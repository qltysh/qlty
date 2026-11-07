use super::utils::{
    print_authentication_info, print_initial_messages, print_metadata, print_settings,
    validate_minimal_metadata,
};
use crate::{CommandError, CommandSuccess};
use anyhow::{Context, Result};
use clap::Args;
use console::style;
use qlty_cloud::{get_legacy_api_url, Client as QltyClient};
use qlty_coverage::{publish::Settings, token::load_auth_token};
use qlty_types::tests::v1::CoverageMetadata;
use serde_json::Value;
use std::time::Instant;

#[derive(Debug, Args, Default)]
pub struct Skip {
    #[arg(long)]
    /// The tag whose coverage was intentionally not run
    pub tag: Option<String>,

    #[arg(long)]
    /// Why coverage was not run, shown in the commit status
    pub reason: Option<String>,

    #[arg(long)]
    /// Override the commit SHA from the CI environment
    pub override_commit_sha: Option<String>,

    #[arg(long)]
    /// Override the branch from the CI environment
    pub override_branch: Option<String>,

    #[arg(long)]
    /// Override the pull request number from the CI environment
    pub override_pr_number: Option<String>,

    #[arg(long, short)]
    /// The token to use for authentication.
    /// By default, it retrieves the token from the QLTY_COVERAGE_TOKEN environment variable.
    pub token: Option<String>,

    #[arg(long)]
    /// The name of the project. Only needed when coverage token represents a
    /// workspace and if it cannot be inferred from the git origin.
    pub project: Option<String>,

    #[arg(long)]
    /// Perform a dry-run without contacting Qlty Cloud
    pub dry_run: bool,

    #[clap(long, short)]
    pub quiet: bool,
}

impl Skip {
    pub fn execute(&self, _args: &crate::Arguments) -> Result<CommandSuccess, CommandError> {
        print_initial_messages(self.quiet);

        let settings = self.build_settings();

        self.print_section_header(" SETTINGS ");
        print_settings(&settings);

        let token = load_auth_token(&self.token, self.project.as_deref())?;
        let metadata =
            qlty_coverage::publish::MetadataPlanner::new(&settings, qlty_coverage::ci::current())
                .compute_from_ci();

        validate_minimal_metadata(&metadata)?;

        self.print_section_header(" METADATA ");
        print_metadata(&metadata, self.quiet);

        self.print_section_header(" AUTHENTICATION ");
        print_authentication_info(&token, self.quiet);

        let timer = Instant::now();
        self.print_section_header(" SKIPPING... ");

        if !self.dry_run {
            let legacy_api_url = get_legacy_api_url();
            QltyClient::new(Some(&legacy_api_url), Some(token))
                .post_coverage_data("/coverage/skip", &self.request_data(&metadata)?)
                .context("Failed to skip coverage")?;
        }

        if !self.quiet {
            eprintln!(
                "    Coverage skip accepted in {:.2}s!",
                timer.elapsed().as_secs_f32()
            );
            eprintln!();
        }

        CommandSuccess::ok()
    }

    fn request_data(&self, metadata: &CoverageMetadata) -> Result<Value> {
        let mut data =
            serde_json::to_value(metadata).context("Failed to serialize coverage metadata")?;
        if let Some(reason) = &self.reason {
            data["skipReason"] = Value::from(reason.as_str());
        }
        Ok(data)
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
            tag: self.tag.clone(),
            quiet: self.quiet,
            project: self.project.clone(),
            dry_run: self.dry_run,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata() -> CoverageMetadata {
        CoverageMetadata {
            commit_sha: "2ca1bc45a94e37c8dbae6fd9e19fc069ba64bd67".to_string(),
            tag: Some("backend".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn request_data_adds_the_reason_to_the_metadata() {
        let skip = Skip {
            reason: Some("No backend changes".to_string()),
            ..Default::default()
        };

        let data = skip.request_data(&metadata()).unwrap();

        assert_eq!(
            data["commitSha"],
            "2ca1bc45a94e37c8dbae6fd9e19fc069ba64bd67"
        );
        assert_eq!(data["tag"], "backend");
        assert_eq!(data["skipReason"], "No backend changes");
    }

    #[test]
    fn request_data_omits_the_reason_when_none_is_given() {
        let data = Skip::default().request_data(&metadata()).unwrap();

        assert_eq!(data.get("skipReason"), None);
    }
}
