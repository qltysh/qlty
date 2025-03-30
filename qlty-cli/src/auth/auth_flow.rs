use super::credentials::write_token;
use anyhow::{anyhow, bail, Context, Result};
use http::Uri;
use serde::Deserialize;
use serde_querystring::ParseMode;
use std::{
    fmt::Display,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, RwLock,
    },
    thread::spawn,
};
use tiny_http::{Header, Request, Response, ResponseBox, Server};
use tracing::error;
use uuid::Uuid;

const LOGIN_URL: &str = "https://qlty.sh/login";
const LOGIN_URL_ENV: &str = "QLTY_AUTH_LOGIN_URL";

#[derive(Deserialize, Debug)]
struct AuthFlowQueryParams {
    state: String,
    code: String,
    redirect_uri: String,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub login_url: String,
    pub original_state: String,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            login_url: std::env::var(LOGIN_URL_ENV).unwrap_or(LOGIN_URL.to_string()),
            original_state: Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Clone)]
pub struct ServerResponse {
    pub base_url: String,
    shutdown_send: Sender<()>,
    server: Arc<RwLock<Server>>,
}

impl Drop for ServerResponse {
    fn drop(&mut self) {
        self.server.read().unwrap().unblock();
        if let Err(e) = self.shutdown_send.send(()) {
            error!("Failed to send shutdown signal: {}", e);
        }
    }
}

impl Display for ServerResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.base_url)
    }
}

fn redirect_matches(expected: String, actual: String) -> bool {
    if let Ok(expected_uri) = expected.parse::<Uri>() {
        if let Ok(actual_uri) = actual.parse::<Uri>() {
            return expected_uri.scheme() == actual_uri.scheme()
                && expected_uri.host() == actual_uri.host()
                && expected_uri.port_u16() == actual_uri.port_u16();
        }
    }

    false
}

fn run_handler(request: &Request, state: &AppState) -> Result<ResponseBox> {
    let uri = request.url().parse::<Uri>()?;
    let params: AuthFlowQueryParams;
    if let Some(query) = uri.query() {
        params = serde_querystring::from_str::<AuthFlowQueryParams>(query, ParseMode::UrlEncoded)
            .with_context(|| "invalid parameter")?;
    } else {
        bail!("No query parameters found in request");
    }

    if params.state != state.original_state {
        bail!("State does not match original state");
    }

    if !redirect_matches(state.login_url.clone(), params.redirect_uri.clone()) {
        bail!("Redirect URI does not match login URL");
    }

    if let Err(e) = write_token(&params.code) {
        bail!("Failed to store auth token in credential storage: {}", e);
    }

    let redirect = format!("Location: {}", params.redirect_uri)
        .parse::<Header>()
        .map_err(|_| anyhow!("Failed to generate redirect"))?;
    Ok(Response::empty(307).with_header(redirect).boxed())
}

pub fn launch_login_server(state: AppState) -> Result<ServerResponse> {
    let server =
        Server::http("127.0.0.1:0").map_err(|e| anyhow!("Failed to start server: {}", e))?;

    let (shutdown_send, shutdown_recv): (Sender<()>, Receiver<()>) = mpsc::channel();
    let ip = server
        .server_addr()
        .to_ip()
        .ok_or_else(|| anyhow!("Failed to determine server address"))?;
    let base_url = format!("http://{}", ip);

    let server = Arc::new(RwLock::new(server));
    let server_copy = server.clone();
    spawn(move || run_login_server_loop(server, &state, shutdown_recv));

    Ok(ServerResponse {
        base_url,
        shutdown_send,
        server: server_copy,
    })
}

