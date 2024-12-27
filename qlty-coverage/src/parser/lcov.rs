use crate::Parser;
use anyhow::Result;
use qlty_types::tests::v1::FileCoverage;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Lcov {}

impl Lcov {
    pub fn new() -> Self {
        Self {}
    }
}

impl Parser for Lcov {
    fn parse_text(&self, text: &str) -> Result<Vec<FileCoverage>> {
        let mut file_coverages = vec![];
        let mut lcov_lines = text.lines();

        while let Some(lcov_line) = lcov_lines.next() {
            if let Some(path_string) = lcov_line.strip_prefix(SF) {
                let file_coverage_path = path_string.to_string();

                let mut file_coverage = FileCoverage {
                    path: file_coverage_path,
                    ..Default::default()
                };

                let mut line_numbers_to_hits = HashMap::new();

                for lcov_line in lcov_lines.by_ref() {
                    if let Some(lcov_line) = lcov_line.strip_prefix(DA) {
                        let mut split = lcov_line.split(',');

                        let line_number = split.next().unwrap();
                        let line_number = line_number.parse::<u32>().unwrap();

                        let hits_count = split.next().unwrap();
                        let hits_count = hits_count.parse::<u64>().unwrap();

                        *line_numbers_to_hits.entry(line_number).or_insert(0) += hits_count;
                    } else if lcov_line.starts_with(END_OF_RECORD) {
                        break;
                    }
                }

                let maximum_line_number = line_numbers_to_hits.keys().max().unwrap();
                let mut line_hits: Vec<i64> = vec![-1; *maximum_line_number as usize];

                for (line_number, hits) in line_numbers_to_hits {
                    line_hits[(line_number - 1) as usize] = hits as i64;
                }

                file_coverage.hits = line_hits;
                file_coverages.push(file_coverage);
            }
        }

        Ok(file_coverages)
    }
}

const SF: &str = "SF:";
const DA: &str = "DA:";
const END_OF_RECORD: &str = "end_of_record";

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple() {
        let input = r#"
SF:src/lib.rs
DA:1,1
DA:2,0
DA:5,10
"#;

        insta::assert_yaml_snapshot!(Lcov::new().parse_text(input).unwrap(), @r#"
        - path: src/lib.rs
          hits:
            - "1"
            - "0"
            - "-1"
            - "-1"
            - "10"
        "#);
    }

    #[test]
    fn test_fixture() {
        let input = include_str!("../../tests/fixtures/lcov/sample.lcov");
        insta::assert_yaml_snapshot!(Lcov::new().parse_text(input).unwrap(), @r#"
    - path: formatter.js
      hits:
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "-1"
        - "1"
        - "1"
        - "-1"
        - "-1"
        - "1"
        - "5"
        - "-1"
        - "-1"
        - "1"
        - "2"
        - "1"
        - "1"
        - "1"
        - "-1"
        - "0"
        - "-1"
        - "-1"
        - "-1"
        - "1"
        - "2"
        - "-1"
        - "2"
        - "2"
        - "0"
        - "-1"
        - "-1"
        - "2"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "2"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "2"
    "#);
    }
}
