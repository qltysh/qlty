use super::WorkspaceEntryMatcher;
use crate::WorkspaceEntry;

/// Matches workspace entries that match ANY of the provided matchers
#[derive(Default, Debug)]
pub struct OrMatcher {
    matchers: Vec<Box<dyn WorkspaceEntryMatcher>>,
}

impl OrMatcher {
    pub fn new(matchers: Vec<Box<dyn WorkspaceEntryMatcher>>) -> Self {
        Self { matchers }
    }

    pub fn push(&mut self, matcher: Box<dyn WorkspaceEntryMatcher>) {
        self.matchers.push(matcher);
    }
}

impl WorkspaceEntryMatcher for OrMatcher {
    fn matches(&self, workspace_entry: WorkspaceEntry) -> Option<WorkspaceEntry> {
        for matcher in &self.matchers {
            if let Some(matched_workspace_entry) = matcher.matches(workspace_entry.clone()) {
                return Some(matched_workspace_entry);
            }
        }

        None
    }
}
