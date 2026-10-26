//! A throwaway Git repository with commits at chosen committer times.

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate};
use git2::{IndexAddOption, Repository, RepositoryInitOptions, Signature, Time};
use qlty_slop_one_trends::periods::Interval;
use qlty_slop_one_trends::run::RunDir;
use qlty_slop_one_trends::snapshot::FreezeOptions;
use tempfile::TempDir;

pub const ORIGIN_URL: &str = "https://github.com/owner/demo.git";

pub struct TestRepo {
    root: TempDir,
    repository: Repository,
}

impl TestRepo {
    pub fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let mut options = RepositoryInitOptions::new();
        options.initial_head("main");
        let repository = Repository::init_opts(root.path().join("checkout"), &options).unwrap();
        repository.remote("origin", ORIGIN_URL).unwrap();
        let mut config = repository.config().unwrap();
        config.set_bool("core.autocrlf", false).unwrap();
        config.set_bool("core.safecrlf", false).unwrap();
        Self { root, repository }
    }

    pub fn path(&self) -> &Path {
        self.repository.workdir().unwrap()
    }

    pub fn repository(&self) -> &Repository {
        &self.repository
    }

    pub fn run_dir(&self, name: &str) -> RunDir {
        RunDir::new(self.root.path().join("runs").join(name))
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.root.path().join("cache")
    }

    pub fn write(&self, path: &str, contents: &str) {
        self.write_bytes(path, contents.as_bytes());
    }

    pub fn write_bytes(&self, path: &str, contents: &[u8]) {
        let target = self.path().join(path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, contents).unwrap();
    }

    #[cfg(unix)]
    pub fn write_executable(&self, path: &str, contents: &str) {
        use std::os::unix::fs::PermissionsExt as _;

        self.write(path, contents);
        let target = self.path().join(path);
        fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[cfg(unix)]
    pub fn symlink(&self, path: &str, target: &str) {
        let link = self.path().join(path);
        fs::create_dir_all(link.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(target, link).unwrap();
    }

    pub fn remove(&self, path: &str) {
        fs::remove_file(self.path().join(path)).unwrap();
    }

    /// Stages every change and commits it with `when` as the author and
    /// committer time, returning the commit id.
    pub fn commit(&self, when: &str) -> String {
        let stamp = DateTime::parse_from_rfc3339(when).unwrap();
        let time = Time::new(stamp.timestamp(), stamp.offset().local_minus_utc() / 60);
        let signature = Signature::new("History test", "history@example.invalid", &time).unwrap();
        let mut index = self.repository.index().unwrap();
        index.add_all(["*"], IndexAddOption::DEFAULT, None).unwrap();
        index.update_all(["*"], None).unwrap();
        index.write().unwrap();
        let tree = self
            .repository
            .find_tree(index.write_tree().unwrap())
            .unwrap();
        let parent = self
            .repository
            .head()
            .ok()
            .and_then(|head| head.peel_to_commit().ok());
        let parents: Vec<_> = parent.iter().collect();
        self.repository
            .commit(Some("HEAD"), &signature, &signature, when, &tree, &parents)
            .unwrap()
            .to_string()
    }

    pub fn head(&self) -> String {
        self.repository
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id()
            .to_string()
    }

    /// Points `origin/main` at the current head and makes it `origin/HEAD`.
    pub fn track_origin_main(&self) {
        let head = self
            .repository
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id();
        self.repository
            .reference("refs/remotes/origin/main", head, true, "track")
            .unwrap();
        self.repository
            .reference_symbolic(
                "refs/remotes/origin/HEAD",
                "refs/remotes/origin/main",
                true,
                "track",
            )
            .unwrap();
    }

    pub fn options(&self, since: &str, interval: Interval, as_of: &str) -> FreezeOptions {
        FreezeOptions {
            repo: self.path().to_path_buf(),
            since: NaiveDate::parse_from_str(since, "%Y-%m-%d").unwrap(),
            interval,
            timezone: chrono_tz::America::New_York,
            git_ref: None,
            as_of: Some(DateTime::parse_from_rfc3339(as_of).unwrap()),
            project: None,
            repository_url: None,
            cache_directory: self.cache_dir(),
        }
    }
}
