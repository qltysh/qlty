use crate::auth::clear_auth_token;
use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct Logout {}

impl Logout {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        clear_auth_token()?;
        CommandSuccess::ok()
    }
}