fn run_login_server_loop(
    server: Arc<RwLock<Server>>,
    state: &AppState,
    shutdown_recv: Receiver<()>,
) {
    loop {
        match server.read().unwrap().recv() {
            Ok(request) => {
                match run_handler(&request, state) {
                    Ok(response) => {
                        if let Err(e) = request.respond(response) {
                            error!("Failed to send response: {}", e);
                        } else {
                            // shutdown server
                            return;
                        }
                    }
                    Err(e) => {
                        let response =
                            Response::from_string(format!("Error: {}", e)).with_status_code(400);
                        request.respond(response).ok();
                        error!("Failed to process request: {}", e);
                    }
                };
            }
            Err(e) => error!("Failed to receive request: {}", e),
        }

        if shutdown_recv.try_recv().is_ok() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::credentials::{read_token, set_mock_entry};
    use keyring::{mock, set_default_credential_builder, Entry};
    use std::sync::Once;
    use std::{thread, time::Duration};

    static INIT: Once = Once::new();

    impl AppState {
        fn test(original_state: &str) -> Self {
            INIT.call_once(|| {
                set_default_credential_builder(mock::default_credential_builder());
                set_mock_entry(Arc::new(
                    Entry::new("qlty-cli-test", "qlty-cli-test").unwrap(),
                ));
            });

            let mut state = AppState::default();
            state.original_state = original_state.to_string();
            state.login_url = "https://example.com".to_string();
            state
        }
    }

    #[test]
    fn test_auth_flow_get() {
        let state = AppState::test("123");
        let server = launch_login_server(state).unwrap();
        let resp = ureq::builder()
            .redirects(0)
            .build()
            .get(&server.base_url)
            .query("state", "123")
            .query("code", "ABCDEFG")
            .query("redirect_uri", "https://example.com")
            .call()
            .unwrap();

        assert_eq!(resp.status(), 307);
        assert_eq!(resp.header("Location").unwrap(), "https://example.com");
    }

    #[test]
    fn test_auth_flow_wrong_state() {
        let state = AppState::test("123");
        let server = launch_login_server(state).unwrap();
        assert!(ureq::get(&server.base_url)
            .query("state", "456")
            .query("code", "ABCDEFG")
            .query("redirect_uri", "https://example.com")
            .call()
            .is_err());
    }

    #[test]
    fn test_auth_flow_no_redirect_uri() {
        let state = AppState::test("123");
        let server = launch_login_server(state).unwrap();
        assert!(ureq::get(&server.base_url)
            .query("state", "123")
            .query("code", "ABCDEFG")
            .call()
            .is_err());
    }

    #[test]
    fn test_auth_flow_invalid_redirect_uri() {
        let state = AppState::test("123");
        let server = launch_login_server(state).unwrap();
        assert!(ureq::get(&server.base_url)
            .query("state", "123")
            .query("code", "ABCDEFG")
            .query("redirect_uri", "http://example.com")
            .call()
            .is_err());
    }

    #[test]
    fn test_auth_flow_server() {
        let state = AppState::test("123");
        let server = launch_login_server(state).unwrap();

        let response = ureq::builder()
            .redirects(0)
            .build()
            .get(&server.base_url)
            .query("state", "123")
            .query("code", "ABCDEFG")
            .query("redirect_uri", "https://example.com/?complete")
            .call()
            .unwrap();

        assert_eq!(response.status(), 307);
        assert_eq!(
            response.header("Location").unwrap(),
            "https://example.com/?complete"
        );

        thread::sleep(Duration::from_millis(100));

        // ensure server has shut-down
        assert!(ureq::get(&server.base_url)
            .timeout(Duration::from_millis(10))
            .call()
            .is_err());

        assert_eq!(read_token().unwrap(), "ABCDEFG");
    }

    #[test]
    fn test_redirect_matches() {
        assert!(redirect_matches(
            "https://example.com".to_string(),
            "https://example.com".to_string()
        ));
        assert!(redirect_matches(
            "https://example.com/a".to_string(),
            "https://example.com/b".to_string()
        ));
        assert!(!redirect_matches(
            "https://example.com".to_string(),
            "http://example.com".to_string()
        ));
        assert!(!redirect_matches(
            "https://example.com".to_string(),
            "https://example.com:8080".to_string()
        ));
        assert!(!redirect_matches(
            "https://example.com".to_string(),
            "https://not-example.com".to_string()
        ));
    }
}
