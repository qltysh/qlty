mod publish;
mod transform;
mod validate;
pub use publish::Publish;
pub use transform::Transform;
pub use validate::Validate;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::{CommandError, CommandSuccess};

#[derive(Debug, Args)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Upload coverage reports to the Quality Cloud
    Publish(Publish),
    /// Transforms third party coverage reports to the Qlty format
    Transform(Transform),
    /// Validates coverage reports files exist on the filesystem
    Validate(Validate),
}

impl Arguments {
    pub fn execute(&self, args: &crate::Arguments) -> Result<CommandSuccess, CommandError> {
        match &self.command {
            Commands::Transform(command) => command.execute(args),
            Commands::Publish(command) => command.execute(args),
            Commands::Validate(command) => command.execute(args),
        }
    }
}
