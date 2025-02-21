use super::WorkspaceEntryMatcher;
use crate::{utils::fs::path_to_string, WorkspaceEntry};
use std::path::PathBuf;

/// Matches workspace entries that are within a path prefix (a directory)
#[derive(Debug)]
pub struct PrefixMatcher {
    path_prefix: String,
    root: PathBuf,
}

impl PrefixMatcher {
    pub fn new(path_prefix: String, root: PathBuf) -> Self {
        Self { path_prefix, root }
    }
}

impl WorkspaceEntryMatcher for PrefixMatcher {
    fn matches(&self, workspace_entry: WorkspaceEntry) -> Option<WorkspaceEntry> {
        match workspace_entry.full_path(&self.root) {
            Ok(full_path) => {
                if full_path.starts_with(path_to_string(&self.path_prefix)) {
                    Some(workspace_entry)
                } else {
                    None
                }
            }
            Err(_e) => None,
        }
    }
}
