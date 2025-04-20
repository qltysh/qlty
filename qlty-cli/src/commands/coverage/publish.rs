use crate::{CommandError, CommandSuccess};
use anyhow::{bail, Result};
use clap::Args;
use console::style;
use git2::Repository;
use indicatif::HumanBytes;
use num_format::{Locale, ToFormattedString as _};
use qlty_config::version::LONG_VERSION;
use qlty_config::{QltyConfig, Workspace};
use qlty_coverage::ci::{GitHub, CI};
use qlty_coverage::eprintln_unless;
use qlty_coverage::formats::Formats;
use qlty_coverage::print::{print_report_as_json, print_report_as_text};
use qlty_coverage::publish::{Plan, Planner, Processor, Reader, Report, Settings, Upload};
use regex::Regex;
use std::io::Write as _;
use std::path::PathBuf;
use std::time::Instant;
use tabwriter::TabWriter;
use tracing::debug;

const COVERAGE_TOKEN_WORKSPACE_PREFIX: &str = "qltcw_";
const COVERAGE_TOKEN_PROJECT_PREFIX: &str = "qltcp_";
const OIDC_REGEX: &str = r"^([a-zA-Z0-9\-_]+)\.([a-zA-Z0-9\-_]+)\.([a-zA-Z0-9\-_]+)$";

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

    #[arg(long)]
    /// Verbose
    pub verbose: bool,

    #[arg(long, hide = true, requires = "print")]
    /// JSON output
    pub json: bool,

    #[clap(long, short)]
    pub quiet: bool,

    #[arg(long, hide = true)]
    pub skip_missing_files: bool,

    #[arg(long)]
    /// The total number of parts that qlty cloud should expect. Each call to qlty publish will upload one part.
    /// (The total parts count is per coverage tag e.g. if you have 2 tags each with 3 parts, you should set this to 3)
    pub total_parts_count: Option<u32>,

    // Paths to coverage reports
    pub paths: Vec<String>,
}

