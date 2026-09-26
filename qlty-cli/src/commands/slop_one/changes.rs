//! Find the changed files to compare, and their earlier versions, in Git.

use anyhow::{anyhow, Context as _, Result};
use git2::{Delta, DiffDelta, DiffFindOptions, DiffOptions, FileMode, Oid, Repository};
use qlty_slop_one::{BaseVersion, Change, Contents, RevisionPair};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::git_hook::RefUpdate;

/// The changes a push makes, and the refs that had nothing to compare with.
#[derive(Debug)]
pub struct PushedChanges {
    pub changes: Vec<Change>,
    /// Each pushed commit and its base, abbreviated.
    pub revisions: Vec<RevisionPair>,
    /// Local refs none of whose commits are on a remote yet.
    pub without_base: Vec<String>,
}

/// The working tree's changes since its merge base with the upstream.
#[derive(Debug)]
pub struct UpstreamChanges {
    pub changes: Vec<Change>,
    /// The merge base, abbreviated.
    pub merge_base: String,
}

/// Files changed in the working tree since its merge base with `upstream`,
/// as `qlty check --upstream` finds them. Paths are relative to `cwd`.
pub fn since_upstream(cwd: &Path, upstream: &str) -> Result<UpstreamChanges> {
    let repository = open(cwd)?;
    let root = workdir(&repository)?;
    let head = repository
        .head()
        .and_then(|head| head.peel_to_commit())
        .context("Could not read HEAD")?;
    let upstream_commit = repository
        .revparse_single(upstream)
        .and_then(|object| object.peel_to_commit())
        .with_context(|| format!("Could not find the upstream commit {upstream:?}"))?;
    let base = repository
        .merge_base(upstream_commit.id(), head.id())
        .with_context(|| format!("{upstream:?} and HEAD have no common ancestor"))?;
    let base_tree = repository.find_commit(base)?.tree()?;

    let mut options = DiffOptions::new();
    options.include_untracked(true).recurse_untracked_dirs(true);
    let mut diff =
        repository.diff_tree_to_workdir_with_index(Some(&base_tree), Some(&mut options))?;
    diff.find_similar(Some(
        DiffFindOptions::new().renames(true).for_untracked(true),
    ))?;

    let paths = Paths::new(&root, cwd)?;
    let mut changes = vec![];
    for delta in diff.deltas() {
        let Some(new_path) = delta.new_file().path() else {
            continue;
        };
        if !is_changed(delta.status()) || !is_regular_file(&root.join(new_path)) {
            continue;
        }
        changes.push(Change {
            path: paths.shown(new_path),
            contents: Contents::OnDisk,
            base: base_version(&repository, &delta, base, &paths)?,
        });
    }
    Ok(UpstreamChanges {
        changes,
        merge_base: abbreviate(&repository, base)?,
    })
}

/// Files the pushed refs change, each compared with the last commit on its
/// first-parent line that a remote already has. Paths are relative to `cwd`.
pub fn being_pushed(cwd: &Path, updates: &[RefUpdate]) -> Result<PushedChanges> {
    let repository = open(cwd)?;
    let root = workdir(&repository)?;
    let paths = Paths::new(&root, cwd)?;
    let mut seen = HashSet::new();
    let mut pushed = PushedChanges {
        changes: vec![],
        revisions: vec![],
        without_base: vec![],
    };
    for update in updates.iter().filter(|update| !update.is_deletion()) {
        let local = Oid::from_str(&update.local_object)
            .with_context(|| format!("Invalid object name {:?}", update.local_object))?;
        let remote = if update.creates_remote_ref() {
            None
        } else {
            Oid::from_str(&update.remote_object).ok()
        };
        let Some(base) = pushed_base(&repository, local, remote)? else {
            pushed.without_base.push(update.local_ref.clone());
            continue;
        };
        let revision = RevisionPair {
            base: abbreviate(&repository, base)?,
            pushed: abbreviate(&repository, local)?,
        };
        if !pushed.revisions.contains(&revision) {
            pushed.revisions.push(revision);
        }
        let base_tree = repository.find_commit(base)?.tree()?;
        let head_tree = repository.find_commit(local)?.tree()?;
        let mut diff = repository.diff_tree_to_tree(Some(&base_tree), Some(&head_tree), None)?;
        diff.find_similar(Some(DiffFindOptions::new().renames(true)))?;
        for delta in diff.deltas() {
            let Some(new_path) = delta.new_file().path() else {
                continue;
            };
            if !is_changed(delta.status()) || !is_blob(delta.new_file().mode()) {
                continue;
            }
            let key = (
                new_path.to_path_buf(),
                delta.old_file().id(),
                delta.new_file().id(),
            );
            if !seen.insert(key) {
                continue;
            }
            let blob = repository.find_blob(delta.new_file().id())?;
            pushed.changes.push(Change {
                path: paths.shown(new_path),
                contents: Contents::InMemory(blob.content().to_vec()),
                base: base_version(&repository, &delta, base, &paths)?,
            });
        }
    }
    Ok(pushed)
}

