use anyhow::Result;
use clap::Args;
use qlty_analysis::code::File;
use qlty_analysis::workspace_entries::TargetMode;
use qlty_analysis::workspace_entries::WorkspaceEntryFinderBuilder;
use qlty_config::Workspace;
use std::io::Write;
use std::path::PathBuf;

use crate::{Arguments, CommandError, CommandSuccess};

#[derive(Args, Debug)]
pub struct Parse {
    /// Print location information
    #[arg(short, long)]
    pub locations: bool,

    /// The path to a file to parse
    pub file: PathBuf,
}

impl Parse {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let workspace = Workspace::new()?;
        workspace.fetch_sources()?;

        let config = workspace.config()?;

        let mut workspace_entry_finder_builder = WorkspaceEntryFinderBuilder {
            mode: TargetMode::Paths(1),
            paths: ([self.file.clone()]).to_vec(),
            config: config.clone(),
            ..Default::default()
        };

        let mut workspace_entry_finder = workspace_entry_finder_builder.build()?;
        let workspace_entries = workspace_entry_finder.workspace_entries("qlty")?;
        let source_file = File::from_workspace_entry(workspace_entries.first().unwrap())?;

        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();

        let tree = source_file.parse();
        let mut cursor = tree.walk();

        let mut needs_newline = false;
        let mut indent_level = 0;
        let mut did_visit_children = false;

        loop {
            let node = cursor.node();
            let is_named = node.is_named();
            if did_visit_children {
                if is_named {
                    stdout.write_all(b")").unwrap();
                    needs_newline = true;
                }
                if cursor.goto_next_sibling() {
                    did_visit_children = false;
                } else if cursor.goto_parent() {
                    did_visit_children = true;
                    indent_level -= 1;
                } else {
                    break;
                }
            } else {
                if is_named {
                    if needs_newline {
                        stdout.write_all(b"\n").unwrap();
                    }
                    for _ in 0..indent_level {
                        stdout.write_all(b"  ").unwrap();
                    }

                    if let Some(field_name) = cursor.field_name() {
                        write!(&mut stdout, "{}: ", field_name).unwrap();
                    }

                    if self.locations {
                        let start = node.start_position();
                        let end = node.end_position();

                        write!(
                            &mut stdout,
                            "({} [{}, {}] - [{}, {}]",
                            node.kind(),
                            start.row,
                            start.column,
                            end.row,
                            end.column
                        )
                        .unwrap();
                    } else {
                        write!(&mut stdout, "({}", node.kind(),).unwrap();
                    }

                    needs_newline = true;
                }
                if cursor.goto_first_child() {
                    did_visit_children = false;
                    indent_level += 1;
                } else {
                    did_visit_children = true;
                }
            }
        }
        cursor.reset(tree.root_node());
        println!();
        CommandSuccess::ok()
    }
}
