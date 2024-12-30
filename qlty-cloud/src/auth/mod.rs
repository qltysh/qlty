mod auth_flow;
mod token;

use crate::Client;
use anyhow::Result;
use auth_flow::{launch_login_server, AppState};
use console::style;
use std::{thread, time::Duration};
use token::Token;
use tracing::{info, warn};

pub fn load_or_retrieve_auth_token() -> Result<String> {
    let mut has_token = false;
    let auth_token = match Token::default().get() {
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

pub fn clear_auth_token() -> Result<()> {
    Token::default().delete()
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
    let user = &state.credential_user;
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

    let token = Token::new(user);
    loop {
        if let Result::Ok(value) = token.get() {
            eprintln!("Login successful! Your credentials have been stored for future use.");
            return Ok(value);
        }
        thread::sleep(Duration::from_secs(1));
    }
}