/// Walks the first parents of `local` back to the first commit a remote
/// already has: a remote-tracking ref or `remote` reaches it. `None` when
/// no commit on that line has been pushed anywhere.
fn pushed_base(repository: &Repository, local: Oid, remote: Option<Oid>) -> Result<Option<Oid>> {
    let mut walk = repository.revwalk()?;
    walk.push(local)?;
    for reference in repository.references_glob("refs/remotes/*")? {
        if let Ok(commit) = reference?.peel_to_commit() {
            walk.hide(commit.id())?;
        }
    }
    if let Some(remote) = remote {
        if repository.find_commit(remote).is_ok() {
            walk.hide(remote)?;
        }
    }
    let unpushed = walk.collect::<std::result::Result<HashSet<Oid>, _>>()?;

    let mut commit = repository.find_commit(local)?;
    loop {
        if !unpushed.contains(&commit.id()) {
            return Ok(Some(commit.id()));
        }
        match commit.parent(0) {
            Ok(parent) => commit = parent,
            Err(_) => return Ok(None),
        }
    }
}

fn base_version(
    repository: &Repository,
    delta: &DiffDelta,
    base: Oid,
    paths: &Paths,
) -> Result<Option<BaseVersion>> {
    let old = delta.old_file();
    let has_base = matches!(delta.status(), Delta::Modified | Delta::Renamed);
    let Some(old_path) = old.path().filter(|_| has_base && is_blob(old.mode())) else {
        return Ok(None);
    };
    let blob = repository.find_blob(old.id())?;
    Ok(Some(BaseVersion {
        path: paths.shown(old_path),
        commit: base.to_string(),
        contents: blob.content().to_vec(),
    }))
}

/// The shortest unambiguous abbreviation of a commit id.
fn abbreviate(repository: &Repository, oid: Oid) -> Result<String> {
    let object = repository.find_object(oid, None)?;
    let short = object.short_id()?;
    Ok(short.as_str().unwrap_or_default().to_owned())
}

fn is_changed(status: Delta) -> bool {
    matches!(
        status,
        Delta::Added | Delta::Modified | Delta::Renamed | Delta::Untracked
    )
}

fn is_blob(mode: FileMode) -> bool {
    matches!(mode, FileMode::Blob | FileMode::BlobExecutable)
}

fn is_regular_file(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_file())
}

fn open(cwd: &Path) -> Result<Repository> {
    Repository::discover(cwd).context("This directory is not inside a Git repository")
}

fn workdir(repository: &Repository) -> Result<PathBuf> {
    repository
        .workdir()
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow!("The Git repository has no working tree"))
}

/// Turns repository paths into paths relative to the working directory, so
/// they resolve and read the way paths typed on the command line do.
struct Paths {
    root: PathBuf,
    cwd: PathBuf,
}

impl Paths {
    fn new(root: &Path, cwd: &Path) -> Result<Self> {
        Ok(Self {
            root: fs::canonicalize(root)?,
            cwd: fs::canonicalize(cwd)?,
        })
    }

    fn shown(&self, relative: &Path) -> PathBuf {
        if self.root == self.cwd {
            return relative.to_path_buf();
        }
        let absolute = self.root.join(relative);
        pathdiff::diff_paths(&absolute, &self.cwd).unwrap_or(absolute)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{IndexAddOption, Signature};

    const ZERO: &str = "0000000000000000000000000000000000000000";

    fn write(root: &Path, relative: &str, text: &str) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn commit(repository: &Repository) -> Oid {
        let mut index = repository.index().unwrap();
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .unwrap();
        index.update_all(["*"].iter(), None).unwrap();
        index.write().unwrap();
        let tree = repository.find_tree(index.write_tree().unwrap()).unwrap();
        let signature = Signature::now("Test", "test@example.invalid").unwrap();
        let parent = repository
            .head()
            .ok()
            .and_then(|head| head.peel_to_commit().ok());
        let parents: Vec<_> = parent.iter().collect();
        repository
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                "commit",
                &tree,
                &parents,
            )
            .unwrap()
    }

