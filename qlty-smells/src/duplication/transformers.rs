use anyhow::Result;
use globset::{escape, Glob, GlobSet, GlobSetBuilder};
use qlty_config::config::issue_transformer::IssueTransformer;
use qlty_types::analysis::v1::Issue;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct InclusionPathMatcher {
    inclusion_glob_set: GlobSet,
}

impl InclusionPathMatcher {
    pub fn new(paths: Vec<PathBuf>) -> Result<Self> {
        let mut builder = GlobSetBuilder::new();
        for path in paths {
            if path.is_file() {
                if let Some(file_path) = path.to_str() {
                    builder.add(Glob::new(&escape(file_path))?);
                }
            } else if path.is_dir() {
                if let Some(dir_path) = path.join("**").to_str() {
                    builder.add(Glob::new(dir_path)?);
                }
            }
        }

        Ok(InclusionPathMatcher {
            inclusion_glob_set: builder.build()?,
        })
    }
}

impl IssueTransformer for InclusionPathMatcher {
    fn transform(&self, issue: Issue) -> Option<Issue> {
        match issue.location {
            Some(ref location) => {
                if self.inclusion_glob_set.is_match(&location.path) {
                    Some(issue)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    fn clone_box(&self) -> Box<dyn IssueTransformer> {
        Box::new(self.clone())
    }
}
