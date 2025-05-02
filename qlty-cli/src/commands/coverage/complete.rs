use crate::{CommandError, CommandSuccess};
use anyhow::{anyhow, Context};
use anyhow::{bail, Result};
use clap::Args;
use console::style;
use git2::Repository;
use qlty_cloud::Client as QltyClient;
use qlty_config::version::LONG_VERSION;
use qlty_config::{QltyConfig, Workspace};
use qlty_coverage::ci::{GitHub, CI};
use qlty_coverage::eprintln_unless;
use qlty_coverage::publish::{Plan, Planner, Settings};
use regex::Regex;
use serde_json::Value;
use std::path::PathBuf;
use std::time::Instant;
use tracing::debug;

const COVERAGE_TOKEN_WORKSPACE_PREFIX: &str = "qltcw_";
const COVERAGE_TOKEN_PROJECT_PREFIX: &str = "qltcp_";
const OIDC_REGEX: &str = r"^([a-zA-Z0-9\-_]+)\.([a-zA-Z0-9\-_]+)\.([a-zA-Z0-9\-_]+)$";
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
        self.print_initial_messages();

        let settings = self.build_settings();

        self.print_settings(&settings);

        let token = self.load_auth_token()?;
        let plan = Planner::new(&Self::load_config(), &settings).compute()?;

        self.validate_plan(&plan)?;

        self.print_metadata(&plan);

        self.print_section_header(" PREPARING TO UPLOAD... ");
        self.print_authentication_info(&token);

        self.print_section_header(" UPLOADING... ");
        let timer = Instant::now();
        self.request_complete(&plan.metadata, &token)
            .context("Failed to complete coverage")?;
        self.print_complete_success(timer.elapsed().as_secs_f32());

        CommandSuccess::ok()
    }

    fn build_settings(&self) -> Settings {
        Settings {
            override_commit_sha: self.override_commit_sha.clone(),
            override_branch: self.override_branch.clone(),
            override_pull_request_number: self.override_pr_number.clone(),
            tag: self.tag.clone(),
            // Set empty values for parameters not needed in complete
            paths: vec![],
            add_prefix: None,
            strip_prefix: None,
            report_format: None,
            skip_missing_files: false,
            total_parts_count: None,
            override_build_id: None,
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

    fn print_initial_messages(&self) {
        eprintln_unless!(self.quiet, "qlty {}", LONG_VERSION.as_str());
        eprintln_unless!(self.quiet, "{}", style("https://qlty.sh/d/coverage").dim());
        eprintln_unless!(self.quiet, "");
    }

    fn print_section_header(&self, title: &str) {
        eprintln_unless!(self.quiet, "{}", style(title).bold().reverse());
        eprintln_unless!(self.quiet, "");
    }

    fn print_settings(&self, _settings: &Settings) {
        self.print_section_header(" SETTINGS ");

        eprintln_unless!(
            self.quiet,
            "    cwd: {}",
            std::env::current_dir()
                .unwrap_or(PathBuf::from("ERROR"))
                .to_string_lossy()
        );

        if let Some(tag) = &self.tag {
            eprintln_unless!(self.quiet, "    tag: {}", tag);
        }
        if let Some(override_branch) = &self.override_branch {
            eprintln_unless!(self.quiet, "    override-branch: {}", override_branch);
        }
        if let Some(override_commit_sha) = &self.override_commit_sha {
            eprintln_unless!(
                self.quiet,
                "    override-commit-sha: {}",
                override_commit_sha
            );
        }
        if let Some(override_pr_number) = &self.override_pr_number {
            eprintln_unless!(self.quiet, "    override-pr-number: {}", override_pr_number);
        }
        if let Some(project) = &self.project {
            eprintln_unless!(self.quiet, "    project: {}", project);
        }

        eprintln_unless!(self.quiet, "");
    }

    fn print_metadata(&self, plan: &Plan) {
        self.print_section_header(" METADATA ");
        if !plan.metadata.ci.is_empty() {
            eprintln_unless!(self.quiet, "    CI: {}", plan.metadata.ci);
        }

        eprintln_unless!(self.quiet, "    Commit: {}", plan.metadata.commit_sha);
        if !plan.metadata.pull_request_number.is_empty() {
            eprintln_unless!(
                self.quiet,
                "    Pull Request: #{}",
                plan.metadata.pull_request_number
            );
        }

        if !plan.metadata.branch.is_empty() {
            eprintln_unless!(self.quiet, "    Branch: {}", plan.metadata.branch);
        }

        if !plan.metadata.build_id.is_empty() {
            eprintln_unless!(self.quiet, "    Build ID: {}", plan.metadata.build_id);
        }

        eprintln_unless!(self.quiet, "");
    }

    fn print_authentication_info(&self, token: &str) {
        let token_type = if token.starts_with(COVERAGE_TOKEN_WORKSPACE_PREFIX) {
            "Workspace Token"
        } else if token.starts_with(COVERAGE_TOKEN_PROJECT_PREFIX) {
            "Project Token"
        } else if let Ok(oidc_regex) = Regex::new(OIDC_REGEX) {
            if oidc_regex.is_match(token) {
                "OIDC"
            } else {
                "Unknown"
            }
        } else {
            "ERROR"
        };
        eprintln_unless!(self.quiet, "    Auth Method: {}", token_type);
        eprintln_unless!(self.quiet, "    Token: {}", token);
        eprintln_unless!(self.quiet, "");
    }

    fn print_complete_success(&self, elapsed_seconds: f32) {
        eprintln_unless!(
            self.quiet,
            "    Coverage marked as complete in {:.2}s!",
            elapsed_seconds
        );
        eprintln_unless!(self.quiet, "");
    }

    fn load_auth_token(&self) -> Result<String> {
        self.expand_token(match &self.token {
            Some(token) => Ok(token.to_owned()),
            None => std::env::var("QLTY_COVERAGE_TOKEN").map_err(|_| {
                anyhow::Error::msg("QLTY_COVERAGE_TOKEN environment variable is required.")
            }),
        }?)
    }

    fn request_complete(
        &self,
        metadata: &qlty_types::tests::v1::CoverageMetadata,
        token: &str,
    ) -> Result<Value> {
        let client = QltyClient::new(Some(LEGACY_API_URL), Some(token.into()));
        let response_result = client.post("/coverage/complete").send_json(ureq::json!({
            "data": metadata,
        }));

        match response_result {
            Ok(resp) => resp.into_json::<Value>().map_err(|err| {
                anyhow!(
                    "JSON Error: {}: Unable to parse JSON response from success: {:?}",
                    client.base_url,
                    err
                )
            }),

            Err(ureq::Error::Status(code, resp)) => match resp.into_string() {
                Ok(body) => match serde_json::from_str::<Value>(&body) {
                    Ok(json) => match json.get("error") {
                        Some(error) => {
                            bail!("HTTP Error {}: {}: {}", code, client.base_url, error)
                        }
                        None => {
                            bail!("HTTP Error {}: {}: {}", code, client.base_url, body);
                        }
                    },
                    Err(_) => bail!(
                        "HTTP Error {}: {}: Unable to parse JSON response: {}",
                        code,
                        client.base_url,
                        body
                    ),
                },
                Err(err) => bail!(
                    "HTTP Error {}: {}: Error reading response body: {:?}",
                    code,
                    client.base_url,
                    err
                ),
            },
            Err(ureq::Error::Transport(transport_error)) => bail!(
                "Transport Error: {}: {:?}",
                client.base_url,
                transport_error
            ),
        }
    }

    /// Appends repository name to token if it is a workspace token
    fn expand_token(&self, token: String) -> Result<String> {
        if token.starts_with(COVERAGE_TOKEN_WORKSPACE_PREFIX) {
            if token.contains('/') {
                return Ok(token);
            }
            let project = if let Some(project) = &self.project {
                project.clone()
            } else if let Some(repository) = self.find_repository_name_from_env() {
                repository
            } else {
                match self.find_repository_name_from_repository() {
                    Ok(repository) => repository,
                    Err(err) => {
                        debug!("Find repository name: {}", err);
                        bail!(
                            "Could not infer project name from environment, please provide it using --project"
                        )
                    }
                }
            };
            Ok(format!("{token}/{project}"))
        } else {
            Ok(token)
        }
    }

    fn find_repository_name_from_env(&self) -> Option<String> {
        let repository = GitHub::default().repository_name();
        if repository.is_empty() {
            None
        } else {
            Self::extract_repository_name(&repository)
        }
    }

    fn find_repository_name_from_repository(&self) -> Result<String> {
        let root = Workspace::assert_within_git_directory()?;
        let repo = Repository::open(root)?;
        let remote = repo.find_remote("origin")?;
        if let Some(name) = Self::extract_repository_name(remote.url().unwrap_or_default()) {
            Ok(name)
        } else {
            bail!(
                "Could not find repository name from git remote: {:?}",
                remote.url()
            )
        }
    }

    fn extract_repository_name(value: &str) -> Option<String> {
        value
            .split('/')
            .next_back()
            .map(|s| s.strip_suffix(".git").unwrap_or(s).to_string())
            .take_if(|v| !v.is_empty())
    }

    fn load_config() -> QltyConfig {
        Workspace::new()
            .and_then(|workspace| workspace.config())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn complete(project: Option<&str>) -> Complete {
        Complete {
            project: project.map(|s| s.to_string()),
            quiet: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_expand_token_project() -> Result<()> {
        let token = complete(None).expand_token("qltcp_123".to_string())?;
        assert_eq!(token, "qltcp_123");
        Ok(())
    }

    #[test]
    fn test_expand_token_workspace_with_project() -> Result<()> {
        let token = complete(Some("test")).expand_token("qltcw_123".to_string())?;
        assert_eq!(token, "qltcw_123/test");
        Ok(())
    }

    #[test]
    fn test_expand_token_workspace_with_env() -> Result<()> {
        let token = complete(None).expand_token("qltcw_123".to_string())?;
        assert!(token.starts_with("qltcw_123/"));

        std::env::set_var("GITHUB_REPOSITORY", "");
        let token = complete(None).expand_token("qltcw_123".to_string())?;
        assert!(token.starts_with("qltcw_123/"));

        std::env::set_var("GITHUB_REPOSITORY", "a/b.git");
        let token = complete(None).expand_token("qltcw_123".to_string())?;
        assert_eq!(token, "qltcw_123/b");

        std::env::set_var("GITHUB_REPOSITORY", "b/c");
        let token = complete(None).expand_token("qltcw_123".to_string())?;
        assert_eq!(token, "qltcw_123/c");

        Ok(())
    }

    #[test]
    fn test_expand_token_already_expanded() -> Result<()> {
        let token = complete(Some("test")).expand_token("qltcw_123/abc".to_string())?;
        assert_eq!(token, "qltcw_123/abc");
        Ok(())
    }

    #[test]
    fn test_extract_repository_name() {
        assert_eq!(Complete::extract_repository_name(""), None);
        assert_eq!(Complete::extract_repository_name("a/"), None);
        assert_eq!(
            Complete::extract_repository_name("git@example.org:a/b"),
            Some("b".into())
        );
        assert_eq!(
            Complete::extract_repository_name("ssh://x@example.org:a/b"),
            Some("b".into())
        );
        assert_eq!(
            Complete::extract_repository_name("https://x:y@example.org/a/b"),
            Some("b".into())
        );
    }
}
