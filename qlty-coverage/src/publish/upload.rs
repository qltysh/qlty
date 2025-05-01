use crate::export::CoverageExport;
use crate::publish::Report;
use anyhow::{anyhow, bail};
use anyhow::{Context, Result};
use qlty_cloud::Client as QltyClient;
use qlty_types::tests::v1::CoverageMetadata;
use serde_json::Value;
use std::path::PathBuf;
use ureq::Error;

const LEGACY_API_URL: &str = "https://qlty.sh/api";

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
        let response = ureq::put(url)
            .set("Content-Type", content_type)
            .send_bytes(&data)
            .map_err(|err| {
                anyhow!(
                    "HTTP Error: PUT {}: Error sending upload bytes: {:?}",
                    url,
                    err
                )
            })?;

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
        let client = QltyClient::new(Some(LEGACY_API_URL), Some(token.into()));
        let response_result = client.post("/coverage").send_json(ureq::json!({
            "data": metadata,
        }));

        match response_result {
            Ok(resp) => resp.into_json::<Value>().map_err(|err| {
                anyhow!(
                    "JSON Error: {}: Unable to parse JSON response from success: {:?}",
                    client.base_url,
                    err
                )
            }),

            Err(Error::Status(code, resp)) => match resp.into_string() {
                Ok(body) => match serde_json::from_str::<Value>(&body) {
                    Ok(json) => match json.get("error") {
                        Some(error) => {
                            bail!("HTTP Error {}: {}: {}", code, client.base_url, error)
                        }
                        None => {
                            bail!("HTTP Error {}: {}: {}", code, client.base_url, body);
                        }
                    },
                    Err(_) => bail!(
                        "HTTP Error {}: {}: Unable to parse JSON response: {}",
                        code,
                        client.base_url,
                        body
                    ),
                },
                Err(err) => bail!(
                    "HTTP Error {}: {}: Error reading response body: {:?}",
                    code,
                    client.base_url,
                    err
                ),
            },
            Err(Error::Transport(transport_error)) => bail!(
                "Transport Error: {}: {:?}",
                client.base_url,
                transport_error
            ),
        }
    }
}
