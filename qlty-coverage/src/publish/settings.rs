use crate::formats::Formats;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Settings {
    pub tag: Option<String>,

    pub override_build_id: Option<String>,
    pub override_branch: Option<String>,
    pub override_commit_sha: Option<String>,
    pub override_pull_request_number: Option<String>,

    pub paths: Vec<String>,

    pub report_format: Option<Formats>,

    pub add_prefix: Option<String>,
    pub strip_prefix: Option<String>,

    pub skip_missing_files: bool,
    pub total_parts_count: Option<u32>,
}
