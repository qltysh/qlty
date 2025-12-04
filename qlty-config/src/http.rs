use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use rustls_platform_verifier::BuilderVerifierExt;
use std::env;
use std::sync::Arc;
use url::Url;

static AGENT: Lazy<ureq::Agent> = Lazy::new(|| {
    // Use platform-verifier to build a TLS config with native roots
    let config = rustls::ClientConfig::builder()
        .with_platform_verifier()
        .with_no_client_auth();

    ureq::AgentBuilder::new()
        .tls_config(Arc::new(config))
        .build()
});

fn validate_https_url(url_str: &str) -> Result<()> {
    let url = Url::parse(url_str)?;

    if url.scheme() == "https" {
        return Ok(());
    }

    if env::var("QLTY_INSECURE_ALLOW_HTTP").ok() == Some("true".to_string()) {
        return Ok(());
    }

    Err(anyhow!(
        "HTTP URLs are not allowed. Use HTTPS or set QLTY_INSECURE_ALLOW_HTTP=true for testing: {}",
        url_str
    ))
}

pub fn get(url: &str) -> Result<ureq::Request> {
    validate_https_url(url)?;
    Ok(AGENT.get(url))
}

pub fn post(url: &str) -> Result<ureq::Request> {
    validate_https_url(url)?;
    Ok(AGENT.post(url))
}

pub fn put(url: &str) -> Result<ureq::Request> {
    validate_https_url(url)?;
    Ok(AGENT.put(url))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup_test_http() {
        INIT.call_once(|| {
            std::env::set_var("QLTY_INSECURE_ALLOW_HTTP", "true");
        });
    }

    fn setup_crypto_provider() {
        std::sync::Once::new().call_once(|| {
            let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        });
    }

    #[test]
    fn test_get_creates_request() {
        setup_crypto_provider();
        let request = get("https://example.com").unwrap();
        assert_eq!(request.url(), "https://example.com");
    }

    #[test]
    fn test_post_creates_request() {
        setup_crypto_provider();
        let request = post("https://example.com").unwrap();
        assert_eq!(request.url(), "https://example.com");
    }

    #[test]
    fn test_put_creates_request() {
        setup_crypto_provider();
        let request = put("https://example.com").unwrap();
        assert_eq!(request.url(), "https://example.com");
    }

    #[test]
    fn test_get_rejects_invalid_url() {
        setup_crypto_provider();
        setup_test_http();
        let result = get("not a url");
        assert!(result.is_err());
    }
}
