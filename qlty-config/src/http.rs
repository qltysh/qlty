use crate::env::{EnvSource, SystemEnv};
use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use rustls_platform_verifier::BuilderVerifierExt;
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
    validate_https_url_with_env(url_str, &SystemEnv)
}

fn validate_https_url_with_env(url_str: &str, env: &dyn EnvSource) -> Result<()> {
    let url = Url::parse(url_str)?;
    let scheme = url.scheme();

    if scheme == "https" {
        return Ok(());
    }

    if scheme == "http" && env.var("QLTY_INSECURE_ALLOW_HTTP") == Some("true".to_string()) {
        return Ok(());
    }

    Err(anyhow!(
        "HTTP URLs are not allowed. Use HTTPS or set QLTY_INSECURE_ALLOW_HTTP=true for development/testing: {}",
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
    use std::collections::HashMap;

    #[derive(Debug, Default)]
    struct HashMapEnv {
        inner: HashMap<String, String>,
    }

    impl EnvSource for HashMapEnv {
        fn var(&self, name: &str) -> Option<String> {
            self.inner.get(name).cloned()
        }
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
    fn test_rejects_invalid_url() {
        let env = HashMapEnv::default();
        let result = validate_https_url_with_env("not a url", &env);
        assert!(result.is_err());
    }

    #[test]
    fn test_https_allowed() {
        let env = HashMapEnv::default();
        let result = validate_https_url_with_env("https://example.com", &env);
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_rejected_without_env_var() {
        let env = HashMapEnv::default();
        let result = validate_https_url_with_env("http://example.com", &env);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_allowed_with_env_var() {
        let mut env = HashMapEnv::default();
        env.inner
            .insert("QLTY_INSECURE_ALLOW_HTTP".to_string(), "true".to_string());
        let result = validate_https_url_with_env("http://example.com", &env);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ftp_rejected_even_with_env_var() {
        let mut env = HashMapEnv::default();
        env.inner
            .insert("QLTY_INSECURE_ALLOW_HTTP".to_string(), "true".to_string());
        let result = validate_https_url_with_env("ftp://example.com", &env);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_rejected_even_with_env_var() {
        let mut env = HashMapEnv::default();
        env.inner
            .insert("QLTY_INSECURE_ALLOW_HTTP".to_string(), "true".to_string());
        let result = validate_https_url_with_env("file:///etc/passwd", &env);
        assert!(result.is_err());
    }
}
