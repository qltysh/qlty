use crate::planner::Plan;
use crate::source_reader::SourceReader;
use crate::ui::ProgressBar as _;
use crate::{executor::staging_area::StagingArea, Progress};
use anyhow::{bail, Result};
use itertools::Itertools;
use lazy_static::lazy_static;
use qlty_cloud::Client;
use qlty_config::issue_transformer::IssueTransformer;
use qlty_types::analysis::v1::{Issue, Suggestion};
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};
use ureq::json;

const MAX_FIXES: usize = 500;
const MAX_FIXES_PER_FILE: usize = 30;
const MAX_CONCURRENT_FIXES: usize = 10;
const MAX_BATCH_SIZE: usize = 15;

lazy_static! {
    static ref API_THREAD_POOL: ThreadPool = ThreadPoolBuilder::new()
        .num_threads(MAX_CONCURRENT_FIXES)
        .build()
        .unwrap();
}

#[derive(Clone, Debug)]
pub struct Fixer {
    progress: Progress,
    staging_area: StagingArea,
    r#unsafe: bool,
    attempts_per_file: Arc<Mutex<HashMap<Option<String>, AtomicUsize>>>,
    total_attempts: Arc<AtomicUsize>,
}

impl IssueTransformer for Fixer {
    fn transform(&self, issue: Issue) -> Option<Issue> {
        Some(issue)
    }

    fn transform_batch(&self, issues: &[Issue]) -> Option<Vec<Issue>> {
        if issues.is_empty() {
            return None;
        }
        Some(
            issues
                .chunks(MAX_BATCH_SIZE)
                .flat_map(|chunk| self.fix_issue(chunk))
                .flatten()
                .collect_vec(),
        )
    }

    fn clone_box(&self) -> Box<dyn IssueTransformer> {
        Box::new(self.clone())
    }
}

impl Fixer {
    pub fn new(plan: &Plan, progress: Progress) -> Self {
        Self {
            progress,
            staging_area: plan.staging_area.clone(),
            r#unsafe: plan.settings.r#unsafe,
            attempts_per_file: Arc::new(Mutex::new(HashMap::new())),
            total_attempts: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn common_path(&self, issues: &[Issue]) -> Option<String> {
        issues
            .iter()
            .find(|issue| issue.path().is_some())
            .and_then(|issue| issue.path())
    }

    fn reached_max_fixes(&self, issues: &[Issue]) -> bool {
        if self.total_attempts.load(Ordering::Relaxed) + issues.len() >= MAX_FIXES {
            debug!(
                "Skipping all issue due to max attempts of {} reached",
                MAX_FIXES
            );
            return true;
        }

        let mut attempts_per_file = self.attempts_per_file.lock().unwrap();
        let path = self.common_path(issues);
        let file_attempts = attempts_per_file
            .entry(path.clone())
            .or_insert(AtomicUsize::new(0));
        if file_attempts.load(Ordering::Relaxed) >= MAX_FIXES_PER_FILE {
            warn!(
                "Skipping more issues in file with too many attempts: {}",
                path.unwrap_or_default()
            );
            return true;
        }

        false
    }

    fn update_max_fixes(&self, issues: &[Issue]) {
        self.total_attempts
            .fetch_add(issues.len(), Ordering::Relaxed);
        self.attempts_per_file
            .lock()
            .unwrap()
            .get(&self.common_path(issues))
            .map(|a| a.fetch_add(1, Ordering::Relaxed));
    }

    fn fix_issue(&self, issues: &[Issue]) -> Option<Vec<Issue>> {
        if self.reached_max_fixes(issues) {
            return Some(issues.to_vec());
        }

        let tasks = issues
            .iter()
            .map(|issue| {
                let task = self.progress.task("Generating AI Fix:", "");

                let trimmed_message = if issue.message.len() > 80 {
                    format!("{}...", &issue.message[..80])
                } else {
                    issue.message.clone()
                };
                task.set_dim_message(&trimmed_message);
                task
            })
            .collect_vec();

        match self.try_fix(issues) {
            Ok(issues) => {
                self.update_max_fixes(&issues);
                for issue in issues.iter() {
                    if issue.suggestions.is_empty() {
                        debug!(
                            "No AI fix generated for issue: {}:{}",
                            &issue.tool, &issue.rule_key
                        );
                    } else {
                        info!("Generated AI autofix for issue: {:?}", &issue.suggestions);
                    }
                }
                return Some(issues);
            }
            Err(error) => {
                warn!("Failed to generate AI autofix: {:?}", error);
            }
        };

        self.progress.increment(issues.len() as u64);
        tasks.iter().for_each(|task| task.clear());
        None
    }

    fn try_fix(&self, issues: &[Issue]) -> Result<Vec<Issue>> {
        if let Some(path) = self.common_path(issues) {
            let client = Client::authenticated()?;
            let content = self.staging_area.read(path.clone().into())?;
            let response = API_THREAD_POOL.scope(|_| {
                client.post("/fixes/batch").send_json(json!({
                    "issues": issues,
                    "files": [{ "content": content, "path": path }],
                    "options": {
                        "unsafe": self.r#unsafe
                    },
                }))
            })?;
            debug!("Response [/fixes/batch]: {:?}", &response);

            let suggestion_groups: Vec<Vec<Suggestion>> = response.into_json()?;
            debug!("Suggestions: {:?}", suggestion_groups);

            let issues = issues
                .iter()
                .zip(suggestion_groups)
                .map(|(issue, suggestions)| {
                    let mut issue = issue.clone();
                    issue.suggestions = suggestions;
                    issue
                })
                .collect_vec();

            Ok(issues)
        } else {
            bail!("Issues have no path");
        }
    }
}
