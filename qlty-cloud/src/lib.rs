use anyhow::{anyhow, bail, Result};
use qlty_config::http;
use qlty_config::version::QLTY_VERSION;
use qlty_types::tests::v1::CoverageMetadata;
use serde_json::Value;
use ureq::{json, serde_json, Error, Request};

const ENV_QLTY_API_URL: &str = "QLTY_API_URL";
const QLTY_API_URL: &str = "https://api.qlty.sh";
const LEGACY_API_URL: &str = "https://qlty.sh/api";

pub fn get_api_url() -> String {
    std::env::var(ENV_QLTY_API_URL).unwrap_or_else(|_| QLTY_API_URL.to_string())
}

pub fn get_legacy_api_url() -> String {
    std::env::var(ENV_QLTY_API_URL).unwrap_or_else(|_| LEGACY_API_URL.to_string())
}
const USER_AGENT_PREFIX: &str = "qlty";

#[derive(Debug, Clone)]
pub struct Client {
    pub base_url: String,
    pub token: Option<String>,
}

impl Default for Client {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl Client {
    pub fn new(base_url: Option<&str>, token: Option<String>) -> Self {
        Self {
            base_url: if let Some(url) = base_url {
                url.to_string()
            } else {
                get_api_url()
            },
            token,
        }
    }

    pub fn post(&self, path: &str) -> Result<Request> {
        let url = self.build_url(path);
        self.build_request(http::post(&url)?)
    }

    pub fn get(&self, path: &str) -> Result<Request> {
        let url = self.build_url(path);
        self.build_request(http::get(&url)?)
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn build_request(&self, request: Request) -> Result<Request> {
        let mut request = request;

        request = request.set(
            "User-Agent",
            &format!("{}/{}", USER_AGENT_PREFIX, QLTY_VERSION),
        );

        if let Some(header_value) = self.authorization_header()? {
            request = request.set("Authorization", &header_value);
        }

        Ok(request)
    }

    // The bytes ureq accepts in a header value.
    fn authorization_header(&self) -> Result<Option<String>> {
        match &self.token {
            Some(token)
                if !token
                    .bytes()
                    .all(|byte| matches!(byte, b' ' | b'\t' | 0x21..=0x7E)) =>
            {
                bail!(
                    "The token contains a character that can't be sent in an HTTP header, such as a line break or a non-ASCII character."
                )
            }
            Some(token) => Ok(Some(format!("Bearer {}", token))),
            None => Ok(None),
        }
    }

    pub fn post_coverage_metadata(&self, url: &str, metadata: &CoverageMetadata) -> Result<Value> {
        self.post_coverage_data(url, &serde_json::to_value(metadata)?)
    }

    pub fn post_coverage_data(&self, url: &str, data: &Value) -> Result<Value> {
        let response_result = self.post(url)?.send_json(json!({
            "data": data,
        }));

        match response_result {
            Ok(resp) => resp.into_json::<Value>().map_err(|err| {
                anyhow!(
                    "JSON Error: {}: Unable to parse JSON response from success: {:?}",
                    self.base_url,
                    err
                )
            }),

            Err(Error::Status(code, resp)) => match resp.into_string() {
                Ok(body) => match serde_json::from_str::<Value>(&body) {
                    Ok(json) => match json.get("error") {
                        Some(error) => {
                            bail!("HTTP Error {}: {}: {}", code, self.base_url, error)
                        }
                        None => {
                            bail!("HTTP Error {}: {}: {}", code, self.base_url, body);
                        }
                    },
                    Err(_) => bail!(
                        "HTTP Error {}: {}: Unable to parse JSON response: {}",
                        code,
                        self.base_url,
                        body
                    ),
                },
                Err(err) => bail!(
                    "HTTP Error {}: {}: Error reading response body: {:?}",
                    code,
                    self.base_url,
                    err
                ),
            },
            Err(Error::Transport(transport_error)) => {
                bail!("Transport Error: {}: {:?}", self.base_url, transport_error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_token_with_line_break() {
        let client = Client::new(None, Some("qltcp_SECRET\n".to_string()));
        assert!(!client
            .authorization_header()
            .unwrap_err()
            .to_string()
            .contains("SECRET"));
    }

    #[test]
    fn rejects_token_with_non_ascii_project() {
        let client = Client::new(None, Some("qltcw_SECRET/café".to_string()));
        assert!(!client
            .authorization_header()
            .unwrap_err()
            .to_string()
            .contains("SECRET"));
    }

    #[test]
    fn builds_bearer_authorization_header() {
        let client = Client::new(None, Some("qltcp_123".to_string()));
        assert_eq!(
            client.authorization_header().unwrap(),
            Some("Bearer qltcp_123".to_string())
        );
    }
}
