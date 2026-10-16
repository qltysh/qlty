use serde_json::Value;
use std::process::{Command, Output};

const PUBLIC_COMMANDS: &[&str] = &[
    "cache",
    "cache clean",
    "cache dir",
    "cache prune",
    "cache status",
    "check",
    "completions",
    "config",
    "config migrate",
    "config show",
    "config validate",
    "coverage",
    "coverage complete",
    "coverage publish",
    "coverage transform",
    "dashboard",
    "deinit",
    "discord",
    "docs",
    "fmt",
    "help",
    "init",
    "install",
    "metrics",
    "plugins",
    "plugins disable",
    "plugins enable",
    "plugins list",
    "plugins upgrade",
    "slop-one",
    "smells",
    "sources",
    "sources fetch",
    "upgrade",
    "upgrade source",
    "version",
];

const HIDDEN_COMMANDS: &[&str] = &[
    "auth",
    "build",
    "cli-reference",
    "githooks",
    "panic",
    "parse",
    "patch",
    "slop-one trends",
    "telemetry",
    "validate",
];

fn run_cli_reference() -> Output {
    Command::new(env!("CARGO_BIN_EXE_qlty"))
        .env("QLTY_TELEMETRY", "off")
        .arg("cli-reference")
        .output()
        .unwrap()
}

fn exported_reference() -> Value {
    serde_json::from_slice(&run_cli_reference().stdout).unwrap()
}

fn command_paths(commands: &Value) -> Vec<String> {
    commands
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|command| {
            let path = command["path"]
                .as_array()
                .unwrap()
                .iter()
                .map(|segment| segment.as_str().unwrap())
                .collect::<Vec<_>>()
                .join(" ");
            std::iter::once(path).chain(command_paths(&command["subcommands"]))
        })
        .collect()
}

fn exported_command_paths() -> Vec<String> {
    let mut paths = command_paths(&exported_reference()["commands"]);
    paths.sort();
    paths
}

#[test]
fn cli_reference_succeeds() {
    assert!(run_cli_reference().status.success());
}

#[test]
fn cli_reference_prints_json() {
    assert!(serde_json::from_slice::<Value>(&run_cli_reference().stdout).is_ok());
}

#[test]
fn cli_reference_exports_every_public_command() {
    assert_eq!(exported_command_paths(), PUBLIC_COMMANDS);
}

#[test]
fn cli_reference_omits_hidden_commands() {
    let paths = exported_command_paths();
    assert!(!HIDDEN_COMMANDS
        .iter()
        .any(|hidden| paths.contains(&hidden.to_string())));
}