    fn mark_pushed(repository: &Repository, oid: Oid) {
        repository
            .reference("refs/remotes/origin/main", oid, true, "test")
            .unwrap();
    }

    fn update(local: Oid, remote: &str) -> RefUpdate {
        RefUpdate {
            local_ref: "refs/heads/main".to_owned(),
            local_object: local.to_string(),
            remote_ref: "refs/heads/main".to_owned(),
            remote_object: remote.to_owned(),
        }
    }

    fn paths(changes: &[Change]) -> Vec<String> {
        changes
            .iter()
            .map(|change| change.path.display().to_string())
            .collect()
    }

    fn base_paths(changes: &[Change]) -> Vec<Option<String>> {
        changes
            .iter()
            .map(|change| {
                change
                    .base
                    .as_ref()
                    .map(|base| base.path.display().to_string())
            })
            .collect()
    }

    fn sandbox() -> (tempfile::TempDir, Repository) {
        let dir = tempfile::tempdir().unwrap();
        let repository = Repository::init(dir.path()).unwrap();
        (dir, repository)
    }

    #[test]
    fn the_base_is_the_last_commit_a_remote_has() {
        let (dir, repository) = sandbox();
        write(dir.path(), "a.py", "a = 1\n");
        let pushed = commit(&repository);
        mark_pushed(&repository, pushed);
        write(dir.path(), "a.py", "a = 2\n");
        let local = commit(&repository);
        assert_eq!(pushed_base(&repository, local, None).unwrap(), Some(pushed));
    }

    #[test]
    fn there_is_no_base_when_nothing_was_pushed() {
        let (dir, repository) = sandbox();
        write(dir.path(), "a.py", "a = 1\n");
        let local = commit(&repository);
        assert_eq!(pushed_base(&repository, local, None).unwrap(), None);
    }

    #[test]
    fn the_remote_object_counts_as_pushed() {
        let (dir, repository) = sandbox();
        write(dir.path(), "a.py", "a = 1\n");
        let remote = commit(&repository);
        write(dir.path(), "a.py", "a = 2\n");
        let local = commit(&repository);
        assert_eq!(
            pushed_base(&repository, local, Some(remote)).unwrap(),
            Some(remote)
        );
    }

    #[test]
    fn a_push_compares_modified_files_with_their_base() {
        let (dir, repository) = sandbox();
        write(dir.path(), "a.py", "a = 1\n");
        let pushed = commit(&repository);
        mark_pushed(&repository, pushed);
        write(dir.path(), "a.py", "a = 2\n");
        write(dir.path(), "b.py", "b = 1\n");
        let local = commit(&repository);
        let changes = being_pushed(dir.path(), &[update(local, ZERO)])
            .unwrap()
            .changes;
        assert_eq!(base_paths(&changes), [Some("a.py".to_owned()), None]);
    }

    #[test]
    fn a_push_reads_the_pushed_contents_not_the_working_tree() {
        let (dir, repository) = sandbox();
        write(dir.path(), "a.py", "a = 1\n");
        let pushed = commit(&repository);
        mark_pushed(&repository, pushed);
        write(dir.path(), "a.py", "a = 2\n");
        let local = commit(&repository);
        write(dir.path(), "a.py", "a = 3\n");
        let changes = being_pushed(dir.path(), &[update(local, ZERO)])
            .unwrap()
            .changes;
        assert!(matches!(
            &changes[0].contents,
            Contents::InMemory(bytes) if bytes == b"a = 2\n"
        ));
    }

    #[test]
    fn a_push_compares_a_renamed_file_with_its_old_path() {
        let (dir, repository) = sandbox();
        write(
            dir.path(),
            "old.py",
            "def f():\n    return 1\n\n\ndef g():\n    return 2\n",
        );
        let pushed = commit(&repository);
        mark_pushed(&repository, pushed);
        fs::rename(dir.path().join("old.py"), dir.path().join("new.py")).unwrap();
        let local = commit(&repository);
        let changes = being_pushed(dir.path(), &[update(local, ZERO)])
            .unwrap()
            .changes;
        assert_eq!(base_paths(&changes), [Some("old.py".to_owned())]);
    }

