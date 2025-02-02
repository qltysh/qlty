use crate::{
    cache::{IssueCache, IssuesCacheKey},
    command::ExecResult,
    planner::InvocationPlan,
};
use anyhow::{Context, Result};
use chrono::prelude::*;
use qlty_analysis::utils::fs::path_to_string;
use qlty_config::{
    config::{OutputDestination, OutputMissing, TargetType},
    version::QLTY_VERSION,
};
use qlty_types::analysis::v1::{
    Category, ExecutionVerb, ExitResult, Invocation, Issue, Level, Location, Message, MessageLevel,
};
use serde::Serialize;
use std::{collections::HashMap, path::PathBuf};
use std::{process::Output, sync::Arc};
use tracing::{debug, error};

#[derive(Debug, Clone)]
pub struct InvocationResult {
    pub plan: InvocationPlan,
    pub messages: Vec<Message>,
    pub invocation: Invocation,
    pub file_results: Option<Vec<FileResult>>,
    pub formatted: Option<Vec<PathBuf>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileResult {
    pub path: String,
    // pub contents_digest: String,
    pub issues: Vec<Issue>,
}

#[derive(Debug, PartialEq)]
pub enum InvocationStatus {
    Success,
    LintError,
    ParseError,
}

impl InvocationResult {
    pub fn from_command_output(
        plan: &InvocationPlan,
        rerun: &str,
        output: &Output,
        duration: f64,
    ) -> Result<Self> {
        let exec_result = ExecResult::from_process_output(output);

        let now = Utc::now();
        let start_time = now - chrono::Duration::seconds(duration as i64);

        let mut invocation = Self {
            plan: plan.clone(),
            invocation: Invocation {
                workspace_id: Default::default(),
                project_id: Default::default(),
                reference: Default::default(),
                build_id: Default::default(),
                build_timestamp: Default::default(),
                commit_sha: Default::default(),
                id: plan.invocation_id.clone(),
                qlty_cli_version: QLTY_VERSION.to_string(),
                plugin_name: plan.tool.name(),
                driver_name: plan.driver_name.clone(),
                prefix: plan.plugin.prefix.clone().unwrap_or_default(),
                plugin_version: plan.tool.version().unwrap_or_default(),
                verb: plan.verb.into(),
                targets_count: plan.targets.len() as u32,
                target_paths: plan
                    .targets
                    .iter()
                    .map(|t| t.path_string())
                    .collect::<Vec<_>>(),
                config_paths: plan
                    .plugin_configs
                    .iter()
                    .map(|c| path_to_string(&c.path))
                    .collect(),
                script: rerun.to_string(),
                cwd: path_to_string(&plan.invocation_directory),
                env: plan.tool.env(),
                started_at: Some(start_time.into()),
                duration_secs: duration as f32,
                exit_code: exec_result.exit_code,
                stdout: exec_result.stdout.clone(),
                stderr: exec_result.stderr.clone(),
                tmpfile_path: if plan.uses_tmpfile() {
                    Some(plan.tmpfile_path())
                } else {
                    None
                },
                exit_result: match plan.driver.exit_result(exec_result.exit_code) {
                    Ok(ExitResult::Success) => qlty_types::analysis::v1::ExitResult::Success.into(),
                    Ok(ExitResult::NoIssues) => {
                        qlty_types::analysis::v1::ExitResult::NoIssues.into()
                    }
                    Ok(ExitResult::KnownError) => {
                        qlty_types::analysis::v1::ExitResult::KnownError.into()
                    }
                    _ => qlty_types::analysis::v1::ExitResult::UnknownError.into(),
                },
                tmpfile_contents: Default::default(),
                parser_error: Default::default(),
                issues_count: Default::default(),
                rewrites_count: Default::default(),
            },
            messages: Default::default(),
            file_results: Default::default(),
            formatted: Default::default(),
        };

        invocation.process_results()?;
        invocation.write_out_file()?;

        match qlty_types::analysis::v1::ExitResult::try_from(invocation.invocation.exit_result) {
            Ok(qlty_types::analysis::v1::ExitResult::Success) | Ok(ExitResult::NoIssues) => {
                invocation.push_message(
                    MessageLevel::Info,
                    "invocation.concluded.success".to_string(),
                    format!(
                        "Successfully ran {} in {:.2}s",
                        invocation.plan.plugin_name, duration
                    ),
                    Default::default(),
                );
            }
            _ => {
                invocation.push_message(
                    MessageLevel::Error,
                    "invocation.concluded.error".to_string(),
                    format!(
                        "Error occurred while running {} ({:.2}s)",
                        invocation.plan.plugin_name, duration
                    ),
                    Default::default(),
                );
            }
        }

        if invocation.invocation.parser_error.is_some() {
            invocation.push_message(
                MessageLevel::Error,
                "invocation.parser.error".to_string(),
                format!("Error parsing output from {}", invocation.plan.plugin_name),
                invocation.invocation.parser_error.clone().unwrap(),
            );
        }

        Ok(invocation)
    }

