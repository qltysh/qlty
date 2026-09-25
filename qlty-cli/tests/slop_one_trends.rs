use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use git2::{IndexAddOption, Repository, Signature, Time};
use serde_json::Value;

const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/slop_one_trends"
);
const REPOSITORY_URL: &str = "https://github.com/example/trendrepo";

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn write(root: &Path, relative: &str, fixture: &str) {
    let target = root.join(relative);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::copy(Path::new(FIXTURES).join(fixture), target).unwrap();
}

fn commit(repository: &Repository, message: &str, seconds: i64) {
    let mut index = repository.index().unwrap();
    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .unwrap();
    index.update_all(["*"].iter(), None).unwrap();
    index.write().unwrap();
    let tree = repository.find_tree(index.write_tree().unwrap()).unwrap();
    let signature = Signature::new(
        "Trend Test",
        "trend@example.invalid",
        &Time::new(seconds, 0),
    )
    .unwrap();
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
            message,
            &tree,
            &parents,
        )
        .unwrap();
}

/// Three weekly commits: a baseline, a change plus an addition, and a rename.
fn synthetic_repository(root: &Path) {
    let repository = Repository::init(root).unwrap();
    write(root, "lib/simple.py", "v1/lib/simple.py");
    write(root, "lib/tangled.py", "v1/lib/tangled.py");
    write(root, "tests/test_simple.py", "v1/tests/test_simple.py");
    commit(&repository, "baseline", 1_751_457_600);
    write(root, "lib/tangled.py", "v2/lib/tangled.py");
    write(root, "lib/other.py", "v2/lib/other.py");
    commit(&repository, "simplify tangled, add other", 1_752_062_400);
    fs::rename(
        root.join("lib/simple.py"),
        root.join("lib/simple_renamed.py"),
    )
    .unwrap();
    commit(&repository, "rename simple", 1_752_667_200);
}

struct Sandbox {
    _dir: tempfile::TempDir,
    repo: PathBuf,
    cache: PathBuf,
    output: PathBuf,
}

fn sandbox() -> Sandbox {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    let cache = dir.path().join("cache");
    let output = dir.path().join("out");
    synthetic_repository(&repo);
    copy_dir(&Path::new(FIXTURES).join("cache"), &cache);
    fs::create_dir_all(&output).unwrap();
    Sandbox {
        _dir: dir,
        repo,
        cache,
        output,
    }
}

fn qlty(sandbox: &Sandbox, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_qlty"))
        .current_dir(&sandbox.repo)
        .env("QLTY_TELEMETRY", "off")
        .env("RUST_BACKTRACE", "0")
        .args(["slop-one", "trends"])
        .args(args)
        .args([
            "--repository-url",
            REPOSITORY_URL,
            "--cache-dir",
            sandbox.cache.to_str().unwrap(),
            "--output",
            sandbox.output.to_str().unwrap(),
        ])
        .output()
        .unwrap()
}

fn build(sandbox: &Sandbox) -> Output {
    qlty(
        sandbox,
        &[
            "build",
            "--since",
            "2025-07-01",
            "--as-of",
            "2025-07-20T12:00:00+00:00",
            "--timezone",
            "UTC",
            "--offline",
            "--jobs",
            "2",
        ],
    )
}

fn data(sandbox: &Sandbox) -> Value {
    serde_json::from_str(&fs::read_to_string(sandbox.output.join("trendrepo-weekly.json")).unwrap())
        .unwrap()
}

#[test]
fn build_freezes_three_weekly_snapshots_and_writes_every_artifact() {
    let sandbox = sandbox();
    let output = build(&sandbox);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let weeks = data(&sandbox)["weeks"].as_array().unwrap().len();
    assert_eq!(weeks, 3);
    let artifacts: Vec<bool> = [
        ".json",
        "-files.json.gz",
        "-exclusions.json",
        "-explanations.json.gz",
        ".html",
    ]
    .iter()
    .map(|suffix| {
        sandbox
            .output
            .join(format!("trendrepo-weekly{suffix}"))
            .is_file()
    })
    .collect();
    assert_eq!(artifacts, [true; 5]);
}

