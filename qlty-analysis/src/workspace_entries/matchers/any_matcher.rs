use super::WorkspaceEntryMatcher;
use crate::WorkspaceEntry;

#[derive(Debug)]
pub struct AnyMatcher;

impl WorkspaceEntryMatcher for AnyMatcher {
    fn matches(&self, workspace_entry: WorkspaceEntry) -> Option<WorkspaceEntry> {
        Some(workspace_entry)
    }
}
