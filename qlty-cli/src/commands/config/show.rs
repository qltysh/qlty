use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::Args;
use qlty_config::Workspace;

#[derive(Args, Debug, Clone)]
pub struct Show {}

impl Show {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let workspace = Workspace::require_initialized()?;
        let config = workspace.load_config(false)?;
        let yaml_string = serde_yaml::to_string(&config).unwrap();
        println!("{}", yaml_string);
        CommandSuccess::ok()
    }
}
