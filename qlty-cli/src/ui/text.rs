use super::invocations::print_invocations;
use super::issues::print_issues;
use super::unformatted::print_unformatted;
use super::{fixes::print_fixes, ApplyMode};
use anyhow::Result;
use console::style;
use num_format::{Locale, ToFormattedString as _};
use qlty_check::Report;
use qlty_config::Workspace;
use qlty_types::analysis::v1::ExecutionVerb;

#[derive(Debug)]
pub struct TextFormatter {
    report: Report,
    workspace: Workspace,
    verbose: usize,
    summary: bool,
    apply_mode: ApplyMode,
}

impl TextFormatter {
    pub fn new(
        report: &Report,
        workspace: &Workspace,
        verbose: usize,
        summary: bool,
        apply_mode: ApplyMode,
    ) -> Self {
        Self {
            report: report.clone(),
            workspace: workspace.clone(),
            verbose,
            summary,
            apply_mode,
        }
    }
}

impl TextFormatter {
    pub fn write_to(&mut self, writer: &mut dyn std::io::Write) -> anyhow::Result<bool> {
        if !self.summary {
            if print_unformatted(writer, &self.report.issues)? {
                return Ok(true);
            }

            if print_fixes(
                writer,
                &self.report.issues,
                &self.workspace.root,
                self.apply_mode,
            )? {
                return Ok(true);
            }

            print_issues(writer, &self.report)?;
        }

        print_invocations(writer, &self.report, self.verbose)?;
        self.print_conclusion(writer)?;

        Ok(false)
    }
}

impl TextFormatter {
    pub fn print_conclusion(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        if self.verbose >= 1 && self.report.targets_count() > 0 {
            self.print_processed_files(writer)?;
        } else if self.report.targets_count() == 0 && self.report.target_mode.is_diff() {
            self.print_no_modified_files(writer)?;
        }

        Ok(())
    }

    pub fn print_processed_files(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        writeln!(
            writer,
            "{} {} {}{}",
            match self.report.verb {
                ExecutionVerb::Check => "Checked",
                ExecutionVerb::Fmt => "Formatted",
                _ => "Processed",
            },
            self.report.targets_count().to_formatted_string(&Locale::en),
            if self.report.target_mode.is_diff() {
                "modified "
            } else {
                ""
            },
            if self.report.targets_count() == 1 {
                "file"
            } else {
                "files"
            },
        )?;

        Ok(())
    }

    pub fn print_no_modified_files(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        writeln!(
            writer,
            "{}",
            style(format!(
                "No modified files for {} were found on your branch.",
                if self.report.verb == ExecutionVerb::Fmt {
                    "formatting"
                } else {
                    "checks"
                }
            ))
            .dim()
        )?;

        Ok(())
    }
}
