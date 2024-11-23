use crate::ui::Steps;
use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::{Args, ValueEnum};
use cli_table::{
    format::{Border, HorizontalLine, Justify, Separator, VerticalLine},
    print_stdout, Cell, Table,
};
use console::style;
use console::Emoji;
use qlty_analysis::git::compute_upstream;
use qlty_analysis::workspace_entries::{TargetMode, WorkspaceEntryFinderBuilder};
use qlty_analysis::Report;
use qlty_config::Workspace;
use qlty_smells::metrics::{Executor, MetricsMode, Plan, Planner, Processor, Settings};
use qlty_types::analysis::v1::Stats;
use std::path::{Path, PathBuf};

static EYES: Emoji<'_, '_> = Emoji("👀  ", "");
static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
static SPARKLES: Emoji<'_, '_> = Emoji("✨  ", "");
static THINKING: Emoji<'_, '_> = Emoji("🤔  ", "");

const DEFAULT_DEPTH: usize = 100;

#[derive(Args, Debug)]
pub struct Metrics {
    /// Compute metrics for all files, not just changed
    #[arg(short, long, conflicts_with = "upstream")]
    pub all: bool,

    /// Print per-directory stats
    #[arg(short, long)]
    pub dirs: bool,

    /// Print function stats
    #[arg(long, conflicts_with = "dirs")]
    pub functions: bool,

    /// Directory depth to print, this flag will also set to print per-directory stats
    #[arg(long, conflicts_with = "functions")]
    pub max_depth: Option<usize>,

    /// Sort output by column
    #[arg(long, value_enum)]
    pub sort: Option<Sort>,

    /// Maximum rows to print
    #[arg(long)]
    pub limit: Option<usize>,

    /// Exclude tests
    #[arg(long)]
    pub exclude_tests: bool,

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Sort {
    Name,
    Classes,
    Functions,
    Fields,
    Lines,
    Loc,
    Complexity,
    Lcom,
}

impl Metrics {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        self.run_assertions()?;

        let workspace = Workspace::new()?;
        workspace.fetch_sources()?;

        let mut steps = Steps::new(self.quiet, 3);
        steps.start(THINKING, "Planning... ");

        let config = workspace.config()?;
        let target_mode = self.compute_target_mode(&workspace);
        let mut workspace_entry_finder_builder = WorkspaceEntryFinderBuilder {
            mode: target_mode.clone(),
            paths: self.paths.clone(),
            config: config.clone(),
            ..Default::default()
        };
        let files = workspace_entry_finder_builder.build()?.files()?;

        let settings = Settings {
            functions: self.functions,
            exclude_tests: self.exclude_tests,
            target_mode: target_mode.clone(),
        };

        let planner = Planner::new(&config, &settings, files);
        let plan = planner.compute()?;

        steps.start(
            LOOKING_GLASS,
            format!("Analyzing {}...", plan.description()),
        );

        let mut executor = Executor::new(&plan);
        let results = executor.execute();

        steps.start(
            EYES,
            format!("Parsing {} files... ", plan.source_files.len()),
        );

        let mut processor = Processor::new(results);
        let report = processor.compute()?;

        steps.start(SPARKLES, "Reporting... ");
        self.print(plan.mode, &report)?;
        self.print_target_suggestion_if_necessary(&plan, &target_mode);

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

    fn print_target_suggestion_if_necessary(&self, plan: &Plan, target_mode: &TargetMode) {
        if !plan.source_files.is_empty() {
            return;
        }

        eprintln!();

        let target_name = match target_mode {
            TargetMode::UpstreamDiff(upstream) => upstream.to_owned(),
            TargetMode::HeadDiff => String::from("HEAD"),
            _ => return,
        };

        eprintln!(
            "{} No changes detected vs. {}. Consider running {}",
            style("NOTE:").bold(),
            style(target_name).bold(),
            style("qlty metrics --all").cyan().bold()
        );
    }

    fn print(&self, mode: MetricsMode, report: &Report) -> Result<()> {
        if self.json {
            self.print_json(report)
        } else {
            self.print_text(mode, report)
        }
    }

    fn print_json(&self, report: &Report) -> Result<()> {
        let json = serde_json::to_string_pretty(report)?;
        println!("{}", json);
        Ok(())
    }

