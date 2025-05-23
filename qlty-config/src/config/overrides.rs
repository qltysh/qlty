use super::IssueMode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// DEPRECATED -- Use Triage instead
#[derive(Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
pub struct Override {
    #[serde(default)]
    pub level: Option<String>,

    #[serde(default)]
    pub category: Option<String>,

    #[serde(default)]
    pub plugins: Vec<String>,

    #[serde(default)]
    pub rules: Vec<String>,

    #[serde(default)]
    pub file_patterns: Vec<String>,

    #[serde(default)]
    pub mode: Option<IssueMode>,
}
