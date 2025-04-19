use std::path::PathBuf;

use crate::export::CoverageExport;
use crate::publish::processor::CoverageMetrics;
use anyhow::Result;
use qlty_types::tests::v1::{CoverageMetadata, FileCoverage, ReportFile};
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct Report {
    pub metadata: CoverageMetadata,
    pub report_files: Vec<ReportFile>,
    pub file_coverages: Vec<FileCoverage>,
    pub coverage_metrics: CoverageMetrics,
    pub missing_files: Vec<String>,
}

impl Report {
    pub fn set_upload_id(&mut self, upload_id: &str) {
        self.metadata.upload_id = upload_id.to_string();
    }

    pub fn set_project_id(&mut self, project_id: &str) {
        self.metadata.project_id = Some(project_id.to_string());

        self.report_files.iter_mut().for_each(|f| {
            f.project_id = Some(project_id.to_string());
        });

        self.file_coverages.iter_mut().for_each(|f| {
            f.project_id = Some(project_id.to_string());
        });
    }

    pub fn export_to(&self, directory: Option<PathBuf>) -> Result<CoverageExport> {
        let mut exporter = CoverageExport {
            metadata: self.metadata.clone(),
            report_files: self.report_files.clone(),
            file_coverages: self.file_coverages.clone(),
            ..Default::default()
        };

        exporter.export_to(directory)?;
        Ok(exporter)
    }
}
