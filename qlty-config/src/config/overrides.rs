use super::IssueMode;
use crate::config::issue_transformer::IssueTransformer;
use globset::{Glob, GlobSet, GlobSetBuilder};
use qlty_types::category_from_str;
use qlty_types::{analysis::v1::Issue, level_from_str};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, Default, JsonSchema)]
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

    #[serde(skip)]
    glob_set: RwLock<Option<GlobSet>>,
}

impl Clone for Override {
    fn clone(&self) -> Self {
        Self {
            level: self.level.clone(),
            category: self.category.clone(),
            plugins: self.plugins.clone(),
            rules: self.rules.clone(),
            file_patterns: self.file_patterns.clone(),
            mode: self.mode,
            glob_set: RwLock::new(None),
        }
    }
}

impl IssueTransformer for Override {
    fn initialize(&self) {
        let mut globset_builder = GlobSetBuilder::new();

        for glob in &self.file_patterns {
            globset_builder.add(Glob::new(glob).unwrap());
        }

        let mut glob_set = self.glob_set.write().unwrap();
        *glob_set = Some(globset_builder.build().unwrap());
    }

    fn transform(&self, issue: Issue) -> Option<Issue> {
        if self.applies_to_issue(&issue) {
            let mut new_issue = issue.clone();

            if let Some(level) = &self.level {
                new_issue.level = level_from_str(level).into();
            }

            if let Some(category) = &self.category {
                new_issue.category = category_from_str(category).into();
            }

            if let Some(mode) = &self.mode {
                new_issue.mode = *mode as i32;
            }

            Some(new_issue)
        } else {
            Some(issue)
        }
    }

    fn clone_box(&self) -> Box<dyn IssueTransformer> {
        Box::new(self.clone())
    }
}

impl Override {
    fn applies_to_issue(&self, issue: &Issue) -> bool {
        self.plugin_applies_to_issue(issue)
            && self.rule_applies_to_issue(issue)
            && self.glob_applies_to_issue(issue)
    }

    fn plugin_applies_to_issue(&self, issue: &Issue) -> bool {
        self.plugins.is_empty() || self.plugins.contains(&issue.tool.to_string())
    }

    fn rule_applies_to_issue(&self, issue: &Issue) -> bool {
        self.rules.is_empty() || self.rules.contains(&issue.rule_key.to_string())
    }

    fn glob_applies_to_issue(&self, issue: &Issue) -> bool {
        if self.file_patterns.is_empty() {
            return true;
        }

        let glob_set = self.glob_set.read().unwrap();

        if let Some(path) = issue.path() {
            glob_set.as_ref().unwrap().is_match(path)
        } else {
            // TODO: Issues without a path are not filterable
            false
        }
    }
}
