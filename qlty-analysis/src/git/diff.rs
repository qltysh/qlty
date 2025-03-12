use crate::code::FileIndex;
use anyhow::Result;
use git2::{Diff, DiffOptions, Index, Repository};
use ignore::{DirEntry, Walk, WalkBuilder};
use qlty_config::issue_transformer::IssueTransformer;
use qlty_types::analysis::v1::Issue;
use std::cell::RefCell;
use std::rc::Rc;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use tracing::{debug, trace, warn};

const PLUS: char = '+';

pub enum DiffMode {
    HeadToIndex,
    HeadToIndexFile(PathBuf),
    HeadToWorkdir,
    UpstreamToWorkdir(String),
}

#[derive(Debug, Clone)]
pub struct GitDiff {
    pub changed_files: Vec<PathBuf>,
    pub line_filter: DiffLineTransformer,
}

impl GitDiff {
    pub fn compute(mode: DiffMode, repository_root: &Path) -> Result<Self> {
        let repository = Repository::open(repository_root)?;
        let head_commit = repository.head()?.peel_to_commit()?;

        let commit = match mode {
            DiffMode::HeadToIndex | DiffMode::HeadToIndexFile(_) => head_commit,
            DiffMode::HeadToWorkdir => head_commit,
            DiffMode::UpstreamToWorkdir(ref upstream_ref) => {
                let upstream_head = repository.revparse_single(upstream_ref)?.peel_to_commit()?;
                let merge_base = repository.merge_base(upstream_head.id(), head_commit.id())?;
                repository.find_commit(merge_base)?
            }
        };

        debug!(
            "Computing diff_tree_to_workdir_with_index for commit {}",
            commit.id()
        );

        let mut diff_opts = DiffOptions::new();

        let diff = match mode {
            DiffMode::HeadToIndex => {
                let index = repository.index()?;

                repository.diff_tree_to_index(
                    Some(&commit.tree()?),
                    Some(&index),
                    Some(&mut diff_opts),
                )?
            }
            DiffMode::HeadToIndexFile(index_path) => {
                let index = Index::open(&index_path)?;

                repository.diff_tree_to_index(
                    Some(&commit.tree()?),
                    Some(&index),
                    Some(&mut diff_opts),
                )?
            }
            _ => {
                diff_opts.include_untracked(true);

                repository
                    .diff_tree_to_workdir_with_index(Some(&commit.tree()?), Some(&mut diff_opts))?
            }
        };

        let changed_files = Self::diff_to_paths(&diff, &repository)?;

        debug!("Found {} changed files", changed_files.len());
        trace!("Changed files: {:?}", changed_files);

        let line_filter = DiffLineTransformer::new(Self::plus_lines_index(
            &diff,
            repository_root.to_path_buf(),
        )?);

        Ok(Self {
            changed_files,
            line_filter,
        })
    }

    fn plus_lines_index(diff: &git2::Diff, repo_path: PathBuf) -> Result<FileIndex> {
        let index = Rc::new(RefCell::new(FileIndex::new()));

        diff.foreach(
            &mut |delta, _progress| {
                if delta.status() == git2::Delta::Untracked {
                    if let Some(new_path) = delta.new_file().path() {
                        // Construct the absolute path for checking fs
                        let absolute_path = repo_path.join(new_path);

                        if absolute_path.is_dir() {
                            // If it's a directory, traverse it to get all files
                            if let Ok(files) = GitDiff::traverse_directory(absolute_path) {
                                for file in files {
                                    // Convert back to a relative path
                                    let relative_path = file.strip_prefix(&repo_path).unwrap();
                                    index.borrow_mut().insert_file(relative_path);
                                }
                            }
                        } else {
                            index.borrow_mut().insert_file(new_path);
                        }
                    }
                }
                true
            },
            None,
            None,
            Some(&mut |delta, _hunk, line| {
                if line.origin() == PLUS {
                    if let Some(new_path) = delta.new_file().path() {
                        if let Some(new_lineno) = line.new_lineno() {
                            index.borrow_mut().insert_line(new_path, new_lineno);
                        }
                    }
                }
                true
            }),
        )?;

        Ok(Rc::try_unwrap(index).unwrap().into_inner())
    }

    fn traverse_directory(path: PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut files = Vec::new();

        for entry in Self::walk_for_path(&path) {
            let entry = entry.unwrap();

            if let Some(file_type) = entry.file_type() {
                if file_type.is_file() {
                    let relative_path = entry.path().to_path_buf();
                    files.push(relative_path);
                }
            } else {
                warn!(
                    "Git diff returned a path that is neither a file nor a directory: {:?}",
                    entry.path()
                );
            }
        }

        Ok(files)
    }

    fn diff_to_paths(diff: &Diff, repository: &Repository) -> Result<Vec<PathBuf>> {
        let delta_paths = Self::collect_delta_paths(diff);
        let delta_file_paths = Self::collect_file_paths(&delta_paths, repository)?;

        Ok(delta_file_paths.into_iter().collect())
    }

    fn collect_delta_paths(diff: &Diff) -> HashSet<PathBuf> {
        let mut delta_paths = HashSet::new();

        for delta in diff.deltas() {
            if let Some(path) = delta.new_file().path() {
                delta_paths.insert(path.to_owned());
            }
        }

        delta_paths
    }

