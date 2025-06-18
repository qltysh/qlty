use duct::cmd;
use glob::glob;
use itertools::Itertools;
use qlty_analysis::join_path_string;
use std::{
    ffi::OsStr,
    fs,
    path::{Component, Path, PathBuf},
    time::Duration,
};
use tempfile::TempDir;
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

pub fn setup_and_run_git_free_test_cases(glob: &str) {
    let (cases, fixtures) = detect_cases_and_fixtures(glob);

    let _isolated_fixtures: Vec<_> = cases
        .iter()
        .map(|case| setup_isolated_git_free_test(case, &fixtures))
        .collect();
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

fn setup_isolated_git_free_test(case: &Path, fixtures: &[PathBuf]) -> GitFreeFixture {
    let temp_dir = tempfile::tempdir().unwrap();

    // Find matching fixture for this test case
    let case_name = case.file_stem().unwrap().to_str().unwrap();
    let matching_fixture = fixtures.iter().find(|fixture| {
        fixture
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with(case_name)
    });

    // Read the original test case file
    let test_content = fs::read_to_string(case).unwrap();

    // Create a new isolated test that runs in the temp directory
    let temp_test_dir = temp_dir.path().join("test");
    fs::create_dir_all(&temp_test_dir).unwrap();

    let test_stdout = case
        .parent()
        .unwrap()
        .join(case_name)
        .with_extension("stdout");

    if test_stdout.exists() {
        // If a .stdout file exists, read its content
        let stdout_str = fs::read_to_string(test_stdout).unwrap();
        // Append the expected stdout to the test content
        let temp_stdout_path = temp_test_dir.join(case_name).with_extension("stdout");
        fs::write(temp_stdout_path, stdout_str).unwrap();
    }

    let test_stderr = case
        .parent()
        .unwrap()
        .join(case_name)
        .with_extension("stderr");
    if test_stderr.exists() {
        // If a .stdout file exists, read its content
        let stdout_str = fs::read_to_string(test_stderr).unwrap();
        // Append the expected stdout to the test content
        let temp_stdout_path = temp_test_dir.join(case_name).with_extension("stderr");
        fs::write(temp_stdout_path, stdout_str).unwrap();
    }

    // Write test case to temp directory with modified working directory
    let temp_test_case = temp_test_dir.join(format!("{}.toml", case_name));
    fs::write(&temp_test_case, &test_content).unwrap();

    // Create .in directory in temp location
    let temp_in_dir = temp_test_dir.join(format!("{}.in", case_name));
    fs::create_dir_all(&temp_in_dir).unwrap();

    if let Some(fixture_path) = matching_fixture {
        copy_dir_contents(fixture_path, &temp_in_dir, true);
    }

    // Run the test case from the isolated temp directory
    TestCases::new()
        .case(temp_test_case)
        .env("RUST_BACKTRACE", "0")
        .timeout(Duration::from_secs(DEFAULT_TEST_TIMEOUT));

    GitFreeFixture {
        _temp_dir: temp_dir,
    }
}

fn copy_dir_contents(source: &Path, destination: &Path, exclude_git: bool) {
    if !destination.exists() {
        fs::create_dir_all(destination).unwrap();
    }

    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();

        // Skip .git directories in git-free tests
        if exclude_git && file_name == ".git" {
            continue;
        }

        let src_path = entry.path();
        let dst_path = destination.join(file_name);

        if src_path.is_dir() {
            copy_dir_contents(&src_path, &dst_path, exclude_git);
        } else {
            fs::copy(&src_path, &dst_path).unwrap();
        }
    }
}

struct GitFreeFixture {
    // Need to keep reference to TempDir to prevent it from being dropped
    _temp_dir: TempDir,
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

        let (shell, flag) = Self::get_shell_and_flag();

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
        self.reset_git();
        std::fs::remove_dir_all(&self.git_dir()).unwrap_or_default();
    }

    fn reset_git(&self) {
        let (shell, flag) = Self::get_shell_and_flag();
        cmd!(shell, flag, "git reset --hard")
            .dir(&self.path)
            .read()
            .unwrap();
    }

    fn git_dir(&self) -> PathBuf {
        self.path.join(".git")
    }

    fn get_shell_and_flag() -> (&'static str, &'static str) {
        if cfg!(windows) {
            ("cmd", "/c")
        } else {
            ("sh", "-c")
        }
    }
}

impl Drop for RepositoryFixture {
    fn drop(&mut self) {
        self.destroy();
    }
}
