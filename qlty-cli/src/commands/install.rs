use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::{Context, Result};
use clap::Args;
use qlty_check::planner::Plan;
use qlty_check::tool::tool_builder::ToolBuilder;
use qlty_check::{CheckFilter, Executor, Planner, Progress, Settings, Tool};
use qlty_config::Workspace;
use qlty_types::analysis::v1::ExecutionVerb;
use tracing::{debug, warn};

#[derive(Args, Clone, Debug)]
pub struct Install {
    /// Disable progress bar
    #[arg(long)]
    pub no_progress: bool,

    /// Maximum number of concurrent jobs
    #[arg(short, long)]
    pub jobs: Option<u32>,

    /// Filter by plugin or check
    #[arg(long)]
    filter: Option<String>,
    // /// Print verbose output
    // #[arg(short, long, action = clap::ArgAction::Count)]
    // pub verbose: u8,
}

impl Install {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let workspace = Workspace::require_initialized()?;
        workspace.fetch_sources()?;
        let config = workspace.config()?;

        let mut settings = Settings::default();
        settings.root = workspace.root.clone();

        if let Some(filter) = &self.filter {
            warn!("Filtering plugins: {}", filter);
            settings.filters = vec![CheckFilter {
                plugin: filter.clone(),
                rule_key: None,
            }];
        }

        let mut planner = Planner::new(ExecutionVerb::Unspecified, &settings)?;

        planner.compute_workspace_entries_strategy()?;
        planner.compute_enabled_plugins()?;

        let active_plugins = planner.active_plugins.clone();

        let mut tools = vec![];
        for active_plugin in active_plugins {
            debug!("Building tool for plugin: {}", active_plugin.name);
            let tool = ToolBuilder::new(&config, &active_plugin.name, &active_plugin.plugin)
                .build_tool()
                .with_context(|| format!("Failed to build tool for {}", active_plugin.name))?;
            tools.push(tool);
        }

        let tools = Plan::all_unique_sorted_tools(tools);
        self.install(tools)?;

        CommandSuccess::ok()
    }

    fn install(&self, tools: Vec<(String, Box<dyn Tool>)>) -> Result<()> {
        let progress = Progress::new(!self.no_progress, tools.len() as u64);
        let jobs = Planner::jobs_count(self.jobs);

        let results = Executor::install_tools(tools, jobs, progress);
        for (name, result) in results {
            result.with_context(|| format!("Failed to install {}", name))?;
        }

        Ok(())
    }
}
