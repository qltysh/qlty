use anyhow::Result;
use qlty_analysis::Report;
use qlty_config::Workspace;
use qlty_formats::{
    CopyFormatter, Formatter, GzFormatter, InvocationJsonFormatter, JsonEachRowFormatter,
    JsonFormatter,
};
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
        Box::new(JsonFormatter::new(self.report.metadata.clone()))
            .write_to_file(&self.path.join("metadata.json"))?;

        Box::new(JsonEachRowFormatter::new(self.report.messages.clone()))
            .write_to_file(&self.path.join("messages.jsonl"))?;

        Box::new(InvocationJsonFormatter::new(
            self.report.invocations.clone(),
        ))
        .write_to_file(&self.path.join("invocations.jsonl"))?;

        Box::new(JsonEachRowFormatter::new(self.report.issues.clone()))
            .write_to_file(&self.path.join("issues.jsonl"))?;

        Box::new(JsonEachRowFormatter::new(self.report.stats.clone()))
            .write_to_file(&self.path.join("stats.jsonl"))?;

        Box::new(CopyFormatter::new(Self::qlty_config_path()?))
            .write_to_file(&self.path.join("qlty.toml"))?;

        Ok(())
    }

    fn export_json_gz(&self) -> Result<()> {
        Box::new(JsonFormatter::new(self.report.metadata.clone()))
            .write_to_file(&self.path.join("metadata.json"))?;

        Box::new(GzFormatter::new(Box::new(JsonEachRowFormatter::new(
            self.report.messages.clone(),
        ))))
        .write_to_file(&self.path.join("messages.json.gz"))?;

        Box::new(GzFormatter::new(Box::new(InvocationJsonFormatter::new(
            self.report.invocations.clone(),
        ))))
        .write_to_file(&self.path.join("invocations.json.gz"))?;

        Box::new(GzFormatter::new(Box::new(JsonEachRowFormatter::new(
            self.report.issues.clone(),
        ))))
        .write_to_file(&self.path.join("issues.json.gz"))?;

        Box::new(GzFormatter::new(Box::new(JsonEachRowFormatter::new(
            self.report.stats.clone(),
        ))))
        .write_to_file(&self.path.join("stats.json.gz"))?;

        Box::new(GzFormatter::new(Box::new(CopyFormatter::new(
            Self::qlty_config_path()?,
        ))))
        .write_to_file(&self.path.join("qlty.toml.gz"))?;

        Ok(())
    }

    fn qlty_config_path() -> Result<PathBuf> {
        Ok(Workspace::new()?.library()?.qlty_config_path())
    }
}
