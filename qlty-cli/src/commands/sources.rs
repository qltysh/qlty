use crate::{CommandError, CommandSuccess};
use anyhow::Result;
use clap::{Args, Subcommand};

mod fetch;

pub use fetch::Fetch;

#[derive(Debug, Args)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Fetch plugin sources
    Fetch(Fetch),
}

impl Arguments {
    pub fn execute(&self, args: &crate::Arguments) -> Result<CommandSuccess, CommandError> {
        match &self.command {
            Commands::Fetch(command) => command.execute(args),
        }
    }
}
