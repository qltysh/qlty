use anyhow::Result;
use console::style;
use num_format::{Locale, ToFormattedString as _};
use qlty_analysis::utils::fs::path_to_string;
use qlty_check::{executor::InvocationStatus, Report};

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