    pub fn push_message(
        &mut self,
        level: MessageLevel,
        ty: String,
        message: String,
        details: String,
    ) {
        let now = Utc::now();

        let mut message_tags = HashMap::new();
        message_tags.insert("invocation.id".to_string(), self.invocation.id.clone());
        message_tags.insert(
            "plugin.name".to_string(),
            self.invocation.plugin_name.clone(),
        );
        message_tags.insert(
            "driver.name".to_string(),
            self.invocation.driver_name.clone(),
        );

        self.messages.push(Message {
            timestamp: Some(now.into()),
            module: "qlty_check::executor".to_string(),
            ty: ty.to_string(),
            level: level.into(),
            message,
            details,
            tags: message_tags,
            ..Default::default()
        });
    }

    pub fn process_results(&mut self) -> Result<()> {
        if self.plan.driver.output == OutputDestination::PassFail {
            self.handle_pass_fail_case()?;
        } else if self.invocation.exit_result
            == qlty_types::analysis::v1::ExitResult::Success as i32
        {
            self.handle_success_case()?;
        } else if self.invocation.exit_result
            == qlty_types::analysis::v1::ExitResult::NoIssues as i32
        {
            self.file_results = Some(vec![]);
        } else {
            self.log_error_output();
        }

        self.invocation.issues_count = self
            .file_results
            .as_ref()
            .map(|frs| frs.iter().map(|fr| fr.issues.len()).sum::<usize>() as u32)
            .unwrap_or_default();

        Ok(())
    }

    fn handle_pass_fail_case(&mut self) -> Result<()> {
        if self.invocation.exit_result == qlty_types::analysis::v1::ExitResult::Success as i32
            || self.invocation.exit_result == qlty_types::analysis::v1::ExitResult::NoIssues as i32
        {
            self.file_results = Some(vec![]);
        } else {
            let result = FileResult {
                path: "".to_string(),
                issues: vec![Issue {
                    message: format!("{} failed", self.invocation.plugin_name),
                    category: Category::Lint.into(),
                    level: Level::High.into(),
                    rule_key: "fail".to_string(),
                    tool: self.invocation.plugin_name.clone(),
                    location: None,
                    ..Default::default()
                }],
            };
            self.file_results = Some(vec![result]);
        }

        Ok(())
    }

    fn handle_success_case(&mut self) -> Result<()> {
        if self.plan.uses_tmpfile() {
            self.handle_tmpfile()?;
        }

        if self.plan.driver.output == OutputDestination::Rewrite {
            if self.invocation.verb == ExecutionVerb::Check as i32 {
                self.file_results = Some(self.create_file_result_for_autofmts()?);
            } else {
                self.handle_output_rewrite()?;
            }
        } else {
            self.handle_output_parsing()?;
        }

        Ok(())
    }

    fn log_error_output(&self) {
        let invocation_label = self.plan.invocation_label();

        if !self.invocation.stdout.is_empty() {
            error!(
                "{}: {} STDOUT: {}",
                self.invocation.id, invocation_label, self.invocation.stdout
            );
        }

        if !self.invocation.stderr.is_empty() {
            error!(
                "{}: {} STDERR: {}",
                self.invocation.id, invocation_label, self.invocation.stderr
            );
        }
    }

    fn handle_tmpfile(&mut self) -> Result<()> {
        let tmpfile_path = self.invocation.tmpfile_path.as_ref().unwrap();
        let read_result = std::fs::read_to_string(tmpfile_path)
            .with_context(|| format!("Failed to read tmpfile contents from {}", tmpfile_path));
        self.invocation.tmpfile_contents = read_result.ok();
        Ok(())
    }

    fn handle_output_rewrite(&mut self) -> Result<()> {
        let mut formatted = vec![];

        for workspace_entry in self.plan.workspace_entries.iter() {
            let workspace_path = self.plan.workspace.root.join(&workspace_entry.path);
            let staged_path = self.plan.target_root.join(&workspace_entry.path);

            let workspace_contents = match std::fs::read_to_string(&workspace_path) {
                Ok(content) => content,
                Err(_) => {
                    error!("Failed to read workspace file {:?}", &workspace_path);
                    continue; // Skip unreadable files
                }
            };
            // if we can read the workspace file, we can assume the staged file exists
            let staged_contents = std::fs::read_to_string(&staged_path)
                .with_context(|| format!("Failed to read staged file {:?}", &staged_path))?;

            if workspace_contents != staged_contents {
                std::fs::copy(&staged_path, &workspace_path)?;
                self.invocation.rewrites_count += 1;
                formatted.push(workspace_entry.path.to_owned())
            }
        }

        self.formatted = Some(formatted);
        Ok(())
    }

    fn handle_output_parsing(&mut self) -> Result<()> {
        let output = if self.plan.uses_tmpfile() {
            self.invocation
                .tmpfile_contents
                .as_ref()
                .unwrap_or(&String::new())
                .to_owned()
        } else if self.plan.driver.output == OutputDestination::Stderr {
            self.invocation.stderr.to_owned()
        } else {
            self.invocation.stdout.to_owned()
        };

        if output.is_empty() {
            match self.plan.driver.output_missing {
                OutputMissing::Error => {
                    self.invocation.exit_result =
                        qlty_types::analysis::v1::ExitResult::UnknownError.into();
                    self.log_error_output();
                }
                OutputMissing::NoIssues => {
                    self.invocation.exit_result =
                        qlty_types::analysis::v1::ExitResult::NoIssues.into();
                }
                OutputMissing::Parse => self.parse_output(output),
            }
        } else {
            self.parse_output(output);
        }

        Ok(())
    }

