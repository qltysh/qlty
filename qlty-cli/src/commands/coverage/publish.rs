use crate::{CommandError, CommandSuccess};
use anyhow::{bail, Result};
use clap::Args;
use console::style;
use git2::Repository;
use indicatif::HumanBytes;
use qlty_config::version::LONG_VERSION;
use qlty_config::{QltyConfig, Workspace};
use qlty_coverage::ci::{GitHub, CI};
use qlty_coverage::eprintln_unless;
use qlty_coverage::formats::Formats;
use qlty_coverage::print::{print_report_as_json, print_report_as_text};
use qlty_coverage::publish::{Planner, Processor, Reader, Report, Settings, Upload};
use std::path::PathBuf;
use std::time::Instant;
use tracing::debug;

const COVERAGE_TOKEN_WORKSPACE_PREFIX: &str = "qltcw_";

#[derive(Debug, Args)]
pub struct Publish {
    #[clap(long)]
    /// Do not upload the coverage report, only export it to the output directory.
    pub dry_run: bool,

    #[arg(long, value_enum)]
    /// The format of the coverage report to transform. If not specified, the format will be inferred from the file extension or contents.
    pub report_format: Option<Formats>,

    #[arg(long, hide = true)]
    pub output_dir: Option<PathBuf>,

    #[arg(long)]
    pub tag: Option<String>,

    #[arg(long)]
    /// Override the build identifier from the CI environment
    pub override_build_id: Option<String>,

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
    /// The prefix to add to file paths in coverage payloads, to make them match the project's directory structure.
    pub transform_add_prefix: Option<String>,

    #[arg(long)]
    /// The prefix to remove from absolute paths in coverage payloads to make them relative to the project root.
    /// This is usually the directory in which the tests were run. Defaults to the root of the git repository.
    pub transform_strip_prefix: Option<String>,

    #[arg(long, short)]
    /// The token to use for authentication when uploading the report. By default, it retrieves the token from the QLTY_COVERAGE_TOKEN environment variable.
    pub token: Option<String>,

    #[arg(long)]
    /// The name of the project to associate the coverage report with. Only needed when coverage token represents a
    /// workspace and if it cannot be inferred from the git origin.
    pub project: Option<String>,

    #[arg(long)]
    /// Print coverage
    pub print: bool,

    #[arg(long, hide = true, requires = "print")]
    /// JSON output
    pub json: bool,

    #[clap(long, short)]
    pub quiet: bool,

    // Paths to coverage reports
    pub paths: Vec<String>,
}

impl Publish {
    // TODO: Use CommandSuccess and CommandError, which is not straight forward since those types aren't available here.
    pub fn execute(&self, _args: &crate::Arguments) -> Result<CommandSuccess, CommandError> {
        self.print_initial_messages();

        let token = match self.load_auth_token() {
            Ok(token) => token,
            Err(err) => {
                eprintln!("{}", style(format!("{}", err)).red());
                std::process::exit(1);
            }
        };

        eprintln_unless!(self.quiet, "  Retrieving CI metadata...");
        let plan = Planner::new(
            &Self::load_config(),
            &Settings {
                override_build_id: self.override_build_id.clone(),
                override_commit_sha: self.override_commit_sha.clone(),
                override_branch: self.override_branch.clone(),
                override_pull_request_number: self.override_pr_number.clone(),
                add_prefix: self.transform_add_prefix.clone(),
                strip_prefix: self.transform_strip_prefix.clone(),
                tag: self.tag.clone(),
                report_format: self.report_format,
                paths: self.paths.clone(),
            },
        )
        .compute()?;

        eprintln_unless!(
            self.quiet,
            "{}",
            style(format!(
                "  → {} CI commit {:?} on branch {:?}",
                plan.metadata.ci, plan.metadata.commit_sha, plan.metadata.branch
            ))
            .dim()
        );
        eprintln_unless!(self.quiet, "");

        eprintln_unless!(self.quiet, "  Reading code coverage data...");
        let results = Reader::new(&plan).read()?;
        let mut report = Processor::new(&plan, results).compute()?;
        eprintln_unless!(
            self.quiet,
            "{}",
            style(format!(
                "  → Found {} files with code coverage data",
                report.report_files.len()
            ))
            .dim()
        );
        eprintln_unless!(self.quiet, "");

        if self.print {
            self.show_report(&report)?;
        }

        if self.dry_run {
            eprintln_unless!(self.quiet, "  Exporting code coverage data...");
            let export = report.export_to(self.output_dir.clone())?;
            eprintln_unless!(
                self.quiet,
                "{}",
                style(format!("  → Exported to {:?}", export.to.as_ref().unwrap())).dim()
            );
            return CommandSuccess::ok();
        }

        eprintln_unless!(self.quiet, "  Authenticating with Qlty...");

        match Upload::prepare(&token, &mut report) {
            Ok(upload) => {
                eprintln_unless!(self.quiet, "  Exporting code coverage data...");
                let export = report.export_to(self.output_dir.clone())?;

                eprintln_unless!(
                    self.quiet,
                    "{}",
                    style(format!("  → Exported to {:?}", export.to.as_ref().unwrap())).dim()
                );
                eprintln_unless!(self.quiet, "");

                eprintln_unless!(
                    self.quiet,
                    "{}",
                    style(format!("  → Using coverage token {:?}", token)).dim()
                );
                eprintln_unless!(self.quiet, "");

                eprintln_unless!(self.quiet, "  Uploading coverage data...");

                let timer = Instant::now();
                upload.upload(&export)?;

                let bytes = export.total_size_bytes()?;
                eprintln_unless!(
                    self.quiet,
                    "{}",
                    style(format!(
                        "  → Uploaded {} in {:.2}s!",
                        HumanBytes(bytes),
                        timer.elapsed().as_secs_f32()
                    ))
                    .dim()
                );

                eprintln_unless!(self.quiet, "");
                eprintln_unless!(self.quiet, "View upload at https://qlty.sh");
            }
            Err(err) => {
                eprintln!("{}", style(format!("  → {}", err)).red());
                std::process::exit(1);
            }
        }

        CommandSuccess::ok()
    }

