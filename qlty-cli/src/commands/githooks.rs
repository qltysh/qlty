use crate::{CommandError, CommandSuccess};
use anyhow::Result;
use clap::{Args, Subcommand};

mod install;
mod uninstall;

pub use install::Install;
pub use uninstall::Uninstall;

#[derive(Debug, Args)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]

pub enum Commands {
    /// Install the git hooks
    Install(Install),

    /// Uninstall the git hooks
    Uninstall(Uninstall),
}

impl Arguments {
    pub fn execute(&self, args: &crate::Arguments) -> Result<CommandSuccess, CommandError> {
        match &self.command {
            Commands::Install(command) => command.execute(args),
            Commands::Uninstall(command) => command.execute(args),
        }
    }
}
