use crate::docs_export::export_command_tree;
use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::{Args, CommandFactory};

#[derive(Args, Debug)]
pub struct CliReference {}

impl CliReference {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let export = export_command_tree(Arguments::command());
        println!("{}", serde_json::to_string_pretty(&export)?);
        CommandSuccess::ok()
    }
}
