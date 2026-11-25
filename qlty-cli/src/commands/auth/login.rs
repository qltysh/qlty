use crate::auth::{load_or_retrieve_auth_token, store_auth_token};
use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::{Context, Result};
use clap::Args;
use console::style;
use dialoguer::Input;

#[derive(Args, Debug)]
pub struct Login {
    /// Provide a CLI token manually or use "-" to read from standard input (see https://qlty.sh/user/settings/cli)
    #[arg(long)]
    pub token: Option<String>,
}

impl Login {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        if let Some(mut token) = self.token.clone() {
            if token.is_empty() || token == "-" {
                eprintln!(
                    "Generate a token from {} and paste it here.",
                    style("https://qlty.sh/user/settings/cli")
                        .underlined()
                        .green()
                );
                token = Input::<String>::new()
                    .with_prompt("Token")
                    .interact_text()
                    .with_context(|| "Invalid input")?;
            }

            store_auth_token(checked_token(&token)?)?;
            eprintln!("{}", style("Token saved successfully.").green());
            return CommandSuccess::ok();
        }
        load_or_retrieve_auth_token()?;
        CommandSuccess::ok()
    }
}

fn checked_token(token: &str) -> Result<&str, CommandError> {
    let token = token.trim();
    if !token.starts_with("qltyp_") || token.len() < 32 {
        return Err(CommandError::new("Token is invalid"));
    }
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_token_trims_line_endings() {
        assert_eq!(
            checked_token("qltyp_0123456789abcdefghijklmnop\r\n").unwrap(),
            "qltyp_0123456789abcdefghijklmnop"
        );
    }

    #[test]
    fn checked_token_rejects_short_token_padded_with_whitespace() {
        assert!(checked_token("qltyp_0123456789          \n").is_err());
    }
}
