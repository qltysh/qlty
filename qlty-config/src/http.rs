use once_cell::sync::Lazy;
use rustls_platform_verifier::BuilderVerifierExt;
use std::sync::Arc;
use std::time::Duration;

static AGENT: Lazy<ureq::Agent> = Lazy::new(|| {
    // Use platform-verifier to build a TLS config with native roots
    let config = rustls::ClientConfig::builder()
        .with_platform_verifier()
        .with_no_client_auth();

    ureq::AgentBuilder::new()
        .tls_config(Arc::new(config))
        .build()
});

pub fn get(url: &str) -> ureq::Request {
    AGENT.get(url)
}

pub fn get_with_timeout(url: &str, timeout: Duration) -> ureq::Request {
    AGENT.get(url).timeout(timeout)
}

pub fn post(url: &str) -> ureq::Request {
    AGENT.post(url)
}

pub fn put(url: &str) -> ureq::Request {
    AGENT.put(url)
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
        let request = get("https://example.com");
        assert_eq!(request.url(), "https://example.com");
    }

    #[test]
    fn test_post_creates_request() {
        setup_crypto_provider();
        let request = post("https://example.com");
        assert_eq!(request.url(), "https://example.com");
    }

    #[test]
    fn test_put_creates_request() {
        setup_crypto_provider();
        let request = put("https://example.com");
        assert_eq!(request.url(), "https://example.com");
    }
}
