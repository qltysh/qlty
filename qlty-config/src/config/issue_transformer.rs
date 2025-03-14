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
            .cloned()
            .filter_map(|issue| self.transform(issue))
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

#[cfg(test)]
mod tests {
    use super::*;
    use qlty_types::analysis::v1::Issue;

    #[derive(Debug, Clone)]
    struct DropTransformer;
    impl IssueTransformer for DropTransformer {
        fn transform(&self, _issue: Issue) -> Option<Issue> {
            None
        }

        fn clone_box(&self) -> Box<dyn IssueTransformer> {
            Box::new(self.clone())
        }
    }

    #[derive(Debug, Clone)]
    struct FilterBatchTransformer;
    impl IssueTransformer for FilterBatchTransformer {
        fn transform_batch(&self, issues: Vec<Issue>) -> Vec<Issue> {
            issues
                .iter()
                .filter(|issue| issue.id.starts_with("KEEP"))
                .cloned()
                .collect()
        }

        fn clone_box(&self) -> Box<dyn IssueTransformer> {
            Box::new(self.clone())
        }
    }

    fn create_test_issue(id: &str) -> Issue {
        Issue {
            id: id.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_null_transformer_single() {
        let transformer = NullIssueTransformer;
        let issue = create_test_issue("TEST-1");
        let result = transformer.transform(issue.clone());
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, "TEST-1");
    }

    #[test]
    fn test_null_transformer_batch() {
        let transformer = NullIssueTransformer;
        let issues = vec![
            create_test_issue("TEST-1"),
            create_test_issue("TEST-2"),
            create_test_issue("TEST-3"),
        ];

        let result = transformer.transform_batch(issues.clone());
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].id, "TEST-1");
        assert_eq!(result[1].id, "TEST-2");
        assert_eq!(result[2].id, "TEST-3");
    }

    #[test]
    fn test_transformer_clone() {
        let transformer: Box<dyn IssueTransformer> = Box::new(NullIssueTransformer);
        let cloned = transformer.clone();
        let issue = create_test_issue("TEST-1");
        let original_result = transformer.transform(issue.clone());
        let cloned_result = cloned.transform(issue.clone());
        assert_eq!(original_result.unwrap().id, cloned_result.unwrap().id);
    }

    #[test]
    fn test_drop_transformer_single() {
        let transformer = DropTransformer;
        let issue = create_test_issue("TEST-1");
        let result = transformer.transform(issue);
        assert!(result.is_none());
    }

    #[test]
    fn test_drop_transformer_batch() {
        let transformer = DropTransformer;
        let issues = vec![
            create_test_issue("TEST-1"),
            create_test_issue("TEST-2"),
            create_test_issue("TEST-3"),
        ];
        assert!(transformer.transform_batch(issues).is_empty());
    }

    #[test]
    fn test_filter_batch_transformer() {
        let transformer = FilterBatchTransformer;
        let issues = vec![
            create_test_issue("KEEP-1"),
            create_test_issue("DROP-1"),
            create_test_issue("KEEP-2"),
            create_test_issue("DROP-2"),
        ];

        let result = transformer.transform_batch(issues);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "KEEP-1");
        assert_eq!(result[1].id, "KEEP-2");
    }

    #[test]
    fn test_filter_batch_transformer_empty() {
        let transformer = FilterBatchTransformer;
        let issues = vec![create_test_issue("DROP-1"), create_test_issue("DROP-2")];
        assert!(transformer.transform_batch(issues).is_empty());
    }
}
