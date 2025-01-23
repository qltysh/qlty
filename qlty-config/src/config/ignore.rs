use std::sync::RwLock;

use crate::config::issue_transformer::IssueTransformer;
use globset::{Glob, GlobSet, GlobSetBuilder};
use qlty_types::analysis::v1::Issue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const ALL_WILDCARD: &'static str = "ALL";

#[derive(Debug, Serialize, Deserialize, Default, JsonSchema)]
pub struct Ignore {
    #[serde(default = "default_ignore_file_patterns")]
    pub file_patterns: Vec<String>,

    #[serde(default)]
    pub plugins: Vec<String>,

    #[serde(default)]
    pub rules: Vec<String>,

    #[serde(default)]
    pub levels: Vec<String>,

    #[serde(skip)]
    pub glob_set: RwLock<Option<GlobSet>>,
}

impl Clone for Ignore {
    fn clone(&self) -> Self {
        Self {
            file_patterns: self.file_patterns.clone(),
            plugins: self.plugins.clone(),
            rules: self.rules.clone(),
            levels: self.levels.clone(),
            glob_set: RwLock::new(None),
        }
    }
}

fn default_ignore_file_patterns() -> Vec<String> {
    vec!["*".to_string()]
}

impl IssueTransformer for Ignore {
    fn initialize(&self) {
        self.initialize_globset();
    }

    fn transform(&self, issue: Issue) -> Option<Issue> {
        if self.applies_to_issue(issue.clone()) {
            None
        } else {
            Some(issue)
        }
    }

    fn clone_box(&self) -> Box<dyn IssueTransformer> {
        Box::new(self.clone())
    }
}

impl Ignore {
    pub fn initialize_globset(&self) {
        let mut globset_builder = GlobSetBuilder::new();

        for glob in &self.file_patterns {
            globset_builder.add(Glob::new(glob).unwrap());
        }

        let mut glob_set = self.glob_set.write().unwrap();
        *glob_set = Some(globset_builder.build().unwrap());
    }

    pub fn matches_path(&self, path: &str) -> bool {
        if self.file_patterns.is_empty() {
            return true;
        }

        let glob_set = self.glob_set.read().unwrap();

        if let Some(glob_set) = glob_set.as_ref() {
            glob_set.is_match(path)
        } else {
            false
        }
    }

    fn applies_to_issue(&self, issue: Issue) -> bool {
        self.plugin_applies_to_issue(&issue)
            && self.rule_applies_to_issue(&issue)
            && self.glob_applies_to_issue(&issue)
            && self.level_applies_to_issue(&issue)
    }

    fn level_applies_to_issue(&self, issue: &Issue) -> bool {
        if self.levels.is_empty() {
            return true;
        }
        self.levels.contains(&self.level_as_string(issue.level))
    }

    fn plugin_applies_to_issue(&self, issue: &Issue) -> bool {
        if self.plugins.is_empty() {
            return true;
        }
        self.plugins.contains(&issue.tool.to_string())
    }

    fn rule_applies_to_issue(&self, issue: &Issue) -> bool {
        if self.rules.is_empty() {
            return true;
        }
        self.rules.contains(&issue.rule_key.to_string())
    }

    fn glob_applies_to_issue(&self, issue: &Issue) -> bool {
        if let Some(path) = issue.path() {
            self.matches_path(&path)
        } else {
            // TODO: Issues without a path are not filterable
            false
        }
    }

    fn level_as_string(&self, level: i32) -> String {
        match level {
            0 => "unspecified".to_string(),
            1 => "fmt".to_string(),
            2 => "low".to_string(),
            3 => "medium".to_string(),
            4 => "high".to_string(),
            _ => "unspecified".to_string(),
        }
    }
}
