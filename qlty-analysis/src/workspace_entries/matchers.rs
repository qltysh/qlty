mod and_matcher;
mod any_matcher;
mod file_matcher;
mod globs_matcher;
mod ignore_groups_matcher;
mod language_globs_matcher;
mod languages_shebang_matcher;
mod or_matcher;
mod prefix_matcher;

pub use and_matcher::AndMatcher;
pub use any_matcher::AnyMatcher;
pub use file_matcher::FileMatcher;
pub use globs_matcher::GlobsMatcher;
pub use ignore_groups_matcher::IgnoreGroupsMatcher;
pub use language_globs_matcher::LanguageGlobsMatcher;
pub use languages_shebang_matcher::LanguagesShebangMatcher;
pub use or_matcher::OrMatcher;
pub use prefix_matcher::PrefixMatcher;

use super::workspace_entry::WorkspaceEntry;

pub trait WorkspaceEntryMatcher: core::fmt::Debug {
    fn matches(&self, workspace_entry: WorkspaceEntry) -> Option<WorkspaceEntry>;
}
