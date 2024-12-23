use crate::auth::{token::DEFAULT_USER, token::SERVICE, Token};
use actix_web::{
    dev::{Server, ServerHandle},
    get,
    http::{header::LOCATION, Uri},
    rt,
    web::{Data, Query},
    App, HttpResponse, HttpResponseBuilder, HttpServer,
};
use anyhow::Result;
use serde::Deserialize;
use std::{
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tracing::{error, info};
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
    server_handle: Arc<Mutex<Option<ServerHandle>>>,
    pub login_url: String,
    pub original_state: String,
    pub credential_user: String,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            login_url: std::env::var(LOGIN_URL_ENV).unwrap_or(LOGIN_URL.to_string()),
            original_state: Uuid::new_v4().to_string(),
            server_handle: Arc::new(Mutex::new(None)),
            credential_user: DEFAULT_USER.to_string(),
        }
    }
}

#[get("/")]
async fn auth_flow(
    query: Query<AuthFlowQueryParams>,
    state: Data<Arc<AppState>>,
) -> HttpResponseBuilder {
    if state.original_state != query.state {
        error!(
            "Invalid state parameter in auth flow: {:?} did not match received state {:?}",
            state.original_state, query.state
        );
        return HttpResponse::BadRequest();
    }

    if !redirect_matches(state.login_url.clone(), query.redirect_uri.clone()) {
        error!(
            "Invalid redirect_uri parameter in auth flow (expecting {}): {}",
            state.login_url, query.redirect_uri
        );
        return HttpResponse::BadRequest();
    }

    let token = Token::new(&state.credential_user);
    match token.set(&query.code) {
        Result::Ok(_) => {
            info!(
                "Auth token stored in credential storage: {}.{}",
                SERVICE, state.credential_user
            );
            stop_login_server(state);
            HttpResponse::TemporaryRedirect()
                .insert_header((LOCATION, query.redirect_uri.clone()))
                .take()
        }
        Err(e) => {
            error!("Failed to store auth token in credential storage: {}", e);
            HttpResponse::BadRequest()
        }
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

fn login_server(state: Arc<AppState>) -> Result<(Server, u16)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .service(auth_flow)
    })
    .shutdown_timeout(0)
    .workers(1)
    .listen(listener)?
    .run();
    Ok((server, port))
}

pub fn launch_login_server(state: AppState) -> Result<u16> {
    let handle = state.server_handle.clone();
    let (server, port) = login_server(Arc::new(state))?;

    handle.lock().unwrap().replace(server.handle());

    thread::spawn(move || {
        let _ = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(server);
    });

    Ok(port)
}

fn stop_login_server(state: Data<Arc<AppState>>) {
    rt::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let handle = state.server_handle.lock().unwrap().take().unwrap();
        handle.stop(false).await;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{
        test::{self, init_service, TestRequest},
        App,
    };

    impl AppState {
        fn test(original_state: &str) -> Self {
            let mut state = AppState::default();
            state.original_state = original_state.to_string();
            state.credential_user = "qlty-cli-test".to_string();
            state.login_url = "https://example.com".to_string();
            state
        }
    }

    #[actix_web::test]
    async fn test_auth_flow_get() {
        let state = Arc::new(AppState::test("123"));
        let req = TestRequest::default()
            .uri("/?state=123&code=ABCDEFG&redirect_uri=https://example.com/complete")
            .to_request();
        let app = init_service(App::new().app_data(Data::new(state)).service(auth_flow)).await;
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 307);
        assert_eq!(
            resp.headers().get(LOCATION).unwrap(),
            "https://example.com/complete"
        );
    }

    #[actix_web::test]
    async fn test_auth_flow_wrong_state() {
        let state = Arc::new(AppState::test("123"));
        let req = TestRequest::default()
            .uri("/?state=456&code=ABCDEFG&redirect_uri=https://example.com")
            .to_request();
        let app = init_service(App::new().app_data(Data::new(state)).service(auth_flow)).await;
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_web::test]
    async fn test_auth_flow_no_redirect_uri() {
        let state = Arc::new(AppState::test("123"));
        let req = TestRequest::default()
            .uri("/?state=456&code=ABCDEFG")
            .to_request();
        let app = init_service(App::new().app_data(Data::new(state)).service(auth_flow)).await;
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_web::test]
    async fn test_auth_flow_invalid_redirect_uri() {
        let state = Arc::new(AppState::test("123"));
        let req = TestRequest::default()
            .uri("/?state=456&code=ABCDEFG&redirect_uri=http://example.com")
            .to_request();
        let app = init_service(App::new().app_data(Data::new(state)).service(auth_flow)).await;
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_web::test]
    async fn test_auth_flow_server() {
        let state = AppState::test("123");
        let port = launch_login_server(state).unwrap();
        let url = format!("http://127.0.0.1:{}", port);

        ureq::get(&url)
            .query("state", "123")
            .query("code", "ABCDEFG")
            .query("redirect_uri", "https://example.com")
            .call()
            .unwrap();

        thread::sleep(Duration::from_millis(100));

        // ensure server has shut-down
        assert!(ureq::get(&url)
            .timeout(Duration::from_millis(10))
            .call()
            .is_err());

        let token = Token::new("qlty-cli-test").get().unwrap();
        assert_eq!(token, "ABCDEFG");
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
