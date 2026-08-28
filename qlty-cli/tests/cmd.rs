use crate::helpers::{
    setup_and_run_diff_test_cases, setup_and_run_test_cases, setup_and_run_test_cases_without_git,
};
use indoc::indoc;
use qlty_config::Library;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use trycmd::TestCases;

struct CacheCleanup(PathBuf);

impl Drop for CacheCleanup {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run_qlty(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_qlty"))
        .current_dir(root)
        .env("QLTY_TELEMETRY", "off")
        .args(args)
        .output()
        .unwrap()
}

fn command_output(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn version_tests() {
    TestCases::new().case("tests/cmd/version/**/*.toml");
}

#[test]
fn help_tests() {
    TestCases::new().case("tests/cmd/help/**/*.toml");
}

#[test]
fn metrics_tests() {
    setup_and_run_test_cases("tests/cmd/metrics/**/*.toml");
}

#[test]
fn duplication_tests() {
    setup_and_run_test_cases("tests/cmd/duplication/**/*.toml");
}

#[test]
fn check_tests() {
    // only run .toml files in check directory
    // prevent running toml files from *.in
    setup_and_run_test_cases("tests/cmd/check/*.toml");
}

#[test]
fn filtered_check_does_not_pollute_unfiltered_issue_cache() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    fs::create_dir_all(root.join(".qlty")).unwrap();
    fs::write(
        root.join(".gitignore"),
        ".qlty/logs\n.qlty/out\n.qlty/results\n.qlty/sources\n.qlty/tmp\n",
    )
    .unwrap();
    fs::write(root.join("sample.js"), "const answer = 42;\n").unwrap();
    fs::write(
        root.join(".qlty/qlty.toml"),
        indoc! {r#"
            config_version = "0"

            [plugins.definitions.cache-repro]
            file_types = ["javascript"]
            known_good_version = "1.0.0"

            [plugins.definitions.cache-repro.drivers.lint]
            script = "echo sample.js:1 LINT: reproducible lint issue"
            success_codes = [0]
            output = "stdout"
            output_format = "regex"
            output_regex = '((?P<path>.*):(?P<line>-?\d+) (?P<code>\S+): (?P<message>.+))'
            output_level = "high"
            cache_results = true

            [plugins.definitions.cache-repro.drivers.format]
            script = "exit 0"
            success_codes = [0]
            output = "rewrite"
            driver_type = "formatter"

            [[plugin]]
            name = "cache-repro"
            version = "1.0.0"
        "#},
    )
    .unwrap();

    let _repository = qlty_test_utilities::git::init(root);
    let cache_directory = Library::new(root).unwrap().cache_directory().unwrap();
    let _cache_cleanup = CacheCleanup(cache_directory);

    let filtered = run_qlty(
        root,
        &[
            "check",
            "--all",
            "--filter=cache-repro:fmt",
            "--no-upgrade-check",
            "--no-progress",
        ],
    );
    assert!(filtered.status.success(), "{}", command_output(&filtered));

    let unfiltered = run_qlty(
        root,
        &[
            "check",
            "--all",
            "--no-formatters",
            "--no-upgrade-check",
            "--no-progress",
        ],
    );
    assert_eq!(
        unfiltered.status.code(),
        Some(1),
        "an unfiltered check reused the filtered cache entry\n{}",
        command_output(&unfiltered)
    );
    assert!(
        command_output(&unfiltered).contains("reproducible lint issue"),
        "{}",
        command_output(&unfiltered)
    );
}

#[test]
fn fmt_tests() {
    setup_and_run_test_cases("tests/cmd/fmt/*.toml");
}

#[test]
#[ignore] // ignore tests that may require network connection
fn network_tests() {
    // only run .toml files in check/network/*/ directory
    // Run check and fmt network in sequence
    setup_and_run_test_cases("tests/cmd/network/*/*.toml");
}

#[test]
fn smells_tests() {
    setup_and_run_test_cases("tests/cmd/smells/**/*.toml");
}

#[test]
fn sources_tests() {
    setup_and_run_test_cases("tests/cmd/sources/**/*.toml");
}

#[test]
fn coverage_tests() {
    setup_and_run_test_cases("tests/cmd/coverage/**/*.toml");
}

#[test]
fn without_git_tests() {
    setup_and_run_test_cases_without_git("tests/cmd/without_git/**/*.toml");
}

#[test]
fn build_tests() {
    setup_and_run_test_cases("tests/cmd/build/**/*.toml");
}

#[test]
fn init_tests() {
    setup_and_run_test_cases("tests/cmd/init/*.toml");
}

#[test]
fn config_migrate_tests() {
    setup_and_run_test_cases("tests/cmd/config/migrate/*.toml");
}

#[test]
#[ignore] // ignore tests that require network connection
fn init_network_tests() {
    setup_and_run_test_cases("tests/cmd/init/network/*.toml");
}

#[test]
fn git_based_check_tests() {
    setup_and_run_diff_test_cases("tests/cmd/check/diff_tests/*.toml");
}
