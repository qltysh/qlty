use qlty_types::analysis::v1::Issue;
use rayon::prelude::*;
use std::fmt::Debug;

pub trait IssueTransformer: Debug + Send + Sync + 'static {
    fn initialize(&self) {}

    fn transform(&self, issue: Issue) -> Option<Issue> {
        Some(issue)
    }

    fn transform_batch(&self, issues: Vec<Issue>) -> Vec<Issue> {
        issues
            .par_iter()
            .filter_map(|issue| self.transform(issue.clone()))
            .collect()
    }

    fn clone_box(&self) -> Box<dyn IssueTransformer>;
}

impl Clone for Box<dyn IssueTransformer> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct NullIssueTransformer;

impl IssueTransformer for NullIssueTransformer {
    fn clone_box(&self) -> Box<dyn IssueTransformer> {
        Box::new(self.clone())
    }
}
