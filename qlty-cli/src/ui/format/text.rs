use anyhow::{Context as _, Result};
use console::{style, Style};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use diffy::Patch;
use num_format::{Locale, ToFormattedString as _};
use qlty_analysis::utils::fs::path_to_string;
use qlty_check::Report;
use qlty_check::{executor::InvocationStatus, results::FixedResult};
use qlty_cloud::format::Formatter;
use qlty_config::Workspace;
use qlty_types::analysis::v1::{ExecutionVerb, Issue, Level, SuggestionSource};
use similar::{ChangeTag, TextDiff};
use std::collections::HashSet;
use std::fmt;
use std::io::{IsTerminal as _, Write};
use tabwriter::TabWriter;
use tracing::warn;

#[derive(Debug)]
pub struct TextFormatter {
    report: Report,
    workspace: Workspace,
    verbose: usize,
    summary: bool,
}

impl<'a> TextFormatter {
    // qlty-ignore: clippy:new_ret_no_self
    pub fn new(
        report: &Report,
        workspace: &Workspace,
        verbose: usize,
        summary: bool,
    ) -> Box<dyn Formatter> {
        Box::new(Self {
            report: report.clone(),
            workspace: workspace.clone(),
            verbose,
            summary,
        })
    }
}

struct Line(Option<usize>);

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

impl Formatter for TextFormatter {
    fn write_to(&self, writer: &mut dyn std::io::Write) -> anyhow::Result<()> {
        if !self.summary {
            self.print_unformatted(writer)?;
            self.print_fixes(writer)?;
            self.print_issues(writer)?;
        }

        self.print_invocations(writer)?;
        self.print_conclusion(writer)?;
        Ok(())
    }
}

struct PatchCandidate {
    issue: Issue,
    source: SuggestionSource,
    path: String,
    patch: String,
    original_code: String,
    modified_code: String,
}

enum AskMode {
    All,
    None,
    Ask,
}