    #[test]
    fn a_push_skips_deleted_refs() {
        let (dir, repository) = sandbox();
        write(dir.path(), "a.py", "a = 1\n");
        let local = commit(&repository);
        let deletion = RefUpdate {
            local_ref: "(delete)".to_owned(),
            local_object: ZERO.to_owned(),
            remote_ref: "refs/heads/main".to_owned(),
            remote_object: local.to_string(),
        };
        let pushed = being_pushed(dir.path(), &[deletion]).unwrap();
        assert_eq!((pushed.changes.len(), pushed.without_base.len()), (0, 0));
    }

    #[test]
    fn a_push_of_an_unpublished_history_has_no_base() {
        let (dir, repository) = sandbox();
        write(dir.path(), "a.py", "a = 1\n");
        let local = commit(&repository);
        let pushed = being_pushed(dir.path(), &[update(local, ZERO)]).unwrap();
        assert_eq!(pushed.without_base, ["refs/heads/main"]);
    }

    #[test]
    fn a_push_counts_a_file_changed_by_two_refs_once() {
        let (dir, repository) = sandbox();
        write(dir.path(), "a.py", "a = 1\n");
        let pushed = commit(&repository);
        mark_pushed(&repository, pushed);
        write(dir.path(), "a.py", "a = 2\n");
        let local = commit(&repository);
        let changes = being_pushed(
            dir.path(),
            &[update(local, ZERO), update(local, &pushed.to_string())],
        )
        .unwrap()
        .changes;
        assert_eq!(paths(&changes), ["a.py"]);
    }

    #[test]
    fn a_push_reports_its_revisions() {
        let (dir, repository) = sandbox();
        write(dir.path(), "a.py", "a = 1\n");
        let pushed = commit(&repository);
        mark_pushed(&repository, pushed);
        write(dir.path(), "a.py", "a = 2\n");
        let local = commit(&repository);
        let revisions = being_pushed(dir.path(), &[update(local, ZERO)])
            .unwrap()
            .revisions;
        assert_eq!(
            revisions,
            [RevisionPair {
                base: pushed.to_string()[..7].to_owned(),
                pushed: local.to_string()[..7].to_owned(),
            }]
        );
    }

    #[test]
    fn upstream_reports_the_merge_base() {
        let (dir, repository) = sandbox();
        write(dir.path(), "a.py", "a = 1\n");
        let base = commit(&repository);
        assert_eq!(
            since_upstream(dir.path(), "HEAD").unwrap().merge_base,
            base.to_string()[..7]
        );
    }

    #[test]
    fn upstream_includes_uncommitted_and_untracked_files() {
        let (dir, repository) = sandbox();
        write(dir.path(), "a.py", "a = 1\n");
        commit(&repository);
        repository
            .reference(
                "refs/heads/upstream",
                repository.head().unwrap().target().unwrap(),
                true,
                "test",
            )
            .unwrap();
        write(dir.path(), "a.py", "a = 2\n");
        write(dir.path(), "lib/b.py", "b = 1\n");
        let changes = since_upstream(dir.path(), "upstream").unwrap().changes;
        assert_eq!(base_paths(&changes), [Some("a.py".to_owned()), None]);
    }

    #[test]
    fn upstream_skips_deleted_files() {
        let (dir, repository) = sandbox();
        write(dir.path(), "a.py", "a = 1\n");
        commit(&repository);
        fs::remove_file(dir.path().join("a.py")).unwrap();
        assert_eq!(since_upstream(dir.path(), "HEAD").unwrap().changes.len(), 0);
    }

    #[test]
    fn upstream_paths_are_relative_to_the_working_directory() {
        let (dir, repository) = sandbox();
        write(dir.path(), "a.py", "a = 1\n");
        commit(&repository);
        write(dir.path(), "a.py", "a = 2\n");
        fs::create_dir(dir.path().join("sub")).unwrap();
        let changes = since_upstream(&dir.path().join("sub"), "HEAD")
            .unwrap()
            .changes;
        assert_eq!(changes[0].path, Path::new("..").join("a.py"));
    }
}
