use crate::Parser;
use anyhow::{Context, Result};
use qlty_types::tests::v1::FileCoverage;
use std::io::BufReader;
use std::str::FromStr;
use xml::reader::{EventReader, XmlEvent};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Clover {}

struct CloverFile {
    name: String,
    path: Option<String>,
    lines: Vec<(i64, i64)>, // (line_num, count)
    loc: Option<i64>,
}

impl Clover {
    pub fn new() -> Self {
        Self {}
    }

    fn parse_xml(&self, text: &str) -> Result<Vec<CloverFile>> {
        let reader = BufReader::new(text.as_bytes());
        let parser = EventReader::new(reader);

        let mut files = Vec::new();
        // We're not using packages, but removing it to avoid errors
        let mut current_file: Option<CloverFile> = None;
        let mut in_project = false;

        for event in parser {
            match event {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => {
                    match name.local_name.as_str() {
                        "project" => {
                            in_project = true;
                        }
                        "package" => {
                            if in_project {
                                // Start of a package element inside project
                            }
                        }
                        "file" => {
                            if in_project {
                                let mut file_name = String::new();
                                let mut file_path = None;

                                for attr in attributes {
                                    match attr.name.local_name.as_str() {
                                        "name" => file_name = attr.value,
                                        "path" => file_path = Some(attr.value),
                                        _ => {}
                                    }
                                }

                                current_file = Some(CloverFile {
                                    name: file_name,
                                    path: file_path,
                                    lines: Vec::new(),
                                    loc: None,
                                });
                            }
                        }
                        "line" => {
                            if let Some(ref mut file) = current_file {
                                let mut num = 0;
                                let mut count = 0;

                                for attr in attributes {
                                    match attr.name.local_name.as_str() {
                                        "num" => {
                                            num = match i64::from_str(&attr.value) {
                                                Ok(value) => value,
                                                Err(e) => {
                                                    eprintln!("Failed to parse 'num' attribute: {}. Value: {}", e, &attr.value);
                                                    continue;
                                                }
                                            };
                                        }
                                        "count" => {
                                            count = i64::from_str(&attr.value).with_context(|| {
                                                format!("Failed to parse 'count' attribute value: {}", attr.value)
                                            })?;
                                        }
                                        _ => {}
                                    }
                                }

                                if num > 0 {
                                    file.lines.push((num, count));
                                }
                            }
                        }
                        "metrics" => {
                            if let Some(ref mut file) = current_file {
                                for attr in attributes {
                                    if attr.name.local_name == "loc" {
                                        file.loc = i64::from_str(&attr.value).ok();
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(XmlEvent::EndElement { name }) => {
                    match name.local_name.as_str() {
                        "project" => {
                            in_project = false;
                        }
                        "file" => {
                            if let Some(file) = current_file.take() {
                                files.push(file);
                            }
                        }
                        "package" => {
                            // End of a package element
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("XML parsing error: {}", e));
                }
                _ => {}
            }
        }

        Ok(files)
    }
}

impl Parser for Clover {
    fn parse_text(&self, text: &str) -> Result<Vec<FileCoverage>> {
        let clover_files = self
            .parse_xml(text)
            .with_context(|| "Failed to parse XML text")?;

        let mut file_coverages = vec![];

        for file in clover_files {
            let mut line_hits = Vec::new();

            // Sort lines by line number to ensure they're processed in order
            let mut sorted_lines = file.lines;
            sorted_lines.sort_by_key(|(num, _)| *num);

            for (num, count) in sorted_lines {
                // Fill in gaps with -1 (no coverage information)
                for _x in (line_hits.len() as i64)..(num - 1) {
                    line_hits.push(-1);
                }

                line_hits.push(count);
            }

            // If file has a loc (lines of code) metric, fill remaining lines
            if let Some(loc) = file.loc {
                for _x in (line_hits.len() as i64)..loc {
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
