use crate::ui::Highlighter;
use crate::ui::Steps;
use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::{Context, Result};
use clap::Args;
use console::{style, Emoji};
use itertools::Itertools;
use qlty_analysis::code::File;
use qlty_analysis::git::compute_upstream;
use qlty_analysis::workspace_entries::TargetMode;
use qlty_analysis::workspace_entries::WorkspaceEntryFinderBuilder;
use qlty_analysis::Report;
use qlty_config::{QltyConfig, Workspace};
use qlty_types::analysis::v1::Issue;
use std::{path::PathBuf, sync::Arc};

static EYES: Emoji<'_, '_> = Emoji("👀  ", "");
static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
static THINKING: Emoji<'_, '_> = Emoji("🤔  ", "");
static SPARKLES: Emoji<'_, '_> = Emoji("✨  ", "");

#[derive(Args, Debug, Default)]
pub struct Smells {
    /// Compute smells for all files, not just changed
    #[arg(short, long, conflicts_with = "upstream")]
    pub all: bool,

    /// Don't check for duplication
    #[arg(long)]
    pub no_duplication: bool,

    /// Include tests
    #[arg(long)]
    pub include_tests: bool,

    /// Don't show code snippets
    #[arg(long)]
    pub no_snippets: bool,

    /// Upstream base ref to compare against
    #[arg(long)]
    pub upstream: Option<String>,

    /// Only show results
    #[arg(long)]
    pub quiet: bool,

    /// JSON output
    #[arg(long, hide = true)]
    json: bool,

    /// Files to analyze
    pub paths: Vec<PathBuf>,
}

impl Smells {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        self.run_assertions()?;

        let workspace = Workspace::new()?;
        workspace.fetch_sources()?;

        let config = workspace.config()?;

        let mut steps_count = 2;

        if !self.no_duplication {
            steps_count += 1;
        }
        let target_mode = self.compute_target_mode(&workspace);

        let mut steps = Steps::new(self.quiet, steps_count);
        steps.start(LOOKING_GLASS, self.analyzing_message(&target_mode));

        let target_mode = self.compute_target_mode(&workspace);
        let mut workspace_entry_finder_builder = WorkspaceEntryFinderBuilder {
            mode: target_mode.clone(),
            paths: self.paths.clone(),
            config: config.clone(),
            exclude_tests: !self.include_tests,
            ..Default::default()
        };
        let files = workspace_entry_finder_builder.build()?.files()?;

        steps.start(
            EYES,
            format!("Checking structure of {} files... ", files.len()),
        );
        let mut report = self.run_structure(&config, &files)?;

        if !self.no_duplication {
            steps.start(
                THINKING,
                format!("Looking for duplication across {} files... ", files.len()),
            );

            report.merge(&self.run_duplication(&target_mode, &config, &files)?);
        }

        report.relativeize_paths(&workspace.root);

        steps.start(SPARKLES, "Reporting... ");
        println!();
        self.write_stdout(&workspace, &report.issues)?;