    fn print_initial_messages(&self) {
        eprintln_unless!(self.quiet, "qlty {}", LONG_VERSION.as_str());
        eprintln_unless!(self.quiet, "{}", style("https://qlty.sh/d/coverage").dim());
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

    /// Appends repository name to token if it is a workspace token
    fn expand_token(&self, token: String) -> Result<String> {
        if token.starts_with(COVERAGE_TOKEN_WORKSPACE_PREFIX) {
            if token.contains("/") {
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
                        bail!("Could not infer project name from environment, please provide it using --project")
                    }
                }
            };
            Ok(format!("{}/{}", token, project))
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
            .last()
            .map(|s| s.strip_suffix(".git").unwrap_or(s).to_string())
            .take_if(|v| !v.is_empty())
    }

    fn show_report(&self, report: &Report) -> Result<()> {
        if self.json {
            print_report_as_json(report)
        } else {
            print_report_as_text(report)
        }
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

    fn publish(project: Option<&str>) -> Publish {
        Publish {
            dry_run: true,
            report_format: None,
            output_dir: None,
            tag: None,
            override_build_id: None,
            override_branch: None,
            override_commit_sha: None,
            override_pr_number: None,
            transform_add_prefix: None,
            transform_strip_prefix: None,
            token: None,
            project: project.map(|s| s.to_string()),
            print: false,
            json: false,
            quiet: true,
            paths: vec![],
        }
    }

    #[test]
    fn test_expand_token_project() -> Result<()> {
        let token = publish(None).expand_token("qltcp_123".to_string())?;
        assert_eq!(token, "qltcp_123");
        Ok(())
    }

    #[test]
    fn test_expand_token_workspace_with_project() -> Result<()> {
        let token = publish(Some("test")).expand_token("qltcw_123".to_string())?;
        assert_eq!(token, "qltcw_123/test");
        Ok(())
    }

    #[test]
    fn test_expand_token_workspace_with_env() -> Result<()> {
        let token = publish(None).expand_token("qltcw_123".to_string())?;
        assert_eq!(token, "qltcw_123/qlty");

        std::env::set_var("GITHUB_REPOSITORY", "");
        let token = publish(None).expand_token("qltcw_123".to_string())?;
        assert_eq!(token, "qltcw_123/qlty");

        std::env::set_var("GITHUB_REPOSITORY", "a/b.git");
        let token = publish(None).expand_token("qltcw_123".to_string())?;
        assert_eq!(token, "qltcw_123/b");

        std::env::set_var("GITHUB_REPOSITORY", "b/c");
        let token = publish(None).expand_token("qltcw_123".to_string())?;
        assert_eq!(token, "qltcw_123/c");

        Ok(())
    }

    #[test]
    fn test_expand_token_already_expanded() -> Result<()> {
        let token = publish(Some("test")).expand_token("qltcw_123/abc".to_string())?;
        assert_eq!(token, "qltcw_123/abc");
        Ok(())
    }

    #[test]
    fn test_extract_repository_name() {
        assert_eq!(Publish::extract_repository_name(""), None);
        assert_eq!(Publish::extract_repository_name("a/"), None);
        assert_eq!(
            Publish::extract_repository_name("git@example.org:a/b"),
            Some("b".into())
        );
        assert_eq!(
            Publish::extract_repository_name("ssh://x@example.org:a/b"),
            Some("b".into())
        );
        assert_eq!(
            Publish::extract_repository_name("https://x:y@example.org/a/b"),
            Some("b".into())
        );
    }
}
