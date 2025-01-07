use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::Args;
use qlty_check::planner::{plugin_supported_on_platform, Plan};
use qlty_check::tool::tool_builder::ToolBuilder;
use qlty_check::{Executor, Planner, Progress, Tool};
use qlty_config::config::IssueMode;
use qlty_config::Workspace;

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

        let mut tools = vec![];
        for plugin in &config.plugin {
            if plugin.mode == IssueMode::Disabled {
                continue;
            }

            if let Some(plugin_def) = config.plugins.definitions.get(&plugin.name) {
                if !plugin_supported_on_platform(plugin_def, &plugin.name) {
                    continue;
                }

                let mut plugin_def = plugin_def.clone();
                if plugin.version != "latest" {
                    plugin_def.version = Some(plugin.version.clone());
                }

                let tool = ToolBuilder::new(&config, &plugin.name, &plugin_def)
                    .build_tool()
                    .unwrap();
                tools.push(tool);
            } else {
                log::warn!("Plugin {} not found in plugins definitions", plugin.name);
            }
        }

        let tools = Plan::all_unique_sorted_tools(tools);
        self.install(tools)?;

        CommandSuccess::ok()
    }

    fn install(&self, tools: Vec<(String, Box<dyn Tool>)>) -> Result<()> {
        let progress = Progress::new(!self.no_progress, tools.len() as u64);
        let jobs = Planner::jobs_count(self.jobs);

        Executor::install_tools(tools, jobs, progress)
    }
}
