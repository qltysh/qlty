use crate::Parser;
use anyhow::{Context, Result};
use qlty_types::tests::v1::FileCoverage;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename = "coverage")]
struct CloverSource {
    project: Project,
}

#[derive(Debug, Deserialize)]
struct Project {
    file: Option<Vec<File>>,
    package: Option<Vec<Package>>,
}

#[derive(Debug, Deserialize)]
struct Package {
    file: Option<Vec<File>>,
}

#[derive(Debug, Deserialize, Clone)]
struct File {
    name: String,
    path: Option<String>,
    line: Option<Vec<Line>>,
    metrics: Metrics,
}

#[derive(Debug, Deserialize, Clone)]
struct Metrics {
    loc: Option<i64>,
}

#[derive(Debug, Deserialize, Clone)]
struct Line {
    num: i64,
    count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Clover {}

impl Clover {
    pub fn new() -> Self {
        Self {}
    }
}

impl Parser for Clover {
    fn parse_text(&self, text: &str) -> Result<Vec<FileCoverage>> {
        let source: CloverSource =
            serde_xml_rs::from_str(text).with_context(|| "Failed to parse XML text")?;

        let mut files = source.project.file.unwrap_or_default().clone();

        let files_from_packages = source
            .project
            .package
            .unwrap_or_default()
            .iter()
            .flat_map(|package| package.file.clone().unwrap_or_default())
            .collect::<Vec<_>>();

        files.extend(files_from_packages);

        let mut file_coverages = vec![];

        for file in files {
            let mut line_hits = Vec::new();

            if let Some(ref lines) = file.line {
                for line in lines.iter() {
                    for _x in (line_hits.len() as i64)..(line.num - 1) {
                        line_hits.push(-1);
                    }

                    line_hits.push(line.count);
                }
            }

            if let Some(file_metrics_loc) = file.metrics.loc {
                for _x in (line_hits.len() as i64)..file_metrics_loc {
                    line_hits.push(-1);
                }
            }

            let file_coverage = FileCoverage {
                path: file.path.unwrap_or(file.name),
                hits: line_hits,
                ..Default::default()
            };

            file_coverages.push(file_coverage);
        }

        Ok(file_coverages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clover_results2() {
        let input = include_str!("../../tests/fixtures/clover/sample2.xml");
        insta::assert_yaml_snapshot!(Clover::new().parse_text(input).unwrap());
    }

    #[test]
    fn clover_results() {
        // Make sure that the <?xml version="1.0"?> tag is always right at the beginning of the string to avoid parsing errors
        let input = include_str!("../../tests/fixtures/clover/sample.xml");
        insta::assert_yaml_snapshot!(Clover::new().parse_text(input).unwrap());
    }
}
