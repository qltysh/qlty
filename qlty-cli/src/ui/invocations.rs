use std::io::Write as _;

use anyhow::Result;
use console::style;
use num_format::{Locale, ToFormattedString as _};
use qlty_analysis::utils::fs::path_to_string;
use qlty_check::{executor::InvocationStatus, Report};
use tabwriter::TabWriter;

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

    let cwd = std::env::current_dir().expect("Unable to identify current directory");
    let mut tw = TabWriter::new(vec![]);

    // Print a JOBS summary in verbose mode
    if verbose >= 1 {
        for invocation in &report.invocations {
            let absolute_outfile_path = invocation.outfile_path();
            let outfile_path = pathdiff::diff_paths(absolute_outfile_path, &cwd).unwrap();

            match invocation.status() {
                InvocationStatus::Success => {
                    tw.write_all(
                        format!(
                            "{}\t{}\t{} {}\t{:.2}s\t{}\n",
                            invocation.invocation.plugin_name,
                            style("Success").green(),
                            invocation.invocation.targets_count,
                            if invocation.invocation.targets_count == 1 {
                                "target"
                            } else {
                                "targets"
                            },
                            invocation.invocation.duration_secs,
                            style(path_to_string(outfile_path)).dim().underlined(),
                        )
                        .as_bytes(),
                    )
                    .unwrap();
                }
                InvocationStatus::LintError => {
                    tw.write_all(
                        format!(
                            "{}\t{}\t{} {}\t{:.2}s\t{}\n",
                            invocation.invocation.plugin_name,
                            style("Error").red(),
                            invocation.invocation.targets_count,
                            if invocation.invocation.targets_count == 1 {
                                "target"
                            } else {
                                "targets"
                            },
                            invocation.invocation.duration_secs,
                            style(path_to_string(outfile_path)).dim().underlined(),
                        )
                        .as_bytes(),
                    )
                    .unwrap();
                }
                InvocationStatus::ParseError => {
                    tw.write_all(
                        format!(
                            "{}\t{}\t{} {}\t{:.2}s\t{}\n",
                            invocation.invocation.plugin_name,
                            style("Parse error").red(),
                            invocation.invocation.targets_count,
                            if invocation.invocation.targets_count == 1 {
                                "target"
                            } else {
                                "targets"
                            },
                            invocation.invocation.duration_secs,
                            style(path_to_string(outfile_path)).dim().underlined(),
                        )
                        .as_bytes(),
                    )
                    .unwrap();
                }
            }
        }

        tw.flush().unwrap();
        let written = String::from_utf8(tw.into_inner().unwrap()).unwrap();

        if !written.is_empty() {
            writeln!(writer, "{}", written)?;
        }
    }

    let mut tw = TabWriter::new(vec![]);
    let mut errors_count = 0;

    for invocation in &report.invocations {
        let absolute_outfile_path = invocation.outfile_path();
        let outfile_path = pathdiff::diff_paths(absolute_outfile_path, &cwd).unwrap();

        match invocation.status() {
            InvocationStatus::Success => {}
            InvocationStatus::LintError => {
                errors_count += 1;

                match invocation.invocation.exit_code {
                    Some(code) => {
                        tw.write_all(
                            format!(
                                "{}\t{}\t{}\t{}\n",
                                invocation.invocation.plugin_name,
                                style("Error").red(),
                                format!(
                                    "Exited with code {:?} in {:.2}s",
                                    code, invocation.invocation.duration_secs
                                ),
                                style(path_to_string(outfile_path)).dim().underlined(),
                            )
                            .as_bytes(),
                        )
                        .unwrap();

                        if invocation.invocation.stderr.is_empty() {
                            if !invocation.invocation.stdout.is_empty() {
                                let text: String =
                                    invocation.invocation.stdout.chars().take(2048).collect();

                                for line in text.lines() {
                                    tw.write_all(format!("\t{}", style(line).red()).as_bytes())?;
                                }
                            }
                        } else {
                            let text: String =
                                invocation.invocation.stderr.chars().take(2048).collect();

                            for line in text.lines() {
                                tw.write_all(format!("\t{}", style(line).red()).as_bytes())?;
                            }
                        }
                    }
                    None => {
                        tw.write_all(
                            format!(
                                "{}\t{}\tExited with unknown status in {:.2}s\t{}\n",
                                invocation.invocation.plugin_name,
                                style("Error").red(),
                                invocation.invocation.duration_secs,
                                style(path_to_string(outfile_path)).dim().underlined(),
                            )
                            .as_bytes(),
                        )
                        .unwrap();
                    }
                }
            }
            InvocationStatus::ParseError => {
                errors_count += 1;

                tw.write_all(
                    format!(
                        "{}\t{}\t{}\t{}\n",
                        invocation.invocation.plugin_name,
                        style("Parse error").red(),
                        invocation.invocation.parser_error.as_ref().unwrap(),
                        style(path_to_string(outfile_path)).dim().underlined(),
                    )
                    .as_bytes(),
                )
                .unwrap();
            }
        }
    }

    tw.flush().unwrap();
    let written = String::from_utf8(tw.into_inner().unwrap()).unwrap();

    if !written.is_empty() {
        writeln!(
            writer,
            "{}{}{}",
            style(" ERRORS: ").bold().reverse(),
            style(errors_count.to_formatted_string(&Locale::en))
                .bold()
                .reverse(),
            style(" ").bold().reverse()
        )?;
        writeln!(writer)?;
        writeln!(writer, "{}", written)?;
    }

    Ok(())
}
