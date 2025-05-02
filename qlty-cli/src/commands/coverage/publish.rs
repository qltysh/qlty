use super::utils::{
    load_config, print_authentication_info, print_initial_messages, print_metadata, print_settings,
};
use crate::{CommandError, CommandSuccess};
use anyhow::{bail, Result};
use clap::Args;
use console::style;
use indicatif::HumanBytes;
use num_format::{Locale, ToFormattedString as _};
use qlty_coverage::formats::Formats;
use qlty_coverage::print::{print_report_as_json, print_report_as_text};
use qlty_coverage::publish::{Plan, Planner, Processor, Reader, Report, Settings, Upload};
use qlty_coverage::token::load_auth_token;
use std::io::Write as _;
use std::path::PathBuf;
use std::time::Instant;
use tabwriter::TabWriter;

#[derive(Debug, Args, Default)]
pub struct Publish {
    #[clap(long)]
    /// Do not upload the coverage report, only export it to the output directory.
    pub dry_run: bool,

    #[arg(long, value_enum, hide = true)]
    /// [DEPRECATED, use --format] The format of the coverage report to transform.
    /// If not specified, the format will be inferred from the file extension or contents.
    pub report_format: Option<Formats>,

    #[arg(long, value_enum)]
    /// The format of the coverage report to transform.
    /// If not specified, the format will be inferred from the file extension or contents.
    pub format: Option<Formats>,

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

    #[arg(long, hide = true)]
    /// [DEPRECATED, use --add-prefix] The prefix to add to file paths in coverage payloads, to make them match the project's directory structure.
    pub transform_add_prefix: Option<String>,

    #[arg(long)]
    /// The prefix to add to file paths in coverage payloads, to make them match the project's directory structure.
    pub add_prefix: Option<String>,

    #[arg(long, hide = true)]
    /// [DEPRECATED, use --strip-prefix] The prefix to remove from absolute paths in coverage payloads to make them relative to the project root.
    /// This is usually the directory in which the tests were run. Defaults to the root of the git repository.
    pub transform_strip_prefix: Option<String>,

    #[arg(long)]
    /// The prefix to remove from absolute paths in coverage payloads to make them relative to the project root.
    /// This is usually the directory in which the tests were run. Defaults to the root of the git repository.
    pub strip_prefix: Option<String>,

    #[arg(long, short)]
    /// The token to use for authentication when uploading the report.
    /// By default, it retrieves the token from the QLTY_COVERAGE_TOKEN environment variable.
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

    #[arg(long)]
    /// Mark this upload as incomplete. This is useful when issuing multiple qlty coverage publish commands for the same coverage tag.
    /// The server will merge the uploads into a single report when qlty coverage complete is called.
    pub incomplete: bool,

    // Paths to coverage reports
    pub paths: Vec<String>,
}

impl Publish {
    // TODO: Use CommandSuccess and CommandError, which is not straight forward since those types aren't available here.
    pub fn execute(&self, _args: &crate::Arguments) -> Result<CommandSuccess, CommandError> {
        print_initial_messages(self.quiet);
        self.print_deprecation_warnings();

        let settings = self.build_settings();

        self.print_section_header(" SETTINGS ");
        print_settings(&settings);
        self.validate_options()?;

        let token = load_auth_token(&self.token, self.project.as_deref())?;
        let plan = Planner::new(&load_config(), &settings).compute()?;

        self.validate_plan(&plan)?;

        self.print_section_header(" METADATA ");
        print_metadata(&plan, self.quiet);
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

        self.print_section_header(" AUTHENTICATION ");
        print_authentication_info(&token, self.quiet);

        self.print_section_header(" PREPARING TO UPLOAD... ");
        let upload = Upload::prepare(&token, &mut report)?;

        self.print_section_header(" UPLOADING... ");
        let timer = Instant::now();
        upload.upload(&export)?;
        let bytes = export.total_size_bytes()?;
        self.print_upload_complete(bytes, timer.elapsed().as_secs_f32(), &upload.url);

        CommandSuccess::ok()
    }

    fn build_settings(&self) -> Settings {
        let format = Self::coalesce_args(&self.format, &self.report_format);
        let add_prefix = Self::coalesce_args(&self.add_prefix, &self.transform_add_prefix);
        let strip_prefix = Self::coalesce_args(&self.strip_prefix, &self.transform_strip_prefix);

        let incomplete: bool = self.incomplete || self.total_parts_count.unwrap_or(1) > 1;

        Settings {
            override_build_id: self.override_build_id.clone(),
            override_commit_sha: self.override_commit_sha.clone(),
            override_branch: self.override_branch.clone(),
            override_pull_request_number: self.override_pr_number.clone(),
            add_prefix,
            strip_prefix,
            tag: self.tag.clone(),
            report_format: format,
            paths: self.paths.clone(),
            skip_missing_files: self.skip_missing_files,
            total_parts_count: self.total_parts_count,
            incomplete,
            quiet: self.quiet,
            project: self.project.clone(),
            dry_run: self.dry_run,
            output_dir: self.output_dir.clone(),
        }
    }

