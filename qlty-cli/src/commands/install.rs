use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::Args;
use qlty_check::{CheckFilter, Executor};
use qlty_check::{Planner, Settings};
use qlty_config::Workspace;
use qlty_types::analysis::v1::ExecutionVerb;

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

        let plan = Planner::new(
            ExecutionVerb::Check,
            &Settings {
                root: workspace.root,
                all: true,
                cache: false,
                filters: CheckFilter::from_optional_list(self.filter.clone()),
                jobs: self.jobs,
                ..Default::default()
            },
        )?
        .compute()?;

        Executor::new(&plan).install()?;
        CommandSuccess::ok()
    }
}
