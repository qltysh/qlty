use std::io::Write as _;

use anyhow::Result;
use console::style;
use num_format::{Locale, ToFormattedString as _};
use qlty_analysis::utils::fs::path_to_string;
use qlty_check::Report;
use tabwriter::TabWriter;

use super::{level::formatted_level, source::formatted_source};

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
                    "{:>7}\t{}\t{}\t{}\n",
                    style(format!(
                        "{}:{}",
                        issue.range().unwrap_or_default().start_line,
                        issue.range().unwrap_or_default().end_line,
                    ))
                    .dim(),
                    formatted_level(issue.level()),
                    issue.message.replace('\n', " ").trim(),
                    formatted_source(issue)
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
