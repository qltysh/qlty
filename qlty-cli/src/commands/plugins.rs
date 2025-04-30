use crate::{CommandError, CommandSuccess};
use anyhow::Result;
use clap::{Args, Subcommand};

mod disable;
mod enable;
mod list;
mod upgrade;

pub use disable::Disable;
pub use enable::Enable;
pub use list::List;
pub use upgrade::Upgrade;

#[derive(Debug, Args)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]

pub enum Commands {
    /// Enable plugins for the current project
    Enable(Enable),

    /// Disable plugins for the current project
    Disable(Disable),

    /// List all available plugins
    List(List),

    /// Upgrade a plugin for the current project
    Upgrade(Upgrade),
}

impl Arguments {
    pub fn execute(&self, args: &crate::Arguments) -> Result<CommandSuccess, CommandError> {
        match &self.command {
            Commands::Enable(command) => command.execute(args),
            Commands::Disable(command) => command.execute(args),
            Commands::List(command) => command.execute(args),
            Commands::Upgrade(command) => command.execute(args),
        }
    }
}
