use anyhow::Result;
use clap::Args;
use console::style;
use indicatif::HumanBytes;
use qlty_config::{QltyConfig, Workspace};
use qlty_coverage::eprintln_unless;
use qlty_coverage::formats::Formats;
use qlty_coverage::print::{print_report_as_json, print_report_as_text};
use qlty_coverage::publish::{Planner, Processor, Reader, Report, Settings, Upload};
use std::path::PathBuf;
use std::time::Instant;

use crate::version::LONG_VERSION;
use crate::{CommandError, CommandSuccess};

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
    /// The prefix to remove from absolute paths in coverage payloads, to make them relative to the project root. This is usually the directory in which the tests were run. Defaults to current working directory.
    pub transform_strip_prefix: Option<String>,

    #[arg(long, short)]
    /// The token to use for authentication when uploading the report. By default, it retrieves the token from the QLTY_COVERAGE_TOKEN environment variable.
    pub token: Option<String>,

    #[arg(long)]
    /// Print coverage
    pub print: bool,

    #[arg(long, requires = "print")]
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
                report_format: self.report_format.clone(),
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
        match &self.token {
            Some(token) => Ok(token.to_owned()),
            None => std::env::var("QLTY_COVERAGE_TOKEN").map_err(|_| {
                return anyhow::Error::msg("QLTY_COVERAGE_TOKEN environment variable is required.");
            }),
        }
    }

    fn show_report(&self, report: &Report) -> Result<()> {
        if self.json {
            print_report_as_json(report)
        } else {
            print_report_as_text(report)
        }
    }

    fn load_config() -> QltyConfig {
        if let Ok(workspace) = Workspace::new() {
            if let Ok(cfg) = workspace.config() {
                cfg
            } else {
                QltyConfig::default()
            }
        } else {
            QltyConfig::default()
        }
    }
}
