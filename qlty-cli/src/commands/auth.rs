use crate::{CommandError, CommandSuccess};
use anyhow::Result;
use clap::{Args, Subcommand};

mod login;
mod logout;
mod signup;
mod whoami;

pub use login::Login;
pub use logout::Logout;
pub use signup::Signup;
pub use whoami::Whoami;

#[derive(Debug, Args)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]

pub enum Commands {
    /// Log in a user
    Login(Login),

    /// Log out the currently logged in user
    Logout(Logout),

    /// Open the Qlty Cloud sign up flow in the browser
    Signup(Signup),

    ///  Print the email address of the authenticated Qlty Cloud user
    Whoami(Whoami),
}

impl Arguments {
    pub fn execute(&self, args: &crate::Arguments) -> Result<CommandSuccess, CommandError> {
        match &self.command {
            Commands::Login(command) => command.execute(args),
            Commands::Logout(command) => command.execute(args),
            Commands::Signup(command) => command.execute(args),
            Commands::Whoami(command) => command.execute(args),
        }
    }
}
