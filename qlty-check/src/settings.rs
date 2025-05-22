use qlty_config::config::CheckTrigger;
use qlty_types::analysis::v1::{Issue, Level};
use std::{fmt::Formatter, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Settings {
    pub root: PathBuf,
    pub all: bool,
    pub fix: bool,
    pub ai: bool,
    pub r#unsafe: bool,
    pub verbose: usize,
    pub progress: bool,
    pub formatters: bool,
    pub cache: bool,
    pub jobs: Option<u32>,
    pub sample: Option<usize>,
    pub filters: Vec<CheckFilter>,
    pub upstream: Option<String>,
    pub index: bool,
    pub index_file: Option<PathBuf>,
    pub level: Level,
    pub fail_level: Option<Level>,
    pub paths: Vec<PathBuf>,
    pub trigger: CheckTrigger,
    pub skip_errored_plugins: bool,
    pub emit_existing_issues: bool,
    pub auth_token: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            root: std::env::current_dir().expect("current dir"),
            all: false,
            fix: false,
            ai: false,
            r#unsafe: false,
            verbose: 0,
            progress: true,
            formatters: true,
            cache: true,
            jobs: None,
            sample: None,
            filters: vec![],
            upstream: None,
            index: false,
            index_file: None,
            level: Level::Unspecified,
            fail_level: Some(Level::Fmt),
            paths: vec![],
            trigger: CheckTrigger::Manual,
            skip_errored_plugins: false,
            emit_existing_issues: false,
            auth_token: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CheckFilter {
    pub plugin: String,
    pub rule_key: Option<String>,
}

impl CheckFilter {
    pub fn from_optional_list(list: Option<String>) -> Vec<Self> {
        list.map_or_else(Vec::new, |s| Self::from_list(&s))
    }

    // Converts a comma-separated list of strings into a Vec<CheckFilter>
    fn from_list(list: &str) -> Vec<Self> {
        list.split(',').map(|s| s.to_string().into()).collect()
    }

    pub fn matches_issue(&self, issue: &Issue) -> bool {
        if let Some(expected_rule_key) = &self.rule_key {
            self.matches_tool(&issue.tool) && issue.rule_key == *expected_rule_key
        } else {
            self.matches_tool(&issue.tool)
        }
    }

    fn matches_tool(&self, issue_tool: &str) -> bool {
        self.plugin == issue_tool || self.plugin.replace('-', "_") == issue_tool
    }
}

impl std::fmt::Display for CheckFilter {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match &self.rule_key {
            Some(rule_key) => write!(f, "{}:{}", self.plugin, rule_key),
            None => write!(f, "{}", self.plugin),
        }
    }
}

// Converts a string in the form of "plugin_name" or "plugin_name:rule_key"
// into a single CheckFilter
impl From<String> for CheckFilter {
    fn from(string: String) -> Self {
        let parts: Vec<&str> = string.split(':').collect();
        let plugin = parts[0].to_string();
        let rule_key = parts.get(1).map(|s| s.to_string());
        Self { plugin, rule_key }
    }
}
