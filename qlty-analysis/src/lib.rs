use serde::Serialize;

pub mod cache;
pub mod code;
pub mod git;
pub mod issue_muter;
pub mod lang;
mod report;
pub mod snippet;
pub mod source_reader;
pub mod utils;
mod walker;
pub mod workspace_entries;

pub use lang::Language;
pub use report::Report;
pub use workspace_entries::{
    AllSource, ArgsSource, DiffSource, FileMatcher, GlobsMatcher, PrefixMatcher, WorkspaceEntry,
    WorkspaceEntryFinder, WorkspaceEntryKind, WorkspaceEntryMatcher, WorkspaceEntrySource,
};

#[derive(Debug, Default, Serialize, Clone, Copy)]
pub struct IssueCount {
    pub total_issues: usize,
    pub failure_issues: usize,
    pub total_security_issues: usize,
}
