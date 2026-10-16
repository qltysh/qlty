use clap::{Arg, ArgAction, Command};
use qlty_config::version::{GIT_COMMIT_QLTY, QLTY_VERSION};
use serde_json::{json, Value};

const SCHEMA_VERSION: u32 = 1;
const BIN_NAME: &str = "qlty";
const HELP_SUBCOMMAND: &str = "help";

pub fn export_command_tree(command: Command) -> Value {
    let help_usage = render_help_subcommand_usage(&command);
    let mut root = command.bin_name(BIN_NAME).term_width(0);
    root.build();

    let globals: Vec<Value> = root
        .get_arguments()
        .filter(|arg| !arg.is_hide_set())
        .filter(|arg| arg.is_global_set() || matches!(arg.get_action(), ArgAction::Help))
        .map(export_arg)
        .collect();

    let commands: Vec<Value> = root
        .get_subcommands()
        .filter(|subcommand| !subcommand.is_hide_set())
        .map(|subcommand| {
            if subcommand.get_name() == HELP_SUBCOMMAND {
                export_help_command(subcommand, &help_usage)
            } else {
                export_command(subcommand, &[])
            }
        })
        .collect();

    json!({
        "schema_version": SCHEMA_VERSION,
        "cli_version": QLTY_VERSION,
        "cli_commit": GIT_COMMIT_QLTY,
        "globals": globals,
        "commands": commands,
    })
}

fn export_command(command: &Command, parent_path: &[String]) -> Value {
    let mut path = parent_path.to_vec();
    path.push(command.get_name().to_owned());

    let args: Vec<Value> = command
        .get_arguments()
        .filter(|arg| !arg.is_hide_set() && !arg.is_global_set() && !is_auto_flag(arg))
        .map(export_arg)
        .collect();

    let subcommands: Vec<Value> = command
        .get_subcommands()
        .filter(|subcommand| !subcommand.is_hide_set() && subcommand.get_name() != HELP_SUBCOMMAND)
        .map(|subcommand| export_command(subcommand, &path))
        .collect();

    json!({
        "name": command.get_name(),
        "path": path,
        "about": command.get_about().map(ToString::to_string),
        "long_about": command.get_long_about().map(ToString::to_string),
        "usage": render_usage(command),
        "runnable": !command.is_subcommand_required_set(),
        "args": args,
        "subcommands": subcommands,
    })
}

fn export_help_command(command: &Command, usage: &str) -> Value {
    json!({
        "name": command.get_name(),
        "path": [command.get_name()],
        "about": command.get_about().map(ToString::to_string),
        "long_about": command.get_long_about().map(ToString::to_string),
        "usage": usage,
        "runnable": true,
        "args": [],
        "subcommands": [],
    })
}

fn export_arg(arg: &Arg) -> Value {
    let takes_values = arg.get_action().takes_values();

    let default_values: Vec<String> = if takes_values {
        arg.get_default_values()
            .iter()
            .map(|value| value.to_string_lossy().into_owned())
            .collect()
    } else {
        vec![]
    };

    let possible_values: Vec<String> = if takes_values {
        arg.get_possible_values()
            .iter()
            .filter(|value| !value.is_hide_set())
            .map(|value| value.get_name().to_owned())
            .collect()
    } else {
        vec![]
    };

    let value_names: Vec<String> = if takes_values {
        arg.get_value_names()
            .unwrap_or_default()
            .iter()
            .map(ToString::to_string)
            .collect()
    } else {
        vec![]
    };

    json!({
        "id": arg.get_id().as_str(),
        "display": render_display(arg),
        "short": arg.get_short().map(String::from),
        "long": arg.get_long(),
        "value_names": value_names,
        "positional": arg.is_positional(),
        "required": arg.is_required_set(),
        "help": arg.get_help().map(ToString::to_string),
        "long_help": arg.get_long_help().map(ToString::to_string),
        "help_heading": arg.get_help_heading(),
        "default_values": default_values,
        "possible_values": possible_values,
        "env": arg.get_env().map(|env| env.to_string_lossy().into_owned()),
    })
}

fn is_auto_flag(arg: &Arg) -> bool {
    matches!(
        arg.get_action(),
        ArgAction::Help | ArgAction::HelpShort | ArgAction::HelpLong | ArgAction::Version
    )
}

