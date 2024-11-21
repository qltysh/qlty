use console::style;
use qlty_analysis::Report;
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;

pub fn report_duplications(report: &Report, diff: bool) {
    println!();
    let cwd = std::env::current_dir().expect("Unable to identify current directory");
    let mut i = 0;

    for (_, issues) in report.duplication_issues_by_duplication() {
        let identical = issues[0].get_property_bool("identical");
        let node_kind = issues[0].get_property_string("node_kind");
        let mass = issues[0].get_property_number("mass");
        i += 1;

        let description = match identical {
            false => "Similar",
            true => "IDENTICAL",
        };

        let bonus = match identical {
            false => String::from(""),
            true => format!("*{}", issues.len()),
        };

        println!(
            "{}) {} code found in :{} (mass={}{})",
            i, description, node_kind, mass, bonus,
        );

        let mut examples = vec![];
        let mut letter = 'A';

        for issue in issues {
            if examples.len() < 2 {
                examples.push(issue.snippet.to_owned());
            }

            let path = PathBuf::from(issue.path().unwrap());

            let source_file_relative_path = match path.strip_prefix(&cwd) {
                Ok(relative_path) => relative_path.display().to_string(),
                Err(_) => path.display().to_string(),
            };

            let range = issue.range().unwrap();
            println!(
                "        {}: {}:{}-{}",
                letter, source_file_relative_path, range.start_line, range.end_line,
            );

            letter = std::char::from_u32(letter as u32 + 1).unwrap_or('A');
        }

        if diff {
            println!(
                "{}",
                style(
                    "   ==============================================================================="
                )
                .dim()
            );

            if identical {
                println!("{}", examples.first().unwrap());
            } else {
                let diff = TextDiff::from_lines(examples.first().unwrap(), examples.get(1).unwrap());

                for change in diff.iter_all_changes() {
                    match change.tag() {
                        ChangeTag::Delete => {
                            print!("   {}", style(format!("A: {}", change)).cyan())
                        }
                        ChangeTag::Insert => {
                            print!("   {}", style(format!("B: {}", change)).green())
                        }
                        ChangeTag::Equal => print!("      {}", change),
                    };
                }
            }

            println!(
                "{}",
                style(
                    "   ==============================================================================="
                )
                .dim()
            );
        }

        println!();
    }
}
