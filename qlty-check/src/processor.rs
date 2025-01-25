use std::{collections::HashSet, time::Instant};

use crate::{
    patcher::Patcher,
    planner::Plan,
    results::{FixedResult, Results},
    Report,
};
use anyhow::Result;
use qlty_analysis::IssueCount;
use qlty_types::analysis::v1::{Category, ExecutionVerb, Issue};
use tracing::info;

pub struct Processor {
    plan: Plan,
    results: Results,
    issues: Vec<Issue>,
    fixed: HashSet<FixedResult>,
    fixable: HashSet<FixedResult>,
    counts: IssueCount,
}

impl Processor {
    pub fn new(plan: &Plan, results: Results) -> Self {
        Self {
            plan: plan.clone(),
            results,
            issues: Vec::new(),
            counts: IssueCount::default(),
            fixed: HashSet::new(),
            fixable: HashSet::new(),
        }
    }

    pub fn compute(&mut self) -> Result<Report> {
        let timer = Instant::now();
        self.compute_issues();
        self.compute_fixes();
        self.compute_fixable();
        self.compute_counts();

        // Sort issues for consistent ordering of results
        self.issues.sort();

        info!(
            "Processed {} issue results in {:.2}s",
            self.results.issues.len(),
            timer.elapsed().as_secs_f32()
        );

        Ok(Report {
            verb: self.plan.verb,
            target_mode: self.plan.target_mode.clone(),
            messages: self.results.messages.clone(),
            invocations: self.results.invocations.clone(),
            formatted: self.results.formatted.clone(),
            fixed: self.fixed.clone(),
            fixable: self.fixable.clone(),
            issues: self.issues.clone(),
            counts: self.counts,
        })
    }

    fn compute_counts(&mut self) {
        self.counts.total_issues = self.issues.len();

        for issue in &self.issues {
            if let Some(fail_level) = self.plan.fail_level {
                if issue.level >= fail_level as i32 {
                    self.counts.failure_issues += 1;
                }
            }

            if issue.category == Category::Vulnerability as i32
                || issue.category == Category::Secret as i32
                || issue.category == Category::DependencyAlert as i32
            {
                self.counts.total_security_issues += 1;
            }
        }
    }

    fn compute_issues(&mut self) {
        for transformer in self.plan.transformers.iter() {
            transformer.initialize();
        }

        for issue in self.results.issues.iter() {
            if let Some(issue) = self.transform_issue(issue) {
                self.issues.push(issue);
            }
        }
    }

    fn compute_fixes(&mut self) {
        if self.plan.verb == ExecutionVerb::Check && self.plan.settings.fix {
            self.fixed = Patcher::new(&self.plan.staging_area)
                .try_apply(&self.fixable_issues(), self.plan.settings.r#unsafe);
        }
    }

    fn compute_fixable(&mut self) {
        for issue in self.fixable_issues().iter() {
            if !issue.suggestions.is_empty() && issue.location.is_some() {
                self.fixable.insert(FixedResult {
                    rule_key: issue.rule_key.clone(),
                    location: issue.location().unwrap(),
                });
            }
        }
    }

    fn fixable_issues(&self) -> Vec<Issue> {
        Patcher::filter_issues(&self.issues, self.plan.settings.r#unsafe)
    }

    fn transform_issue(&self, issue: &Issue) -> Option<Issue> {
        let mut transformed_issue: Option<Issue> = Some(issue.clone());

        for transformer in self.plan.transformers.iter() {
            if transformed_issue.is_some() {
                transformed_issue = transformer.transform(transformed_issue.unwrap());
            } else {
                return None;
            }
        }

        transformed_issue
    }
}
