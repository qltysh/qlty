use super::invocations::print_invocations;
use super::issues::print_issues;
use super::messages::print_installation_error_messages;
use super::unformatted::print_unformatted;
use super::{fixes::print_fixes, ApplyMode};
use anyhow::Result;
use console::style;
use num_format::{Locale, ToFormattedString as _};
use qlty_check::{Report, Settings};
use qlty_config::Workspace;
use qlty_types::analysis::v1::ExecutionVerb;

#[derive(Debug)]
pub struct TextFormatter<'a> {
    report: Report,
    workspace: Workspace,
    settings: &'a Settings,
    summary: bool,
    apply_mode: ApplyMode,
}

impl<'a> TextFormatter<'a> {
    pub fn new(
        report: &Report,
        workspace: &Workspace,
        settings: &'a Settings,
        summary: bool,
        apply_mode: ApplyMode,
    ) -> Self {
        Self {
            report: report.clone(),
            workspace: workspace.clone(),
            settings,
            summary,
            apply_mode,
        }
    }

    pub fn write_to(&mut self, writer: &mut dyn std::io::Write) -> anyhow::Result<bool> {
        if !self.summary {
            if print_unformatted(writer, &self.report.issues, self.settings, self.apply_mode)? {
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

        print_invocations(writer, &self.report, self.settings.verbose)?;
        print_installation_error_messages(writer, &self.report)?;

        self.print_conclusion(writer)?;

        Ok(false)
    }

    pub fn print_conclusion(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        if self.settings.verbose >= 1 && self.report.targets_count() > 0 {
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