    fn print_deprecation_warnings(&self) {
        if self.quiet {
            return;
        }

        if self.report_format.is_some() {
            eprintln!("WARNING: --report-format is deprecated, use --format instead\n");
        }
        if self.transform_add_prefix.is_some() {
            eprintln!("WARNING: --transform-add-prefix is deprecated, use --add-prefix instead\n");
        }
        if self.transform_strip_prefix.is_some() {
            eprintln!(
                "WARNING: --transform-strip-prefix is deprecated, use --strip-prefix instead\n"
            );
        }
    }

    fn coalesce_args<T: Clone>(primary: &Option<T>, fallback: &Option<T>) -> Option<T> {
        if let Some(val) = primary {
            Some(val.clone())
        } else {
            fallback.clone()
        }
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

    fn print_coverage_files(&self, plan: &Plan) {
        if self.quiet {
            return;
        }

        eprintln!(
            "{}{}{}",
            style(" COVERAGE FILES: ").bold().reverse(),
            style(plan.report_files.len().to_formatted_string(&Locale::en))
                .bold()
                .reverse(),
            style(" ").bold().reverse()
        );
        eprintln!();

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

        eprintln!("{written}");
    }

    fn print_section_header(&self, title: &str) {
        if self.quiet {
            return;
        }

        eprintln!("{}", style(title).bold().reverse());
        eprintln!();
    }

    fn print_coverage_data(&self, report: &Report) {
        if self.quiet {
            return;
        }

        self.print_section_header(" COVERAGE DATA ");

        let total_files_count = report.found_files.len() + report.missing_files.len();

        eprintln!(
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

            eprintln!(
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

            eprintln!("\n    {}\n", style("Missing code files:").bold().yellow());

            for path in missing_files.iter().take(paths_to_show) {
                eprintln!("      {}", style(path.to_string()).yellow());
            }

            if !show_all && paths_to_show < missing_files.len() {
                let remaining = missing_files.len() - paths_to_show;
                eprintln!(
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

            eprintln!();

            if missing_percent > 10.0 {
                eprintln!(
                    "    {} {}",
                    style("TIP:").bold().yellow(),
                    style("Consider using add-prefix or strip-prefix to fix paths").bold()
                );
            } else {
                eprintln!(
                    "    {} Consider excluding these paths with your code coverage tool.",
                    style("TIP:").bold()
                )
            }

            eprintln!(
                "    {}",
                style("https://qlty.sh/d/coverage-path-fixing").dim()
            );

            eprintln!();
        } else {
            eprintln!(
                "    {}",
                style("All code files in the coverage data were found on disk.").dim()
            );
        }

        eprintln!();

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

        eprintln!("    Covered Lines:      {covered_lines:>max_length$}");
        eprintln!("    Uncovered Lines:    {uncovered_lines:>max_length$}");
        eprintln!("    Omitted Lines:      {omitted_lines:>max_length$}");
        eprintln!();
        eprintln!(
            "    {}",
            style(format!(
                "Line Coverage:       {:.2}%",
                report.totals.coverage_percentage
            ))
            .bold()
        );
        eprintln!();
    }

    fn print_export_status(&self, export_path: &Option<PathBuf>) {
        if self.quiet {
            return;
        }

        self.print_section_header(" EXPORTING... ");
        eprintln!(
            "    Exported: {}/coverage.zip",
            export_path
                .as_ref()
                .unwrap_or(&PathBuf::from("ERROR"))
                .to_string_lossy()
        );
        eprintln!();
    }

    fn print_upload_complete(&self, bytes: u64, elapsed_seconds: f32, url: &str) {
        if self.quiet {
            return;
        }

        eprintln!(
            "    Uploaded {} in {:.2}s!",
            HumanBytes(bytes),
            elapsed_seconds
        );

        if !url.is_empty() {
            eprintln!("    {}", style(format!("View report: {url}")).bold());
        }

        eprintln!();
    }

    fn validate_options(&self) -> Result<(), CommandError> {
        if let Some(total_parts) = self.total_parts_count {
            if total_parts == 0 {
                return Err(CommandError::InvalidOptions {
                    message: String::from("Total parts count must be greater than 0"),
                });
            }

            if total_parts == 1 && self.incomplete {
                return Err(CommandError::InvalidOptions {
                    message: String::from("Cannot specify both --incomplete and --total-parts-count 1 as this is ambiguous. See https://qlty.sh/d/server-side-merging for more information."),
                });
            }
        }
        Ok(())
    }

    fn show_report(&self, report: &Report) -> Result<()> {
        if self.json {
            print_report_as_json(report)
        } else {
            print_report_as_text(report)
        }
    }
}
