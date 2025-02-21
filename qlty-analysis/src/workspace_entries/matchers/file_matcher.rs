use super::WorkspaceEntryMatcher;
use crate::{WorkspaceEntry, WorkspaceEntryKind};

/// Matches workspace entries that are of type `WorkspaceEntryKind::File`
#[derive(Debug)]
pub struct FileMatcher;

impl WorkspaceEntryMatcher for FileMatcher {
    fn matches(&self, workspace_entry: WorkspaceEntry) -> Option<WorkspaceEntry> {
        if workspace_entry.kind == WorkspaceEntryKind::File {
            Some(workspace_entry)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::WorkspaceEntryKind;
    use std::path::PathBuf;
    use std::time::SystemTime;

    // Tests for FileWorkspaceEntryMatcher
    #[test]
    fn test_file_workspace_entry_matcher_matches_file() {
        let workspace_entry = WorkspaceEntry {
            path: PathBuf::from("/path/to/file.txt"),
            kind: WorkspaceEntryKind::File,
            content_modified: SystemTime::now(),
            contents_size: 100,
            language_name: None,
        };

        let matcher = FileMatcher;
        assert!(
            matcher.matches(workspace_entry).is_some(),
            "Expected workspace_entry to match as it is of type File"
        );
    }

    #[test]
    fn test_file_workspace_entry_matcher_does_not_match_directory() {
        let workspace_entry = WorkspaceEntry {
            path: PathBuf::from("/path/to/directory/"),
            kind: WorkspaceEntryKind::Directory,
            content_modified: SystemTime::now(),
            contents_size: 0, // Assuming directories have a size of 0
            language_name: None,
        };

        let matcher = FileMatcher;
        assert!(
            matcher.matches(workspace_entry).is_none(),
            "Expected workspace_entry not to match as it is of type Directory"
        );
    }
}
