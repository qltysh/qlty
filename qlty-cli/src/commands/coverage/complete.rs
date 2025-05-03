use super::utils::{
    load_config, print_authentication_info, print_initial_messages, print_metadata, print_settings,
};
use crate::{CommandError, CommandSuccess};
use anyhow::{bail, Context, Result};
use clap::Args;
use console::style;
use qlty_cloud::Client as QltyClient;
use qlty_coverage::{
    publish::{Plan, Planner, Settings},
    token::load_auth_token,
};
use serde_json::Value;
use std::time::Instant;

const LEGACY_API_URL: &str = "https://qlty.sh/api";

#[derive(Debug, Args, Default)]
pub struct Complete {
    #[arg(long)]
    pub tag: Option<String>,

    #[arg(long)]
    /// Override the branch from the CI environment
    pub override_branch: Option<String>,

    #[arg(long)]
    /// Override the commit SHA from the CI environment
    pub override_commit_sha: Option<String>,

    #[arg(long)]
    /// Override the pull request number from the CI environment
    pub override_pr_number: Option<String>,

    #[arg(long)]
    /// Override the build identifier from the CI environment
    pub override_build_id: Option<String>,

    #[arg(long, short)]
    /// The token to use for authentication when uploading the report.
    /// By default, it retrieves the token from the QLTY_COVERAGE_TOKEN environment variable.
    pub token: Option<String>,

    #[arg(long)]
    /// The name of the project to associate the coverage report with. Only needed when coverage token represents a
    /// workspace and if it cannot be inferred from the git origin.
    pub project: Option<String>,

    #[clap(long, short)]
    pub quiet: bool,
}

impl Complete {
    pub fn execute(&self, _args: &crate::Arguments) -> Result<CommandSuccess, CommandError> {
        print_initial_messages(self.quiet);

        let settings = self.build_settings();

        self.print_section_header(" SETTINGS ");
        print_settings(&settings);

        let token = load_auth_token(&self.token, self.project.as_deref())?;
        let plan = Planner::new(&load_config(), &settings).compute()?;

        self.validate_plan(&plan)?;

        self.print_section_header(" METADATA ");
        print_metadata(&plan, self.quiet);

        self.print_section_header(" AUTHENTICATION ");
        print_authentication_info(&token, self.quiet);

        let timer = Instant::now();
        Self::request_complete(&plan.metadata, &token).context("Failed to complete coverage")?;
        self.print_complete_success(timer.elapsed().as_secs_f32());

        CommandSuccess::ok()
    }

    fn print_section_header(&self, title: &str) {
        if self.quiet {
            return;
        }

        eprintln!("{}", style(title).bold().reverse());
        eprintln!();
    }

    fn build_settings(&self) -> Settings {
        Settings {
            override_commit_sha: self.override_commit_sha.clone(),
            override_branch: self.override_branch.clone(),
            override_pull_request_number: self.override_pr_number.clone(),
            override_build_id: self.override_build_id.clone(),
            tag: self.tag.clone(),
            quiet: self.quiet,
            project: self.project.clone(),
            ..Default::default()
        }
    }

    fn validate_plan(&self, plan: &Plan) -> Result<()> {
        if plan.metadata.commit_sha.is_empty() {
            bail!(
                "Unable to determine commit SHA from the environment.\nPlease provide it using --override-commit-sha"
            )
        }

        Ok(())
    }

    fn print_complete_success(&self, elapsed_seconds: f32) {
        if self.quiet {
            return;
        }

        eprintln!("    Coverage marked as complete in {elapsed_seconds:.2}s!");
        eprintln!();
    }

    fn request_complete(
        metadata: &qlty_types::tests::v1::CoverageMetadata,
        token: &str,
    ) -> Result<Value> {
        let client = QltyClient::new(Some(LEGACY_API_URL), Some(token.into()));
        client.post_coverage_metadata("/coverage/complete", metadata)
    }
}