fn render_display(arg: &Arg) -> String {
    match (arg.get_short(), arg.get_long()) {
        (Some(short), Some(_)) => format!("-{short}, {arg}"),
        _ => arg.to_string(),
    }
}

fn render_usage(command: &Command) -> String {
    let usage = command.clone().render_usage().to_string();
    usage.strip_prefix("Usage: ").unwrap_or(&usage).to_owned()
}

// Command::build() expands the help subcommand into a copy of the whole tree, which
// changes its usage. The parser builds it shallowly, so render from a shallow build.
fn render_help_subcommand_usage(command: &Command) -> String {
    let mut shallow = command.clone().bin_name(BIN_NAME).term_width(0);
    shallow.render_usage();
    shallow
        .find_subcommand(HELP_SUBCOMMAND)
        .map(|help| {
            render_usage(
                &help
                    .clone()
                    .bin_name(format!("{BIN_NAME} {HELP_SUBCOMMAND}")),
            )
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Arguments;
    use clap::CommandFactory;

    fn export() -> Value {
        export_command_tree(Arguments::command())
    }

    fn find_command<'a>(export: &'a Value, path: &[&str]) -> &'a Value {
        path.iter()
            .fold(None, |current: Option<&'a Value>, name| {
                let list = match current {
                    None => &export["commands"],
                    Some(command) => &command["subcommands"],
                };
                list.as_array()
                    .unwrap()
                    .iter()
                    .find(|command| command["name"] == *name)
            })
            .unwrap()
    }

    fn find_arg<'a>(command: &'a Value, id: &str) -> &'a Value {
        command["args"]
            .as_array()
            .unwrap()
            .iter()
            .find(|arg| arg["id"] == id)
            .unwrap()
    }

    fn command_names(list: &Value) -> Vec<String> {
        list.as_array()
            .unwrap()
            .iter()
            .map(|command| command["name"].as_str().unwrap().to_owned())
            .collect()
    }

    fn arg_ids(list: &Value) -> Vec<String> {
        list.as_array()
            .unwrap()
            .iter()
            .map(|arg| arg["id"].as_str().unwrap().to_owned())
            .collect()
    }

    fn all_command_arg_ids(list: &Value) -> Vec<String> {
        list.as_array()
            .unwrap()
            .iter()
            .flat_map(|command| {
                let mut ids = arg_ids(&command["args"]);
                ids.extend(all_command_arg_ids(&command["subcommands"]));
                ids
            })
            .collect()
    }

    #[test]
    fn test_full_export_snapshot() {
        insta::assert_json_snapshot!(export(), {
            ".cli_version" => "[version]",
            ".cli_commit" => "[commit]",
        });
    }

    #[test]
    fn test_hidden_command_absent() {
        assert!(!command_names(&export()["commands"]).contains(&"auth".to_owned()));
    }

    #[test]
    fn test_hidden_arg_absent() {
        let export = export();
        let check = find_command(&export, &["check"]);
        assert!(!arg_ids(&check["args"]).contains(&"trigger".to_owned()));
    }

    #[test]
    fn test_globals() {
        assert_eq!(
            arg_ids(&export()["globals"]),
            vec!["debug", "no_upgrade_check", "help"]
        );
    }

    #[test]
    fn test_global_help_display() {
        assert_eq!(export()["globals"][2]["display"], "-h, --help");
    }

    #[test]
    fn test_globals_absent_from_subcommands() {
        let ids = all_command_arg_ids(&export()["commands"]);
        assert!(!ids
            .iter()
            .any(|id| ["debug", "no_upgrade_check", "help"].contains(&id.as_str())));
    }

    #[test]
    fn test_version_flag_absent() {
        assert!(!arg_ids(&export()["globals"]).contains(&"version".to_owned()));
    }

    #[test]
    fn test_help_command_is_leaf() {
        let export = export();
        let help = find_command(&export, &["help"]);
        assert_eq!(
            (&help["runnable"], &help["args"], &help["subcommands"]),
            (&json!(true), &json!([]), &json!([]))
        );
    }

    #[test]
    fn test_help_subcommand_absent_from_nested_commands() {
        let export = export();
        let cache = find_command(&export, &["cache"]);
        assert!(!command_names(&cache["subcommands"]).contains(&"help".to_owned()));
    }

    #[test]
    fn test_help_command_usage() {
        let export = export();
        assert_eq!(
            find_command(&export, &["help"])["usage"],
            "qlty help [COMMAND]..."
        );
    }

    #[test]
    fn test_switch_has_no_value_names() {
        let export = export();
        let all = find_arg(find_command(&export, &["check"]), "all");
        assert_eq!(all["value_names"], json!([]));
    }

    #[test]
    fn test_upgrade_version_option() {
        let export = export();
        let upgrade = find_command(&export, &["upgrade"]);
        assert_eq!(
            find_arg(upgrade, "version")["display"],
            "--version <VERSION>"
        );
    }

    #[test]
    fn test_plugins_upgrade_version_option() {
        let export = export();
        let plugins_upgrade = find_command(&export, &["plugins", "upgrade"]);
        assert_eq!(
            find_arg(plugins_upgrade, "version")["display"],
            "--version <VERSION>"
        );
    }

    #[test]
    fn test_switch_has_no_default_or_possible_values() {
        let export = export();
        let all = find_arg(find_command(&export, &["check"]), "all");
        assert_eq!(
            (&all["default_values"], &all["possible_values"]),
            (&json!([]), &json!([]))
        );
    }

    #[test]
    fn test_switch_display() {
        let export = export();
        let all = find_arg(find_command(&export, &["check"]), "all");
        assert_eq!(all["display"], "-a, --all");
    }

    #[test]
    fn test_count_flag_has_no_default() {
        let export = export();
        let verbose = find_arg(find_command(&export, &["check"]), "verbose");
        assert_eq!(verbose["default_values"], json!([]));
    }

    #[test]
    fn test_count_flag_display() {
        let export = export();
        let verbose = find_arg(find_command(&export, &["check"]), "verbose");
        assert_eq!(verbose["display"], "-v, --verbose...");
    }

    #[test]
    fn test_positional_display() {
        let export = export();
        let paths = find_arg(find_command(&export, &["check"]), "paths");
        assert_eq!(paths["display"], "[PATHS]...");
    }

    #[test]
    fn test_level_default_value() {
        let export = export();
        let level = find_arg(find_command(&export, &["check"]), "level");
        assert_eq!(level["default_values"], json!(["note"]));
    }

    #[test]
    fn test_level_possible_values() {
        let export = export();
        let level = find_arg(find_command(&export, &["check"]), "level");
        assert_eq!(
            level["possible_values"],
            json!(["note", "fmt", "low", "medium", "high"])
        );
    }

    #[test]
    fn test_level_display() {
        let export = export();
        let level = find_arg(find_command(&export, &["check"]), "level");
        assert_eq!(level["display"], "--level <LEVEL>");
    }

    #[test]
    fn test_env_var() {
        let export = export();
        let verify = find_arg(find_command(&export, &["upgrade"]), "verify_attestations");
        assert_eq!(verify["env"], "QLTY_VERIFY_ATTESTATIONS");
    }

    #[test]
    fn test_help_heading() {
        let export = export();
        let since = find_arg(find_command(&export, &["slop-one"]), "since");
        assert_eq!(since["help_heading"], "Report options");
    }

    #[test]
    fn test_comparison_help_heading() {
        let export = export();
        let upstream = find_arg(find_command(&export, &["slop-one"]), "upstream");
        assert_eq!(upstream["help_heading"], "Comparison options");
    }

    #[test]
    fn test_upgrade_runnable() {
        let export = export();
        assert_eq!(find_command(&export, &["upgrade"])["runnable"], true);
    }

    #[test]
    fn test_upgrade_subcommands() {
        let export = export();
        let upgrade = find_command(&export, &["upgrade"]);
        assert_eq!(command_names(&upgrade["subcommands"]), vec!["source"]);
    }

    #[test]
    fn test_coverage_not_runnable() {
        let export = export();
        assert_eq!(find_command(&export, &["coverage"])["runnable"], false);
    }

    #[test]
    fn test_nested_usage() {
        let export = export();
        let dir = find_command(&export, &["cache", "dir"]);
        assert!(dir["usage"].as_str().unwrap().starts_with("qlty cache dir"));
    }

    #[test]
    fn test_nested_path() {
        let export = export();
        assert_eq!(
            find_command(&export, &["cache", "dir"])["path"],
            json!(["cache", "dir"])
        );
    }

    #[test]
    fn test_deterministic() {
        assert_eq!(export(), export());
    }
}
