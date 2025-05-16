use crate::{utils::fs::path_to_string, WorkspaceEntry};
use qlty_config::config::Exclude;
use std::path::PathBuf;

use super::WorkspaceEntryMatcher;

#[derive(Debug)]
pub struct PluginSpecificExcludeMatcher {
    plugin_name: String,
    excludes: Vec<Exclude>,
    root: PathBuf,
}

impl PluginSpecificExcludeMatcher {
    pub fn new(plugin_name: String, excludes: Vec<Exclude>, root: PathBuf) -> Self {
        for exclude in &excludes {
            exclude.initialize_globset();
        }

        Self {
            plugin_name,
            excludes,
            root,
        }
    }
}

impl WorkspaceEntryMatcher for PluginSpecificExcludeMatcher {
    fn matches(&self, entry: WorkspaceEntry) -> Option<WorkspaceEntry> {
        let path_str = path_to_string(entry.path.strip_prefix(&self.root).unwrap_or(&entry.path));

        if self.excludes.iter().any(|exclude| {
            exclude.plugins.contains(&self.plugin_name) && exclude.matches_path(&path_str)
        }) {
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

        let matcher =
            PluginSpecificExcludeMatcher::new(plugin_name.clone(), vec![exclude], root.clone());

        let entry = WorkspaceEntry {
            path: root.join("file.log"),
            kind: WorkspaceEntryKind::File,
            content_modified: SystemTime::now(),
            contents_size: 100,
            language_name: None,
        };

        let result = matcher.matches(entry);

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

        let matcher = PluginSpecificExcludeMatcher::new(plugin_name, vec![exclude], root.clone());

        let entry = WorkspaceEntry {
            path: root.join("file.log"),
            kind: WorkspaceEntryKind::File,
            content_modified: SystemTime::now(),
            contents_size: 100,
            language_name: None,
        };

        let result = matcher.matches(entry);

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

        let matcher = PluginSpecificExcludeMatcher::new(plugin_name, vec![exclude], root.clone());

        let entry = WorkspaceEntry {
            path: root.join("file.txt"),
            kind: WorkspaceEntryKind::File,
            content_modified: SystemTime::now(),
            contents_size: 100,
            language_name: None,
        };

        let result = matcher.matches(entry);

        assert!(result.is_some(), "Expected file.txt not to be excluded");
    }

    #[test]
    fn test_handles_multiple_excludes() {
        let plugin_name = "test-plugin".to_string();
        let root = PathBuf::from("/project");

        let mut exclude1 = Exclude::default();
        exclude1.file_patterns = vec!["*.log".to_string()];
        exclude1.plugins = vec![plugin_name.clone()];

        let mut exclude2 = Exclude::default();
        exclude2.file_patterns = vec!["*.tmp".to_string()];
        exclude2.plugins = vec![plugin_name.clone()];

        let matcher =
            PluginSpecificExcludeMatcher::new(plugin_name, vec![exclude1, exclude2], root.clone());

        let entry1 = WorkspaceEntry {
            path: root.join("file.log"),
            kind: WorkspaceEntryKind::File,
            content_modified: SystemTime::now(),
            contents_size: 100,
            language_name: None,
        };

        let entry2 = WorkspaceEntry {
            path: root.join("file.tmp"),
            kind: WorkspaceEntryKind::File,
            content_modified: SystemTime::now(),
            contents_size: 100,
            language_name: None,
        };

        let entry3 = WorkspaceEntry {
            path: root.join("file.txt"),
            kind: WorkspaceEntryKind::File,
            content_modified: SystemTime::now(),
            contents_size: 100,
            language_name: None,
        };

        assert!(
            matcher.matches(entry1).is_none(),
            "Expected file.log to be excluded"
        );
        assert!(
            matcher.matches(entry2).is_none(),
            "Expected file.tmp to be excluded"
        );
        assert!(
            matcher.matches(entry3).is_some(),
            "Expected file.txt not to be excluded"
        );
    }

    #[test]
    fn test_correctly_strips_root_prefix() {
        let plugin_name = "test-plugin".to_string();
        let root = PathBuf::from("/project");

        let mut exclude = Exclude::default();
        exclude.file_patterns = vec!["src/*.log".to_string()];
        exclude.plugins = vec![plugin_name.clone()];

        let matcher = PluginSpecificExcludeMatcher::new(plugin_name, vec![exclude], root.clone());

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
            matcher.matches(entry_in_src).is_none(),
            "Expected src/file.log to be excluded"
        );
        assert!(
            matcher.matches(entry_in_root).is_some(),
            "Expected root/file.log not to be excluded"
        );
    }
}
