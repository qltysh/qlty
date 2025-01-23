use crate::planner::check_filters::CheckFilters;
use crate::source_reader::SourceReader;
use crate::ui::ProgressBar as _;
use crate::PATCH_CONTEXT_LENGTH;
use crate::{executor::staging_area::StagingArea, Progress};
use crate::{Results, Settings};
use anyhow::{bail, Result};
use qlty_cloud::Client;
use qlty_config::issue_transformer::IssueTransformer;
use qlty_types::analysis::v1::{Issue, Suggestion};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tracing::info;
use tracing::{debug, warn};
use ureq::json;

const MAX_FIXES: usize = 500;
const MAX_FIXES_PER_FILE: usize = 30;
const THREADS: usize = 12;

#[derive(Clone, Debug)]
pub struct Fixer {
    staging_area: StagingArea,
    results: Results,
    r#unsafe: bool,
    progress: bool,
    pre_transformers: Vec<Box<dyn IssueTransformer>>,
    issues: Vec<Issue>,
    fixes_to_attempt: Vec<usize>,
    fixes_generated: Arc<AtomicUsize>,
}

impl Fixer {
    pub fn new(settings: &Settings, staging_area: &StagingArea, results: &Results) -> Self {
        Self {
            staging_area: staging_area.clone(),
            r#unsafe: settings.r#unsafe,
            progress: settings.progress,
            results: results.clone(),
            issues: vec![],
            pre_transformers: vec![Box::new(CheckFilters {
                filters: settings.filters.clone(),
            })],
            fixes_to_attempt: vec![],
            fixes_generated: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn completions_count(&self) -> usize {
        self.fixes_to_attempt.len()
    }

    fn compute_issues(&mut self) {
        for transformer in self.pre_transformers.iter() {
            transformer.initialize();
        }

        for issue in self.results.issues.iter() {
            if let Some(issue) = self.transform_issue(issue.to_owned()) {
                self.issues.push(issue);
            }
        }
    }

    fn transform_issue(&self, issue: Issue) -> Option<Issue> {
        let mut transformed_issue: Option<Issue> = Some(issue.clone());

        for transformer in self.pre_transformers.iter() {
            if transformed_issue.is_some() {
                transformed_issue = transformer.transform(transformed_issue.unwrap());
            } else {
                return None;
            }
        }

        transformed_issue
    }

    pub fn plan(&mut self) {
        self.compute_issues();

        let mut total_attempts = 0;
        let mut attempts_per_file = HashMap::new();

        let mut previous_issue_path = String::new();
        let mut previous_issue_line = 0;

        // Iterate over the issues and determine which ones to attempt to fix
        // This allows us to display progress to the user and preserve the order
        for (index, issue) in self.issues.iter().enumerate() {
            if issue.path().is_none() {
                continue;
            }
            let issue_path = issue.path().unwrap();

            if total_attempts >= MAX_FIXES {
                debug!(
                    "Skipping all issue due to max attempts of {} reached",
                    MAX_FIXES
                );
                break;
            }

            let file_attempts = attempts_per_file.entry(issue.path()).or_insert(0);
            if *file_attempts >= MAX_FIXES_PER_FILE {
                warn!(
                    "Skipping more issues in file with too many attempts: {}",
                    issue_path
                );
                continue;
            }

            if previous_issue_path == issue_path
                && issue.range().is_some()
                && previous_issue_line + PATCH_CONTEXT_LENGTH
                    >= *issue.line_range().unwrap().start()
            {
                debug!(
                    "Skipping issue too close to previous at {}:{}",
                    issue_path,
                    issue.line_range().unwrap().start()
                );
                continue;
            }

            total_attempts += 1;
            *file_attempts += 1;

            previous_issue_path = issue_path;

            if issue.range().is_some() {
                previous_issue_line = *issue.line_range().unwrap().start();
            }

            self.fixes_to_attempt.push(index);
        }
    }

    pub fn generate_fixes(&mut self) -> Result<Results> {
        info!(
            "Attempting AI autofix for {} of {} issues...",
            self.completions_count(),
            self.results.issues.len()
        );

        let progress =
            Progress::new_with_position(self.progress, self.fixes_to_attempt.len() as u64);
        progress.set_prefix("AI Autofixing");

        let original_issues = self.issues.clone();
        let mut modified_issues = vec![];

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(THREADS)
            .build()
            .unwrap();

        pool.install(|| {
            modified_issues = original_issues
                .into_par_iter()
                .enumerate()
                .map(|(index, issue)| self.maybe_fix_issue(index, &issue, progress.clone()))
                .collect::<Vec<Issue>>();
        });

        progress.clear();

        info!(
            "Generated AI autofixes for {} of {} attempted issues",
            self.fixes_generated
                .load(std::sync::atomic::Ordering::Relaxed),
            self.completions_count()
        );

        Ok(Results {
            issues: modified_issues,
            ..self.results.clone()
        })
    }

    fn maybe_fix_issue(&self, index: usize, issue: &Issue, progress: Progress) -> Issue {
        if self.fixes_to_attempt.contains(&index) {
            self.fix_issue(issue, progress)
        } else {
            issue.clone()
        }
    }

    fn fix_issue(&self, issue: &Issue, progress: Progress) -> Issue {
        let task = progress.task("AI Autofixing", "");
        task.set_prefix(&issue.tool);

        let trimmed_message = if issue.message.len() > 80 {
            format!("{}...", &issue.message[..80])
        } else {
            issue.message.clone()
        };
        task.set_dim_message(&trimmed_message);

        let issue = match self.try_fix(issue) {
            Ok(issue) => {
                if issue.suggestions.is_empty() {
                    debug!(
                        "No AI fix generated for issue: {}:{}",
                        &issue.tool, &issue.rule_key
                    );
                } else {
                    info!("Generated AI autofix for issue: {:?}", &issue.suggestions);
                    self.fixes_generated
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }

                issue
            }
            Err(error) => {
                warn!("Failed to generate AI autofix: {:?}", error);
                issue.clone()
            }
        };

        progress.increment(1);
        task.clear();
        issue
    }

    fn try_fix(&self, issue: &Issue) -> Result<Issue> {
        if let Some(path) = issue.path() {
            let client = Client::authenticated()?;
            let content = self.staging_area.read(issue.path().unwrap().into())?;
            let response = client.post("/fixes").send_json(json!({
                "issue": issue.clone(),
                "files": [{ "content": content, "path": path }],
                "options": {
                    "unsafe": self.r#unsafe
                },
            }))?;

            let response_debug = format!("{:?}", &response);
            let suggestions: Vec<Suggestion> = response.into_json()?;
            debug!("{} with {} suggestions", response_debug, suggestions.len());

            let mut issue = issue.clone();
            issue.suggestions = suggestions.clone();

            Ok(issue)
        } else {
            bail!("Issue {} has no path", issue.id);
        }
    }
}
