use anyhow::Result;
use console::style;
use num_format::{Locale, ToFormattedString as _};
use qlty_analysis::utils::fs::path_to_string;
use qlty_check::Report;
use qlty_check::{executor::InvocationStatus, results::FixedResult};
use qlty_config::Workspace;
use qlty_types::analysis::v1::{ExecutionVerb, Issue};
use std::io::Write;
use tabwriter::TabWriter;

use super::fixes::print_fixes;
use super::level::formatted_level;
use super::source::formatted_source;
use super::unformatted::print_unformatted;

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
    pub fn write_to(&mut self, writer: &mut dyn std::io::Write) -> anyhow::Result<()> {
        if !self.summary {
            print_unformatted(writer, &self.report.issues)?;
            print_fixes(
                writer,
                &self.report.issues,
                &self.workspace.root,
                self.apply_mode,
            )?;
            self.print_issues(writer)?;
        }

        self.print_invocations(writer)?;
        self.print_conclusion(writer)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ApplyMode {
    All,
    None,
    Ask,
}

impl TextFormatter {
    pub fn print_issues(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        let issues_by_path = self.report.issues_by_path();
        let mut paths: Vec<_> = issues_by_path.keys().collect();
        paths.sort();

        if !paths.is_empty() {
            writeln!(writer)?;
            writeln!(
                writer,
                "{}{}{}",
                style(" ISSUES: ").bold().reverse(),
                style(self.report.issues.len().to_formatted_string(&Locale::en))
                    .bold()
                    .reverse(),
                style(" ").bold().reverse()
            )?;
            writeln!(writer)?;
        }

        for path in paths {
            let issues = issues_by_path.get(path).unwrap();

            let first_issue = issues.first().unwrap();
            let start_line = first_issue.range().unwrap_or_default().start_line;
            let end_line = first_issue.range().unwrap_or_default().end_line;

            writeln!(
                writer,
                "{}{}",
                style(path_to_string(path.clone().unwrap_or_default())).underlined(),
                style(format!(":{}:{}", start_line, end_line)).dim()
            )?;

            let mut tw = TabWriter::new(vec![]);

            for issue in issues {
                tw.write_all(
                    format!(
                        "{:>7}\t{}\t{}\t{}{}\n",
                        style(format!(
                            "{}:{}",
                            issue.range().unwrap_or_default().start_line,
                            issue.range().unwrap_or_default().end_line,
                        ))
                        .dim(),
                        formatted_level(issue.level()),
                        issue.message.replace('\n', " ").trim(),
                        formatted_source(issue),
                        formatted_fix_message(&self.report, issue),
                    )
                    .as_bytes(),
                )
                .unwrap();
            }

            tw.flush().unwrap();
            let written = String::from_utf8(tw.into_inner().unwrap()).unwrap();
            writeln!(writer, "{}", written)?;
        }

        Ok(())
    }

    pub fn print_invocations(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        for formatted_path in &self.report.formatted {
            writeln!(
                writer,
                "{} Formatted {}",
                style("✔").green().bold(),
                style(path_to_string(formatted_path)).underlined()
            )?;
        }

        if self.verbose >= 1 {
            writeln!(writer)?;
            writeln!(
                writer,
                "{}{}{}",
                style(" JOBS: ").bold().reverse(),
                style(
                    self.report
                        .invocations
                        .len()
                        .to_formatted_string(&Locale::en)
                )
                .bold()
                .reverse(),
                style(" ").bold().reverse()
            )?;
            writeln!(writer)?;
        }

        let mut printed_summary = false;
        let cwd = std::env::current_dir().expect("Unable to identify current directory");

        for invocation in &self.report.invocations {
            let absolute_outfile_path = invocation.outfile_path();
            let outfile_path = pathdiff::diff_paths(absolute_outfile_path, &cwd).unwrap();

            match invocation.status() {
                InvocationStatus::Success => {
                    if self.verbose >= 1 {
                        writeln!(
                            writer,
                            "{} {} checked {} files in {:.2}s {}",
                            style("Success").green(),
                            invocation.invocation.plugin_name,
                            invocation.plan.workspace_entries.len(),
                            invocation.invocation.duration_secs,
                            style(path_to_string(outfile_path)).dim(),
                        )?;

                        printed_summary = true;
                    }
                }
                InvocationStatus::LintError => match invocation.invocation.exit_code {
                    Some(code) => {
                        writeln!(
                            writer,
                            "{} {}: Exited with code {:?} {}",
                            style("Lint error").red(),
                            style(&invocation.invocation.plugin_name).red().bold(),
                            code,
                            style(path_to_string(outfile_path)).dim(),
                        )?;

                        if invocation.invocation.stderr.is_empty() {
                            if !invocation.invocation.stdout.is_empty() {
                                let text: String =
                                    invocation.invocation.stdout.chars().take(2048).collect();

                                for line in text.lines() {
                                    writeln!(writer, "        {}", style(line).red())?;
                                }
                            }
                        } else {
                            let text: String =
                                invocation.invocation.stderr.chars().take(2048).collect();

                            for line in text.lines() {
                                writeln!(writer, "        {}", style(line).red())?;
                            }
                        }

                        printed_summary = true;
                    }
                    None => {
                        writeln!(
                            writer,
                            "{} {}: Exited with unknown status {}",
                            style("Lint error").red(),
                            style(&invocation.invocation.plugin_name).red().bold(),
                            style(path_to_string(invocation.outfile_path())).dim(),
                        )?;
                        printed_summary = true;
                    }
                },
                InvocationStatus::ParseError => {
                    writeln!(
                        writer,
                        "{} {}: {} {}",
                        style("Parse error").red(),
                        invocation.invocation.plugin_name,
                        invocation.invocation.parser_error.as_ref().unwrap(),
                        style(path_to_string(outfile_path)).dim(),
                    )?;

                    printed_summary = true;
                }
            }
        }

        if printed_summary {
            writeln!(writer)?;
        }

        Ok(())
    }

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

fn formatted_fix_message(report: &Report, issue: &Issue) -> String {
    if issue.location().is_none() {
        return "".to_string();
    }

    let fixed_result = FixedResult {
        rule_key: issue.rule_key.clone(),
        location: issue.location().unwrap(),
    };
    if report.fixed.contains(&fixed_result) {
        format!(" [{}]", style("fixed").green())
    } else if report.fixable.contains(&fixed_result) {
        format!(" [{}]", style("fixable").yellow())
    } else {
        "".to_string()
    }
}
