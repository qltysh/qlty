use crate::planner::Plan;
use crate::source_reader::SourceReader;
use crate::ui::ProgressBar as _;
use crate::{executor::staging_area::StagingArea, Progress};
use anyhow::{bail, Result};
use qlty_cloud::Client;
use qlty_config::issue_transformer::IssueTransformer;
use qlty_types::analysis::v1::{Issue, Suggestion};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use tracing::{debug, warn};
use tracing::{info, trace};
use ureq::json;

const MAX_FIXES: usize = 500;
const MAX_FIXES_PER_FILE: usize = 30;
const MAX_CONCURRENT_FIXES: usize = 10;

#[derive(Clone, Debug)]
pub struct Fixer {
    progress: Progress,
    staging_area: StagingArea,
    r#unsafe: bool,
    attempts_per_file: Arc<Mutex<HashMap<Option<String>, AtomicUsize>>>,
    total_attempts: Arc<AtomicUsize>,
    api_concurrency_lock: Arc<AtomicUsize>,
    api_concurrency_guard: Arc<Mutex<()>>,
}

impl IssueTransformer for Fixer {
    fn transform(&self, mut issue: Issue) -> Option<Issue> {
        if !self.reached_max_fixes(&issue) {
            issue = self
                .fix_issue(&issue)
                .inspect(|issue| self.update_max_fixes(issue))
                .unwrap_or(issue);
        }

        Some(issue)
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
            api_concurrency_lock: Arc::new(AtomicUsize::new(0)),
            api_concurrency_guard: Arc::new(Mutex::new(())),
        }
    }

    fn reached_max_fixes(&self, issue: &Issue) -> bool {
        if self.total_attempts.load(Ordering::Relaxed) >= MAX_FIXES {
            debug!(
                "Skipping all issue due to max attempts of {} reached",
                MAX_FIXES
            );
            return true;
        }

        let mut attempts_per_file = self.attempts_per_file.lock().unwrap();
        let file_attempts = attempts_per_file
            .entry(issue.path())
            .or_insert(AtomicUsize::new(0));
        if file_attempts.load(Ordering::Relaxed) >= MAX_FIXES_PER_FILE {
            warn!(
                "Skipping more issues in file with too many attempts: {}",
                issue.path().unwrap_or_default()
            );
            return true;
        }

        false
    }

    fn update_max_fixes(&self, issue: &Issue) {
        self.total_attempts.fetch_add(1, Ordering::Relaxed);
        self.attempts_per_file
            .lock()
            .unwrap()
            .get(&issue.path())
            .map(|a| a.fetch_add(1, Ordering::Relaxed));
    }

    fn fix_issue(&self, issue: &Issue) -> Option<Issue> {
        let task = self.progress.task("Generating AI Fix:", "");

        let trimmed_message = if issue.message.len() > 80 {
            format!("{}...", &issue.message[..80])
        } else {
            issue.message.clone()
        };
        task.set_dim_message(&trimmed_message);

        match self.try_fix(issue) {
            Ok(issue) => {
                if issue.suggestions.is_empty() {
                    debug!(
                        "No AI fix generated for issue: {}:{}",
                        &issue.tool, &issue.rule_key
                    );
                } else {
                    info!("Generated AI autofix for issue: {:?}", &issue.suggestions);
                    return Some(issue);
                }
            }
            Err(error) => {
                warn!("Failed to generate AI autofix: {:?}", error);
            }
        };

        self.progress.increment(1);
        task.clear();
        None
    }

    fn try_fix(&self, issue: &Issue) -> Result<Issue> {
        if let Some(path) = issue.path() {
            let client = Client::authenticated()?;
            let content = self.staging_area.read(issue.path().unwrap().into())?;
            self.try_fix_barrier();
            let response = client.post("/fixes").send_json(json!({
                "issue": issue.clone(),
                "files": [{ "content": content, "path": path }],
                "options": {
                    "unsafe": self.r#unsafe
                },
            }));
            self.api_concurrency_lock.fetch_sub(1, Ordering::SeqCst);

            let response_debug = format!("{:?}", &response);
            let suggestions: Vec<Suggestion> = response?.into_json()?;
            debug!("{} with {} suggestions", response_debug, suggestions.len());

            let mut issue = issue.clone();
            issue.suggestions = suggestions.clone();

            Ok(issue)
        } else {
            bail!("Issue {} has no path", issue.id);
        }
    }

    fn try_fix_barrier(&self) {
        let guard = self.api_concurrency_guard.lock().unwrap();
        while self.api_concurrency_lock.load(Ordering::SeqCst) >= MAX_CONCURRENT_FIXES {
            sleep(Duration::from_millis(100));
        }
        let value = self.api_concurrency_lock.fetch_add(1, Ordering::SeqCst);
        trace!("API request made with {} concurrent fixes", value);
        drop(guard);
    }
}
