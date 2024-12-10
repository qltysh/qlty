use super::compute_invocation_script;
use super::invocation_result::FileResult;
use crate::parser::actionlint::Actionlint;
use crate::parser::bandit::Bandit;
use crate::parser::biome::Biome;
use crate::parser::clippy::Clippy;
use crate::parser::coffeelint::Coffeelint;
use crate::parser::eslint::Eslint;
use crate::parser::golangci_lint::GolangciLint;
use crate::parser::hadolint::Hadolint;
use crate::parser::knip::Knip;
use crate::parser::markdownlint::Markdownlint;
use crate::parser::mypy::Mypy;
use crate::parser::php_codesniffer::PhpCodesniffer;
use crate::parser::phpstan::Phpstan;
use crate::parser::pylint::Pylint;
use crate::parser::radarlint::Radarlint;
use crate::parser::reek::Reek;
use crate::parser::regex::Regex;
use crate::parser::ripgrep::Ripgrep;
use crate::parser::rubocop::Rubocop;
use crate::parser::ruff::Ruff;
use crate::parser::sarif::Sarif;
use crate::parser::shellcheck::Shellcheck;
use crate::parser::sqlfluff::Sqlfluff;
use crate::parser::stylelint::Stylelint;
use crate::parser::taplo::Taplo;
use crate::parser::trivy_sarif::TrivySarif;
use crate::parser::trufflehog::Trufflehog;
use crate::parser::tsc::Tsc;
use crate::parser::Parser;
use crate::planner::InvocationPlan;
use crate::tool::command_builder::Command;
use crate::ui::ProgressBar;
use crate::ui::ProgressTask;
use crate::InvocationResult;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use duct::Expression;
use itertools::Itertools;
use qlty_analysis::utils::fs::path_to_native_string;
use qlty_analysis::utils::fs::path_to_string;
use qlty_config::config::DriverDef;
use qlty_config::config::InvocationDirectoryType;
use qlty_config::config::OutputFormat;
use qlty_config::config::TargetType;
use qlty_types::analysis::v1::ExitResult;
use qlty_types::analysis::v1::Issue;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use sysinfo::{Pid, ProcessesToUpdate, System};
use tracing::warn;
use tracing::{debug, error, info};

const DEFAULT_SUCCESS_EXIT_CODE: i64 = 0;
const MAX_OUTPUT_SIZE_BYTES: usize = 1024 * 1024 * 100; // 100 MB

#[derive(Debug, Clone)]
pub struct Driver {
    def: DriverDef,
}

impl Deref for Driver {
    type Target = DriverDef;

    fn deref(&self) -> &Self::Target {
        &self.def
    }
}

impl DerefMut for Driver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.def
    }
}

impl From<DriverDef> for Driver {
    fn from(def: DriverDef) -> Self {
        Self { def }
    }
}

impl From<&DriverDef> for Driver {
    fn from(def: &DriverDef) -> Self {
        Self {
            def: def.to_owned(),
        }
    }
}

impl Driver {
    pub fn run(&self, plan: &InvocationPlan, task: &ProgressTask) -> Result<InvocationResult> {
        plan.tool.pre_setup(task)?;
        plan.tool.setup(task)?;
        task.set_message(&plan.description());

        let cmd_wrapper = Command::new(None, compute_invocation_script(plan)?);
        let rerun = cmd_wrapper.script;
        let cmd = cmd_wrapper
            .cmd
            .dir(&plan.invocation_directory)
            .full_env(plan.tool.env())
            .stderr_capture()
            .stdout_capture()
            .unchecked();

        Self::run_with_timeout(cmd, plan, &rerun)
    }

