use super::ignore::is_rule_issue_match;
use super::plugin::PluginMode;
use crate::config::issue_transformer::IssueTransformer;
use globset::{Glob, GlobSet, GlobSetBuilder};
use qlty_types::analysis::v1::{Category, Issue, Level};
use qlty_types::{category_from_str, level_from_str};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, Default, JsonSchema)]
pub struct Match {
    #[serde(default)]
    pub plugins: Vec<String>,

    #[serde(default)]
    pub rules: Vec<String>,

    #[serde(default)]
    pub file_patterns: Vec<String>,

    #[serde(skip)]
    pub glob_set: RwLock<Option<GlobSet>>,
}

impl Clone for Match {
    fn clone(&self) -> Self {
        Self {
            plugins: self.plugins.clone(),
            rules: self.rules.clone(),
            file_patterns: self.file_patterns.clone(),
            glob_set: RwLock::new(None),
        }
    }
}

#[derive(Debug, Serialize, Default, Clone, JsonSchema)]
pub struct Set {
    #[serde(default)]
    pub level: Option<Level>,

    #[serde(default)]
    pub category: Option<Category>,

    #[serde(default)]
    pub mode: Option<PluginMode>,

    #[serde(default)]
    pub ignored: bool,
}

impl<'de> Deserialize<'de> for Set {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SetHelper {
            #[serde(default)]
            level: Option<String>,

            #[serde(default)]
            category: Option<String>,

            #[serde(default)]
            mode: Option<PluginMode>,

            #[serde(default)]
            ignored: bool,
        }

        let helper = SetHelper::deserialize(deserializer)?;

        Ok(Set {
            level: helper.level.as_deref().map(level_from_str),
            category: helper.category.as_deref().map(category_from_str),
            mode: helper.mode,
            ignored: helper.ignored,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
pub struct Triage {
    #[serde(default)]
    #[serde(rename = "match")]
    pub _match: Match,

    #[serde(default)]
    pub set: Set,
}

impl Match {
    fn initialize(&self) {
        let mut globset_builder = GlobSetBuilder::new();

        for glob in &self.file_patterns {
            globset_builder.add(Glob::new(glob).unwrap());
        }

        let mut glob_set = self.glob_set.write().unwrap();
        *glob_set = Some(globset_builder.build().unwrap());
    }

    fn applies_to_issue(&self, issue: &Issue) -> bool {
        self.plugin_applies_to_issue(issue)
            && is_rule_issue_match(&self.rules, issue)
            && self.glob_applies_to_issue(issue)
    }

    fn plugin_applies_to_issue(&self, issue: &Issue) -> bool {
        self.plugins.is_empty() || self.plugins.contains(&issue.tool.to_string())
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

impl IssueTransformer for Triage {
    fn initialize(&self) {
        self._match.initialize();
    }

    fn transform(&self, issue: Issue) -> Option<Issue> {
        if self._match.applies_to_issue(&issue) {
            if self.set.ignored {
                return None;
            }

            let mut new_issue = issue.clone();

            if let Some(level) = &self.set.level {
                new_issue.level = *level as i32;
            }

            if let Some(category) = &self.set.category {
                new_issue.category = *category as i32;
            }

            if let Some(mode) = &self.set.mode {
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