        CommandSuccess::ok()
    }

    fn compute_target_mode(&self, workspace: &Workspace) -> TargetMode {
        if self.all {
            TargetMode::All
        } else if !self.paths.is_empty() {
            TargetMode::Paths(self.paths.len())
        } else if let Some(upstream) = compute_upstream(workspace, &self.upstream) {
            TargetMode::UpstreamDiff(upstream)
        } else {
            TargetMode::HeadDiff
        }
    }

    fn run_assertions(&self) -> Result<(), CommandError> {
        self.assert_mutually_exclusive_options()
    }

    fn assert_mutually_exclusive_options(&self) -> Result<(), CommandError> {
        if self.all && !self.paths.is_empty() {
            let message = format!(
                "the argument '{}' cannot be used with specified {}",
                style("--all").yellow(),
                style("[PATHS]").yellow()
            );

            return Err(CommandError::InvalidOptions { message });
        }

        if self.upstream.is_some() && !self.paths.is_empty() {
            let message = format!(
                "the argument '{}' cannot be used with specified {}",
                style("--upstream").yellow(),
                style("[PATHS]").yellow()
            );

            return Err(CommandError::InvalidOptions { message });
        }

        Ok(())
    }

    fn run_structure(&self, config: &QltyConfig, files: &[Arc<File>]) -> Result<Report> {
        let planner = qlty_smells::structure::Planner::new(config, files.to_vec())?;
        let plan = planner.compute()?;

        let mut executor = qlty_smells::structure::Executor::new(&plan);
        executor.execute();

        Ok(executor.report())
    }

    fn run_duplication(
        &self,
        target_mode: &TargetMode,
        config: &QltyConfig,
        files: &[Arc<File>],
    ) -> Result<Report> {
        // No need to run duplication if there are no files to analyze
        if files.is_empty() {
            return Ok(Report::default());
        }

        let paths = if self.paths.is_empty() {
            match target_mode {
                TargetMode::HeadDiff | TargetMode::UpstreamDiff(_) => {
                    files.iter().map(|file| file.path.clone()).collect()
                }
                _ => vec![],
            }
        } else {
            self.paths.clone()
        };

        let settings = qlty_smells::duplication::Settings {
            paths,
            include_tests: self.include_tests,
        };

        let mut workspace_entry_finder_builder = WorkspaceEntryFinderBuilder {
            mode: TargetMode::All,
            paths: vec![],
            config: config.clone(),
            exclude_tests: !self.include_tests,
            ..Default::default()
        };

        let planner = qlty_smells::duplication::Planner::new(
            config,
            &settings,
            workspace_entry_finder_builder.build()?.files()?.to_vec(),
        )?;
        let plan = planner.compute()?;

        let mut executor = qlty_smells::duplication::Executor::new(&plan);
        executor.execute();

        Ok(executor.report())
    }

    fn write_stdout(&self, workspace: &Workspace, issues: &[Issue]) -> Result<()> {
        if self.json {
            self.write_stdout_json(issues)
        } else {
            self.write_stdout_text(workspace, issues)
        }
    }

    fn write_stdout_json(&self, issues: &[Issue]) -> Result<()> {
        let sorted_issues: Vec<_> = issues
            .iter()
            .sorted_by_key(|issue| {
                (
                    &issue.tool,
                    &issue.driver,
                    &issue.rule_key,
                    issue.path().unwrap(),
                    issue.range().unwrap_or_default().start_line,
                )
            })
            .collect();

        let json = serde_json::to_string_pretty(&sorted_issues)?;
        println!("{}", json);
        Ok(())
    }

    fn analyzing_message(&self, target_mode: &TargetMode) -> String {
        let suffix = match target_mode {
            TargetMode::All => " all targets".to_string(),
            TargetMode::Sample(ref samples) => {
                if *samples == 1 {
                    " 1 sample".to_string()
                } else {
                    format!(" {} samples", samples)
                }
            }
            TargetMode::Paths(ref paths) => {
                if *paths == 1 {
                    " 1 path".to_string()
                } else {
                    format!(" {} paths", paths)
                }
            }
            TargetMode::UpstreamDiff(ref upstream) => format!(" vs. {}", upstream),
            TargetMode::HeadDiff => " vs. HEAD".to_string(),
            TargetMode::Index => " index".to_string(),
            TargetMode::IndexFile(ref file) => format!(" index file {}", file.display()),
        };

        format!("Analyzing{}...", suffix)
    }

    fn write_stdout_text(&self, workspace: &Workspace, issues: &[Issue]) -> Result<()> {
        let cwd = std::env::current_dir().expect("Unable to identify current directory");
        let issues_by_path = issues
            .iter()
            .into_group_map_by(|issue| issue.path().map(PathBuf::from));

        println!();

        for path in issues_by_path.keys().sorted() {
            let mut path_buf = workspace.root.clone();
            path_buf.push(&path.clone().unwrap());

            let path_relative_to_cwd = path_buf
                .strip_prefix(&cwd)
                .with_context(|| format!("Unable to strip prefix {:?} from {:?}", cwd, path_buf))?;

            println!("{}", style(path_relative_to_cwd.display()).cyan());

            let mut highlighter = None;

            for issue in issues_by_path.get(path).unwrap() {
                if self.no_snippets {
                    println!(
                        "{} {}",
                        style(format!("{:>4} ", issue.range().unwrap().start_line)).dim(),
                        style(&issue.message).bold()
                    );
                } else {
                    println!("    {}", style(&issue.message).bold());
                }

                if !self.no_snippets {
                    println!();

                    if highlighter.as_ref().is_none() {
                        highlighter = Some(Highlighter::new(&path_buf)?);
                    }

                    highlighter
                        .as_ref()
                        .unwrap()
                        .print_range(issue.line_range().unwrap(), 8, 6);
                }

                for other_location in &issue.other_locations {
                    println!("        also found at {}", other_location.path);
                }

                if !self.no_snippets {
                    println!();
                }
            }

            println!();
        }

        Ok(())
    }
}
