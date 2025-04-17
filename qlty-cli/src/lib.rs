pub mod allocator;
mod arguments;
pub mod auth;
mod commands;
mod errors;
pub mod export;
pub mod format;
mod initializer;
mod logging;
pub mod shell;
mod success;
mod telemetry;
mod ui;
mod upgrade;

pub use arguments::Arguments;
pub use auth::{clear_auth_token, load_or_retrieve_auth_token, store_auth_token};
pub use errors::CommandError;
pub use initializer::Initializer;
pub use success::{CommandSuccess, Trigger};
pub use telemetry::Telemetry;
pub use upgrade::QltyRelease;

use crate::logging::init_logs;
use clap::Parser;
use console::style;
use qlty_config::Workspace;
use std::panic;
use std::time::Instant;
use tracing::{debug, error, info, trace};

pub fn run_command_line() {
    let start_time = Instant::now();
    let current = std::env::current_dir().expect("current dir");
    let repository_path = Workspace::closest_git_repository_path(&current);

    // WARNING: Do NOT refactor this logging code! It is critical that the _guards variable
    // be loaded into the main() function scope, otherwise the logs will not be flushed to disk.
    let _guards = init_logs(repository_path.clone());
    trace!("Repository path: {:?}", repository_path);

    let command = std::env::args().collect::<Vec<String>>().join(" ");
    info!("Executing command: {}", command);

    let arguments = Arguments::parse();
    debug!("Arguments: {:?}", arguments);

    let telemetry = Telemetry::new(&command, start_time, repository_path.clone());
    setup_panic_hook(telemetry.clone());

    let result = arguments.execute();
    debug!("Command result: {:?}", result);
    handle_result(&telemetry, &command, result);
}

fn handle_result(
    telemetry: &Telemetry,
    command: &str,
    result: Result<CommandSuccess, CommandError>,
) {
    match result {
        Ok(command_success) => {
            info!(
                "Command executed successfully in {:.2}s: {}",
                telemetry.start_time.elapsed().as_secs_f32(),
                command,
            );
            telemetry.track_command_success(&command_success).ok();

            if let Some(count) = command_success.issues_count {
                let fixed_count = command_success.fixed_count;
                if count == 0 {
                    eprintln!("{}", style("✔ No issues").green().bold());
                } else if fixed_count > 0 {
                    let remaining = count - fixed_count;

                    if remaining > 0 {
                        eprintln!(
                            "{}",
                            style(format!(
                                "✖ {}/{} fixed {}",
                                fixed_count,
                                count,
                                if count == 1 { "issue" } else { "issues" }
                            ))
                            .red()
                            .bold()
                        );
                    } else {
                        eprintln!(
                            "{}",
                            style(format!(
                                "✔ {} fixed {}",
                                count,
                                if count == 1 { "issue" } else { "issues" }
                            ))
                            .green()
                            .bold()
                        );
                    }
                } else {
                    eprintln!(
                        "{}",
                        style(format!(
                            "✖ {} {}",
                            count,
                            if count == 1 { "issue" } else { "issues" }
                        ))
                        .red()
                        .bold()
                    );
                }

                if let Some(count) = command_success.security_issues_count {
                    if count > 0 {
                        eprintln!(
                            "{}",
                            style(format!(
                                "✖ {} security {}",
                                count,
                                if count == 1 { "issue" } else { "issues" }
                            ))
                            .red()
                            .bold()
                        );
                    }
                }

                if let Some(count) = command_success.unformatted_count {
                    if count > 0 {
                        eprintln!(
                            "{}",
                            style(format!(
                                "✖ {} unformatted {}",
                                count,
                                if count == 1 { "file" } else { "files" }
                            ))
                            .red()
                            .bold()
                        );
                    }
                }
            }

            std::process::exit(command_success.exit_code());
        }
        Err(command_error) => {
            match command_error {
                CommandError::InvalidOptions { ref message } => {
                    error!("Invalid options: {}", message);
                    eprintln!("{} {}", style("error:").red().bold(), message);
                    eprintln!();
                    eprintln!("For more information, try {}.", style("'--help'").bold());
                }
                CommandError::Config => {
                    error!("Config error");
                    eprintln!("❌ Config error");
                }
                CommandError::Lint => {
                    error!("Lint error");
                    eprintln!("❌ Lint error");
                }
                CommandError::Unknown { ref source } => {
                    error!("Command failed: {}", command);
                    error!("{:?}", source);
                    eprintln!();
                    eprintln!("{}", style("   ERROR   ").red().bold().reverse());
                    eprintln!();

                    let error_message = format!("{:?}", source);
                    let error_message = error_message
                        .lines()
                        .map(|line| format!(" {} {}", style(">").red().bold(), line))
                        .collect::<Vec<_>>()
                        .join("\n");
                    eprintln!("{}", error_message);
                }
            }

            telemetry.track_command_error(&command_error).ok();
            std::process::exit(command_error.exit_code());
        }
    }
}

fn setup_panic_hook(telemetry: Telemetry) {
    trace!("Setting up panic hook");
    let telemetry = telemetry.clone();

    let next = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        telemetry.panic(info).ok();
        next(info);
    }));
}
