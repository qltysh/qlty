use super::{ApplyMode, TextFormatter};
use crate::Trigger;
use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Input};
use num_format::{Locale, ToFormattedString as _};
use qlty_analysis::utils::fs::path_to_string;
use qlty_check::{Executor, Planner, Processor, Settings};
use qlty_config::Workspace;
use qlty_types::analysis::v1::{ExecutionVerb, Issue, Level};
use std::{collections::HashSet, io::IsTerminal as _, path::PathBuf};

pub fn print_unformatted(
    writer: &mut dyn std::io::Write,
    issues: &[Issue],
    settings: &Settings,
    apply_mode: ApplyMode,
) -> Result<bool> {
    let issues = issues
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

    let mut printed_output = false;

    for path in paths.clone().into_iter().flatten() {
        writeln!(
            writer,
            "{} {}",
            style("✖").red().bold(),
            style(path_to_string(path)).underlined(),
        )?;

        printed_output = true;
    }

    if printed_output {
        writeln!(writer)?;
    }

    if std::io::stdin().is_terminal() && !paths.is_empty() {
        let mut settings = settings.clone();
        settings.trigger = Trigger::Manual.into();
        settings.paths = paths
            .clone()
            .into_iter()
            .map(|p| PathBuf::from(p.clone().unwrap()))
            .collect();

        match apply_mode {
            ApplyMode::None => {}
            ApplyMode::All => {
                return apply_fmt(writer, &settings);
            }
            ApplyMode::Ask => {
                let mut answered = false;

                // Loop until we get a valid answer
                while !answered {
                    if let Ok(answer) = prompt_fmt() {
                        match answer.to_lowercase().as_str() {
                            "y" | "yes" => return apply_fmt(writer, &settings),
                            "n" | "no" => answered = true,
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    Ok(false)
}

fn apply_fmt(writer: &mut dyn std::io::Write, settings: &Settings) -> Result<bool> {
    let workspace = Workspace::require_initialized()?;
    workspace.fetch_sources()?;

    let mut settings = settings.clone();

    // pass absolute paths to the formatter
    // so cwd does not affect targeting
    settings.paths = settings
        .paths
        .iter()
        .map(|p| {
            if p.is_absolute() {
                p.clone()
            } else {
                settings.root.join(p)
            }
        })
        .collect::<Vec<_>>();

    let plan = Planner::new(ExecutionVerb::Fmt, &settings)?.compute()?;
    let executor = Executor::new(&plan);
    let results = executor.install_and_invoke()?;

    let mut processor = Processor::new(&plan, results);
    let report = processor.compute()?;

    let mut formatter =
        TextFormatter::new(&report, &plan.workspace, &settings, false, ApplyMode::None);
    formatter.write_to(writer)?;

    Ok(true)
}

fn prompt_fmt() -> Result<String> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Format these files? [Yes/no]")
        .default("Y".to_string())
        .show_default(false)
        .allow_empty(true)
        .interact_text()?)
}
