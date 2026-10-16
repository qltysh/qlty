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
