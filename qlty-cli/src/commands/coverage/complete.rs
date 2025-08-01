use super::utils::{
    print_authentication_info, print_initial_messages, print_metadata, print_settings,
    validate_metadata,
};
use crate::{CommandError, CommandSuccess};
use anyhow::{Context, Result};
use clap::Args;
use console::style;
use qlty_cloud::{get_legacy_api_url, Client as QltyClient};
use qlty_coverage::{publish::Settings, token::load_auth_token};
use std::time::Instant;

#[derive(Debug, Default, Clone)]
pub struct CompleteResult {
    pub url: Option<String>,
}

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

    #[arg(long)]
    /// Override the commit time from git metadata. Accepts a Unix timestamp (seconds since epoch) or RFC3339/ISO8601 format
    pub override_commit_time: Option<String>,

    #[arg(long)]
    /// Override the git tag from the CI environment
    pub override_git_tag: Option<String>,

    #[arg(long, short)]
    /// The token to use for authentication when uploading the report.
    /// By default, it retrieves the token from the QLTY_COVERAGE_TOKEN environment variable.
    pub token: Option<String>,

    #[arg(long)]
    /// The name of the project to associate the coverage report with. Only needed when coverage token represents a
    /// workspace and if it cannot be inferred from the git origin.
    pub project: Option<String>,

    #[arg(long)]
    /// Perform a dry-run without actually completing the coverage
    pub dry_run: bool,

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
        let metadata_planner =
            qlty_coverage::publish::MetadataPlanner::new(&settings, qlty_coverage::ci::current());
        let metadata = metadata_planner.compute()?;

        validate_metadata(&metadata)?;

        self.print_section_header(" METADATA ");
        print_metadata(&metadata, self.quiet);

        self.print_section_header(" AUTHENTICATION ");
        print_authentication_info(&token, self.quiet);

        let timer = Instant::now();
        self.print_section_header(" COMPLETING... ");

        if self.dry_run {
            self.print_complete_success(timer.elapsed().as_secs_f32(), &None);
        } else {
            let result =
                Self::request_complete(&metadata, &token).context("Failed to complete coverage")?;
            self.print_complete_success(timer.elapsed().as_secs_f32(), &result.url);
        }

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
            override_commit_time: self.override_commit_time.clone(),
            override_git_tag: self.override_git_tag.clone(),
            tag: self.tag.clone(),
            quiet: self.quiet,
            project: self.project.clone(),
            ..Default::default()
        }
    }

    fn print_complete_success(&self, elapsed_seconds: f32, url: &Option<String>) {
        if self.quiet {
            return;
        }

        eprintln!("    Coverage marked as complete in {elapsed_seconds:.2}s!");

        if let Some(url) = url {
            eprintln!("    {}", style(format!("View report: {url}")).bold());
        }

        eprintln!();
    }

    fn request_complete(
        metadata: &qlty_types::tests::v1::CoverageMetadata,
        token: &str,
    ) -> Result<CompleteResult> {
        let legacy_api_url = get_legacy_api_url();
        let client = QltyClient::new(Some(&legacy_api_url), Some(token.into()));
        let response = client.post_coverage_metadata("/coverage/complete", metadata)?;

        let url = response
            .get("data")
            .and_then(|data| data.get("url"))
            .and_then(|url| url.as_str())
            .map(String::from);

        Ok(CompleteResult { url })
    }
}
