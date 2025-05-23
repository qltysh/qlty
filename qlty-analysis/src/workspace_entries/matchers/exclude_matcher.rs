use crate::{utils::fs::path_to_string, WorkspaceEntry};
use qlty_config::config::Exclude;
use std::path::PathBuf;

use super::WorkspaceEntryMatcher;

#[derive(Debug)]
pub struct ExcludeMatcher {
    exclude: Exclude,
    root: PathBuf,
}

impl ExcludeMatcher {
    pub fn new(exclude: Exclude, root: PathBuf) -> Self {
        exclude.initialize_globset();

        Self { exclude, root }
    }
}

impl WorkspaceEntryMatcher for ExcludeMatcher {
    fn matches(&self, entry: WorkspaceEntry, tool_name: &str) -> Option<WorkspaceEntry> {
        let path_str = path_to_string(entry.path.strip_prefix(&self.root).unwrap_or(&entry.path));

        if self.exclude.plugins.contains(&tool_name.to_string())
            && self.exclude.matches_path(&path_str)
        {
            None
        } else {
            Some(entry)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WorkspaceEntryKind;
    use std::time::SystemTime;

    #[test]
    fn test_matches_excludes_file_for_specific_plugin() {
        let plugin_name = "test-plugin".to_string();
        let root = PathBuf::from("/project");

        let mut exclude = Exclude::default();
        exclude.file_patterns = vec!["*.log".to_string()];
        exclude.plugins = vec![plugin_name.clone()];

        let matcher = ExcludeMatcher::new(exclude, root.clone());

        let entry = WorkspaceEntry {
            path: root.join("file.log"),
            kind: WorkspaceEntryKind::File,
            content_modified: SystemTime::now(),
            contents_size: 100,
            language_name: None,
        };

        let result = matcher.matches(entry, &plugin_name);

        assert!(result.is_none(), "Expected file.log to be excluded");
    }

    #[test]
    fn test_does_not_exclude_file_for_different_plugin() {
        let plugin_name = "test-plugin".to_string();
        let other_plugin = "other-plugin".to_string();
        let root = PathBuf::from("/project");

        let mut exclude = Exclude::default();
        exclude.file_patterns = vec!["*.log".to_string()];
        exclude.plugins = vec![other_plugin];

        let matcher = ExcludeMatcher::new(exclude, root.clone());

        let entry = WorkspaceEntry {
            path: root.join("file.log"),
            kind: WorkspaceEntryKind::File,
            content_modified: SystemTime::now(),
            contents_size: 100,
            language_name: None,
        };

        let result = matcher.matches(entry, &plugin_name);

        assert!(
            result.is_some(),
            "Expected file.log not to be excluded for a different plugin"
        );
    }

    #[test]
    fn test_does_not_exclude_non_matching_file() {
        let plugin_name = "test-plugin".to_string();
        let root = PathBuf::from("/project");

        let mut exclude = Exclude::default();
        exclude.file_patterns = vec!["*.log".to_string()];
        exclude.plugins = vec![plugin_name.clone()];

        let matcher = ExcludeMatcher::new(exclude, root.clone());

        let entry = WorkspaceEntry {
            path: root.join("file.txt"),
            kind: WorkspaceEntryKind::File,
            content_modified: SystemTime::now(),
            contents_size: 100,
            language_name: None,
        };

        let result = matcher.matches(entry, &plugin_name);

        assert!(result.is_some(), "Expected file.txt not to be excluded");
    }

    #[test]
    fn test_correctly_strips_root_prefix() {
        let plugin_name = "test-plugin".to_string();
        let root = PathBuf::from("/project");

        let mut exclude = Exclude::default();
        exclude.file_patterns = vec!["src/*.log".to_string()];
        exclude.plugins = vec![plugin_name.clone()];

        let matcher = ExcludeMatcher::new(exclude, root.clone());

        let entry_in_src = WorkspaceEntry {
            path: root.join("src/file.log"),
            kind: WorkspaceEntryKind::File,
            content_modified: SystemTime::now(),
            contents_size: 100,
            language_name: None,
        };

        let entry_in_root = WorkspaceEntry {
            path: root.join("file.log"),
            kind: WorkspaceEntryKind::File,
            content_modified: SystemTime::now(),
            contents_size: 100,
            language_name: None,
        };

        assert!(
            matcher.matches(entry_in_src, &plugin_name).is_none(),
            "Expected src/file.log to be excluded"
        );
        assert!(
            matcher.matches(entry_in_root, &plugin_name).is_some(),
            "Expected root/file.log not to be excluded"
        );
    }
}
