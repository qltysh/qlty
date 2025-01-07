use std::io::stdin;

use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::Args;
use console::style;
use qlty_cloud::{load_or_retrieve_auth_token, store_auth_token};

#[derive(Args, Debug)]
pub struct Login {
    /// Provide a CLI token manually or use "-" to read from standard input (see https://qlty.sh/user/settings/cli)
    #[arg(long)]
    pub token: Option<String>,
}

impl Login {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        if let Some(mut token) = self.token.clone() {
            if token == "" || token == "-" {
                eprint!(
                    "Generate a token from {} and paste it here.\nToken: ",
                    style("https://qlty.sh/user/settings/cli")
                        .underlined()
                        .green()
                );
                let line = &mut String::new();
                stdin().read_line(line)?;
                token = line.trim().to_string();
            }

            if !token.starts_with("qltyp_") || token.len() < 32 {
                return Err(CommandError::new("Token is invalid"));
            }

            store_auth_token(&token)?;
            eprintln!("{}", style("Token saved successfully.").green());
            return CommandSuccess::ok();
        }
        load_or_retrieve_auth_token()?;
        CommandSuccess::ok()
    }
}
