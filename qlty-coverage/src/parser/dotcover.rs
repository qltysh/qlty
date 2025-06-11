use crate::Parser;
use anyhow::{Context, Result};
use qlty_types::tests::v1::FileCoverage;
use std::collections::HashMap;
use std::io::BufReader;
use std::str::FromStr;
use xml::reader::{EventReader, XmlEvent};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Dotcover {}

#[derive(Debug, Clone)]
struct DotcoverFile {
    name: String,
}

#[derive(Debug, Clone)]
struct Statement {
    file_index: String,
    line: i64,
    covered: bool,
}

impl Dotcover {
    pub fn new() -> Self {
        Self {}
    }

    fn parse_xml(&self, text: &str) -> Result<Vec<FileCoverage>> {
        let reader = BufReader::new(text.as_bytes());
        let parser = EventReader::new(reader);

        let mut file_indices: HashMap<String, DotcoverFile> = HashMap::new();
        let mut statements: Vec<Statement> = Vec::new();
        let mut in_file_indices = false;

        for event in parser {
            match event {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => match name.local_name.as_str() {
                    "FileIndices" => {
                        in_file_indices = true;
                    }
                    "File" if in_file_indices => {
                        let mut index = String::new();
                        let mut file_name = String::new();

                        for attr in attributes {
                            match attr.name.local_name.as_str() {
                                "Index" => index = attr.value,
                                "Name" => file_name = attr.value,
                                _ => {}
                            }
                        }

                        if !index.is_empty() && !file_name.is_empty() {
                            file_indices.insert(index.clone(), DotcoverFile { name: file_name });
                        }
                    }
                    "Statement" => {
                        let mut file_index = String::new();
                        let mut line = 0i64;
                        let mut covered = false;

                        for attr in attributes {
                            match attr.name.local_name.as_str() {
                                "FileIndex" => file_index = attr.value,
                                "Line" => {
                                    line = i64::from_str(&attr.value).with_context(|| {
                                        format!("Failed to parse line number: {}", attr.value)
                                    })?;
                                }
                                "Covered" => {
                                    covered = attr.value.to_lowercase() == "true";
                                }
                                _ => {}
                            }
                        }

                        if !file_index.is_empty() && line > 0 {
                            statements.push(Statement {
                                file_index,
                                line,
                                covered,
                            });
                        }
                    }
                    _ => {}
                },
                Ok(XmlEvent::EndElement { name }) => {
                    if name.local_name == "FileIndices" {
                        in_file_indices = false;
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("XML parsing error: {}", e));
                }
                _ => {}
            }
        }

        self.build_file_coverages(file_indices, statements)
    }

    fn build_file_coverages(
        &self,
        file_indices: HashMap<String, DotcoverFile>,
        statements: Vec<Statement>,
    ) -> Result<Vec<FileCoverage>> {
        let mut file_coverages = Vec::new();
        let mut file_statements: HashMap<String, Vec<&Statement>> = HashMap::new();

        // Group statements by file index
        for statement in &statements {
            file_statements
                .entry(statement.file_index.clone())
                .or_default()
                .push(statement);
        }

        // Build coverage for each file, sorted by file path for consistent output
        let mut sorted_files: Vec<_> = file_indices.into_iter().collect();
        sorted_files.sort_by(|a, b| a.1.name.cmp(&b.1.name));

        for (file_index, dotcover_file) in sorted_files {
            if let Some(file_stmts) = file_statements.get(&file_index) {
                let mut line_hits = Vec::new();
                let mut sorted_statements = file_stmts.clone();
                sorted_statements.sort_by_key(|stmt| stmt.line);

                let max_line = sorted_statements
                    .iter()
                    .map(|stmt| stmt.line)
                    .max()
                    .unwrap_or(0);

                // Initialize all lines as uncovered (-1)
                for _ in 0..max_line {
                    line_hits.push(-1);
                }

                // Set coverage for lines with statements
                for statement in sorted_statements {
                    let line_index = (statement.line - 1) as usize;
                    if line_index < line_hits.len() {
                        line_hits[line_index] = if statement.covered { 1 } else { 0 };
                    }
                }

                file_coverages.push(FileCoverage {
                    path: dotcover_file.name,
                    hits: line_hits,
                    ..Default::default()
                });
            }
        }

        Ok(file_coverages)
    }
}

