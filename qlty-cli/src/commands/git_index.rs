use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::Args;
use qlty_history::GitIndexOptions;
use std::path::PathBuf;
use tracing::info;

#[derive(Args, Debug)]
pub struct GitIndex {
    /// Skip commits without parents (except the initial commit)
    #[arg(long, default_value = "true")]
    pub skip_commits_without_parents: bool,

    /// Skip commits with duplicate diffs
    #[arg(long, default_value = "true")]
    pub skip_commits_with_duplicate_diffs: bool,

    /// Skip paths that match this regular expression
    #[arg(long)]
    pub skip_paths: Option<String>,

    /// Skip commits whose messages match this regular expression
    #[arg(long)]
    pub skip_commits_with_messages: Option<String>,

    /// Skip commits with diffs larger than this size (number of added + removed lines)
    #[arg(long, default_value = "100000")]
    pub diff_size_limit: usize,

    /// Number of threads to use for processing
    #[arg(long)]
    pub threads: Option<usize>,

    /// Output directory for TSV files
    #[arg(long, default_value = ".")]
    pub output_dir: PathBuf,

    /// Skip specific commits by hash
    #[arg(long)]
    pub skip_commit: Vec<String>,

    /// Stop processing after this commit hash
    #[arg(long)]
    pub stop_after_commit: Option<String>,
}

impl GitIndex {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        info!("Starting git-index command");

        let options = GitIndexOptions {
            skip_commits_without_parents: self.skip_commits_without_parents,
            skip_commits_with_duplicate_diffs: self.skip_commits_with_duplicate_diffs,
            skip_paths: self.skip_paths.clone(),
            skip_commits_with_messages: self.skip_commits_with_messages.clone(),
            diff_size_limit: self.diff_size_limit,
            threads: self.threads,
            output_dir: self.output_dir.clone(),
            skip_commit: self.skip_commit.clone(),
            stop_after_commit: self.stop_after_commit.clone(),
        };

        let git_index = qlty_history::GitIndex::new(options);
        git_index
            .run()
            .map_err(|e| CommandError::Unknown { source: e })?;

        CommandSuccess::ok()
    }
}