#[test]
fn second_week_attributes_the_change_and_the_addition() {
    let sandbox = sandbox();
    build(&sandbox);
    let data = data(&sandbox);
    let change = &data["weeks"][1]["change"];
    let changed: Vec<&str> = change["contributions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["after_paths"][0].as_str().unwrap())
        .collect();
    assert_eq!(changed, ["lib/tangled.py"]);
    assert_eq!(
        change["additions"]["files"],
        serde_json::json!(["lib/other.py"])
    );
    assert!(change["contributions"][0]["contribution"].as_f64().unwrap() > 0.0);
}

#[test]
fn third_week_matches_the_rename_with_git_evidence() {
    let sandbox = sandbox();
    build(&sandbox);
    let data = data(&sandbox);
    let change = &data["weeks"][2]["change"];
    assert_eq!(change["additions"]["files"], serde_json::json!([]));
    assert_eq!(change["removals"]["files"], serde_json::json!([]));
    assert_eq!(change["net"], serde_json::json!(0.0));
    assert_eq!(change["compared_groups"], serde_json::json!(3));
}

#[test]
fn test_files_are_skipped_in_every_snapshot() {
    let sandbox = sandbox();
    build(&sandbox);
    let data = data(&sandbox);
    let skipped: Vec<u64> = data["weeks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|week| week["test_paths_excluded"].as_u64().unwrap())
        .collect();
    assert_eq!(skipped, [1, 1, 1]);
}

#[test]
fn report_html_names_the_project_and_has_no_placeholders() {
    let sandbox = sandbox();
    build(&sandbox);
    let html = fs::read_to_string(sandbox.output.join("trendrepo-weekly.html")).unwrap();
    assert!(html.contains("<title>Trendrepo code quality changes</title>"));
    assert!(!html.contains("__"));
    assert_eq!(html.matches("class=\"week-hit\"").count(), 3);
}

#[test]
fn status_reports_every_distinct_content_as_measured() {
    let sandbox = sandbox();
    build(&sandbox);
    let output = qlty(&sandbox, &["status"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("STATUS 4 / 4 measured; 0 analysis errors"));
}

#[test]
fn render_regenerates_the_report_from_exported_data() {
    let sandbox = sandbox();
    build(&sandbox);
    fs::remove_file(sandbox.output.join("trendrepo-weekly.html")).unwrap();
    let output = qlty(&sandbox, &["render"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(sandbox.output.join("trendrepo-weekly.html").is_file());
}

#[test]
fn rebuilding_is_idempotent_and_makes_no_requests() {
    let sandbox = sandbox();
    build(&sandbox);
    let first = fs::read(sandbox.output.join("trendrepo-weekly-files.json.gz")).unwrap();
    let output = build(&sandbox);
    assert!(output.status.success());
    let second = fs::read(sandbox.output.join("trendrepo-weekly-files.json.gz")).unwrap();
    assert_eq!(first, second);
}

#[test]
fn a_frozen_run_rejects_different_settings() {
    let sandbox = sandbox();
    build(&sandbox);
    let output = qlty(
        &sandbox,
        &[
            "build",
            "--since",
            "2025-06-01",
            "--as-of",
            "2025-07-20T12:00:00+00:00",
            "--timezone",
            "UTC",
            "--offline",
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("already frozen"));
}

fn build_with_monthly(sandbox: &Sandbox) -> Output {
    qlty(
        sandbox,
        &[
            "build",
            "--since",
            "2025-07-01",
            "--as-of",
            "2025-08-20T12:00:00+00:00",
            "--timezone",
            "UTC",
            "--offline",
            "--jobs",
            "2",
            "--monthly",
        ],
    )
}

fn manifest(sandbox: &Sandbox, name: &str) -> Value {
    let path = sandbox
        .cache
        .join("trends/github.com/example/trendrepo")
        .join(name)
        .join("manifest.json");
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn monthly_companion_shares_the_weekly_cutoff_and_joins_the_picker() {
    let sandbox = sandbox();
    let output = build_with_monthly(&sandbox);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("month snapshots"));
    let weekly = manifest(&sandbox, "trendrepo-weekly");
    let monthly = manifest(&sandbox, "trendrepo-monthly");
    assert_eq!(monthly["interval"], "month");
    assert_eq!(monthly["created_at"], weekly["created_at"]);
    assert!(sandbox.output.join("trendrepo-monthly.json").is_file());
    let html = fs::read_to_string(sandbox.output.join("trendrepo-weekly.html")).unwrap();
    assert!(html.contains("<option value=\"month\">Monthly</option>"));
}

#[test]
fn a_monthly_companion_needs_a_weekly_run() {
    let sandbox = sandbox();
    let output = qlty(
        &sandbox,
        &[
            "build",
            "--since",
            "2025-07-01",
            "--interval",
            "month",
            "--monthly",
            "--offline",
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--monthly"));
}

fn slop_one(sandbox: &Sandbox, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_qlty"))
        .current_dir(&sandbox.repo)
        .env("QLTY_TELEMETRY", "off")
        .env("RUST_BACKTRACE", "0")
        .arg("slop-one")
        .args(args)
        .args(["--cache-dir", sandbox.cache.to_str().unwrap()])
        .output()
        .unwrap()
}

fn report(sandbox: &Sandbox) -> Output {
    Repository::open(&sandbox.repo)
        .unwrap()
        .remote("origin", REPOSITORY_URL)
        .ok();
    slop_one(
        sandbox,
        &[
            "--since",
            "2025-07-01",
            "--offline",
            "--no-open",
            "--jobs",
            "2",
            "--output",
            sandbox.output.to_str().unwrap(),
        ],
    )
}

fn output_names(sandbox: &Sandbox) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(&sandbox.output)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn bare_slop_one_writes_only_the_report_and_keeps_data_in_the_cache() {
    let sandbox = sandbox();
    let output = report(&sandbox);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output_names(&sandbox), ["trendrepo-trends.html"]);
    let exports = sandbox
        .cache
        .join("trends/github.com/example/trendrepo/exports");
    assert!(exports.join("trendrepo-weekly.json").is_file());
    assert!(exports.join("trendrepo-monthly.json").is_file());
    let html = fs::read_to_string(sandbox.output.join("trendrepo-trends.html")).unwrap();
    assert!(html.contains("<option value=\"month\">Monthly</option>"));
}

#[test]
fn bare_slop_one_refreshes_the_runs_on_every_call() {
    let sandbox = sandbox();
    report(&sandbox);
    let first = manifest(&sandbox, "trendrepo-weekly");
    let output = report(&sandbox);
    assert!(output.status.success());
    let second = manifest(&sandbox, "trendrepo-weekly");
    assert!(second["created_at"].as_str().unwrap() > first["created_at"].as_str().unwrap());
    assert_eq!(
        manifest(&sandbox, "trendrepo-monthly")["created_at"],
        second["created_at"]
    );
}

#[test]
fn scoring_flags_are_rejected_for_the_report() {
    let sandbox = sandbox();
    let output = slop_one(&sandbox, &["--json", "--offline", "--no-open"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--json"));
}

#[test]
fn report_flags_are_rejected_when_scoring_files() {
    let sandbox = sandbox();
    let output = slop_one(
        &sandbox,
        &["--since", "2025-01-01", "--offline", "lib/simple.py"],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--since"));
}

#[test]
fn help_hides_the_trends_plumbing() {
    let sandbox = sandbox();
    let help = String::from_utf8_lossy(&slop_one(&sandbox, &["--help"]).stdout).into_owned();
    assert!(help.contains("Report options"));
    assert!(help.contains("Scoring options"));
    assert!(!help.contains("Commands:"));
    assert!(!help.contains("  trends "));
}

fn slop_one_command(sandbox: &Sandbox) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_qlty"));
    command
        .current_dir(&sandbox.repo)
        .env("QLTY_TELEMETRY", "off")
        .env("RUST_BACKTRACE", "0")
        .arg("slop-one");
    command
}

/// A source file whose content no cache has seen.
fn fresh_source() -> String {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("def fresh_{nonce}(value):\n    return value * {nonce}\n")
}

#[test]
fn report_without_a_key_explains_how_to_get_one() {
    let sandbox = sandbox();
    let repository = Repository::open(&sandbox.repo).unwrap();
    repository.remote("origin", REPOSITORY_URL).unwrap();
    fs::write(sandbox.repo.join("lib/fresh.py"), fresh_source()).unwrap();
    commit(&repository, "add fresh", 1_753_272_000);
    let output = slop_one_command(&sandbox)
        .env_remove("TYPESAFE_API_KEY")
        .args(["--since", "2025-07-01", "--no-open", "--jobs", "2"])
        .args(["--cache-dir", sandbox.cache.to_str().unwrap()])
        .args(["--output", sandbox.output.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Jev API key found"), "{stderr}");
    assert!(stderr.contains("https://typesafe.ai/"));
    assert!(stderr.contains("TYPESAFE_API_KEY"));
    assert!(!stderr.contains("Vercel"));
}

#[test]
fn scoring_a_file_without_a_key_explains_how_to_get_one() {
    let sandbox = sandbox();
    fs::write(sandbox.repo.join("lib/fresh.py"), fresh_source()).unwrap();
    let output = slop_one_command(&sandbox)
        .env_remove("TYPESAFE_API_KEY")
        .args([
            "lib/fresh.py",
            "--cache-dir",
            sandbox.cache.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Jev API key found"), "{stderr}");
    assert!(stderr.contains("https://typesafe.ai/"));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
}

#[test]
fn scoring_through_openrouter_without_a_key_names_the_openrouter_variable() {
    let sandbox = sandbox();
    fs::write(sandbox.repo.join("lib/fresh.py"), fresh_source()).unwrap();
    let output = slop_one_command(&sandbox)
        .env_remove("OPENROUTER_API_KEY")
        .args([
            "lib/fresh.py",
            "--provider",
            "openrouter",
            "--cache-dir",
            sandbox.cache.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No OpenRouter API key found"), "{stderr}");
    assert!(stderr.contains("OPENROUTER_API_KEY"));
    assert!(stderr.contains("https://openrouter.ai/settings/keys"));
    assert!(!stderr.contains("https://typesafe.ai/"));
}

#[test]
fn report_prints_the_steps_and_the_sparkline() {
    let sandbox = sandbox();
    let output = report(&sandbox);
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Reading"), "{stderr}");
    assert!(stderr.contains("weekly"));
    assert!(stderr.contains("monthly snapshots"));
    assert!(stderr.contains("Scoring"));
    assert!(stderr.contains("Rendering"));
    assert!(stderr.contains("this week net"), "{stderr}");
    assert!(String::from_utf8_lossy(&output.stdout).contains("Report: "));
}
