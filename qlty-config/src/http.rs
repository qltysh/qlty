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

    if let Some(host) = url.host_str() {
        if host == "localhost"
            || host == "::1"
            || host.starts_with("127.")
            || host.starts_with("[::1]")
        {
            return Ok(());
        }
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
    fn test_get_rejects_http_url() {
        setup_crypto_provider();
        let result = get("http://example.com");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("HTTP URLs are not allowed"));
    }

    #[test]
    fn test_post_rejects_http_url() {
        setup_crypto_provider();
        let result = post("http://example.com");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("HTTP URLs are not allowed"));
    }

    #[test]
    fn test_put_rejects_http_url() {
        setup_crypto_provider();
        let result = put("http://example.com");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("HTTP URLs are not allowed"));
    }

    #[test]
    fn test_get_rejects_invalid_url() {
        setup_crypto_provider();
        let result = get("not a url");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_allows_localhost_http() {
        setup_crypto_provider();
        let result = get("http://localhost:8080/path");
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_allows_127_0_0_1_http() {
        setup_crypto_provider();
        let result = get("http://127.0.0.1:8080/path");
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_allows_ipv6_loopback_http() {
        setup_crypto_provider();
        let result = get("http://[::1]:8080/path");
        assert!(result.is_ok());
    }
}
