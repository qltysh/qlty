use super::{GlobsMatcher, WorkspaceEntryMatcher};
use crate::WorkspaceEntry;
use qlty_config::config::exclude_group::ExcludeGroup;

#[derive(Default, Debug)]
pub struct ExcludeGroupsMatcher {
    matchers: Vec<GlobsMatcher>,
}

impl ExcludeGroupsMatcher {
    pub fn new(exclude_groups: Vec<ExcludeGroup>) -> Self {
        let matchers = exclude_groups
            .into_iter()
            .filter_map(|exclude_group| {
                GlobsMatcher::new_for_globs(&exclude_group.excludes, exclude_group.negate).ok()
            })
            .collect();

        Self { matchers }
    }
}

impl WorkspaceEntryMatcher for ExcludeGroupsMatcher {
    fn matches(&self, workspace_entry: WorkspaceEntry) -> Option<WorkspaceEntry> {
        // By default, include the file
        let path = workspace_entry.path_string();

        for matcher in self.matchers.iter().rev() {
            if matcher.glob_set.is_match(&path) {
                return if matcher.include {
                    Some(workspace_entry) // No need to clone; directly return ownership
                } else {
                    None // Immediately exclude the file if a normal exclude rule matches
                };
            }
        }

        Some(workspace_entry)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{WorkspaceEntry, WorkspaceEntryKind};
    use qlty_config::config::exclude_group::ExcludeGroup;
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn create_workspace_entry(path: &str) -> WorkspaceEntry {
        WorkspaceEntry {
            path: PathBuf::from(path),
            kind: WorkspaceEntryKind::File,
            content_modified: SystemTime::now(),
            contents_size: 100,
            language_name: None,
        }
    }

    #[test]
    fn test_exclude_groups_matcher_excludes_excluded_paths() {
        let exclude_groups = vec![ExcludeGroup {
            excludes: vec!["logs/**".to_string(), "target/**".to_string()],
            negate: false,
        }];
        let matcher = ExcludeGroupsMatcher::new(exclude_groups);

        assert!(matcher
            .matches(create_workspace_entry("src/main.rs"))
            .is_some());
        assert!(matcher
            .matches(create_workspace_entry("logs/output.log"))
            .is_none());
        assert!(matcher
            .matches(create_workspace_entry("target/debug/app"))
            .is_none());
    }

    #[test]
    fn test_exclude_groups_matcher_allows_negated_patterns() {
        let exclude_groups = vec![
            ExcludeGroup {
                excludes: vec!["logs/**".to_string()],
                negate: false,
            },
            ExcludeGroup {
                excludes: vec!["logs/important.log".to_string()],
                negate: true,
            },
        ];
        let matcher = ExcludeGroupsMatcher::new(exclude_groups);

        assert!(matcher
            .matches(create_workspace_entry("logs/error.log"))
            .is_none());
        assert!(matcher
            .matches(create_workspace_entry("logs/important.log"))
            .is_some());
    }

    #[test]
    fn test_exclude_groups_matcher_empty_list() {
        let matcher = ExcludeGroupsMatcher::new(vec![]);
        assert!(matcher
            .matches(create_workspace_entry("src/main.rs"))
            .is_some());
    }
}
