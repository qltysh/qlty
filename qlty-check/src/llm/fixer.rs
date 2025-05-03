use crate::planner::Plan;
use crate::source_reader::SourceReader;
use crate::ui::ProgressBar as _;
use crate::{executor::staging_area::StagingArea, Progress};
use anyhow::Result;
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
    attempts_per_file: Arc<Mutex<HashMap<String, AtomicUsize>>>,
    total_attempts: Arc<AtomicUsize>,
    auth_token: String,
}

impl IssueTransformer for Fixer {
    fn transform_batch(&self, issues: Vec<Issue>) -> Vec<Issue> {
        issues
            .iter()
            .cloned()
            .into_group_map_by(|issue| issue.path())
            .iter()
            .map(|(path, issues)| {
                if let Some(path) = path {
                    issues
                        .chunks(MAX_BATCH_SIZE)
                        .flat_map(|chunk| self.fix_issue(path, chunk))
                        .collect_vec()
                } else {
                    issues.clone()
                }
            })
            .collect_vec()
            .concat()
    }

    fn clone_box(&self) -> Box<dyn IssueTransformer> {
        Box::new(self.clone())
    }
}

impl Fixer {
    pub fn new(plan: &Plan, progress: Progress) -> Self {
        // If auth_token is missing, use empty string as a fallback
        let auth_token = plan.settings.auth_token.clone().unwrap_or_default();

        Self {
            progress,
            staging_area: plan.staging_area.clone(),
            r#unsafe: plan.settings.r#unsafe,
            attempts_per_file: Arc::new(Mutex::new(HashMap::new())),
            total_attempts: Arc::new(AtomicUsize::new(0)),
            auth_token,
        }
    }

    fn reached_max_fixes(&self, path: &String, issues: &[Issue]) -> bool {
        if self.total_attempts.load(Ordering::Relaxed) + issues.len() >= MAX_FIXES {
            debug!(
                "Skipping all issue due to max attempts of {} reached",
                MAX_FIXES
            );
            return true;
        }

        let mut attempts_per_file = self.attempts_per_file.lock().unwrap();
        let file_attempts = attempts_per_file
            .entry(path.clone())
            .or_insert(AtomicUsize::new(0));
        if file_attempts.load(Ordering::Relaxed) >= MAX_FIXES_PER_FILE {
            warn!(
                "Skipping more issues in file with too many attempts: {}",
                path
            );
            return true;
        }

        false
    }

    fn update_max_fixes(&self, path: &String, issues: &[Issue]) {
        self.total_attempts
            .fetch_add(issues.len(), Ordering::Relaxed);
        self.attempts_per_file
            .lock()
            .unwrap()
            .get(path)
            .map(|a| a.fetch_add(issues.len(), Ordering::Relaxed));
    }

    fn fix_issue(&self, path: &String, issues: &[Issue]) -> Vec<Issue> {
        if self.reached_max_fixes(path, issues) {
            return issues.to_vec();
        }

        let tasks = issues
            .iter()
            .map(|issue| {
                let task = self.progress.task("Generating AI Fix:", "");

                let trimmed_message = if issue.message.chars().count() > 80 {
                    let trimmed: String = issue.message.chars().take(80).collect();
                    format!("{}...", trimmed)
                } else {
                    issue.message.clone()
                };
                task.set_dim_message(&trimmed_message);
                task
            })
            .collect_vec();

        let issues = match self.try_fix(path, issues) {
            Ok(issues) => {
                self.update_max_fixes(path, &issues);
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
                issues
            }
            Err(error) => {
                warn!("Failed to generate AI autofix: {:?}", error);
                issues.to_vec()
            }
        };

        self.progress.increment(issues.len() as u64);
        tasks.iter().for_each(|task| task.clear());
        issues
    }

    fn try_fix(&self, path: &String, issues: &[Issue]) -> Result<Vec<Issue>> {
        let client = Client::new(None, Some(self.auth_token.clone()));
        let content = self.staging_area.read(path.clone().into())?;
        let response = API_THREAD_POOL.scope(|_| {
            client.post("/fixes/batch").send_json(json!({
                "issues": issues,
                "files": [{ "path": path, "content": content }],
                "options": {
                    "unsafe": self.r#unsafe
                },
            }))
        })?;
        debug!(
            "Response [/fixes/batch] (N={}): {:?}",
            issues.len(),
            &response
        );

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
    }
}
