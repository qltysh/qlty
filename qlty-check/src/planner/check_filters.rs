use crate::{CheckFilter, Settings};
use qlty_config::config::issue_transformer::IssueTransformer;
use qlty_types::analysis::v1::Issue;

#[derive(Debug, Clone)]
pub struct CheckFilters {
    pub filters: Vec<CheckFilter>,
    pub skips: Vec<CheckFilter>,
}

impl CheckFilters {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            filters: settings.filters.clone(),
            skips: settings.skips.clone(),
        }
    }
}

impl IssueTransformer for CheckFilters {
    fn transform(&self, issue: Issue) -> Option<Issue> {
        if self.skips.iter().any(|skip| skip.matches_issue(&issue)) {
            return None;
        }

        if self.filters.is_empty()
            || self
                .filters
                .iter()
                .any(|filter| filter.matches_issue(&issue))
        {
            Some(issue)
        } else {
            None
        }
    }

    fn clone_box(&self) -> Box<dyn IssueTransformer> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue(tool: &str, rule_key: &str) -> Issue {
        Issue {
            tool: tool.to_string(),
            rule_key: rule_key.to_string(),
            ..Default::default()
        }
    }

    fn check_filter(value: &str) -> CheckFilter {
        value.to_string().into()
    }

    fn transformer(filters: &[&str], skips: &[&str]) -> CheckFilters {
        CheckFilters {
            filters: filters.iter().map(|value| check_filter(value)).collect(),
            skips: skips.iter().map(|value| check_filter(value)).collect(),
        }
    }

    #[test]
    fn test_no_filters_or_skips_keeps_issue() {
        let result = transformer(&[], &[]).transform(issue("eslint", "no-unused-vars"));

        assert!(result.is_some());
    }

    #[test]
    fn test_skip_plugin_drops_its_issues() {
        let result = transformer(&[], &["eslint"]).transform(issue("eslint", "no-unused-vars"));

        assert!(result.is_none());
    }

    #[test]
    fn test_skip_plugin_keeps_other_plugins() {
        let result =
            transformer(&[], &["eslint"]).transform(issue("rubocop", "Style/StringLiterals"));

        assert!(result.is_some());
    }

    #[test]
    fn test_skip_rule_drops_matching_rule() {
        let result = transformer(&[], &["eslint:no-unused-vars"])
            .transform(issue("eslint", "no-unused-vars"));

        assert!(result.is_none());
    }

    #[test]
    fn test_skip_rule_keeps_other_rules() {
        let result =
            transformer(&[], &["eslint:no-unused-vars"]).transform(issue("eslint", "no-console"));

        assert!(result.is_some());
    }

    #[test]
    fn test_skip_applies_within_filter() {
        let result = transformer(&["eslint"], &["eslint:no-unused-vars"])
            .transform(issue("eslint", "no-unused-vars"));

        assert!(result.is_none());
    }

    #[test]
    fn test_filter_keeps_matching_issue() {
        let result = transformer(&["eslint"], &[]).transform(issue("eslint", "no-unused-vars"));

        assert!(result.is_some());
    }

    #[test]
    fn test_filter_drops_unmatched_issue() {
        let result =
            transformer(&["eslint"], &[]).transform(issue("rubocop", "Style/StringLiterals"));

        assert!(result.is_none());
    }
}
