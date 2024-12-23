use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::Args;
use qlty_cloud::clear_auth_token;

#[derive(Args, Debug)]
pub struct Logout {}

impl Logout {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        clear_auth_token()?;
        CommandSuccess::ok()
    }
}
