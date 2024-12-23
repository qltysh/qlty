use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::Args;
use qlty_cloud::load_or_retrieve_auth_token;

#[derive(Args, Debug)]
pub struct Login {}

impl Login {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        load_or_retrieve_auth_token()?;
        CommandSuccess::ok()
    }
}
