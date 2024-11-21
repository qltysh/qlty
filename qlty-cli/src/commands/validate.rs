use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::Args;
use qlty_check::{Executor, Planner, Settings};
use qlty_config::Workspace;
use qlty_types::analysis::v1::{ExecutionVerb, Level};

#[derive(Args, Debug)]
pub struct Validate {}

impl Validate {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let workspace = Workspace::require_initialized()?;
        workspace.fetch_sources()?;

        let settings = self.build_settings()?;
        let plan = Planner::new(ExecutionVerb::Validate, &settings)?.compute()?;

        let executor = Executor::new(&plan);
        let results = executor.install_and_invoke()?;

        let json = serde_json::to_string_pretty(&results.issues)?;
        println!("{}", json);

        CommandSuccess::ok()
    }

    fn build_settings(&self) -> Result<Settings> {
        Ok(Settings {
            root: Workspace::assert_within_git_directory()?,
            all: true,
            jobs: Some(1),
            verbose: 1,
            fail_level: Some(Level::Low),
            cache: false,
            progress: true,
            ..Default::default()
        })
    }
}
