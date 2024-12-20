use crate::Parser;
use anyhow::{Context, Result};
use qlty_types::tests::v1::FileCoverage;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
#[serde(rename = "coverage")]
struct CoberturaSource {
    packages: Packages,
}

#[derive(Debug, Deserialize)]
struct Packages {
    package: Vec<Package>,
}

#[derive(Debug, Deserialize)]
struct Package {
    classes: Classes,
}

#[derive(Debug, Deserialize)]
struct Classes {
    class: Vec<Class>,
}

#[derive(Debug, Deserialize, Clone)]
struct Class {
    filename: String,
    lines: Lines,
}

#[derive(Debug, Deserialize, Clone)]
struct Lines {
    #[serde(default)]
    line: Option<Vec<Line>>,
}

#[derive(Debug, Deserialize, Clone)]
struct Line {
    number: String,
    hits: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cobertura {}

impl Cobertura {
    pub fn new() -> Self {
        Self {}
    }
}

impl Parser for Cobertura {
    fn parse_text(&self, text: &str) -> Result<Vec<FileCoverage>> {
        let source: CoberturaSource =
            serde_xml_rs::from_str(text).with_context(|| "Failed to parse XML text")?;

        // BTreeMap allows us to index by filename while sorting at the same time
        let mut lines_by_filename: BTreeMap<String, Vec<Line>> = BTreeMap::new();
        let mut file_coverages = vec![];

        for package in source.packages.package.iter() {
            for class in package.classes.class.iter() {
                if let Some(ref lines) = class.lines.line {
                    for line in lines {
                        lines_by_filename
                            .entry(class.filename.clone())
                            .or_default()
                            .push(line.clone());
                    }
                } else {
                    lines_by_filename.entry(class.filename.clone()).or_default();
                }
            }
        }

        for (filename, lines) in lines_by_filename {
            let mut line_hits = Vec::new();
            let mut sorted_lines = lines.clone();
            sorted_lines.sort_by_key(|line| line.number.parse::<i32>().unwrap_or_default());

            if let Some(last_line) = sorted_lines.last() {
                line_hits = vec![-1; last_line.number.parse::<usize>().unwrap_or(0)];

                for line in sorted_lines.iter() {
                    let line_number = line.number.parse::<usize>().ok().unwrap_or(0);

                    if line_number > 0 {
                        let hits = line.hits.parse::<i64>().ok().unwrap_or(-1);
                        if line_hits[line_number - 1] == -1 {
                            line_hits[line_number - 1] = hits;
                        } else {
                            line_hits[line_number - 1] += hits;
                        }
                    }
                }
            }

            let file_coverage = FileCoverage {
                path: filename,
                hits: line_hits,
                ..Default::default()
            };

            file_coverages.push(file_coverage);
        }

        Ok(file_coverages)
    }
}
