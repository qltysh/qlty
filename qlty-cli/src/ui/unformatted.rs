use std::collections::HashSet;

use anyhow::Result;
use console::style;
use num_format::{Locale, ToFormattedString as _};
use qlty_analysis::utils::fs::path_to_string;
use qlty_types::analysis::v1::{Issue, Level};

pub fn print_unformatted(writer: &mut dyn std::io::Write, issues: &[Issue]) -> Result<()> {
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
