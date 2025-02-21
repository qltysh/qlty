use super::WorkspaceEntryMatcher;
use crate::WorkspaceEntry;
use anyhow::Context;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use qlty_config::FileType;

/// Compares workspace entries against a `GlobSet`. Either includes or excludes.
#[derive(Debug)]
pub struct GlobsMatcher {
    pub glob_set: GlobSet,
    pub include: bool,
}

impl GlobsMatcher {
    pub fn new(glob_set: GlobSet, include: bool) -> Self {
        Self { glob_set, include }
    }

    pub fn new_for_globs(globs: &[String], include: bool) -> Result<Self> {
        let glob_set = globs_to_globset(globs)?;
        Ok(Self { glob_set, include })
    }

    pub fn new_for_file_types(file_types: &[FileType]) -> Result<Self> {
        let globs = file_types
            .iter()
            .flat_map(|file_type| file_type.globs.to_owned())
            .collect::<Vec<String>>();
        Self::new_for_globs(&globs, true)
    }
}

impl WorkspaceEntryMatcher for GlobsMatcher {
    fn matches(&self, workspace_entry: WorkspaceEntry) -> Option<WorkspaceEntry> {
        let matches = self.glob_set.is_match(workspace_entry.path_string());

        if matches == self.include {
            Some(workspace_entry)
        } else {
            None
        }
    }
}

fn globs_to_globset(globs: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();

    for glob in globs {
        builder
            .add(Glob::new(glob).context("Failed to create a new Glob from the provided pattern")?);
    }

    Ok(builder.build()?)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::WorkspaceEntryKind;
    use qlty_config::config::Builder;
    use std::path::PathBuf;
    use std::time::SystemTime;

    // Tests for FileWorkspaceEntryMatcher
    #[test]
    fn test_glob_set_workspace_entry_matcher_matches_file() {
        let workspace_entry = WorkspaceEntry {
            path: PathBuf::from("/path/to/file.rs"),
            kind: WorkspaceEntryKind::File,
            content_modified: SystemTime::now(),
            contents_size: 100,
            language_name: None,
        };
        let config = Builder::default_config().unwrap().to_owned();
        let all_file_types = config.file_types.to_owned();
        let file_types_names = vec!["rust".to_owned()];
        let file_types = all_file_types
            .iter()
            .filter_map(|(name, file_type)| {
                if file_types_names.contains(&name) {
                    Some(file_type.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let matcher = GlobsMatcher::new_for_file_types(&file_types).unwrap();

        assert!(
            matcher.matches(workspace_entry).is_some(),
            "Expected workspace_entry to match as it is of type rust"
        );
    }

    #[test]
    fn test_glob_set_workspace_entry_matcher_does_not_match_file() {
        let workspace_entry = WorkspaceEntry {
            path: PathBuf::from("/path/to/file.rb"),
            kind: WorkspaceEntryKind::File,
            content_modified: SystemTime::now(),
            contents_size: 100,
            language_name: None,
        };
        let config = Builder::default_config().unwrap().to_owned();
        let all_file_types = config.file_types.to_owned();
        let file_types_names = vec!["rust".to_owned()];
        let file_types = all_file_types
            .iter()
            .filter_map(|(name, file_type)| {
                if file_types_names.contains(&name) {
                    Some(file_type.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let matcher = GlobsMatcher::new_for_file_types(&file_types).unwrap();

        assert!(
            matcher.matches(workspace_entry).is_none(),
            "Expected workspace_entry not to match as it is of type ruby"
        );
    }
}
