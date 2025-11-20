use crate::export::CoverageExport;
use crate::publish::Report;
use anyhow::{anyhow, bail};
use anyhow::{Context, Result};
use qlty_cloud::{get_legacy_api_url, Client as QltyClient};
use qlty_config::http;
use qlty_types::tests::v1::CoverageMetadata;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Default, Clone, Debug)]
pub struct Upload {
    pub id: String,
    pub project_id: String,
    pub url: String,
    pub coverage_url: String,
}

impl Upload {
    pub fn prepare(token: &str, report: &mut Report) -> Result<Self> {
        let response = Self::request_api(&report.metadata, token)?;

        let coverage_url = response
            .get("data")
            .and_then(|data| data.get("coverage.zip"))
            .and_then(|upload_url| upload_url.as_str())
            .with_context(|| {
                format!(
                    "Unable to find coverage URL in response body: {:?}",
                    response
                )
            })
            .context("Failed to extract coverage URL from response")?;

        let id = response
            .get("data")
            .and_then(|data| data.get("id"))
            .and_then(|upload_url| upload_url.as_str())
            .with_context(|| format!("Unable to find upload ID in response body: {:?}", response))
            .context("Failed to extract upload ID from response")?;

        let project_id = response
            .get("data")
            .and_then(|data| data.get("projectId"))
            .and_then(|project_id| project_id.as_str())
            .with_context(|| format!("Unable to find project ID in response body: {:?}", response))
            .context("Failed to extract project ID from response")?;

        let url = response
            .get("data")
            .and_then(|data| data.get("url"))
            .and_then(|url| url.as_str())
            .unwrap_or_default(); // Optional

        report.set_upload_id(id);
        report.set_project_id(project_id);

        Ok(Self {
            id: id.to_string(),
            project_id: project_id.to_string(),
            coverage_url: coverage_url.to_string(),
            url: url.to_string(),
        })
    }

    pub fn upload(&self, export: &CoverageExport) -> Result<()> {
        self.upload_data(
            &self.coverage_url,
            "application/zip",
            export.read_file(PathBuf::from("coverage.zip"))?,
        )?;

        Ok(())
    }

    fn upload_data(
        &self,
        url: &str,
        content_type: &str,
        data: Vec<u8>,
    ) -> Result<(), anyhow::Error> {
        let response_result = http::put(url)?
            .set("Content-Type", content_type)
            .send_bytes(&data);

        let response = match response_result {
            Ok(resp) => resp,
            Err(ureq::Error::Status(code, resp)) => match resp.into_string() {
                Ok(body) => {
                    bail!(
                        "HTTP Error {}: PUT {}: Upload request failed with response body: {}",
                        code,
                        url,
                        body
                    );
                }
                Err(err) => {
                    bail!(
                            "HTTP Error {}: PUT {}: Upload request failed, error reading response body: {:?}",
                            code,
                            url,
                            err
                        );
                }
            },
            Err(ureq::Error::Transport(transport_error)) => {
                bail!(
                    "Transport Error: PUT {}: Error sending upload bytes: {:?}",
                    url,
                    transport_error
                );
            }
        };

        if response.status() < 200 || response.status() >= 300 {
            bail!(
                "HTTP Error {}: PUT {}: Upload request returned an error: {:?}",
                response.status(),
                url,
                response
                    .into_string()
                    .map_err(|err| anyhow!("Error reading response body: {:?}", err))?,
            );
        }

        Ok(())
    }

