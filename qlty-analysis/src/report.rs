use crate::utils::fs::path_to_string;
use pbjson_types::Timestamp;
use qlty_config::issue_transformer::IssueTransformer;
use qlty_types::analysis::v1::{
    AnalysisResult, ComponentType, Invocation, Issue, Message, Metadata, Stats,
};
use rayon::prelude::*;
use serde::Serialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use time::OffsetDateTime;
use tracing::debug;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Report {
    pub metadata: Metadata,
    pub messages: Vec<Message>,
    pub invocations: Vec<Invocation>,
    pub issues: Vec<Issue>,
    pub stats: Vec<Stats>,
}

impl Report {
    pub fn merge(&mut self, other: &Report) {
        self.messages.extend(other.messages.clone());
        self.invocations.extend(other.invocations.clone());
        self.issues.extend(other.issues.clone());
        self.stats.extend(other.stats.clone());

        if other.metadata.result == AnalysisResult::Error as i32 {
            self.metadata.result = AnalysisResult::Error.into();
        }
    }

    pub fn transform_issues(&mut self, transformer: Box<dyn IssueTransformer>) {
        let mut transformed_issues = vec![];

        for issue in self.issues.iter() {
            if let Some(issue) = transformer.transform(issue.clone()) {
                transformed_issues.push(issue);
            } else {
                debug!("Skipping issue due to transformer: {:?}", issue);
            }
        }

        self.issues = transformed_issues;
    }

    // TODO: Extract this into a Transformer pattern
    pub fn relativeize_paths(&mut self, base_path: &Path) {
        let prefix = base_path.to_path_buf();

        self.issues.iter_mut().for_each(|issue| {
            if let Some(location) = &mut issue.location() {
                location.path = location.relative_path(&prefix);
                issue.location = Some(location.to_owned());
            }

            issue.other_locations.iter_mut().for_each(|other_location| {
                other_location.path = other_location.relative_path(&prefix);
            });

            issue.suggestions.par_iter_mut().for_each(|suggestion| {
                suggestion.replacements.iter_mut().for_each(|replacement| {
                    let location = replacement.location.as_mut().unwrap();
                    location.path = location.relative_path(&prefix);
                    replacement.location = Some(location.clone());
                });
            });
        });

        self.stats.par_iter_mut().for_each(|stats| {
            stats.path = stats
                .path
                .strip_prefix(&path_to_string(&prefix))
                .unwrap_or(&stats.path)
                .to_owned();

            stats.fully_qualified_name = stats
                .fully_qualified_name
                .strip_prefix(&path_to_string(&prefix))
                .unwrap_or(&stats.fully_qualified_name)
                .to_owned();
        });
    }

    pub fn attach_metadata(&mut self) {
        self.invocations.par_iter_mut().for_each(|invocation| {
            invocation.workspace_id = self.metadata.workspace_id.clone();
            invocation.project_id = self.metadata.project_id.clone();
            invocation.reference = self.metadata.reference.clone();
            invocation.build_id = self.metadata.build_id.clone();
            invocation.build_timestamp = self.metadata.start_time;
            invocation.commit_sha = self.metadata.revision_oid.clone();
        });

        self.messages.par_iter_mut().for_each(|message| {
            message.workspace_id = self.metadata.workspace_id.clone();
            message.project_id = self.metadata.project_id.clone();
            message.reference = self.metadata.reference.clone();
            message.build_id = self.metadata.build_id.clone();
            message.build_timestamp = self.metadata.start_time;
            message.commit_sha = self.metadata.revision_oid.clone();
        });

        self.issues.par_iter_mut().for_each(|issue| {
            issue.workspace_id = self.metadata.workspace_id.clone();
            issue.project_id = self.metadata.project_id.clone();
            issue.analyzed_at = Some(self.metadata.start_time.unwrap());
            issue.pull_request_number = self.metadata.pull_request_number.clone();
            issue.tracked_branch_id = self.metadata.tracked_branch_id.clone();

            issue.reference = self.metadata.reference.clone();
            issue.build_id = self.metadata.build_id.clone();
            issue.commit_sha = self.metadata.revision_oid.clone();
        });

        self.stats.par_iter_mut().for_each(|stats| {
            stats.workspace_id = self.metadata.workspace_id.clone();
            stats.project_id = self.metadata.project_id.clone();
            stats.analyzed_at = Some(self.metadata.start_time.unwrap());
            stats.pull_request_number = self.metadata.pull_request_number.clone();
            stats.tracked_branch_id = self.metadata.tracked_branch_id.clone();

            stats.reference = self.metadata.reference.clone();
            stats.build_id = self.metadata.build_id.clone();
            stats.commit_sha = self.metadata.revision_oid.clone();
        });
    }

    pub fn duplication_issues_by_duplication(&self) -> HashMap<String, Vec<Issue>> {
        self.issues
            .iter()
            .filter(|issue| issue.tool == "qlty" && issue.driver == "duplication")
            .fold(HashMap::new(), |mut acc, issue| {
                let structural_hash = issue.get_property_string("structural_hash");
                let issues = acc.entry(structural_hash).or_default();
                issues.push(issue.clone());
                acc
            })
    }

    pub fn function_stats_by_path(&self) -> HashMap<PathBuf, Vec<Stats>> {
        let function_stats = self
            .stats
            .par_iter()
            .filter(|stats| stats.kind.try_into() == Ok(ComponentType::Function))
            .cloned()
            .collect::<Vec<_>>();

        let mut results = HashMap::new();

        for stat in function_stats {
            let path = PathBuf::from(&stat.path);
            let stats = results.entry(path).or_insert(vec![]);
            stats.push(stat);
        }

        results
    }

    pub fn file_stats(&self) -> Vec<Stats> {
        self.stats
            .par_iter()
            .filter(|stats| stats.kind.try_into() == Ok(ComponentType::File))
            .cloned()
            .collect()
    }

    pub fn directory_stats(&self) -> Vec<Stats> {
        self.stats
            .par_iter()
            .filter(|stats| stats.kind.try_into() == Ok(ComponentType::Directory))
            .cloned()
            .collect()
    }

    pub fn finish(&mut self) {
        self.metadata.finish_time = Some(self.now_timestamp());
    }

    fn now_timestamp(&self) -> Timestamp {
        let now = OffsetDateTime::now_utc();
        Timestamp {
            seconds: now.unix_timestamp(),
            nanos: now.nanosecond() as i32,
        }
    }
}
