use std::path::PathBuf;

use crate::formats::Formats;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Settings {
    pub add_prefix: Option<String>,
    pub dry_run: bool,
    pub incomplete: bool,
    pub name: Option<String>,
    pub output_dir: Option<PathBuf>,
    pub override_branch: Option<String>,
    pub override_build_id: Option<String>,
    pub override_commit_sha: Option<String>,
    pub override_pull_request_number: Option<String>,
    pub project: Option<String>,
    pub quiet: bool,
    pub report_format: Option<Formats>,
    pub skip_missing_files: bool,
    pub strip_prefix: Option<String>,
    pub tag: Option<String>,
    pub total_parts_count: Option<u32>,

    pub paths: Vec<String>,
}
