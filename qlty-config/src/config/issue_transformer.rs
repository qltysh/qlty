use qlty_types::analysis::v1::Issue;
use std::fmt::Debug;

pub trait IssueTransformer: Debug + Send + Sync + 'static {
    fn initialize(&self) {}
    fn transform(&self, issue: Issue) -> Option<Issue>;
    fn transform_batch(&self, _issues: &[Issue]) -> Option<Vec<Issue>> {
        None
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
    fn transform(&self, issue: Issue) -> Option<Issue> {
        Some(issue)
    }

    fn clone_box(&self) -> Box<dyn IssueTransformer> {
        Box::new(self.clone())
    }
}