impl TextFormatter {
    pub fn print_unformatted(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        let issues = self
            .report
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

    pub fn print_fixes(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        let mut patch_candidates = vec![];

        for issue in &self.report.issues {
            if let Some(location) = &issue.location {
                if let Some(suggestion) = issue.suggestions.first() {
                    if let Ok(patch) = Patch::from_str(&suggestion.patch) {
                        let full_path = self.workspace.root.join(location.path.clone());
                        let original_code =
                            std::fs::read_to_string(&full_path).with_context(|| {
                                format!("Failed to read file: {}", full_path.display())
                            })?;

                        if let Ok(modified_code) = diffy::apply(&original_code, &patch) {
                            patch_candidates.push(PatchCandidate {
                                issue: issue.clone(),
                                source: SuggestionSource::try_from(suggestion.source)
                                    .unwrap_or_default(),
                                path: location.path.clone(),
                                patch: suggestion.patch.clone(),
                                original_code,
                                modified_code,
                            });
                        } else {
                            warn!("Failed to apply patch: {}", suggestion.patch);
                        }
                    } else {
                        warn!("Failed to parse patch: {}", suggestion.patch);
                    }
                }
            }
        }

        if patch_candidates.is_empty() {
            return Ok(());
        }

        writeln!(writer)?;
        writeln!(
            writer,
            "{}{}{}",
            style(" AUTOFIXES: ").bold().reverse(),
            style(patch_candidates.len().to_formatted_string(&Locale::en))
                .bold()
                .reverse(),
            style(" ").bold().reverse()
        )?;
        writeln!(writer)?;

        let mut ask_mode = AskMode::Ask;

        for candidate in patch_candidates {
            let diff = TextDiff::from_lines(&candidate.original_code, &candidate.modified_code);
            let mut patch_writer = vec![];

            for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
                if idx > 0 {
                    writeln!(patch_writer, "{:-^1$}", "-", 80)?;
                }
                for op in group {
                    for change in diff.iter_inline_changes(op) {
                        let (sign, s) = match change.tag() {
                            ChangeTag::Delete => ("-", Style::new().red()),
                            ChangeTag::Insert => ("+", Style::new().green()),
                            ChangeTag::Equal => (" ", Style::new().dim()),
                        };
                        write!(
                            patch_writer,
                            "{}{} |{}",
                            style(Line(change.old_index())).dim(),
                            style(Line(change.new_index())).dim(),
                            s.apply_to(sign).bold(),
                        )?;
                        for (emphasized, value) in change.iter_strings_lossy() {
                            if emphasized {
                                write!(
                                    patch_writer,
                                    "{}",
                                    s.apply_to(value).underlined().on_black()
                                )?;
                            } else {
                                write!(patch_writer, "{}", s.apply_to(value))?;
                            }
                        }
                        if change.missing_newline() {
                            writeln!(patch_writer)?;
                        }
                    }
                }
            }

            // For a reason that I haven't figured out yet, sometimes we print
            // empty patches. This is a workaround to skip those issues.
            if !patch_writer.is_empty() {
                let start_line = candidate.issue.range().unwrap_or_default().start_line;

                writeln!(
                    writer,
                    "{}{}",
                    style(&candidate.path).underlined(),
                    style(format!(":{}", start_line)).dim()
                )?;

                writeln!(
                    writer,
                    "{} {}",
                    formatted_level(candidate.issue.level()),
                    style(candidate.issue.message.replace('\n', " ").trim())
                )?;

                write!(writer, "{}", String::from_utf8_lossy(&patch_writer))?;
                writeln!(
                    writer,
                    "{} {}",
                    formatted_source(&candidate.issue),
                    match candidate.source {
                        SuggestionSource::Llm => format!("[{}]", style("ai fix").cyan()),
                        _ => "".to_string(),
                    }
                )?;
                writeln!(writer)?;

                if std::io::stdin().is_terminal() {
                    match ask_mode {
                        AskMode::None => {} // Skip and don't ask
                        AskMode::All => {
                            apply_fix(writer, &candidate)?;
                        }
                        AskMode::Ask => {
                            let mut answered = false;

                            // Loop until we get a valid answer
                            while !answered {
                                if let Ok(answer) = prompt_apply_this_fix() {
                                    match answer.as_str() {
                                        "Y" | "y" | "yes" => {
                                            answered = true;
                                            apply_fix(writer, &candidate)?;
                                        }
                                        "A" | "a" | "all" => {
                                            answered = true;
                                            ask_mode = AskMode::All;
                                            apply_fix(writer, &candidate)?;
                                        }
                                        "N" | "n" | "no" => {
                                            answered = true;
                                        }
                                        "none" => {
                                            answered = true;
                                            ask_mode = AskMode::None;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }

                writeln!(writer)?;
            }
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

fn prompt_apply_this_fix() -> Result<String> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Apply this fix? [Yes/no/all/none]")
        .default("Y".to_string())
        .show_default(false)
        .allow_empty(true)
        .interact_text()?)
}

fn apply_fix(writer: &mut dyn std::io::Write, candidate: &PatchCandidate) -> Result<()> {
    if let Ok(patch) = Patch::from_str(&candidate.patch) {
        if let Ok(modified_code) = diffy::apply(&candidate.original_code, &patch) {
            std::fs::write(&candidate.path, &modified_code)
                .with_context(|| format!("Failed to apply path to file: {}", candidate.path))?;

            eprintln!(
                "{} {}",
                style("✔ Fixed:").green().bold(),
                style(&candidate.path).underlined()
            );
        } else {
            warn!("Failed to apply patch: {}", candidate.patch);
            writeln!(
                writer,
                "{} {}",
                style("Failed to apply patch:").red(),
                style(&candidate.path).underlined()
            )?;
        }
    } else {
        warn!("Failed to parse patch: {}", candidate.patch);
        writeln!(
            writer,
            "{} {}",
            style("Failed to parse patch:").red(),
            style(&candidate.path).underlined()
        )?;
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