    fn print_text(&self, mode: MetricsMode, report: &Report) -> Result<()> {
        match mode {
            MetricsMode::Files => print_tabular_report(self, report),
            MetricsMode::Functions => print_functions_report(report),
        }
    }
}

fn print_functions_report(report: &Report) -> Result<()> {
    let cwd = std::env::current_dir()?;

    for (path, function_stats) in report.function_stats_by_path() {
        print_path(&path, &cwd);

        if !function_stats.is_empty() {
            let mut rows = vec![];

            for stats in function_stats {
                rows.push(vec![
                    style(stats.name).cyan().cell(),
                    stats.fields.unwrap().cell().justify(Justify::Right),
                    stats.cyclomatic.unwrap().cell().justify(Justify::Right),
                    stats.complexity.unwrap().cell().justify(Justify::Right),
                    stats.lines.unwrap().cell().justify(Justify::Right),
                    stats.code_lines.unwrap().cell().justify(Justify::Right),
                ]);
            }

            let table = rows
                .table()
                .title(vec![
                    "function".cell(),
                    "fields".cell().justify(Justify::Right),
                    "cyclo".cell().justify(Justify::Right),
                    "cognitive".cell().justify(Justify::Right),
                    "lines".cell().justify(Justify::Right),
                    "loc".cell().justify(Justify::Right),
                ])
                .border(Border::builder().build())
                .separator(
                    Separator::builder()
                        .title(Some(HorizontalLine::default()))
                        .column(Some(VerticalLine::default()))
                        .build(),
                );

            println!();
            print_stdout(table)?;
        }

        println!();
    }

    Ok(())
}

pub fn print_tabular_report(arguments: &Metrics, report: &Report) -> Result<()> {
    let mut stats = vec![];

    if arguments.dirs || arguments.max_depth.is_some() {
        let max_depth = arguments.max_depth.unwrap_or(DEFAULT_DEPTH);
        for stat in report.directory_stats() {
            let directory_path = PathBuf::from(&stat.path);
            let depth = directory_path.components().count();

            if depth <= max_depth {
                stats.push(stat);
            }
        }
    } else {
        stats.extend(report.file_stats());
    }

    stats.sort_by(|stats_a, stats_b| match arguments.sort {
        Some(Sort::Name) => stats_a
            .fully_qualified_name
            .cmp(&stats_b.fully_qualified_name),
        Some(Sort::Classes) => stats_b.classes.cmp(&stats_a.classes),
        Some(Sort::Functions) => stats_b.functions.cmp(&stats_a.functions),
        Some(Sort::Fields) => stats_b.fields.cmp(&stats_a.fields),
        Some(Sort::Lines) => stats_b.lines.cmp(&stats_a.lines),
        Some(Sort::Loc) => stats_b.code_lines.cmp(&stats_a.code_lines),
        Some(Sort::Complexity) => stats_b.complexity.cmp(&stats_a.complexity),
        Some(Sort::Lcom) => stats_b.lcom4.cmp(&stats_a.lcom4),
        None => stats_a
            .fully_qualified_name
            .cmp(&stats_b.fully_qualified_name),
    });

    let total_rows = stats.len();
    let mut rows = stats.clone();

    if let Some(limit) = arguments.limit {
        rows.truncate(limit);
    }

    let mut table_rows: Vec<_> = rows
        .into_iter()
        .map(|stats| {
            vec![
                stats.fully_qualified_name.cell(),
                stats.classes.unwrap().cell().justify(Justify::Right),
                stats.functions.unwrap().cell().justify(Justify::Right),
                stats.fields.unwrap().cell().justify(Justify::Right),
                stats.cyclomatic.unwrap().cell().justify(Justify::Right),
                stats.complexity.unwrap().cell().justify(Justify::Right),
                stats.lcom4.unwrap().cell().justify(Justify::Right),
                stats.lines.unwrap().cell().justify(Justify::Right),
                stats.code_lines.unwrap().cell().justify(Justify::Right),
            ]
        })
        .collect();

    if arguments.limit.is_none() || table_rows.len() == total_rows {
        let mut total = Stats::default();

        for stat in stats {
            total = total + stat;
        }

        table_rows.push(vec![
            style("TOTAL").bold().cell().cell(),
            total
                .classes
                .unwrap_or_default()
                .cell()
                .justify(Justify::Right),
            total
                .functions
                .unwrap_or_default()
                .cell()
                .justify(Justify::Right),
            total
                .fields
                .unwrap_or_default()
                .cell()
                .justify(Justify::Right),
            total
                .cyclomatic
                .unwrap_or_default()
                .cell()
                .justify(Justify::Right),
            total
                .complexity
                .unwrap_or_default()
                .cell()
                .justify(Justify::Right),
            total
                .lcom4
                .unwrap_or_default()
                .cell()
                .justify(Justify::Right),
            total
                .lines
                .unwrap_or_default()
                .cell()
                .justify(Justify::Right),
            total
                .code_lines
                .unwrap_or_default()
                .cell()
                .justify(Justify::Right),
        ]);
    }

    let table = table_rows
        .table()
        .title(vec![
            "name".cell(),
            "classes".cell().justify(Justify::Right),
            "funcs".cell().justify(Justify::Right),
            "fields".cell().justify(Justify::Right),
            "cyclo".cell().justify(Justify::Right),
            "complex".cell().justify(Justify::Right),
            "LCOM".cell().justify(Justify::Right),
            "lines".cell().justify(Justify::Right),
            "LOC".cell().justify(Justify::Right),
        ])
        .border(Border::builder().build())
        .separator(
            Separator::builder()
                .title(Some(HorizontalLine::default()))
                .column(Some(VerticalLine::default()))
                .build(),
        );

    println!();
    print_stdout(table)?;

    Ok(())
}

fn print_path(path: &Path, root: &PathBuf) {
    let path = path.strip_prefix(root).unwrap_or(path);
    println!("{}", style(path.display()).magenta().bold());
}
