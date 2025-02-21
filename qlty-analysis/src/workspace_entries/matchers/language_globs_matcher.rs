use super::{GlobsMatcher, WorkspaceEntryMatcher};
use crate::WorkspaceEntry;
use anyhow::Result;

/// Matches workspace entries that are of a single specific language and
/// amends the `WorkspaceEntry` with the `Language`
///
/// Uses GlobSetWorkspaceEntryMatcher under the hood.
#[derive(Debug)]
pub struct LanguageGlobsMatcher {
    language_name: String,
    matcher: GlobsMatcher,
}

impl LanguageGlobsMatcher {
    pub fn new(language_name: &str, globs: &[String]) -> Result<Self> {
        let matcher = GlobsMatcher::new_for_globs(globs, true)?;

        Ok(Self {
            language_name: language_name.to_string(),
            matcher,
        })
    }
}

impl WorkspaceEntryMatcher for LanguageGlobsMatcher {
    fn matches(&self, workspace_entry: WorkspaceEntry) -> Option<WorkspaceEntry> {
        match self.matcher.matches(workspace_entry) {
            Some(mut workspace_entry) => {
                workspace_entry.language_name = Some(self.language_name.clone());
                Some(workspace_entry)
            }
            None => None,
        }
    }
}