impl Publish {
    // TODO: Use CommandSuccess and CommandError, which is not straight forward since those types aren't available here.
    pub fn execute(&self, _args: &crate::Arguments) -> Result<CommandSuccess, CommandError> {
        self.print_initial_messages();
        self.print_settings();

        self.validate_options()?;

        let token = self.load_auth_token()?;

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
                skip_missing_files: self.skip_missing_files,
                total_parts_count: self.total_parts_count,
            },
        )
        .compute()?;

        self.validate_plan(&plan)?;

        self.print_metadata(&plan);
        self.print_coverage_files(&plan);

        let results = Reader::new(&plan).read()?;
        let mut report = Processor::new(&plan, results).compute()?;

        self.print_coverage_data(&report);

        if self.print {
            self.show_report(&report)?;
        }

        let export = report.export_to(self.output_dir.clone())?;
        self.print_export_status(&export.to);

        if self.dry_run {
            return CommandSuccess::ok();
        }

        self.print_authentication_info(&token);

        let upload = Upload::prepare(&token, &mut report)?;
        let timer = Instant::now();
        upload.upload(&export)?;
        let bytes = export.total_size_bytes()?;
        self.print_upload_complete(bytes, timer.elapsed().as_secs_f32(), &upload.url);

        CommandSuccess::ok()
    }

    fn validate_plan(&self, plan: &Plan) -> Result<()> {
        if plan.metadata.commit_sha.is_empty() {
            bail!(
                "Unable to determine commit SHA from the environment.\nPlease provide it using --override-commit-sha"
            )
        }

        if plan.report_files.is_empty() {
            bail!("No coverage reports data files were provided.")
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

    fn print_settings(&self) {
        self.print_section_header(" SETTINGS ");

        eprintln_unless!(
            self.quiet,
            "    cwd: {}",
            std::env::current_dir()
                .unwrap_or(PathBuf::from("ERROR"))
                .to_string_lossy()
        );

        if self.dry_run {
            eprintln_unless!(self.quiet, "    dry-run: {}", self.dry_run);
        }
        if let Some(report_format) = &self.report_format {
            eprintln_unless!(self.quiet, "    report-format: {}", report_format);
        }
        if let Some(output_dir) = &self.output_dir {
            eprintln_unless!(
                self.quiet,
                "    output-dir: {}",
                output_dir.to_string_lossy()
            );
        }
        if let Some(tag) = &self.tag {
            eprintln_unless!(self.quiet, "    tag: {}", tag);
        }
        if let Some(override_build_id) = &self.override_build_id {
            eprintln_unless!(self.quiet, "    override-build-id: {}", override_build_id);
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
        if let Some(transform_add_prefix) = &self.transform_add_prefix {
            eprintln_unless!(
                self.quiet,
                "    transform-add-prefix: {}",
                transform_add_prefix
            );
        }
        if let Some(transform_strip_prefix) = &self.transform_strip_prefix {
            eprintln_unless!(
                self.quiet,
                "    transform-strip-prefix: {}",
                transform_strip_prefix
            );
        }
        if let Some(project) = &self.project {
            eprintln_unless!(self.quiet, "    project: {}", project);
        }

        if self.skip_missing_files {
            eprintln_unless!(
                self.quiet,
                "    skip-missing-files: {}",
                self.skip_missing_files
            );
        }

        if let Some(total_parts_count) = self.total_parts_count {
            eprintln_unless!(self.quiet, "    total-parts-count: {}", total_parts_count);
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

    fn print_coverage_files(&self, plan: &Plan) {
        eprintln_unless!(
            self.quiet,
            "{}{}{}",
            style(" COVERAGE FILES: ").bold().reverse(),
            style(plan.report_files.len().to_formatted_string(&Locale::en))
                .bold()
                .reverse(),
            style(" ").bold().reverse()
        );
        eprintln_unless!(self.quiet, "");

        let mut tw = TabWriter::new(vec![]);

        tw.write_all(
            format!(
                "    {}\t{}\t{}\n",
                style("Coverage File").bold().underlined(),
                style("Format").bold().underlined(),
                style("Size").bold().underlined(),
            )
            .as_bytes(),
        )
        .ok();

        for report_file in &plan.report_files {
            let mut display_path = report_file.path.clone();

            if let Ok(cwd) = std::env::current_dir() {
                if let Some(relative_path) = pathdiff::diff_paths(display_path.clone(), cwd.clone())
                {
                    if let Some(path) = relative_path.to_str() {
                        display_path = path.to_string();
                    }
                }
            }

            if let Ok(size_bytes) = std::fs::metadata(&report_file.path).map(|m| m.len()) {
                tw.write_all(
                    format!(
                        "    {}\t{}\t{}\n",
                        display_path,
                        report_file.format,
                        HumanBytes(size_bytes),
                    )
                    .as_bytes(),
                )
                .ok();
            } else {
                tw.write_all(
                    format!(
                        "    {}\t{}\t{}\n",
                        report_file.path, report_file.format, "Unknown",
                    )
                    .as_bytes(),
                )
                .ok();
            }
        }

        tw.flush().ok();
        let written =
            String::from_utf8(tw.into_inner().unwrap_or_default()).unwrap_or("ERROR".to_string());

        eprintln_unless!(self.quiet, "{}", written);
    }

    fn print_coverage_data(&self, report: &Report) {
        self.print_section_header(" COVERAGE DATA ");

        let total_files_count = report.found_files.len() + report.missing_files.len();

        eprintln_unless!(
            self.quiet,
            "    {} unique code file {}",
            total_files_count.to_formatted_string(&Locale::en),
            if total_files_count == 1 {
                "path"
            } else {
                "paths"
            }
        );

        let mut missing_files = report.missing_files.iter().collect::<Vec<_>>();
        missing_files.sort();

        if !missing_files.is_empty() {
            let missing_percent = (missing_files.len() as f32 / total_files_count as f32) * 100.0;

            eprintln_unless!(
                self.quiet,
                "    {}",
                style(format!(
                    "{} {} missing on disk ({:.1}%)",
                    missing_files.len().to_formatted_string(&Locale::en),
                    if missing_files.len() == 1 {
                        "path is"
                    } else {
                        "paths are"
                    },
                    missing_percent
                ))
                .bold()
            );

            let (paths_to_show, show_all) = if self.verbose {
                (missing_files.len(), true)
            } else {
                (std::cmp::min(20, missing_files.len()), false)
            };

            eprintln_unless!(
                self.quiet,
                "\n    {}\n",
                style("Missing code files:").bold().yellow()
            );

            for path in missing_files.iter().take(paths_to_show) {
                eprintln_unless!(self.quiet, "      {}", style(path.to_string()).yellow());
            }

            if !show_all && paths_to_show < missing_files.len() {
                let remaining = missing_files.len() - paths_to_show;
                eprintln_unless!(
                    self.quiet,
                    "      {} {}",
                    style(format!(
                        "... and {} more",
                        remaining.to_formatted_string(&Locale::en)
                    ))
                    .dim()
                    .yellow(),
                    style("(Use --verbose to see all)").dim()
                );
            }

            eprintln_unless!(self.quiet, "");

            if missing_percent > 10.0 {
                eprintln_unless!(
                    self.quiet,
                    "    {} {}",
                    style("TIP:").bold().yellow(),
                    style("Consider using add-prefix or strip-prefix to fix paths").bold()
                );
            } else {
                eprintln_unless!(
                    self.quiet,
                    "    {} Consider excluding these paths with your code coverage tool.",
                    style("TIP:").bold()
                )
            }

            eprintln_unless!(
                self.quiet,
                "    {}",
                style("https://qlty.sh/d/coverage-path-fixing").dim()
            );

            eprintln_unless!(self.quiet, "");
        } else {
            eprintln_unless!(
                self.quiet,
                "    {}",
                style("All code files in the coverage data were found on disk.").dim()
            );
        }

        eprintln_unless!(self.quiet, "");

        // Get formatted numbers first
        let covered_lines = report.totals.covered_lines.to_formatted_string(&Locale::en);
        let uncovered_lines = report
            .totals
            .uncovered_lines
            .to_formatted_string(&Locale::en);
        let omitted_lines = report.totals.omitted_lines.to_formatted_string(&Locale::en);

        // Find the longest number for consistent spacing
        let max_length = [&covered_lines, &uncovered_lines, &omitted_lines]
            .iter()
            .map(|s| s.len())
            .max()
            .unwrap_or(0);

        eprintln_unless!(
            self.quiet,
            "    Covered Lines:      {:>width$}",
            covered_lines,
            width = max_length
        );
        eprintln_unless!(
            self.quiet,
            "    Uncovered Lines:    {:>width$}",
            uncovered_lines,
            width = max_length
        );
        eprintln_unless!(
            self.quiet,
            "    Omitted Lines:      {:>width$}",
            omitted_lines,
            width = max_length
        );
        eprintln_unless!(self.quiet, "");
        eprintln_unless!(
            self.quiet,
            "    {}",
            style(format!(
                "Line Coverage:       {:.2}%",
                report.totals.coverage_percentage
            ))
            .bold()
        );
        eprintln_unless!(self.quiet, "");
    }

    fn print_export_status(&self, export_path: &Option<PathBuf>) {
        self.print_section_header(" EXPORTING... ");
        eprintln_unless!(
            self.quiet,
            "    Exported: {}/coverage.zip",
            export_path
                .as_ref()
                .unwrap_or(&PathBuf::from("ERROR"))
                .to_string_lossy()
        );
        eprintln_unless!(self.quiet, "");
    }

    fn print_authentication_info(&self, token: &str) {
        self.print_section_header(" AUTHENTICATING... ");
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
        eprintln_unless!(self.quiet, "    Method: {}", token_type);
        eprintln_unless!(self.quiet, "    Token: {}", token);
        eprintln_unless!(self.quiet, "");
    }

    fn print_upload_complete(&self, bytes: u64, elapsed_seconds: f32, url: &str) {
        eprintln_unless!(
            self.quiet,
            "    Uploaded {} in {:.2}s!",
            HumanBytes(bytes),
            elapsed_seconds
        );

        if !url.is_empty() {
            eprintln_unless!(
                self.quiet,
                "    {}",
                style(format!("View report: {}", url)).bold()
            );
        }

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

    fn validate_options(&self) -> Result<(), CommandError> {
        if let Some(total_parts) = self.total_parts_count {
            if total_parts == 0 {
                return Err(CommandError::InvalidOptions {
                    message: String::from("Total parts count must be greater than 0"),
                });
            }
        }
        Ok(())
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
                        bail!(
                            "Could not infer project name from environment, please provide it using --project"
                        )
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
            skip_missing_files: false,
            total_parts_count: None,
            verbose: false,
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
        assert!(token.starts_with("qltcw_123/"));

        std::env::set_var("GITHUB_REPOSITORY", "");
        let token = publish(None).expand_token("qltcw_123".to_string())?;
        assert!(token.starts_with("qltcw_123/"));

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
