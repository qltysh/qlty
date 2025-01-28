use crate::format::{CopyFormatter, GzFormatter, JsonEachRowFormatter, JsonFormatter};
use anyhow::Result;
use qlty_analysis::Report;
use qlty_config::Workspace;
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Default, Debug)]
pub struct AnalysisExport {
    pub report: Report,
    pub path: PathBuf,
    pub gzip: bool,
}

impl AnalysisExport {
    pub fn new(report: &Report, path: &Path, gzip: bool) -> Self {
        Self {
            report: report.clone(),
            path: path.to_path_buf(),
            gzip,
        }
    }

    pub fn export(&self) -> Result<()> {
        info!("Exporting analysis to: {}", self.path.display());
        std::fs::create_dir_all(&self.path)?;

        if self.gzip {
            self.export_json_gz()
        } else {
            self.export_json()
        }
    }

    fn export_json(&self) -> Result<()> {
        JsonFormatter::new(self.report.metadata.clone())
            .write_to_file(&self.path.join("metadata.json"))?;

        JsonEachRowFormatter::new(self.report.messages.clone())
            .write_to_file(&self.path.join("messages.jsonl"))?;

        JsonEachRowFormatter::new(self.report.invocations.clone())
            .write_to_file(&self.path.join("invocations.jsonl"))?;

        JsonEachRowFormatter::new(self.report.issues.clone())
            .write_to_file(&self.path.join("issues.jsonl"))?;

        JsonEachRowFormatter::new(self.report.stats.clone())
            .write_to_file(&self.path.join("stats.jsonl"))?;

        CopyFormatter::boxed(Self::qlty_config_path()?)
            .write_to_file(&self.path.join("qlty.toml"))?;

        Ok(())
    }

    fn export_json_gz(&self) -> Result<()> {
        JsonFormatter::new(self.report.metadata.clone())
            .write_to_file(&self.path.join("metadata.json"))?;

        GzFormatter::new(JsonEachRowFormatter::new(self.report.messages.clone()))
            .write_to_file(&self.path.join("messages.json.gz"))?;

        GzFormatter::new(JsonEachRowFormatter::new(self.report.invocations.clone()))
            .write_to_file(&self.path.join("invocations.json.gz"))?;

        GzFormatter::new(JsonEachRowFormatter::new(self.report.issues.clone()))
            .write_to_file(&self.path.join("issues.json.gz"))?;

        GzFormatter::new(JsonEachRowFormatter::new(self.report.stats.clone()))
            .write_to_file(&self.path.join("stats.json.gz"))?;

        GzFormatter::new(CopyFormatter::boxed(Self::qlty_config_path()?))
            .write_to_file(&self.path.join("qlty.toml.gz"))?;

        Ok(())
    }

    fn qlty_config_path() -> Result<PathBuf> {
        Ok(Workspace::new()?.library()?.qlty_config_path())
    }
}
