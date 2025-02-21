use super::WorkspaceEntryMatcher;
use crate::WorkspaceEntry;

/// Matches workspace entries that match ALL of the provided matchers
#[derive(Default, Debug)]
pub struct AndMatcher {
    matchers: Vec<Box<dyn WorkspaceEntryMatcher>>,
}

impl AndMatcher {
    pub fn new(matchers: Vec<Box<dyn WorkspaceEntryMatcher>>) -> Self {
        Self { matchers }
    }

    pub fn push(&mut self, matcher: Box<dyn WorkspaceEntryMatcher>) {
        self.matchers.push(matcher);
    }
}

impl WorkspaceEntryMatcher for AndMatcher {
    fn matches(&self, workspace_entry: WorkspaceEntry) -> Option<WorkspaceEntry> {
        let mut matched_workspace_entry = workspace_entry.clone();

        for matcher in &self.matchers {
            if let Some(matched) = matcher.matches(matched_workspace_entry) {
                matched_workspace_entry = matched;
            } else {
                return None;
            }
        }

        Some(matched_workspace_entry)
    }
}
