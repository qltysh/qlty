use crate::publish::Report;
use anyhow::Result;
use cli_table::{
    format::{Border, HorizontalLine, Justify, Separator, VerticalLine},
    print_stdout, Cell, Style, Table,
};
use qlty_types::tests::v1::{CoverageSummary, FileCoverage};
use serde_json::{self};

pub fn print_report_as_text(report: &Report) -> Result<()> {
    let mut total = CoverageSummary::default();

    let mut rows: Vec<_> = report
        .file_coverages
        .clone()
        .into_iter()
        .map(|file_coverage| {
            total += *file_coverage.summary.as_ref().unwrap();

            vec![
                file_coverage.path.cell(),
                file_coverage
                    .summary
                    .as_ref()
                    .unwrap()
                    .covered
                    .to_string()
                    .cell()
                    .justify(Justify::Right),
                file_coverage
                    .summary
                    .as_ref()
                    .unwrap()
                    .missed
                    .to_string()
                    .cell()
                    .justify(Justify::Right),
                (file_coverage.summary.as_ref().unwrap().percent() as u32)
                    .cell()
                    .justify(Justify::Right),
            ]
        })
        .collect();

    rows.push(vec![
        "TOTAL".cell().bold(true),
        total
            .covered
            .to_string()
            .cell()
            .bold(true)
            .justify(Justify::Right),
        total
            .missed
            .to_string()
            .cell()
            .bold(true)
            .justify(Justify::Right),
        (total.percent() as u32)
            .cell()
            .bold(true)
            .justify(Justify::Right),
    ]);

    let table = rows
        .table()
        .title(vec![
            "name".cell(),
            "covered".cell().justify(Justify::Right),
            "missed".cell().justify(Justify::Right),
            "%".cell().justify(Justify::Right),
        ])
        .border(Border::builder().build())
        .separator(
            Separator::builder()
                .title(Some(HorizontalLine::default()))
                .column(Some(VerticalLine::default()))
                .build(),
        );

    print_stdout(table)?;
    Ok(())
}

pub fn print_file_coverages_as_text(file_coverages: &[FileCoverage]) -> Result<()> {
    let rows: Vec<_> = file_coverages
        .iter()
        .map(|file_coverage| vec![file_coverage.clone().path.cell()])
        .collect();

    let table = rows
        .table()
        .title(vec!["name".cell()])
        .border(Border::builder().build())
        .separator(
            Separator::builder()
                .title(Some(HorizontalLine::default()))
                .column(Some(VerticalLine::default()))
                .build(),
        );

    print_stdout(table)?;
    Ok(())
}

pub fn print_report_as_json(report: &Report) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(report)?);
    Ok(())
}

pub fn print_file_coverages_as_json(file_coverages: &Vec<FileCoverage>) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(file_coverages)?);
    Ok(())
}
