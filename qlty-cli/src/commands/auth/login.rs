use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::Args;
use dialoguer::Input;
use qlty_cloud::Client;

#[derive(Args, Debug)]
pub struct Login {}

impl Login {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let access_token: String = Input::new()
            .with_prompt("Access token")
            .interact_text()
            .unwrap();

        let client = Client {
            base_url: "https://api.qlty.sh".to_string(),
            token: Some(access_token.clone()),
        };

        match client.get("/user").call() {
            Ok(_) => {
                let token = qlty_cloud::Token::default();
                token.set(&access_token)?;
                CommandSuccess::ok()
            }
            Err(e) => {
                CommandError::err(&format!("Failed to authenticate: {}", e))
            }
        }
    }
}
