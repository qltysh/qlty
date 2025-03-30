use qlty_config::version::QLTY_VERSION;
use ureq::Request;

const QLTY_API_URL: &str = "https://api.qlty.sh";
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
                match std::env::var("QLTY_API_URL") {
                    Ok(url) => url,
                    Err(_) => QLTY_API_URL.to_string(),
                }
            },
            token,
        }
    }

    pub fn post(&self, path: &str) -> ureq::Request {
        let url = self.build_url(path);
        self.build_request(ureq::post(&url))
    }

    pub fn get(&self, path: &str) -> ureq::Request {
        let url = self.build_url(path);
        self.build_request(ureq::get(&url))
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn build_request(&self, request: Request) -> Request {
        let mut request = request;

        request = request.set(
            "User-Agent",
            &format!("{}/{}", USER_AGENT_PREFIX, QLTY_VERSION),
        );

        if let Some(header_value) = self.authorization_header() {
            request = request.set("Authorization", &header_value);
        }

        request
    }

    fn authorization_header(&self) -> Option<String> {
        self.token.as_ref().map(|token| format!("Bearer {}", token))
    }
}
