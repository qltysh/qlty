use std::fs;
use std::path::Path;
use std::process::Command;

fn sh(cwd: &Path, program: &str, args: &[&str]) {
    let status = Command::new(program)
        .current_dir(cwd)
        .args(args)
        .status()
        .unwrap_or_else(|e| panic!("failed to spawn {program}: {e}"));
    assert!(
        status.success(),
        "{program} {args:?} failed in {}",
        cwd.display()
    );
}

// With a deprecated repository-style default source in qlty.toml,
// `qlty coverage publish` should surface the warning exactly once. The
// message arrives through load_config_for's fallback (which always warns to
// log and screen when the config can't be loaded — covering fetch failures,
// parse errors, and deprecated-source bails uniformly), and neither the
// publish flow nor print_deprecation_warnings should emit a duplicate copy.
#[test]
fn coverage_publish_shows_deprecation_warning_once() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();

    let qlty_dir = root.join(".qlty");
    fs::create_dir_all(&qlty_dir).unwrap();
    fs::write(
        qlty_dir.join("qlty.toml"),
        r#"config_version = "0"

[[source]]
name = "default"
repository = "https://github.com/qltysh/qlty"
tag = "v0.0.0"
"#,
    )
    .unwrap();

    let cached_ref = qlty_dir
        .join("sources")
        .join("https---github-com-qltysh-qlty")
        .join("v0.0.0");
    fs::create_dir_all(&cached_ref).unwrap();

    fs::write(
        root.join("lcov.info"),
        "TN:\nSF:foo.rs\nDA:1,1\nLF:1\nLH:1\nend_of_record\n",
    )
    .unwrap();

    sh(root, "git", &["init", "--quiet", "--initial-branch=main"]);
    sh(root, "git", &["config", "user.email", "test@example.com"]);
    sh(root, "git", &["config", "user.name", "Test"]);
    sh(root, "git", &["add", "."]);
    sh(
        root,
        "git",
        &["commit", "--no-gpg-sign", "-q", "-m", "init"],
    );

    let qlty = env!("CARGO_BIN_EXE_qlty");
    let output = Command::new(qlty)
        .current_dir(root)
        .env("QLTY_COVERAGE_TOKEN", "123")
        .env_remove("GITHUB_ACTIONS")
        .args([
            "coverage",
            "publish",
            "--skip-source-fetch",
            "--dry-run",
            "--no-validate",
            "--override-commit-sha",
            "2ca1bc45a94e37c8dbae6fd9e19fc069ba64bd67",
            "--override-build-id",
            "123",
            "--override-branch",
            "main",
            "lcov.info",
        ])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    let deprecated_phrase_hits = stderr
        .matches("deprecated, repository-based, default source")
        .count();
    let fallback_header_hits = stderr.matches("Failed to load qlty config").count();

    assert_eq!(
        deprecated_phrase_hits, 1,
        "expected the deprecation message to appear exactly once in stderr\n\nstderr was:\n{stderr}"
    );
    assert_eq!(
        fallback_header_hits, 1,
        "expected the 'Failed to load qlty config' fallback header to appear exactly once\n\nstderr was:\n{stderr}"
    );
}
