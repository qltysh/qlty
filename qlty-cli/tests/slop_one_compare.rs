use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use git2::{IndexAddOption, Oid, Repository, Signature, Time};
use serde_json::Value;

const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/slop_one_trends"
);
const ZERO: &str = "0000000000000000000000000000000000000000";
const EXPECTED_PROMPT: &str = include_str!("fixtures/slop_one_compare/pre_push_prompt.txt");

struct Sandbox {
    _dir: tempfile::TempDir,
    repo: PathBuf,
    cache: PathBuf,
    repository: Repository,
}

fn sandbox() -> Sandbox {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    let cache = dir.path().join("cache");
    copy_dir(&Path::new(FIXTURES).join("cache"), &cache);
    let repository = Repository::init(&repo).unwrap();
    Sandbox {
        _dir: dir,
        repo,
        cache,
        repository,
    }
}

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

impl Sandbox {
    fn write(&self, relative: &str, fixture: &str) {
        let target = self.repo.join(relative);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::copy(Path::new(FIXTURES).join(fixture), target).unwrap();
    }

    fn commit(&self) -> Oid {
        let mut index = self.repository.index().unwrap();
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .unwrap();
        index.update_all(["*"].iter(), None).unwrap();
        index.write().unwrap();
        let tree = self
            .repository
            .find_tree(index.write_tree().unwrap())
            .unwrap();
        let signature = Signature::new(
            "Compare Test",
            "compare@example.invalid",
            &Time::new(1_751_457_600, 0),
        )
        .unwrap();
        let parent = self
            .repository
            .head()
            .ok()
            .and_then(|head| head.peel_to_commit().ok());
        let parents: Vec<_> = parent.iter().collect();
        self.repository
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

    fn mark_pushed(&self, oid: Oid) {
        self.repository
            .reference("refs/remotes/origin/main", oid, true, "test")
            .unwrap();
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_qlty"));
        command
            .current_dir(&self.repo)
            .env("QLTY_TELEMETRY", "off")
            .env("RUST_BACKTRACE", "0")
            .arg("slop-one")
            .args(args);
        command
    }

    fn offline_command(&self, args: &[&str]) -> Command {
        let mut command = self.command(args);
        command.arg("--offline").arg("--cache-dir").arg(&self.cache);
        command
    }

    fn upstream(&self, args: &[&str]) -> Output {
        self.offline_command(&[&["--upstream", "HEAD"], args].concat())
            .output()
            .unwrap()
    }

    fn pre_push(&self, mut command: Command, input: &str) -> Output {
        let mut child = command
            .arg("--upstream-from-pre-push")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }
}

fn push_input(local: Oid) -> String {
    format!("refs/heads/main {local} refs/heads/main {ZERO}\n")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn first_line(output: &Output) -> String {
    stdout(output).lines().next().unwrap_or_default().to_owned()
}

fn last_line(output: &Output) -> String {
    stdout(output).lines().last().unwrap_or_default().to_owned()
}

fn lines_from(output: &Output, prefix: &str) -> Vec<String> {
    stdout(output)
        .lines()
        .skip_while(|line| !line.starts_with(prefix))
        .take(2)
        .map(str::to_owned)
        .collect()
}

/// `lib/tangled.py` scores 2.31 at v2 and 1.21 at v1: a drop of 1.10.
fn tangled_gets_worse(sandbox: &Sandbox) {
    sandbox.write("lib/tangled.py", "v2/lib/tangled.py");
    sandbox.commit();
    sandbox.write("lib/tangled.py", "v1/lib/tangled.py");
}

/// A pushed commit that makes `lib/tangled.py` worse and adds `lib/other.py`.
fn push_that_declines(sandbox: &Sandbox) -> Oid {
    sandbox.write("lib/tangled.py", "v2/lib/tangled.py");
    let base = sandbox.commit();
    sandbox.mark_pushed(base);
    sandbox.write("lib/tangled.py", "v1/lib/tangled.py");
    sandbox.write("lib/other.py", "v2/lib/other.py");
    sandbox.commit()
}

/// A pushed commit that adds a file with no cached analysis.
fn push_that_cannot_be_scored(sandbox: &Sandbox) -> Oid {
    sandbox.write("lib/simple.py", "v1/lib/simple.py");
    let base = sandbox.commit();
    sandbox.mark_pushed(base);
    fs::write(
        sandbox.repo.join("lib/fresh.py"),
        "def fresh():\n    return 1\n",
    )
    .unwrap();
    sandbox.commit()
}

#[test]
fn upstream_declines_a_drop_beyond_the_maximum() {
    let sandbox = sandbox();
    tangled_gets_worse(&sandbox);
    let output = sandbox.upstream(&[]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        lines_from(&output, "DECLINED"),
        [
            "DECLINED lib/tangled.py  2.31 → 1.21/10 (-1.10; pass above 5.00; Python)",
            "  The score dropped by 1.10 points; the most allowed is 1.00.",
        ]
    );
}

#[test]
fn upstream_explains_what_changed() {
    let sandbox = sandbox();
    tangled_gets_worse(&sandbox);
    let output = sandbox.upstream(&[]);
    assert!(stdout(&output)
        .contains("Similar code (findings 0 → 1; max duplicated lines 18; lines 17–34, 37–54)"));
}

#[test]
fn upstream_opens_with_the_finding() {
    let sandbox = sandbox();
    tangled_gets_worse(&sandbox);
    let output = sandbox.upstream(&[]);
    assert_eq!(
        first_line(&output),
        "qlty slop-one found 1 changed file that declined in maintainability."
    );
}

#[test]
fn a_larger_max_drop_allows_the_same_change() {
    let sandbox = sandbox();
    tangled_gets_worse(&sandbox);
    let output = sandbox.upstream(&["--max-drop", "1.5"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        first_line(&output),
        "OK lib/tangled.py  2.31 → 1.21/10 (-1.10)"
    );
}

#[test]
fn an_improvement_is_ok() {
    let sandbox = sandbox();
    sandbox.write("lib/tangled.py", "v1/lib/tangled.py");
    sandbox.commit();
    sandbox.write("lib/tangled.py", "v2/lib/tangled.py");
    let output = sandbox.upstream(&[]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        first_line(&output),
        "OK lib/tangled.py  1.21 → 2.31/10 (+1.10)"
    );
}

#[test]
fn falling_below_the_cutoff_declines() {
    let sandbox = sandbox();
    sandbox.write("lib/module.py", "v1/lib/simple.py");
    sandbox.commit();
    sandbox.write("lib/module.py", "v1/lib/tangled.py");
    let output = sandbox.upstream(&[]);
    assert_eq!(
        lines_from(&output, "DECLINED")[1],
        "  The file passed before and now falls below the cutoff."
    );
}

#[test]
fn a_new_file_below_the_cutoff_declines() {
    let sandbox = sandbox();
    sandbox.write("lib/simple.py", "v1/lib/simple.py");
    sandbox.commit();
    sandbox.write("lib/tangled.py", "v1/lib/tangled.py");
    let output = sandbox.upstream(&[]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        lines_from(&output, "DECLINED"),
        [
            "DECLINED lib/tangled.py  1.21/10 (new file; pass above 5.00; Python)",
            "  New files must score above the cutoff.",
        ]
    );
}

#[test]
fn a_new_file_that_passes_is_ok() {
    let sandbox = sandbox();
    sandbox.write("lib/simple.py", "v1/lib/simple.py");
    sandbox.commit();
    sandbox.write("lib/other.py", "v2/lib/other.py");
    let output = sandbox.upstream(&[]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(first_line(&output), "OK lib/other.py  8.80/10 (new file)");
}

#[test]
fn tests_and_other_files_are_skipped() {
    let sandbox = sandbox();
    sandbox.write("lib/simple.py", "v1/lib/simple.py");
    sandbox.commit();
    sandbox.write("tests/test_simple.py", "v1/tests/test_simple.py");
    fs::write(sandbox.repo.join("README.md"), "# Notes\n").unwrap();
    let output = sandbox.upstream(&[]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        last_line(&output),
        "Compared 2 changed files with their earlier versions: 0 declined, 0 OK, 2 skipped, 0 errors. Largest allowed drop: 1.00 points."
    );
}

#[test]
fn json_reports_the_comparison() {
    let sandbox = sandbox();
    tangled_gets_worse(&sandbox);
    let output = sandbox.upstream(&["--json"]);
    let document: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        (
            &document["comparison"]["passed"],
            &document["comparison"]["files"][0]["decline"],
            &document["comparison"]["files"][0]["base"]["status"],
        ),
        (
            &Value::Bool(false),
            &Value::from("dropped-too-far"),
            &Value::from("scored")
        )
    );
}

#[test]
fn json_keeps_the_usual_results() {
    let sandbox = sandbox();
    tangled_gets_worse(&sandbox);
    let output = sandbox.upstream(&["--json"]);
    let document: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(document["results"][0]["path"], "lib/tangled.py");
}

#[test]
fn upstream_fails_when_a_file_cannot_be_scored() {
    let sandbox = sandbox();
    sandbox.write("lib/simple.py", "v1/lib/simple.py");
    sandbox.commit();
    fs::write(
        sandbox.repo.join("lib/fresh.py"),
        "def fresh():\n    return 1\n",
    )
    .unwrap();
    let output = sandbox.upstream(&[]);
    assert_eq!(output.status.code(), Some(99));
}

#[test]
fn comparison_options_need_a_comparison() {
    let sandbox = sandbox();
    sandbox.write("lib/simple.py", "v1/lib/simple.py");
    let output = sandbox
        .offline_command(&["--trigger", "pre-push", "lib/simple.py"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("apply with --upstream or --upstream-from-pre-push"));
}

#[test]
fn a_pre_push_hook_prints_the_full_prompt() {
    let sandbox = sandbox();
    let local = push_that_declines(&sandbox);
    let output = sandbox.pre_push(
        sandbox.offline_command(&["--trigger", "pre-push"]),
        &push_input(local),
    );
    assert_eq!(stdout(&output), EXPECTED_PROMPT);
}

#[test]
fn a_pre_push_hook_blocks_a_declined_file() {
    let sandbox = sandbox();
    let local = push_that_declines(&sandbox);
    let output = sandbox.pre_push(
        sandbox.offline_command(&["--trigger", "pre-push"]),
        &push_input(local),
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("Push blocked: 1 changed file declined."));
}

#[test]
fn the_refactor_prompt_asks_for_changes() {
    let sandbox = sandbox();
    let local = push_that_declines(&sandbox);
    let output = sandbox.pre_push(
        sandbox.offline_command(&["--trigger", "pre-push", "--prompt", "refactor"]),
        &push_input(local),
    );
    assert!(stdout(&output).contains(
        "Refactor the declined files below to improve maintainability for human software engineers."
    ));
}

#[test]
fn pushed_refs_without_the_trigger_are_not_framed_as_a_blocked_push() {
    let sandbox = sandbox();
    let local = push_that_declines(&sandbox);
    let output = sandbox.pre_push(sandbox.offline_command(&[]), &push_input(local));
    assert_eq!(
        first_line(&output),
        "qlty slop-one found 1 changed file that declined in maintainability."
    );
}

#[test]
fn pre_push_ignores_uncommitted_changes() {
    let sandbox = sandbox();
    sandbox.write("lib/tangled.py", "v1/lib/tangled.py");
    let base = sandbox.commit();
    sandbox.mark_pushed(base);
    sandbox.write("lib/tangled.py", "v2/lib/tangled.py");
    let local = sandbox.commit();
    sandbox.write("lib/tangled.py", "v1/lib/tangled.py");
    let output = sandbox.pre_push(
        sandbox.offline_command(&["--trigger", "pre-push"]),
        &push_input(local),
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        first_line(&output),
        "OK lib/tangled.py  1.21 → 2.31/10 (+1.10)"
    );
}

#[test]
fn a_pre_push_hook_lets_the_push_through_when_files_cannot_be_scored() {
    let sandbox = sandbox();
    let local = push_that_cannot_be_scored(&sandbox);
    let output = sandbox.pre_push(
        sandbox.offline_command(&["--trigger", "pre-push"]),
        &push_input(local),
    );
    assert_eq!(output.status.code(), Some(0));
    assert!(stderr(&output).contains("errors do not block the push"));
}

#[test]
fn pushed_refs_without_the_trigger_fail_when_files_cannot_be_scored() {
    let sandbox = sandbox();
    let local = push_that_cannot_be_scored(&sandbox);
    let output = sandbox.pre_push(sandbox.offline_command(&[]), &push_input(local));
    assert_eq!(output.status.code(), Some(99));
}

#[test]
fn a_pre_push_hook_lets_the_push_through_without_an_api_key() {
    let sandbox = sandbox();
    let local = push_that_cannot_be_scored(&sandbox);
    let mut command = sandbox.command(&[
        "--trigger",
        "pre-push",
        "--cache-dir",
        sandbox.cache.to_str().unwrap(),
    ]);
    command.env_remove("TYPESAFE_API_KEY");
    let output = sandbox.pre_push(command, &push_input(local));
    assert_eq!(output.status.code(), Some(0));
    assert!(stderr(&output).contains("SlopOne could not check this push"));
}

#[test]
fn pre_push_with_nothing_to_push_succeeds() {
    let sandbox = sandbox();
    sandbox.write("lib/simple.py", "v1/lib/simple.py");
    sandbox.commit();
    let output = sandbox.pre_push(sandbox.offline_command(&["--trigger", "pre-push"]), "");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_pre_push_hook_still_rejects_invalid_options() {
    let sandbox = sandbox();
    sandbox.write("lib/simple.py", "v1/lib/simple.py");
    let local = sandbox.commit();
    let output = sandbox.pre_push(
        sandbox.offline_command(&["--trigger", "pre-push", "--max-drop=-1"]),
        &push_input(local),
    );
    assert_eq!(output.status.code(), Some(1));
}