    fn parse_output(&mut self, output: String) {
        let file_results = self.plan.driver.parse(&output, &self.plan);

        match file_results {
            Ok(file_results) => {
                self.file_results = Some(file_results);
            }
            Err(e) => {
                self.invocation.parser_error = Some(e.to_string());
            }
        }
    }

    fn create_file_result_for_autofmts(&self) -> Result<Vec<FileResult>> {
        let mut file_results: Vec<FileResult> = Vec::new();

        for workspace_entry in self.plan.targets.iter() {
            let staged_path = self.plan.target_root.join(&workspace_entry.path);
            let staged_contents = match std::fs::read_to_string(&staged_path) {
                Ok(content) => content,
                Err(_) => {
                    error!("Failed to read staged file {:?}", &staged_path);
                    continue; // Skip unreadable files
                }
            };

            let workspace_path = self.plan.workspace.root.join(&workspace_entry.path);
            let workspace_contents = std::fs::read_to_string(&workspace_path)?;

            let mut issues = Vec::new();

            if workspace_contents != staged_contents {
                issues.push(Issue {
                    message: "Incorrect formatting, autoformat by running `qlty fmt`.".to_string(),
                    category: Category::Style.into(),
                    level: Level::Fmt.into(),
                    rule_key: "fmt".to_string(),
                    tool: self.plan.plugin_name.clone(),
                    location: Some(Location {
                        path: workspace_entry.path_string(),
                        ..Default::default()
                    }),
                    on_added_line: true,
                    ..Default::default()
                });
            }

            file_results.push(FileResult {
                path: workspace_entry.path_string(),
                issues,
            });
        }

        Ok(file_results)
    }

    pub fn write_out_file(&self) -> Result<()> {
        if !self.outfile_directory().exists()
            && std::fs::create_dir_all(self.outfile_directory()).is_err()
        {
            error!(
                "Failed to create directory: {}",
                self.outfile_directory().display()
            );
        }

        let contents = serde_yaml::to_string(&self.invocation)?;
        std::fs::write(self.outfile_path(), contents).with_context(|| {
            format!(
                "Failed to write invocation results to {}",
                self.outfile_path().display()
            )
        })?;
        Ok(())
    }

    pub fn cache_issues(&self, cache: &IssueCache) -> Result<()> {
        if self.invocation.verb == ExecutionVerb::Fmt as i32
            || self.plan.driver.target.target_type != TargetType::File
            || !self.is_success()
        {
            return Ok(());
        }

        let file_results_by_path = self.file_results_by_path();
        let configs = &self.plan.plugin_configs;

        let cache_key = IssuesCacheKey::new(
            self.plan.tool.clone(),
            Arc::new(self.plan.plugin.clone()),
            self.invocation.driver_name.clone(),
            Arc::new(configs.clone()),
            self.plan.plugin.affects_cache.clone(),
        );

        for target in &self.plan.targets {
            let mut cache_key = cache_key.clone();
            cache_key.finalize(target);

            if let Some(file_result) = file_results_by_path.get(&target.path_string()) {
                cache.write(&cache_key, &file_result.issues)?;
            } else {
                debug!(
                    "Not writing cache for {}. No file result.",
                    target.path_string()
                );
            }
        }

        Ok(())
    }

    pub fn outfile_path(&self) -> PathBuf {
        self.outfile_directory()
            .join(format!("invoke-{}.yaml", self.invocation.id))
    }

    fn outfile_directory(&self) -> PathBuf {
        self.plan.workspace.root.join(".qlty").join("out")
    }

    pub fn status(&self) -> InvocationStatus {
        if self.plan.driver.output == OutputDestination::PassFail {
            return InvocationStatus::Success;
        }

        match qlty_types::analysis::v1::ExitResult::try_from(self.invocation.exit_result) {
            Ok(qlty_types::analysis::v1::ExitResult::Success) => match self.invocation.parser_error
            {
                None => InvocationStatus::Success,
                Some(_) => InvocationStatus::ParseError,
            },
            Ok(ExitResult::KnownError) => InvocationStatus::LintError,
            Ok(ExitResult::UnknownError) => InvocationStatus::LintError,
            Ok(ExitResult::NoIssues) => InvocationStatus::Success,
            _ => InvocationStatus::LintError,
        }
    }

    pub fn is_success(&self) -> bool {
        self.status() == InvocationStatus::Success
    }

    fn file_results_by_path(&self) -> HashMap<String, FileResult> {
        if self.file_results.is_none() {
            return HashMap::new();
        }

        let mut file_results_by_path: HashMap<String, FileResult> = HashMap::new();

        for file_result in self.file_results.clone().unwrap() {
            file_results_by_path.insert(file_result.path.clone(), file_result.clone());
        }

        file_results_by_path
    }
}
