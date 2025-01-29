use crate::publish::Report;
use anyhow::{Context, Result};
use qlty_cloud::{export::CoverageExport, Client as QltyClient};
use qlty_types::tests::v1::CoverageMetadata;
use serde_json::Value;
use std::path::PathBuf;
use ureq::Error;

const LEGACY_API_URL: &str = "https://qlty.sh/api";

#[derive(Default, Clone, Debug)]
pub struct Upload {
    pub id: String,
    pub project_id: String,
    pub file_coverages_url: String,
    pub report_files_url: String,
    pub metadata_url: String,
    pub raw_files_url: String,
}

impl Upload {
    pub fn prepare(token: &str, report: &mut Report) -> Result<Self> {
        let response = Self::request_api(&report.metadata, token)?;

        let file_coverages_url = response
            .get("data")
            .and_then(|data| data.get("file_coverages.json.gz"))
            .and_then(|upload_url| upload_url.as_str())
            .with_context(|| {
                format!(
                    "Unable to find file coverages URL in response body: {:?}",
                    response
                )
            })
            .context("Failed to extract file coverages URL from response")?;

        let report_files_url = response
            .get("data")
            .and_then(|data| data.get("report_files.json.gz"))
            .and_then(|upload_url| upload_url.as_str())
            .with_context(|| {
                format!(
                    "Unable to find report files URL in response body: {:?}",
                    response
                )
            })
            .context("Failed to extract report files URL from response")?;

        let metadata_url = response
            .get("data")
            .and_then(|data| data.get("metadata.json"))
            .and_then(|upload_url| upload_url.as_str())
            .with_context(|| {
                format!(
                    "Unable to find metadata URL in response body: {:?}",
                    response
                )
            })
            .context("Failed to extract metadata URL from response")?;

        let raw_files_url = response
            .get("data")
            .and_then(|data| data.get("raw_files.zip"))
            .and_then(|upload_url| upload_url.as_str())
            .with_context(|| {
                format!(
                    "Unable to find metadata URL in response body: {:?}",
                    response
                )
            })
            .context("Failed to extract metadata URL from response")?;

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

        report.set_upload_id(id);
        report.set_project_id(project_id);

        Ok(Self {
            id: id.to_string(),
            project_id: project_id.to_string(),
            file_coverages_url: file_coverages_url.to_string(),
            report_files_url: report_files_url.to_string(),
            metadata_url: metadata_url.to_string(),
            raw_files_url: raw_files_url.to_string(),
        })
    }

    pub fn upload(&self, export: &CoverageExport) -> Result<()> {
        self.upload_data(
            &self.file_coverages_url,
            "application/gzip",
            export.read_file(PathBuf::from("file_coverages.json.gz"))?,
        )?;

        self.upload_data(
            &self.report_files_url,
            "application/gzip",
            export.read_file(PathBuf::from("report_files.json.gz"))?,
        )?;

        self.upload_data(
            &self.metadata_url,
            "application/json",
            export.read_file(PathBuf::from("metadata.json"))?,
        )?;

        self.upload_data(
            &self.raw_files_url,
            "application/zip",
            export.read_file(PathBuf::from("raw_files.zip"))?,
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
            .context("Failed to send PUT request")?;

        if response.status() < 200 || response.status() >= 300 {
            let error_message = format!(
                "PUT request for uploading file returned {} status with response: {:?}",
                response.status(),
                response
                    .into_string()
                    .unwrap_or_else(|_| "Unknown error".to_string())
            );
            return Err(anyhow::anyhow!(error_message));
        }

        Ok(())
    }

    fn request_api(metadata: &CoverageMetadata, token: &str) -> Result<Value> {
        let client = QltyClient::new(Some(LEGACY_API_URL), Some(token.into()));
        let response_result = client.post("/coverage").send_json(ureq::json!({
            "data": metadata,
        }));

        match response_result {
            Ok(resp) => resp
                .into_json::<Value>()
                .map_err(|_| anyhow::anyhow!("Invalid JSON response")),

            Err(Error::Status(code, resp)) => {
                let error_message: Value = resp
                    .into_json()
                    .unwrap_or_else(|_| serde_json::json!({"error": "Unknown error"}));

                let error_text = error_message
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");

                Err(anyhow::anyhow!("HTTP Error {}: {}", code, error_text))
            }
            Err(Error::Transport(transport_error)) => {
                Err(anyhow::anyhow!("Transport Error: {:?}", transport_error))
            }
        }
    }
}
