use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::Args;
use qlty_config::Workspace;

#[derive(Args, Debug, Clone)]
pub struct Fetch {}

impl Fetch {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let workspace = Workspace::require_initialized()?;
        workspace.fetch_sources()?;
        CommandSuccess::ok()
    }
}