    fn run_with_timeout(
        cmd: Expression,
        plan: &InvocationPlan,
        rerun: &str,
    ) -> Result<InvocationResult> {
        debug!("Running invocation: {}", &rerun);

        let timer = Instant::now();
        let handle = cmd.start()?;
        let pids = handle.pids();
        let timeout = plan.driver.timeout;
        let invocation_label = plan.invocation_label();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        thread::spawn(move || {
            thread::sleep(Duration::from_secs(timeout));
            if running_clone.load(Ordering::SeqCst) {
                error!("Killing {} process after {}s", invocation_label, timeout);
                Self::terminate_processes(pids);
            }
        });

        let output = handle.into_output()?;
        let duration = timer.elapsed().as_secs_f64();
        running.store(false, Ordering::SeqCst);

        info!(
            "{}: Completed {} in {:.3}s (exit {})",
            plan.invocation_id,
            plan.invocation_label(),
            duration,
            output.status.code().unwrap_or(-1)
        );

        InvocationResult::from_command_output(plan, rerun, &output, duration)
    }

    pub fn terminate_processes(pids: Vec<u32>) {
        let mut system = System::new_all();

        for pid in pids {
            Self::terminate_process_tree(pid, &mut system);
        }
    }

    fn terminate_process_tree(pid: u32, system: &mut System) {
        system.refresh_processes(ProcessesToUpdate::All); // Ensure system info is up-to-date

        // Collect child PIDs first to avoid conflicting borrows
        let child_pids: Vec<u32> = system
            .processes()
            .iter()
            .filter_map(|(cpid, process)| {
                if let Some(parent) = process.parent() {
                    if parent.as_u32() == pid {
                        Some(cpid.as_u32())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Now recursively terminate child processes
        for cpid in child_pids {
            info!("Killing child process: {}", cpid);
            Self::terminate_process_tree(cpid, system); // Recursively kill child processes
            if let Some(process) = system.process(Pid::from_u32(cpid)) {
                if process.kill() {
                    info!("Successfully killed process: {}", cpid);
                } else {
                    warn!("Failed to kill process: {}", cpid);
                }
            }
        }

        // Finally, attempt to kill the parent process itself
        if let Some(parent_process) = system.process(Pid::from_u32(pid)) {
            info!(
                "Killing parent process: {} {:?}",
                pid,
                parent_process.name()
            );
            if parent_process.kill() {
                info!("Successfully killed parent process: {}", pid);
            } else {
                warn!("Failed to kill parent process: {}", pid);
            }
        } else {
            warn!("Parent process with PID {} not found", pid);
        }
    }

    pub fn exit_result(&self, exit_code: Option<i64>) -> Result<ExitResult> {
        let exit_code = match exit_code {
            Some(code) if self.success_codes.contains(&code) => ExitResult::Success,
            Some(DEFAULT_SUCCESS_EXIT_CODE) if self.success_codes.is_empty() => ExitResult::Success,
            Some(code) if self.error_codes.contains(&code) => ExitResult::KnownError,
            Some(code) if self.no_issue_codes.contains(&code) => ExitResult::NoIssues,
            Some(_) => ExitResult::UnknownError,
            None => bail!("inspect_exec returned no exit code"),
        };

        Ok(exit_code)
    }

    pub fn parse(&self, output: &str, plan: &InvocationPlan) -> Result<Vec<FileResult>> {
        if output.len() > MAX_OUTPUT_SIZE_BYTES {
            bail!(
                "Output size exceeds maximum allowed size of {} bytes",
                MAX_OUTPUT_SIZE_BYTES
            );
        }

        let parser = self.parser();
        let issues = parser.parse(&plan.plugin_name, output);
        let path_prefix = self.get_path_prefix(plan);

        match issues {
            Ok(issues) => {
                let issues_by_path = issues
                    .into_iter()
                    .map(|issue| self.fix_issue_path(issue, plan, &path_prefix))
                    .into_group_map_by(|issue| issue.path().map(PathBuf::from));

                debug!("Issues by path: {:?}", issues_by_path);

                let mut file_results = vec![];

                let parent_with_file_results =
                    self.parent_with_file_issues(&issues_by_path, &path_prefix);

                if parent_with_file_results.is_some() {
                    file_results.push(parent_with_file_results.unwrap());
                }

                let fileless_file_results = self.pathless_issues(&issues_by_path);
                if fileless_file_results.is_some() {
                    file_results.push(fileless_file_results.unwrap());
                }

                for workspace_entry in plan.workspace_entries.iter() {
                    let issues = issues_by_path
                        .get(&Some(workspace_entry.path.clone()))
                        .unwrap_or(&vec![])
                        .to_vec();

                    file_results.push(FileResult {
                        path: path_to_string(workspace_entry.path_string()),
                        issues,
                    });
                }

                Ok(file_results)
            }
            Err(e) => {
                error!(
                    "{}: {}: Error parsing output: {:?}",
                    plan.invocation_id,
                    plan.invocation_label(),
                    e
                );
                Err(e)
            }
        }
    }

    fn pathless_issues(
        &self,
        issues_by_path: &std::collections::HashMap<Option<PathBuf>, Vec<Issue>>,
    ) -> Option<FileResult> {
        let pathless_issues = issues_by_path.get(&None).unwrap_or(&vec![]).to_vec();

        if pathless_issues.is_empty() {
            None
        } else {
            Some(FileResult {
                path: "".to_string(),
                issues: pathless_issues,
            })
        }
    }

    fn get_path_prefix(&self, plan: &InvocationPlan) -> Option<PathBuf> {
        if plan.invocation_directory_def.kind == InvocationDirectoryType::Root {
            None
        } else if let Ok(path) = plan
            .invocation_directory
            .strip_prefix(path_to_string(&plan.target_root))
        {
            Some(path.to_path_buf())
        } else {
            None
        }
    }

    fn parent_with_file_issues(
        &self,
        issues_by_path: &std::collections::HashMap<Option<PathBuf>, Vec<Issue>>,
        path_prefix: &Option<PathBuf>,
    ) -> Option<FileResult> {
        if self.target.target_type == TargetType::ParentWith && path_prefix.is_some() {
            let parent_path = path_prefix
                .as_ref()
                .unwrap()
                .join(self.target.path.as_ref().unwrap());

            debug!("Parent path: {:?}", parent_path);
            let parent_issues = issues_by_path
                .get(&Some(parent_path.clone()))
                .unwrap_or(&vec![])
                .to_vec();

            Some(FileResult {
                path: path_to_string(parent_path),
                issues: parent_issues,
            })
        } else {
            None
        }
    }

    fn fix_issue_path(
        &self,
        issue: Issue,
        plan: &InvocationPlan,
        path_prefix: &Option<PathBuf>,
    ) -> Issue {
        if issue.path().is_none() {
            return issue;
        }

        let mut path = path_to_string(issue.path().unwrap());
        let target_root = path_to_string(&plan.target_root);
        let target_root = target_root.strip_suffix('/').unwrap_or(&target_root);

        if PathBuf::from(&path).is_relative() {
            if let Some(path_prefix) = path_prefix {
                path = format!("{}/{}", path_to_string(path_prefix), path);
            }
        }

        // HACK: This is a workaround for plugins where the path can
        // be file:///path/... and
        // staging_directory path may or may not contain '/private'
        path = path.strip_prefix("file://").unwrap_or(&path).into();

        // another possibility is that the path is prefixed with './'
        path = path.strip_prefix("./").unwrap_or(&path).into();

        path = path
            .strip_prefix(&format!("{}/", target_root))
            .unwrap_or(&path)
            .into();

        // HACK: This is a workaround for macOS where the path can
        // be /private/var/... instead of /var/... for reasons
        // I don't understand. -BH
        path = path
            .strip_prefix(&format!("/private{}/", target_root))
            .unwrap_or(&path)
            .into();

        // Some paths seem to be having a leading '/' that needs to be removed
        // in order to match them with their workspace entry paths
        path = path.strip_prefix('/').unwrap_or(&path).into();

        // Some paths seem to be have invocation_directory without /private/
        path = path
            .strip_prefix(&format!(
                "{}/",
                target_root.strip_prefix("/private/").unwrap_or(target_root)
            ))
            .unwrap_or(&path)
            .into();

        if let Some(prefix) = plan.plugin.prefix.as_ref() {
            if !prefix.is_empty() {
                path = format!("{}/{}", prefix, path);
            }
        }

        let mut issue = issue;
        issue.location.as_mut().unwrap().path = path;
        issue.suggestions.iter_mut().for_each(|suggestion| {
            suggestion.replacements.iter_mut().for_each(|replacement| {
                let location = replacement.location.as_mut().unwrap();
                location.path = location.relative_path(&plan.target_root);
                replacement.location = Some(location.clone());
            });
        });

        issue
    }

    fn parser(&self) -> Box<dyn Parser> {
        let parser: Box<dyn Parser> = match self.output_format {
            OutputFormat::Actionlint => Box::new(Actionlint {}),
            OutputFormat::Bandit => Box::new(Bandit {}),
            OutputFormat::Biome => Box::new(Biome {}),
            OutputFormat::Clippy => Box::<Clippy>::default(),
            OutputFormat::Coffeelint => Box::new(Coffeelint {}),
            OutputFormat::Eslint => Box::<Eslint>::default(),
            OutputFormat::GolangciLint => Box::new(GolangciLint {}),
            OutputFormat::Hadolint => Box::new(Hadolint {}),
            OutputFormat::Knip => Box::new(Knip {}),
            OutputFormat::Markdownlint => Box::new(Markdownlint {}),
            OutputFormat::Mypy => Box::new(Mypy {}),
            OutputFormat::PhpCodesniffer => Box::new(PhpCodesniffer {}),
            OutputFormat::Phpstan => Box::new(Phpstan {}),
            OutputFormat::Pylint => Box::new(Pylint {}),
            OutputFormat::Radarlint => Box::new(Radarlint {}),
            OutputFormat::Reek => Box::new(Reek {}),

            OutputFormat::Regex => {
                let level = self.output_level.map(|output_level| output_level.into());

                let category = self
                    .output_category
                    .map(|output_category| output_category.into());

                let regex = &self
                    .output_regex
                    .as_ref()
                    .expect("output = regex was specified, but output_regex is missing")
                    .clone();

                Box::new(Regex::new(regex, level, category))
            }

            OutputFormat::Ripgrep => Box::new(Ripgrep {}),
            OutputFormat::Rubocop => Box::new(Rubocop {}),
            OutputFormat::Ruff => Box::new(Ruff {}),

            OutputFormat::Sarif => {
                let level = self.output_level.map(|output_level| output_level.into());

                let category = self
                    .output_category
                    .map(|output_category| output_category.into());

                Box::new(Sarif::new(level, category))
            }

            OutputFormat::Shellcheck => Box::new(Shellcheck {}),
            OutputFormat::Stylelint => Box::new(Stylelint {}),
            OutputFormat::Sqlfluff => Box::new(Sqlfluff {}),
            OutputFormat::Taplo => Box::new(Taplo {}),
            OutputFormat::Tsc => Box::new(Tsc {}),

            OutputFormat::TrivySarif => {
                let category = self
                    .output_category
                    .map(|output_category| output_category.into());

                Box::new(TrivySarif::new(category))
            }
            OutputFormat::Trufflehog => Box::new(Trufflehog {}),
        };

        parser
    }

    pub fn run_prepare_script(&self, plan: &InvocationPlan, task: &ProgressTask) -> Result<()> {
        plan.tool.pre_setup(task)?;
        plan.tool.setup(task)?;

        let script = self
            .prepare_script
            .clone()
            .unwrap()
            .replace("${linter}", &path_to_native_string(plan.tool.directory()));
        let cmd_wrapper = Command::new(None, script);
        let rerun = cmd_wrapper.script;
        let dir = plan.invocation_directory.clone();

        if !dir.exists() {
            std::fs::create_dir_all(&dir).with_context(|| {
                format!(
                    "Failed to create directory for prepare_script: {}",
                    path_to_string(&dir)
                )
            })?;
        }

        let cmd = cmd_wrapper
            .cmd
            .dir(dir)
            .full_env(plan.tool.env())
            .stderr_capture()
            .stdout_capture()
            .unchecked();

        debug!("Running prepare_script: {}", &rerun);
        let timer = Instant::now();
        let invocation_label = plan.invocation_label();

        let output = cmd.run().with_context(|| {
            format!(
                "Failed to run prepare_script for {}: {}",
                invocation_label, &rerun
            )
        })?;
        let duration = timer.elapsed().as_secs_f64();

        info!(
            "{}: Completed {} in {:.3}s (exit {})",
            plan.invocation_id,
            invocation_label,
            duration,
            output.status.code().unwrap_or(-1)
        );

        Ok(())
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{executor::plan_target_list, planner::target::Target, tool::ruby::Ruby};
    use qlty_analysis::{utils::fs::path_to_string, WorkspaceEntry, WorkspaceEntryKind};
    use qlty_config::{
        config::{DriverType, InvocationDirectoryDef, OutputDestination, PluginDef},
        Workspace,
    };
    use qlty_types::analysis::v1::{ExecutionVerb, Location, Range};
    use std::{path::PathBuf, sync::Arc, time::SystemTime};

    pub fn build_driver(success_codes: Vec<i64>, error_codes: Vec<i64>) -> Driver {
        Driver {
            def: DriverDef {
                script: String::from("mock_script"),
                output: OutputDestination::Stdout,
                output_format: OutputFormat::Sarif,
                output_regex: Some(String::from("mock_regex")),
                driver_type: DriverType::Linter,
                batch: false,
                max_batch: 10,
                success_codes,
                error_codes,
                cache_results: true,
                file_types: None,
                ..Default::default()
            },
        }
    }

    #[test]
    fn test_exit_result_success_codes_empty() {
        let driver = build_driver(vec![], vec![1, 2]);
        assert_eq!(driver.exit_result(Some(0)).unwrap(), ExitResult::Success);
        assert_eq!(driver.exit_result(Some(1)).unwrap(), ExitResult::KnownError);
        assert_eq!(
            driver.exit_result(Some(137)).unwrap(),
            ExitResult::UnknownError
        );
    }

    #[test]
    fn test_exit_result_success_codes_present() {
        let driver = build_driver(vec![0, 1], vec![2]);
        assert_eq!(driver.exit_result(Some(0)).unwrap(), ExitResult::Success);
        assert_eq!(driver.exit_result(Some(1)).unwrap(), ExitResult::Success);
        assert_eq!(driver.exit_result(Some(2)).unwrap(), ExitResult::KnownError);
        assert_eq!(
            driver.exit_result(Some(137)).unwrap(),
            ExitResult::UnknownError
        );
    }

    #[test]
    fn test_exit_result_error_codes_empty() {
        let driver = build_driver(vec![0], vec![]);
        assert_eq!(driver.exit_result(Some(0)).unwrap(), ExitResult::Success);
        assert_eq!(
            driver.exit_result(Some(1)).unwrap(),
            ExitResult::UnknownError
        );
        assert_eq!(
            driver.exit_result(Some(137)).unwrap(),
            ExitResult::UnknownError
        );
    }

    #[test]
    fn test_exit_result_error_codes_present() {
        let driver = build_driver(vec![0], vec![1, 2]);
        assert_eq!(driver.exit_result(Some(1)).unwrap(), ExitResult::KnownError);
        assert_eq!(driver.exit_result(Some(2)).unwrap(), ExitResult::KnownError);
        assert_eq!(
            driver.exit_result(Some(137)).unwrap(),
            ExitResult::UnknownError
        );
    }

    #[test]
    #[should_panic(expected = "inspect_exec returned no exit code")]
    fn test_exit_result_success_none_exit_code() {
        let driver = build_driver(vec![], vec![1, 2]);
        driver.exit_result(None).unwrap();
    }

    #[test]
    fn test_fix_issue_path() {
        let valid_path_staging_directory_pairs = [
            [
                "file:///private/var/some/random/directory2/basic.in.py",
                "/var/some/random/directory2",
            ],
            [
                "file:///var/some/random/directory3/basic.in.py",
                "/var/some/random/directory3",
            ],
            [
                "/private/var/some/random/directory4/basic.in.py",
                "/private/var/some/random/directory4",
            ],
            [
                "/private/var/some/random/directory5/basic.in.py",
                "/var/some/random/directory5",
            ],
            [
                "/var/some/random/directory6/basic.in.py",
                "/var/some/random/directory6",
            ],
            ["/basic.in.py", "/var/some/random/directory7"],
        ];

        for valid_path_staging_directory_pair in valid_path_staging_directory_pairs.iter() {
            let [path, staging_directory] = valid_path_staging_directory_pair;

            let issue = Issue {
                location: Some(Location {
                    path: path_to_string(path.to_string()),
                    range: Some(Range {
                        start_line: 1,
                        start_column: 1,
                        end_line: 1,
                        end_column: 1,
                        ..Default::default()
                    }),
                }),
                ..Default::default()
            };

            let plan = InvocationPlan {
                target_root: PathBuf::from(staging_directory),
                workspace_entries: Arc::new(vec![WorkspaceEntry {
                    path: PathBuf::from(path),
                    content_modified: SystemTime::now(),
                    language_name: None,
                    contents_size: 0,
                    kind: WorkspaceEntryKind::File,
                }]),
                invocation_id: "".to_string(),
                verb: ExecutionVerb::Check,
                workspace: Workspace::new().unwrap(),
                settings: Default::default(),
                runtime: None,
                runtime_version: None,
                plugin_name: "test".to_string(),
                plugin: PluginDef::default(),
                tool: Ruby::new_tool(""),
                driver_name: "test".to_string(),
                driver: build_driver(vec![], vec![]),
                plugin_configs: vec![],
                targets: vec![Target {
                    path: PathBuf::from(path),
                    content_modified: SystemTime::now(),
                    language_name: None,
                    contents_size: 0,
                    kind: WorkspaceEntryKind::File,
                }],
                invocation_directory: PathBuf::from(staging_directory),
                invocation_directory_def: InvocationDirectoryDef::default(),
            };

            let driver = build_driver(vec![], vec![]);
            let fixed_issue = driver.fix_issue_path(issue, &plan, &None);

            assert_eq!(fixed_issue.location.unwrap().path, "basic.in.py");
        }
    }

    #[test]
    fn test_fix_issue_path_with_prefix() {
        let path = "basic.py";
        let staging_directory = "/var/root";
        let prefix = "prefix";

        let issue = Issue {
            location: Some(Location {
                path: path_to_string(path.to_string()),
                range: Some(Range {
                    start_line: 1,
                    start_column: 1,
                    end_line: 1,
                    end_column: 1,
                    ..Default::default()
                }),
            }),
            ..Default::default()
        };

        let plan = InvocationPlan {
            target_root: PathBuf::from(staging_directory),
            workspace_entries: Arc::new(vec![WorkspaceEntry {
                path: PathBuf::from(path),
                content_modified: SystemTime::now(),
                language_name: None,
                contents_size: 0,
                kind: WorkspaceEntryKind::File,
            }]),
            invocation_id: "".to_string(),
            verb: ExecutionVerb::Check,
            workspace: Workspace::new().unwrap(),
            settings: Default::default(),
            runtime: None,
            runtime_version: None,
            plugin_name: "test".to_string(),
            plugin: PluginDef {
                prefix: Some(prefix.to_string()),
                ..Default::default()
            },
            tool: Ruby::new_tool(""),
            driver_name: "test".to_string(),
            driver: build_driver(vec![], vec![]),
            plugin_configs: vec![],
            targets: vec![Target {
                path: PathBuf::from(path),
                content_modified: SystemTime::now(),
                language_name: None,
                contents_size: 0,
                kind: WorkspaceEntryKind::File,
            }],
            invocation_directory: PathBuf::from(staging_directory),
            invocation_directory_def: InvocationDirectoryDef::default(),
        };

        let driver = build_driver(vec![], vec![]);
        let fixed_issue = driver.fix_issue_path(issue, &plan, &None);
        assert_eq!(fixed_issue.location.unwrap().path, "prefix/basic.py");
    }

    #[test]
    fn test_runs_from_tool_dir() {
        let workspace_dir = PathBuf::from("/var/root");
        let target_path = PathBuf::from("basic.py");
        let driver = build_driver(vec![], vec![]);
        let tool = Ruby::new_tool("");

        let plan = InvocationPlan {
            target_root: PathBuf::from(workspace_dir.clone()),
            workspace_entries: Arc::new(vec![WorkspaceEntry {
                path: target_path.clone(),
                content_modified: SystemTime::now(),
                language_name: None,
                contents_size: 0,
                kind: WorkspaceEntryKind::File,
            }]),
            invocation_id: "".to_string(),
            verb: ExecutionVerb::Check,
            workspace: Workspace {
                root: workspace_dir.clone(),
            },
            settings: Default::default(),
            runtime: None,
            runtime_version: None,
            plugin_name: "test".to_string(),
            plugin: PluginDef::default(),
            tool: tool.clone(),
            driver_name: "test".to_string(),
            driver: driver.clone(),
            plugin_configs: vec![],
            targets: vec![Target {
                path: target_path,
                content_modified: SystemTime::now(),
                language_name: None,
                contents_size: 0,
                kind: WorkspaceEntryKind::File,
            }],
            invocation_directory: PathBuf::from(tool.directory()),
            invocation_directory_def: InvocationDirectoryDef {
                kind: InvocationDirectoryType::ToolDir,
                path: None,
            },
        };

        let target_list = plan_target_list(&plan);

        let expected_target_list = workspace_dir.join("basic.py");
        assert_eq!(target_list, path_to_native_string(expected_target_list));
    }

    #[test]
    fn test_runs_from_tool_dir_with_staging() {
        let workspace_dir = PathBuf::from("/var/root");
        let staging_dir = PathBuf::from("/tmp/staging");
        let target_path = PathBuf::from("basic.py");
        let driver = build_driver(vec![], vec![]);
        let tool = Ruby::new_tool("");

        let plan = InvocationPlan {
            target_root: PathBuf::from(staging_dir.clone()),
            workspace_entries: Arc::new(vec![WorkspaceEntry {
                path: target_path.clone(),
                content_modified: SystemTime::now(),
                language_name: None,
                contents_size: 0,
                kind: WorkspaceEntryKind::File,
            }]),
            invocation_id: "".to_string(),
            verb: ExecutionVerb::Check,
            workspace: Workspace {
                root: workspace_dir.clone(),
            },
            settings: Default::default(),
            runtime: None,
            runtime_version: None,
            plugin_name: "test".to_string(),
            plugin: PluginDef::default(),
            tool: tool.clone(),
            driver_name: "test".to_string(),
            driver: driver.clone(),
            plugin_configs: vec![],
            targets: vec![Target {
                path: target_path,
                content_modified: SystemTime::now(),
                language_name: None,
                contents_size: 0,
                kind: WorkspaceEntryKind::File,
            }],
            invocation_directory: PathBuf::from(tool.directory()),
            invocation_directory_def: InvocationDirectoryDef {
                kind: InvocationDirectoryType::ToolDir,
                path: None,
            },
        };

        let target_list = plan_target_list(&plan);

        let expected_target_list = staging_dir.join("basic.py");
        assert_eq!(target_list, path_to_native_string(expected_target_list));
    }
}
