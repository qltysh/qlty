use crate::{results::FixedResult, InvocationResult};
use itertools::Itertools;
use qlty_analysis::IssueCount;
use qlty_types::analysis::v1::{ExecutionVerb, Issue, Message};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[derive(Clone, Debug)]
pub struct Report {
    pub verb: ExecutionVerb,
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
}