impl Parser for Dotcover {
    fn parse_text(&self, text: &str) -> Result<Vec<FileCoverage>> {
        self.parse_xml(text)
            .with_context(|| "Failed to parse dotCover XML text")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dotcover_basic_parsing() {
        let input = r#"<?xml version="1.0" encoding="utf-8"?>
<Root CoveredStatements="17" TotalStatements="24" CoveragePercent="71" ReportType="DetailedXml" DotCoverVersion="2022.3.2">
  <FileIndices>
    <File Index="1" Name="C:\Users\fulano\Desktop\unit-testing-using-mstest\PrimeService.Tests\PrimeService_IsPrimeShould.cs" ChecksumAlgorithm="SHA256" Checksum="23612935A1229C4721145EACB2122A220B0F761AD0DB20CC980DBDB0D5003C2A" />
    <File Index="2" Name="C:\Users\fulano\Desktop\unit-testing-using-mstest\PrimeService\PrimeService.cs" ChecksumAlgorithm="SHA256" Checksum="CA22D548019CFF7919E45E7289DC438D453674E827BBFCE249587AD9DDFAD47D" />
  </FileIndices>
  <Assembly Name="PrimeService" CoveredStatements="5" TotalStatements="12" CoveragePercent="42">
    <Namespace Name="Prime.Services" CoveredStatements="5" TotalStatements="12" CoveragePercent="42">
      <Type Name="PrimeService" CoveredStatements="5" TotalStatements="6" CoveragePercent="83">
        <Method Name="IsPrime(System.Int32):System.Boolean" CoveredStatements="5" TotalStatements="6" CoveragePercent="83">
          <Statement FileIndex="2" Line="8" Column="9" EndLine="8" EndColumn="10" Covered="True" />
          <Statement FileIndex="2" Line="9" Column="13" EndLine="9" EndColumn="31" Covered="True" />
          <Statement FileIndex="2" Line="10" Column="13" EndLine="10" EndColumn="14" Covered="True" />
          <Statement FileIndex="2" Line="11" Column="17" EndLine="11" EndColumn="30" Covered="True" />
          <Statement FileIndex="2" Line="13" Column="13" EndLine="13" EndColumn="77" Covered="False" />
          <Statement FileIndex="2" Line="14" Column="9" EndLine="14" EndColumn="10" Covered="True" />
        </Method>
      </Type>
    </Namespace>
  </Assembly>
</Root>"#;

        let results = Dotcover::new().parse_text(input).unwrap();
        assert_eq!(results.len(), 1);

        let file_coverage = &results[0];
        assert_eq!(
            file_coverage.path,
            r"C:\Users\fulano\Desktop\unit-testing-using-mstest\PrimeService\PrimeService.cs"
        );
        assert_eq!(file_coverage.hits.len(), 14);

        // Check specific line coverage
        assert_eq!(file_coverage.hits[7], 1); // Line 8 - covered
        assert_eq!(file_coverage.hits[8], 1); // Line 9 - covered
        assert_eq!(file_coverage.hits[9], 1); // Line 10 - covered
        assert_eq!(file_coverage.hits[10], 1); // Line 11 - covered
        assert_eq!(file_coverage.hits[12], 0); // Line 13 - not covered
        assert_eq!(file_coverage.hits[13], 1); // Line 14 - covered
    }

    #[test]
    fn dotcover_empty_file() {
        let input = r#"<?xml version="1.0" encoding="utf-8"?>
<Root CoveredStatements="0" TotalStatements="0" CoveragePercent="0" ReportType="DetailedXml" DotCoverVersion="2022.3.2">
  <FileIndices>
  </FileIndices>
</Root>"#;

        let results = Dotcover::new().parse_text(input).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn dotcover_fixture() {
        let input = include_str!("../../tests/fixtures/dotcover/sample.xml");
        let results = Dotcover::new().parse_text(input).unwrap();

        insta::assert_yaml_snapshot!(results, @r###"
        - path: "C:\\Users\\fulano\\Desktop\\unit-testing-using-mstest\\PrimeService.Tests\\PrimeService_IsPrimeShould.cs"
          hits:
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "1"
            - "1"
            - "1"
            - "1"
            - "-1"
            - "-1"
            - "-1"
            - "1"
            - "1"
            - "-1"
            - "1"
            - "1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "1"
            - "1"
            - "-1"
            - "1"
            - "1"
        - path: "C:\\Users\\fulano\\Desktop\\unit-testing-using-mstest\\PrimeService\\PrimeService.cs"
          hits:
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "1"
            - "1"
            - "1"
            - "1"
            - "-1"
            - "0"
            - "1"
        - path: "C:\\Users\\fulano\\Desktop\\unit-testing-using-mstest\\PrimeService\\SecondService.cs"
          hits:
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "0"
            - "0"
            - "0"
            - "0"
            - "-1"
            - "0"
            - "0"
        "###);
    }
}
