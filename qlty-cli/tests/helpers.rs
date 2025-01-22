use duct::cmd;
use glob::glob;
use itertools::Itertools;
use qlty_analysis::join_path_string;
use std::{
    ffi::OsStr,
    path::{Component, Path, PathBuf},
    time::Duration,
};
use trycmd::TestCases;

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
const DEFAULT_TEST_TIMEOUT: u64 = 600;

const GIT_SETUP_SCRIPT: &str = r#"
  git init --initial-branch=main &&
  git add . &&
  git commit --no-gpg-sign --message initial
"#;

const GIT_DIFF_SETUP_SCRIPT: &str = r#"
  git init --initial-branch=main &&
  git add . &&
  git reset -- diff &&
  git commit --no-gpg-sign --message initial &&
  git checkout -b test_branch &&
  git add . &&
  git commit --no-gpg-sign --message initial
"#;

pub fn setup_and_run_diff_test_cases(glob: &str) {
    setup_and_run_test_cases_diff_flag(glob, true);
}

pub fn setup_and_run_test_cases(glob: &str) {
    setup_and_run_test_cases_diff_flag(glob, false);
}

fn setup_and_run_test_cases_diff_flag(glob: &str, diff: bool) {
    let (cases, fixtures) = detect_cases_and_fixtures(glob);

    let _repositories: Vec<_> = fixtures
        .iter()
        .map(|path: &PathBuf| RepositoryFixture::setup(path, diff))
        .collect();

    for case in cases {
        TestCases::new()
            .case(case.strip_prefix(MANIFEST_DIR).unwrap())
            .env("RUST_BACKTRACE", "0")
            .timeout(Duration::from_secs(DEFAULT_TEST_TIMEOUT));
    }
}

fn detect_cases_and_fixtures(path_glob: &str) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut cases = vec![];
    let mut fixtures = vec![];
    let full_path_glob = join_path_string!(MANIFEST_DIR, path_glob);

    for path in glob(&full_path_glob).unwrap() {
        let mut path = path.unwrap();
        let filename = path.file_name().unwrap();

        if filename != "qlty.toml"
            && !path
                .components()
                .contains(&Component::Normal(OsStr::new(".qlty")))
        {
            cases.push(path.clone());

            let basename = filename.to_str().unwrap().split('.').next().unwrap();
            let input_dir = format!("{}.in", basename);

            path.pop();
            let input_path = path.join(input_dir);
            let gitignore_path = input_path.join(".gitignore");

            if gitignore_path.exists() {
                fixtures.push(input_path);
            }
        }
    }

    (cases, fixtures)
}

#[derive(Debug)]
struct RepositoryFixture {
    path: PathBuf,
    diff_tests: bool,
}

impl RepositoryFixture {
    pub fn setup(path: &Path, diff_tests: bool) -> Self {
        let test_repository = Self {
            path: path.to_path_buf(),
            diff_tests,
        };
        test_repository.create();
        test_repository
    }

    pub fn create(&self) {
        if self.git_dir().exists() {
            self.destroy();
        }

        let (shell, flag) = if cfg!(windows) {
            ("cmd", "/c")
        } else {
            ("sh", "-c")
        };

        let script = if self.diff_tests {
            GIT_DIFF_SETUP_SCRIPT
        } else {
            GIT_SETUP_SCRIPT
        };

        cmd!(shell, flag, script.to_string().trim().replace('\n', ""))
            .dir(&self.path)
            .env("GIT_COMMITTER_DATE", "2024-01-01T00:00:00+00:00")
            .env("GIT_COMMITTER_NAME", "TEST")
            .env("GIT_COMMITTER_EMAIL", "test@codeclimate.com")
            .env("GIT_AUTHOR_DATE", "2024-01-01T00:00:00+00:00")
            .env("GIT_AUTHOR_NAME", "TEST")
            .env("GIT_AUTHOR_EMAIL", "test@codeclimate.com")
            .read()
            .unwrap();
    }

    pub fn destroy(&self) {
        std::fs::remove_dir_all(&self.git_dir()).unwrap_or_default();
    }

    fn git_dir(&self) -> PathBuf {
        self.path.join(".git")
    }
}

impl Drop for RepositoryFixture {
    fn drop(&mut self) {
        self.destroy();
    }
}
