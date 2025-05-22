use once_cell::sync::Lazy;
use std::sync::Arc;

static AGENT: Lazy<ureq::Agent> = Lazy::new(|| {
    let mut config_builder = rustls::ClientConfig::builder();
    
    // Use platform-verifier to get system certificate roots
    config_builder = config_builder.with_root_certs(
        rustls_platform_verifier::tls_config_with_native_roots()
            .expect("Failed to get platform certificate roots")
            .root_store
    );
    
    let config = config_builder.with_no_client_auth();
    
    ureq::AgentBuilder::new()
        .tls_config(Arc::new(config))
        .build()
});

pub fn get(url: &str) -> ureq::Request {
    AGENT.get(url)
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

    #[test]
    fn test_get_creates_request() {
        let request = get("https://example.com");
        assert_eq!(request.url(), "https://example.com");
    }

    #[test]
    fn test_post_creates_request() {
        let request = post("https://example.com");
        assert_eq!(request.url(), "https://example.com");
    }

    #[test]
    fn test_put_creates_request() {
        let request = put("https://example.com");
        assert_eq!(request.url(), "https://example.com");
    }
}