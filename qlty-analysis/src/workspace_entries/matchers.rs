use super::workspace_entry::{WorkspaceEntry, WorkspaceEntryKind};
use crate::{code::language_detector::get_language_from_shebang, utils::fs::path_to_string};
use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use qlty_config::{config::ignore_group::IgnoreGroup, FileType};
use std::{collections::HashMap, path::PathBuf};

pub trait WorkspaceEntryMatcher: core::fmt::Debug {
    fn matches(&self, workspace_entry: WorkspaceEntry) -> Option<WorkspaceEntry>;
}

#[derive(Debug)]
pub struct AnyMatcher;

impl WorkspaceEntryMatcher for AnyMatcher {
    fn matches(&self, workspace_entry: WorkspaceEntry) -> Option<WorkspaceEntry> {
        Some(workspace_entry)
    }
}

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

/// Compares workspace entries against a `GlobSet`. Either includes or excludes.
#[derive(Debug)]
pub struct GlobsMatcher {
    glob_set: GlobSet,
    include: bool,
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

fn globs_to_globset(globs: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();

    for glob in globs {
        builder
            .add(Glob::new(glob).context("Failed to create a new Glob from the provided pattern")?);
    }

    Ok(builder.build()?)
}

#[derive(Default, Debug)]
pub struct IgnoreGroupsMatcher {
    matchers: Vec<GlobsMatcher>,
}

impl IgnoreGroupsMatcher {
    pub fn new(ignore_groups: Vec<IgnoreGroup>) -> Self {
        let matchers = ignore_groups
            .into_iter()
            .filter_map(|ignore_group| {
                GlobsMatcher::new_for_globs(&ignore_group.ignores, ignore_group.negate).ok()
            })
            .collect();

        Self { matchers }
    }
}

impl WorkspaceEntryMatcher for IgnoreGroupsMatcher {
    fn matches(&self, workspace_entry: WorkspaceEntry) -> Option<WorkspaceEntry> {
        // By default, include the file
        let path = workspace_entry.path_string();

        for matcher in self.matchers.iter().rev() {
            if matcher.glob_set.is_match(&path) {
                return if matcher.include {
                    Some(workspace_entry) // No need to clone; directly return ownership
                } else {
                    None // Immediately exclude the file if a normal ignore rule matches
                };
            }
        }

        Some(workspace_entry)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use qlty_config::config::Builder;
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
