use crate::publish::Report;
use anyhow::{Context, Result};
use qlty_cloud::{export::CoverageExport, Client as QltyClient};
use std::path::PathBuf;

const LEGACY_API_URL: &str = "https://qlty.sh/api";

#[derive(Default, Clone, Debug)]
pub struct Upload {
    pub id: String,
    pub project_id: String,
    pub file_coverages_url: String,
    pub report_files_url: String,
    pub metadata_url: String,
}

impl Upload {
    pub fn prepare(token: &str, report: &mut Report) -> Result<Self> {
        let client = QltyClient::new(Some(LEGACY_API_URL), Some(token.into()));

        let response = client
            .post("/coverage")
            .send_json(ureq::json!({
                "data": report.metadata,
            }))?
            .into_json::<serde_json::Value>()?;

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
}