    fn collect_file_paths(
        delta_paths: &HashSet<PathBuf>,
        repository: &Repository,
    ) -> Result<HashSet<PathBuf>> {
        let mut delta_file_paths = HashSet::new();
        let repository_work_dir = repository.workdir().unwrap();

        for path in delta_paths {
            let absolute_path = repository_work_dir.join(path);

            if let Ok(metadata) = absolute_path.metadata() {
                if metadata.is_dir() {
                    Self::collect_directory_paths(
                        &absolute_path,
                        repository,
                        &mut delta_file_paths,
                    )?;
                } else {
                    delta_file_paths.insert(path.clone());
                }
            }
        }

        Ok(delta_file_paths)
    }

    fn collect_directory_paths(
        absolute_path: &Path,
        repository: &Repository,
        delta_file_paths: &mut HashSet<PathBuf>,
    ) -> Result<()> {
        for entry in Self::walk_for_path(absolute_path).filter_map(Result::ok) {
            Self::handle_directory_entry(entry, repository, delta_file_paths);
        }

        Ok(())
    }

    fn handle_directory_entry(
        entry: DirEntry,
        repository: &Repository,
        delta_file_paths: &mut HashSet<PathBuf>,
    ) {
        if let Some(file_type) = entry.file_type() {
            if file_type.is_file() {
                if let Ok(relative_path) = Self::get_relative_path(entry.path(), repository) {
                    delta_file_paths.insert(relative_path);
                }
            }
        } else {
            warn!(
                "Git diff returned a path that is neither a file nor a directory: {:?}",
                entry.path()
            );
        }
    }

    fn get_relative_path(path: &Path, repository: &Repository) -> Result<PathBuf> {
        let relative_path = path.strip_prefix(repository.workdir().unwrap())?;
        Ok(relative_path.to_owned())
    }

    fn walk_for_path(path: &Path) -> Walk {
        WalkBuilder::new(path)
            .hidden(false) // Do not ignore hidden files
            .build()
    }
}

/// Mark issues that are on added lines
#[derive(Debug, Clone)]
pub struct DiffLineTransformer {
    index: FileIndex,
}

impl DiffLineTransformer {
    pub fn new(index: FileIndex) -> Self {
        Self { index }
    }
}

impl IssueTransformer for DiffLineTransformer {
    fn transform(&self, issue: Issue) -> Option<Issue> {
        let issue_path = if let Some(path) = issue.path() {
            path
        } else {
            // TODO: Issues without a path are not filterable
            let mut issue = issue;
            issue.on_added_line = true;
            return Some(issue);
        };

        if issue.location.as_ref().unwrap().range.is_none() {
            if self.index.matches_path(&PathBuf::from(&issue_path)) {
                let mut issue = issue;
                issue.on_added_line = true;
                return Some(issue);
            } else {
                return Some(issue);
            }
        }

        if self
            .index
            .matches_line_range(&PathBuf::from(&issue_path), issue.range()?.line_range_u32())
        {
            let mut issue = issue;
            issue.on_added_line = true;
            Some(issue)
        } else {
            Some(issue)
        }
    }

    fn clone_box(&self) -> Box<dyn IssueTransformer> {
        Box::new(self.clone())
    }
}

/// Filter out issues that are not on added lines
#[derive(Debug, Clone)]
pub struct DiffLineFilter;

impl IssueTransformer for DiffLineFilter {
    fn transform(&self, issue: Issue) -> Option<Issue> {
        if issue.on_added_line {
            Some(issue)
        } else {
            None
        }
    }

    fn clone_box(&self) -> Box<dyn IssueTransformer> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use itertools::Itertools;
    use qlty_test_utilities::git::sample_repo;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_changed_files_respects_gitignore() -> Result<()> {
        let (td, repo) = sample_repo();

        // new_file.me
        // .foo/
        // ├── .gitignore  # contains "bar"
        // ├── see.me      # contains some random content
        // └── bar/
        //     └── ignore.me  # contains content

        fs::write(
            td.path().join("new_file.me"),
            "This is some random content.",
        )
        .unwrap();
        let foo_path = td.path().join(".foo");
        fs::create_dir(&foo_path).unwrap();
        fs::write(foo_path.join(".gitignore"), "bar").unwrap();
        fs::write(foo_path.join("see.me"), "This is some random content.").unwrap();
        let bar_path = foo_path.join("bar");
        fs::create_dir(&bar_path).unwrap();
        fs::write(
            bar_path.join("ignore.me"),
            "This file should be ignored according to .gitignore.",
        )
        .unwrap();

        let git_diff = GitDiff::compute(DiffMode::HeadToWorkdir, &repo.path().parent().unwrap())?;
        let paths = git_diff.changed_files;

        let expected_paths = [
            PathBuf::from(".foo/.gitignore"),
            PathBuf::from(".foo/see.me"),
            PathBuf::from("new_file.me"),
        ];

        assert_eq!(
            paths.iter().cloned().sorted().collect::<Vec<PathBuf>>(),
            expected_paths
        );

        // test new file
        assert_eq!(
            git_diff
                .line_filter
                .index
                .matches_line_range(&PathBuf::from("new_file.me"), 1..=1),
            true
        );

        // test new file in a folder
        assert_eq!(
            git_diff
                .line_filter
                .index
                .matches_line_range(&PathBuf::from(".foo/see.me"), 1..=1),
            true
        );

        // test new ignore file in a folder
        assert_eq!(
            git_diff
                .line_filter
                .index
                .matches_line_range(&PathBuf::from(".foo/bar/ignore.me"), 1..=1),
            false
        );

        Ok(())
    }
}
