use super::WorkspaceEntryMatcher;
use crate::{code::language_detector::get_language_from_shebang, WorkspaceEntry};
use std::collections::HashMap;

/// Matches workspace entries that have a sheband line that includes an interpretter
/// This requires reading the contents of the file.
#[derive(Debug)]
pub struct LanguagesShebangMatcher {
    interpreters: HashMap<String, Vec<String>>,
}

impl LanguagesShebangMatcher {
    pub fn new(interpreters: HashMap<String, Vec<String>>) -> Self {
        Self { interpreters }
    }
}

impl WorkspaceEntryMatcher for LanguagesShebangMatcher {
    fn matches(&self, workspace_entry: WorkspaceEntry) -> Option<WorkspaceEntry> {
        let file = std::fs::File::open(&workspace_entry.path);

        if let Ok(file) = file {
            let mut reader = std::io::BufReader::new(file);
            if let Ok(language_name) = get_language_from_shebang(&mut reader, &self.interpreters) {
                if !language_name.is_empty() {
                    return Some(WorkspaceEntry {
                        language_name: Some(language_name),
                        ..workspace_entry
                    });
                }
            }
        }
        None
    }
}
