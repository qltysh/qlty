use crate::{version::LONG_VERSION, CommandError, CommandSuccess};
use anyhow::Result;
use clap::Args;
use console::style;
use qlty_cloud::format::JsonEachRowFormatter;
use qlty_coverage::{
    eprintln_unless,
    formats::Formats,
    print::{print_file_coverages_as_json, print_file_coverages_as_text},
    transform::{Planner, Processor, Settings},
};
use qlty_types::tests::v1::FileCoverage;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct Transform {
    #[clap(long)]
    // Dry run mode, does not output the transformed report.
    pub dry_run: bool,

    #[arg(long, value_enum)]
    /// The format of the coverage report to transform. If not specified, the format will be inferred from the file extension or contents.
    pub report_format: Option<Formats>,

    #[arg(long)]
    /// The prefix to add to file paths in coverage payloads. This helps to match the file paths with the project's directory structure.
    pub add_prefix: Option<String>,

    #[arg(long)]
    /// The prefix to remove from absolute paths in coverage payloads. This makes paths relative to the project root, typically the directory where tests were run. Defaults to the current working directory.
    pub strip_prefix: Option<String>,

    #[arg(long)]
    /// The output file name for the transformed coverage report. If not specified, the report will be saved to 'coverage.jsonl'.
    pub output: Option<String>,

    #[arg(long)]
    /// Print coverage
    pub print: bool,

    #[arg(long, requires = "print")]
    /// JSON output
    pub json: bool,

    #[clap(long, short)]
    /// Suppresses most of the standard output messages.
    pub quiet: bool,

    /// The path to the coverage report, which may include its format (e.g., 'lcov:./coverage/lcov.info').
    pub path: String,
}

impl Transform {
    pub fn execute(&self, _args: &crate::Arguments) -> Result<CommandSuccess, CommandError> {
        self.print_initial_messages();

        if !self.quiet {
            eprintln!("Transforming coverage report {}", self.path);
        }

        let settings = Settings {
            report_format: self.report_format.clone(),
            add_prefix: self.add_prefix.clone(),
            strip_prefix: self.strip_prefix.clone(),
            path: self.path.clone(),
        };

        let planner = Planner::new(&settings);
        let plan = planner.compute()?;

        let file_coverages = Processor::new(&plan).compute()?;

        if self.print {
            self.show_file_coverages(&file_coverages)?;
        }

        if !self.dry_run {
            self.export(&file_coverages)?;

            eprintln_unless!(
                self.quiet,
                "Exported qlty coverage report to {}",
                self.output()
            );
        }

        CommandSuccess::ok()
    }

    fn print_initial_messages(&self) {
        eprintln_unless!(self.quiet, "qlty {}", LONG_VERSION.as_str());
        eprintln_unless!(self.quiet, "{}", style("https://qlty.sh/d/coverage").dim());
        eprintln_unless!(self.quiet, "");
    }

    fn output(&self) -> String {
        self.output
            .clone()
            .unwrap_or_else(|| "coverage.jsonl".to_string())
    }

    fn export(&self, file_coverages: &Vec<FileCoverage>) -> Result<()> {
        let output_path = PathBuf::from(self.output());
        JsonEachRowFormatter::new(file_coverages.clone()).write_to_file(&output_path)?;
        Ok(())
    }

    fn show_file_coverages(&self, file_coverages: &Vec<FileCoverage>) -> Result<()> {
        if self.print {
            if self.json {
                print_file_coverages_as_json(file_coverages)
            } else {
                print_file_coverages_as_text(file_coverages)
            }
        } else {
            Ok(())
        }
    }
}
