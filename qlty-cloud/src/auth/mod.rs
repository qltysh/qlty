mod auth_flow;
mod credentials;

use crate::Client;
use anyhow::Result;
use auth_flow::{launch_login_server, AppState};
use console::style;
use credentials::read_token;
pub use credentials::{delete_token as clear_auth_token, write_token as store_auth_token};
use std::{env, thread, time::Duration};
use tracing::{info, warn};

const TOKEN_ENV_VAR: &str = "QLTY_TOKEN";

pub fn load_or_retrieve_auth_token() -> Result<String> {
    if let Ok(token) = env::var(TOKEN_ENV_VAR) {
        let token = token.trim().to_string();
        if !token.is_empty() {
            // bypass validation when env var is set since this is an intentional override of credential lookup
            return Ok(token);
        }
    }

    let mut has_token = false;
    let auth_token = match read_token() {
        Ok(token) => {
            has_token = true;
            Ok(token)
        }
        Err(_) => auth_via_browser(),
    }?;

    match validate_auth_token(&auth_token) {
        Ok(_) => Ok(auth_token),
        Err(err) => {
            if has_token {
                warn!("Failed to validate existing auth token, attempting to re-authenticate");
                load_or_retrieve_auth_token()
            } else {
                Err(err)
            }
        }
    }
}

fn validate_auth_token(auth_token: &String) -> Result<()> {
    Client::new(None, Some(auth_token.into()))
        .get("/user")
        .call()
        .inspect_err(|_| {
            if let Err(err) = clear_auth_token() {
                warn!("Failed to clear auth token: {}", err);
            }
        })?;

    Ok(())
}

fn auth_via_browser() -> Result<String> {
    let state = AppState::default();
    let original_state = &state.original_state;
    let server = launch_login_server(state.clone())?;
    info!("Auth login server started on port {}", server.base_url);

    eprintln!(
        "Launching {} in your browser. Once you've logged in, come back to the terminal.",
        style("http://qlty.sh/login").bold().green()
    );
    thread::sleep(Duration::from_millis(500));

    let open_url = ureq::get(&state.login_url)
        .query("state", original_state)
        .query("response_type", "token")
        .query("redirect_uri", &server.base_url)
        .request_url()?
        .as_url()
        .to_string();
    info!("Opening browser to {}", open_url);
    webbrowser::open(&open_url)?;

    loop {
        if let Result::Ok(value) = read_token() {
            eprintln!("Login successful! Your credentials have been stored for future use.");
            return Ok(value);
        }
        thread::sleep(Duration::from_secs(1));
    }
}
