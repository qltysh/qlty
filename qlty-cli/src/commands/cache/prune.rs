use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use bytesize::ByteSize;
use clap::Args;
use cli_table::{
    format::{Border, HorizontalLine, Justify, Separator, VerticalLine},
    print_stdout, Cell as _, Table as _,
};
use qlty_config::Workspace;

#[derive(Args, Debug)]
pub struct Prune {}

impl Prune {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let workspace = Workspace::new()?;
        let library = workspace.library()?;

        let before_statuses = library.status()?;
        library.prune()?;
        let after_statuses = library.status()?;

        let rows = after_statuses
            .iter()
            .filter_map(|after| {
                let before = before_statuses.iter().find(|s| s.dir == after.dir);

                match before {
                    Some(before) => {
                        let files_removed = before.files_count - after.files_count;
                        let bytes_removed = before.files_bytes - after.files_bytes;

                        Some(vec![
                            after
                                .dir
                                .to_string_lossy()
                                .to_string()
                                .cell()
                                .justify(Justify::Left),
                            format!("{}", files_removed).cell().justify(Justify::Right),
                            ByteSize(bytes_removed)
                                .to_string_as(true)
                                .cell()
                                .justify(Justify::Right),
                            format!("{}", after.files_count)
                                .cell()
                                .justify(Justify::Right),
                            ByteSize(after.files_bytes)
                                .to_string_as(true)
                                .cell()
                                .justify(Justify::Right),
                        ])
                    }
                    None => None,
                }
            })
            .collect::<Vec<_>>();

        let table = rows
            .table()
            .title(vec![
                "Path".cell(),
                "Files Removed".cell().justify(Justify::Right),
                "Bytes Removed".cell().justify(Justify::Right),
                "Files Remaining".cell().justify(Justify::Right),
                "Bytes Remaining".cell().justify(Justify::Right),
            ])
            .border(Border::builder().build())
            .separator(
                Separator::builder()
                    .title(Some(HorizontalLine::default()))
                    .column(Some(VerticalLine::default()))
                    .build(),
            );
        print_stdout(table)?;

        CommandSuccess::ok()
    }
}
