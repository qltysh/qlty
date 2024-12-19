use anyhow::Result;
use console::style;
use num_format::{Locale, ToFormattedString as _};
use qlty_analysis::utils::fs::path_to_string;
use qlty_check::Report;
use qlty_check::{executor::InvocationStatus, results::FixedResult};
use qlty_cloud::format::Formatter;
use qlty_types::analysis::v1::{ExecutionVerb, Issue, Level};
use std::collections::HashSet;
use std::io::Write;
use tabwriter::TabWriter;

#[derive(Debug)]
pub struct TextFormatter {
    report: Report,
    verbose: usize,
}

impl<'a> TextFormatter {
    pub fn new(report: &Report, verbose: usize) -> Box<dyn Formatter> {
        Box::new(Self {
            report: report.clone(),
            verbose,
        })
    }
}

impl Formatter for TextFormatter {
    fn write_to(&self, writer: &mut dyn std::io::Write) -> anyhow::Result<()> {
        print_unformatted(writer, &self.report)?;
        print_issues(writer, &self.report)?;
        print_invocations(writer, &self.report, self.verbose)?;

        if self.verbose >= 1 && self.report.targets_count() > 0 {
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
        } else if self.report.targets_count() == 0 && self.report.target_mode.is_diff() {
            writeln!(
                writer,
                "{}",
                style("No modified files for linting were found on your branch.").dim()
            )?;
        }

        Ok(())
    }
}

pub fn print_unformatted(writer: &mut dyn std::io::Write, report: &Report) -> Result<()> {
    let issues = report
        .issues
        .iter()
        .filter(|issue| issue.level() == Level::Fmt)
        .collect::<Vec<_>>();

    let paths = issues
        .iter()
        .map(|issue| issue.path().clone())
        .collect::<HashSet<_>>();

    let mut paths: Vec<_> = paths.iter().collect();
    paths.sort();

    if !paths.is_empty() {
        writeln!(writer)?;
        writeln!(
            writer,
            "{}{}{}",
            style(" UNFORMATTED FILES: ").bold().reverse(),
            style(paths.len().to_formatted_string(&Locale::en))
                .bold()
                .reverse(),
            style(" ").bold().reverse()
        )?;
        writeln!(writer)?;
    }

    for path in paths {
        writeln!(
            writer,
            "{} {}",
            style("✖").red().bold(),
            style(path_to_string(path.clone().unwrap_or_default())).underlined(),
        )?;
    }

    Ok(())
}

pub fn print_issues(writer: &mut dyn std::io::Write, report: &Report) -> Result<()> {
    let issues_by_path = report.issues_by_path();
    let mut paths: Vec<_> = issues_by_path.keys().collect();
    paths.sort();

    if !paths.is_empty() {
        writeln!(writer)?;
        writeln!(
            writer,
            "{}{}{}",
            style(" ISSUES: ").bold().reverse(),
            style(report.issues.len().to_formatted_string(&Locale::en))
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
                    formatted_fix_message(report, issue),
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

pub fn print_invocations(
    writer: &mut dyn std::io::Write,
    report: &Report,
    verbose: usize,
) -> Result<()> {
    for formatted_path in &report.formatted {
        writeln!(
            writer,
            "{} Formatted {}",
            style("✔").green().bold(),
            style(path_to_string(formatted_path)).underlined()
        )?;
    }

    if verbose >= 1 {
        writeln!(writer)?;
        writeln!(
            writer,
            "{}{}{}",
            style(" JOBS: ").bold().reverse(),
            style(report.invocations.len().to_formatted_string(&Locale::en))
                .bold()
                .reverse(),
            style(" ").bold().reverse()
        )?;
        writeln!(writer)?;
    }

    let mut printed_summary = false;
    let cwd = std::env::current_dir().expect("Unable to identify current directory");

    for invocation in &report.invocations {
        let absolute_outfile_path = invocation.outfile_path();
        let outfile_path = pathdiff::diff_paths(absolute_outfile_path, &cwd).unwrap();

        match invocation.status() {
            InvocationStatus::Success => {
                if verbose >= 1 {
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

fn formatted_level(level: Level) -> String {
    match level {
        Level::High => style("high  ").red().to_string(),
        Level::Medium => style("medium").magenta().to_string(),
        Level::Low => style("low   ").yellow().to_string(),
        Level::Fmt => style("fmt   ").dim().to_string(),
        _ => format!("{:?}", level),
    }
}

fn formatted_source(issue: &Issue) -> String {
    if !issue.rule_key.is_empty() {
        format!("{}", style(issue.rule_id()).dim())
    } else {
        format!("{}", style(issue.tool.clone()).dim())
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
