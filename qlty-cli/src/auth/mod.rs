mod auth_flow;
mod credentials;
use anyhow::{Context, Error, Result};
use auth_flow::{launch_login_server, AppState};
use console::style;
use credentials::read_token;
pub use credentials::{delete_token as clear_auth_token, write_token as store_auth_token};
use qlty_cloud::InvalidTokenCharacterError;
use qlty_config::http;
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

    resolve_auth_token(
        || read_token().map(|token| token.trim().to_string()),
        clear_auth_token,
        validate_auth_token,
        || auth_via_browser().map(|token| token.trim().to_string()),
    )
}

fn resolve_auth_token(
    read_token: impl Fn() -> Result<String>,
    clear_token: impl Fn() -> Result<()>,
    validate_token: impl Fn(&str) -> Result<()>,
    login: impl Fn() -> Result<String>,
) -> Result<String> {
    if let Ok(token) = read_token() {
        match validate_token(&token) {
            Ok(()) => return Ok(token),
            Err(err) if is_rejected_token(&err) => {
                warn!("Saved auth token was rejected, attempting to re-authenticate");
                clear_token().context("Failed to clear the rejected auth token")?;
            }
            Err(err) => return Err(err),
        }
    }

    let token = login()?;
    if let Err(err) = validate_token(&token) {
        if is_rejected_token(&err) {
            if let Err(clear_err) = clear_token() {
                warn!("Failed to clear auth token: {}", clear_err);
            }
        }
        return Err(err);
    }
    Ok(token)
}

fn validate_auth_token(auth_token: &str) -> Result<()> {
    let client = qlty_cloud::Client::new(None, Some(auth_token.into()));
    client.get("/user")?.call()?;
    Ok(())
}

fn is_rejected_token(err: &Error) -> bool {
    err.is::<InvalidTokenCharacterError>()
        || matches!(
            err.downcast_ref::<ureq::Error>(),
            Some(ureq::Error::Status(401, _))
        )
}

fn auth_via_browser() -> Result<String> {
    let state = AppState::default();
    let original_state = &state.original_state;
    let server = launch_login_server(state.clone())?;
    info!("Auth login server started on port {}", server.base_url);

    eprintln!(
        "Launching {} in your browser. Once you've logged in, come back to the terminal.",
        style("https://qlty.sh/login").bold().green()
    );
    thread::sleep(Duration::from_millis(500));

    let open_url = http::get(&state.login_url)?
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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::bail;
    use std::cell::Cell;

    fn unauthorized() -> Error {
        ureq::Error::Status(401, ureq::Response::new(401, "Unauthorized", "").unwrap()).into()
    }

    #[test]
    fn stops_when_clearing_rejected_token_fails() {
        let logins = Cell::new(0);
        let result = resolve_auth_token(
            || Ok("qltyp_saved".to_string()),
            || bail!("delete denied"),
            |_| Err(unauthorized()),
            || {
                logins.set(logins.get() + 1);
                Ok("qltyp_new".to_string())
            },
        );
        assert!(result.is_err());
        assert_eq!(logins.get(), 0);
    }

    #[test]
    fn keeps_saved_token_when_error_is_not_caused_by_token() {
        let clears = Cell::new(0);
        let result = resolve_auth_token(
            || Ok("qltyp_saved".to_string()),
            || {
                clears.set(clears.get() + 1);
                Ok(())
            },
            |_| bail!("relative URL without a base"),
            || Ok("qltyp_new".to_string()),
        );
        assert!(result.is_err());
        assert_eq!(clears.get(), 0);
    }

    #[test]
    fn replaces_rejected_saved_token_with_new_login() {
        let clears = Cell::new(0);
        let result = resolve_auth_token(
            || Ok("qltyp_saved".to_string()),
            || {
                clears.set(clears.get() + 1);
                Ok(())
            },
            |token| {
                (token != "qltyp_saved")
                    .then_some(())
                    .ok_or_else(unauthorized)
            },
            || Ok("qltyp_new".to_string()),
        );
        assert_eq!(result.unwrap(), "qltyp_new");
        assert_eq!(clears.get(), 1);
    }

    #[test]
    fn clears_rejected_new_login() {
        let clears = Cell::new(0);
        let _ = resolve_auth_token(
            || bail!("no saved token"),
            || {
                clears.set(clears.get() + 1);
                Ok(())
            },
            |_| Err(unauthorized()),
            || Ok("qltyp_new".to_string()),
        );
        assert_eq!(clears.get(), 1);
    }

    #[test]
    fn forbidden_does_not_reject_token() {
        let forbidden: Error =
            ureq::Error::Status(403, ureq::Response::new(403, "Forbidden", "").unwrap()).into();
        assert!(!is_rejected_token(&forbidden));
    }

    #[test]
    fn token_with_invalid_header_character_is_rejected() {
        assert!(is_rejected_token(
            &qlty_cloud::bearer_authorization_header("qltyp_saved\r").unwrap_err()
        ));
    }
}
