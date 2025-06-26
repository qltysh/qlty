use anyhow::Result;
use qlty_analysis::Report;
use qlty_config::Workspace;
use qlty_formats::{
    CopyFormatter, Formatter, GzFormatter, InvocationJsonFormatter, JsonEachRowFormatter,
    JsonFormatter,
};
use std::path::{Path, PathBuf};
use tracing::info;

const INVOCATION_BATCH_SIZE: usize = 200;
const ISSUES_BATCH_SIZE: usize = 2000;
const MESSAGES_BATCH_SIZE: usize = 10000;
const STATS_BATCH_SIZE: usize = 10000;

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

        // Write messages using JsonEachRowFormatter after breaking into chunks
        // to avoid memory issues with large reports, particularly during ingestion
        for (i, chunk) in self.report.messages.chunks(MESSAGES_BATCH_SIZE).enumerate() {
            let filename = format!("messages-{i:03}.jsonl");
            let messages_formatter = JsonEachRowFormatter::new(chunk.to_vec());
            messages_formatter.write_to_file(&self.path.join(filename))?;
        }

        // Write invocations using InvocationJsonFormatter after breaking into chunks
        // to avoid memory issues with large reports, particularly during ingestion
        for (i, chunk) in self
            .report
            .invocations
            .chunks(INVOCATION_BATCH_SIZE)
            .enumerate()
        {
            let filename = format!("invocations-{i:03}.jsonl");
            let invocations_formatter = InvocationJsonFormatter::new(chunk.to_vec());
            invocations_formatter.write_to_file(&self.path.join(filename))?;
        }

        // Write issues using JsonEachRowFormatter after breaking into chunks
        // to avoid memory issues with large reports, particularly during ingestion
        for (i, chunk) in self.report.issues.chunks(ISSUES_BATCH_SIZE).enumerate() {
            let filename = format!("issues-{i:03}.jsonl");
            let issues_formatter = JsonEachRowFormatter::new(chunk.to_vec());
            issues_formatter.write_to_file(&self.path.join(filename))?;
        }

        // Write stats using JsonEachRowFormatter after breaking into chunks
        // to avoid memory issues with large reports, particularly during ingestion
        for (i, chunk) in self.report.stats.chunks(STATS_BATCH_SIZE).enumerate() {
            let filename = format!("stats-{i:03}.jsonl");
            let stats_formatter = JsonEachRowFormatter::new(chunk.to_vec());
            stats_formatter.write_to_file(&self.path.join(filename))?;
        }

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