    fn request_api(metadata: &CoverageMetadata, token: &str) -> Result<Value> {
        let legacy_api_url = get_legacy_api_url();
        let client = QltyClient::new(Some(&legacy_api_url), Some(token.into()));
        client.post_coverage_metadata("/coverage", metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::Once;
    use std::thread;
    use std::time::Duration;

    static INIT: Once = Once::new();

    fn setup_crypto_provider() {
        INIT.call_once(|| {
            let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
            std::env::set_var("QLTY_INSECURE_ALLOW_HTTP", "true");
        });
    }

    struct TestServer {
        base_url: String,
        handle: thread::JoinHandle<()>,
        stop_signal: Arc<Mutex<bool>>,
    }

    impl TestServer {
        fn start(status_code: u16, response_body: &str) -> TestServer {
            let response_body = response_body.to_string();
            let stop_signal = Arc::new(Mutex::new(false));
            let stop_signal_clone = stop_signal.clone();

            let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
            let base_url = format!("http://{}", server.server_addr().to_ip().unwrap());

            let handle = thread::spawn(move || {
                while !*stop_signal_clone.lock().unwrap() {
                    if let Ok(Some(request)) = server.recv_timeout(Duration::from_millis(100)) {
                        let response = tiny_http::Response::from_string(&response_body)
                            .with_status_code(status_code);
                        let _ = request.respond(response);
                        break;
                    }
                }
            });

            TestServer {
                base_url,
                handle,
                stop_signal,
            }
        }

        fn url(&self) -> String {
            self.base_url.clone()
        }

        fn stop(self) {
            *self.stop_signal.lock().unwrap() = true;
            let _ = self.handle.join();
        }
    }

    #[test]
    fn test_upload_data_success() {
        setup_crypto_provider();
        let server = TestServer::start(200, "Upload successful");
        let upload = Upload::default();

        let result = upload.upload_data(&server.url(), "application/zip", vec![1, 2, 3, 4]);

        assert!(result.is_ok());
        server.stop();
    }

    #[test]
    fn test_upload_data_http_error_with_response_body() {
        setup_crypto_provider();
        let error_body = "Access denied: Invalid credentials";
        let server = TestServer::start(403, error_body);
        let upload = Upload::default();

        let result = upload.upload_data(&server.url(), "application/zip", vec![1, 2, 3, 4]);

        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("HTTP Error 403"));
        assert!(error_message.contains("Upload request failed with response body"));
        assert!(error_message.contains(error_body));
        server.stop();
    }

    #[test]
    fn test_upload_data_http_error_4xx() {
        setup_crypto_provider();
        let error_body = "Bad Request: Invalid file format";
        let server = TestServer::start(400, error_body);
        let upload = Upload::default();

        let result = upload.upload_data(&server.url(), "application/zip", vec![1, 2, 3, 4]);

        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("HTTP Error 400"));
        assert!(error_message.contains("Upload request failed with response body"));
        assert!(error_message.contains(error_body));
        server.stop();
    }

    #[test]
    fn test_upload_data_http_error_5xx() {
        setup_crypto_provider();
        let error_body = "Internal Server Error: Database connection failed";
        let server = TestServer::start(500, error_body);
        let upload = Upload::default();

        let result = upload.upload_data(&server.url(), "application/zip", vec![1, 2, 3, 4]);

        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("HTTP Error 500"));
        assert!(error_message.contains("Upload request failed with response body"));
        assert!(error_message.contains(error_body));
        server.stop();
    }

    #[test]
    fn test_upload_data_transport_error() {
        setup_crypto_provider();
        let upload = Upload::default();
        let invalid_url = "http://127.0.0.1:54321/upload";

        let result = upload.upload_data(invalid_url, "application/zip", vec![1, 2, 3, 4]);

        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Transport Error"));
        assert!(error_message.contains("Error sending upload bytes"));
    }

    #[test]
    fn test_upload_data_successful_response_fallback_error_handling() {
        setup_crypto_provider();
        let server = TestServer::start(201, "Created successfully");
        let upload = Upload::default();

        let result = upload.upload_data(&server.url(), "application/zip", vec![1, 2, 3, 4]);

        assert!(result.is_ok());
        server.stop();
    }

    #[test]
    fn test_upload_data_non_2xx_success_response_fallback() {
        setup_crypto_provider();
        let server = TestServer::start(301, "Moved permanently");
        let upload = Upload::default();

        let result = upload.upload_data(&server.url(), "application/zip", vec![1, 2, 3, 4]);

        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("HTTP Error 301"));
        assert!(error_message.contains("Upload request returned an error"));
        server.stop();
    }
}
