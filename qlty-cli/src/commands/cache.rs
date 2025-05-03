use crate::{CommandError, CommandSuccess};
use anyhow::Result;
use clap::{Args, Subcommand};

mod clean;
mod dir;
mod prune;
mod status;

pub use clean::Clean;
pub use dir::Dir;
pub use prune::Prune;
pub use status::Status;

#[derive(Debug, Args)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]

pub enum Commands {
    /// Prune the cache
    Prune(Prune),

    /// Delete the cache for the current project
    Clean(Clean),

    /// Print the cache directory for the current project
    Dir(Dir),

    /// Print the status of the cache directory for the current project
    Status(Status),
}

impl Arguments {
    pub fn execute(&self, args: &crate::Arguments) -> Result<CommandSuccess, CommandError> {
        match &self.command {
            Commands::Clean(command) => command.execute(args),
            Commands::Dir(command) => command.execute(args),
            Commands::Prune(command) => command.execute(args),
            Commands::Status(command) => command.execute(args),
        }
    }
}
