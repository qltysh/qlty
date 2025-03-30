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
        // Write metadata using JsonFormatter
        let metadata_formatter = JsonFormatter::new(self.report.metadata.clone());
        metadata_formatter.write_to_file(&self.path.join("metadata.json"))?;

        // Write messages using JsonEachRowFormatter
        let messages_formatter = JsonEachRowFormatter::new(self.report.messages.clone());
        messages_formatter.write_to_file(&self.path.join("messages.jsonl"))?;

        // Write invocations using InvocationJsonFormatter
        let invocations_formatter = InvocationJsonFormatter::new(self.report.invocations.clone());
        invocations_formatter.write_to_file(&self.path.join("invocations.jsonl"))?;

        // Write issues using JsonEachRowFormatter
        let issues_formatter = JsonEachRowFormatter::new(self.report.issues.clone());
        issues_formatter.write_to_file(&self.path.join("issues.jsonl"))?;

        // Write stats using JsonEachRowFormatter
        let stats_formatter = JsonEachRowFormatter::new(self.report.stats.clone());
        stats_formatter.write_to_file(&self.path.join("stats.jsonl"))?;

        // Write config using CopyFormatter
        let config_path = Self::qlty_config_path()?;
        let copy_formatter = CopyFormatter::new(config_path);
        copy_formatter.write_to_file(&self.path.join("qlty.toml"))?;

        Ok(())
    }

    fn export_json_gz(&self) -> Result<()> {
        // Write metadata using JsonFormatter
        let metadata_formatter = JsonFormatter::new(self.report.metadata.clone());
        metadata_formatter.write_to_file(&self.path.join("metadata.json"))?;

        // Write messages using GzFormatter wrapping JsonEachRowFormatter
        let messages_formatter = JsonEachRowFormatter::new(self.report.messages.clone());
        let gz_messages_formatter = GzFormatter::new(Box::new(messages_formatter));
        gz_messages_formatter.write_to_file(&self.path.join("messages.json.gz"))?;

        // Write invocations using GzFormatter wrapping InvocationJsonFormatter
        let invocations_formatter = InvocationJsonFormatter::new(self.report.invocations.clone());
        let gz_invocations_formatter = GzFormatter::new(Box::new(invocations_formatter));
        gz_invocations_formatter.write_to_file(&self.path.join("invocations.json.gz"))?;

        // Write issues using GzFormatter wrapping JsonEachRowFormatter
        let issues_formatter = JsonEachRowFormatter::new(self.report.issues.clone());
        let gz_issues_formatter = GzFormatter::new(Box::new(issues_formatter));
        gz_issues_formatter.write_to_file(&self.path.join("issues.json.gz"))?;

        // Write stats using GzFormatter wrapping JsonEachRowFormatter
        let stats_formatter = JsonEachRowFormatter::new(self.report.stats.clone());
        let gz_stats_formatter = GzFormatter::new(Box::new(stats_formatter));
        gz_stats_formatter.write_to_file(&self.path.join("stats.json.gz"))?;

        // Write config using GzFormatter wrapping CopyFormatter
        let config_path = Self::qlty_config_path()?;
        let copy_formatter = CopyFormatter::new(config_path);
        let gz_copy_formatter = GzFormatter::new(Box::new(copy_formatter));
        gz_copy_formatter.write_to_file(&self.path.join("qlty.toml.gz"))?;

        Ok(())
    }

    fn qlty_config_path() -> Result<PathBuf> {
        Ok(Workspace::new()?.library()?.qlty_config_path())
    }
}
