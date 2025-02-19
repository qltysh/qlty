use super::{level::formatted_level, source::formatted_source};
use anyhow::{Context as _, Result};
use console::{style, Style};
use dialoguer::{theme::ColorfulTheme, Input};
use diffy::Patch;
use num_format::{Locale, ToFormattedString as _};
use qlty_types::analysis::v1::{Issue, SuggestionSource};
use similar::{ChangeTag, TextDiff};
use std::io::Write as _;
use std::{fmt, io::IsTerminal as _, path::Path};
use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ApplyMode {
    All,
    None,
    Ask,
}

struct PatchCandidate {
    issue: Issue,
    source: SuggestionSource,
    path: String,
    patch: String,
    original_code: String,
    modified_code: String,
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

pub fn print_fixes(
    writer: &mut dyn std::io::Write,
    issues: &[Issue],
    root: &Path,
    apply_mode: ApplyMode,
) -> Result<bool> {
    let mut dirty = false;
    let mut apply_mode = apply_mode;
    let mut patch_candidates = vec![];

    for issue in issues {
        if let Some(location) = &issue.location {
            if let Some(suggestion) = issue.suggestions.first() {
                if let Ok(patch) = Patch::from_str(&suggestion.patch) {
                    let full_path = root.join(location.path.clone());
                    let original_code = std::fs::read_to_string(&full_path)
                        .with_context(|| format!("Failed to read file: {}", full_path.display()))?;

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
        return Ok(dirty);
    }

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
                match apply_mode {
                    ApplyMode::None => {} // Skip and don't ask
                    ApplyMode::All => {
                        apply_fix(writer, &candidate)?;
                        dirty = true;
                    }
                    ApplyMode::Ask => {
                        let mut answered = false;

                        // Loop until we get a valid answer
                        while !answered {
                            if let Ok(answer) = prompt_apply_this_fix() {
                                match answer.as_str() {
                                    "Y" | "y" | "yes" => {
                                        answered = true;
                                        apply_fix(writer, &candidate)?;
                                        dirty = true;
                                    }
                                    "A" | "a" | "all" => {
                                        answered = true;
                                        apply_mode = ApplyMode::All;
                                        apply_fix(writer, &candidate)?;
                                        dirty = true;
                                    }
                                    "N" | "n" | "no" => {
                                        answered = true;
                                    }
                                    "none" => {
                                        answered = true;
                                        apply_mode = ApplyMode::None;
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

    Ok(dirty)
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
