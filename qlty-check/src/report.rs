use crate::{results::FixedResult, InvocationResult};
use itertools::Itertools;
use qlty_analysis::{workspace_entries::TargetMode, IssueCount};
use qlty_formats::SarifTrait;
use qlty_types::analysis::v1::{ExecutionVerb, Issue, Level, Message};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[derive(Clone, Debug)]
pub struct Report {
    pub verb: ExecutionVerb,
    pub target_mode: TargetMode,
    pub messages: Vec<Message>,
    pub invocations: Vec<InvocationResult>,
    pub issues: Vec<Issue>,
    pub formatted: Vec<PathBuf>,
    pub fixed: HashSet<FixedResult>,
    pub fixable: HashSet<FixedResult>,
    pub counts: IssueCount,
}

impl Report {
    pub fn issues_by_path(&self) -> HashMap<Option<PathBuf>, Vec<Issue>> {
        // Sort issues by start line before grouping them by path.
        // They will stay sorted in the group
        self.issues
            .iter()
            .sorted()
            .cloned()
            .into_group_map_by(|issue| issue.path().map(PathBuf::from))
    }

    pub fn is_failure(&self) -> bool {
        self.counts.failure_issues > 0
    }

    pub fn has_errors(&self) -> bool {
        self.invocations
            .iter()
            .any(|invocation| !invocation.is_success())
    }

    pub fn issues_count(&self) -> usize {
        self.issues.len()
    }

    pub fn targets_count(&self) -> usize {
        self.invocations
            .iter()
            .map(|invocation| invocation.plan.targets.len())
            .sum::<usize>()
    }

    pub fn unformatted_count(&self) -> usize {
        self.unformatted_paths().len()
    }

    pub fn unformatted_paths(&self) -> Vec<PathBuf> {
        let issues = self
            .issues
            .iter()
            .filter(|issue| issue.level() == Level::Fmt)
            .collect::<Vec<_>>();

        let mut paths: Vec<PathBuf> = issues
            .iter()
            .filter_map(|issue| issue.path().map(PathBuf::from))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        paths.sort();
        paths
    }
}

impl SarifTrait for Report {
    fn issues(&self) -> Vec<Issue> {
        self.issues.clone()
    }

    fn messages(&self) -> Vec<Message> {
        self.messages.clone()
    }
}
