use crate::Parser;
use anyhow::Result;
use qlty_types::tests::v1::FileCoverage;
use regex::Regex;
use std::collections::BTreeMap;

pub struct Coverprofile {}

impl Default for Coverprofile {
    fn default() -> Self {
        Self::new()
    }
}

impl Coverprofile {
    pub fn new() -> Self {
        Self {}
    }

    fn push_line_hits(
        &self,
        line_count_map: &BTreeMap<i64, i64>,
        file_name: &str,
        file_coverages: &mut Vec<FileCoverage>,
    ) {
        let mut line_hits: Vec<i64> = Vec::new();
        if let Some(last_line) = line_count_map.last_key_value() {
            if *last_line.0 > 0 {
                for x in 1..=*last_line.0 {
                    if line_count_map.contains_key(&x) {
                        line_hits.push(*line_count_map.get(&x).unwrap());
                    } else {
                        line_hits.push(-1);
                    }
                }
            }
        }
        let file_coverage = FileCoverage {
            path: file_name.to_string(),
            hits: line_hits,
            ..Default::default()
        };

        file_coverages.push(file_coverage);
    }
}

impl Parser for Coverprofile {
    fn parse_text(&self, text: &str) -> Result<Vec<FileCoverage>> {
        let mut file_coverages: Vec<FileCoverage> = vec![];
        let mut current_file_name = "";
        let re = Regex::new(r"(?<start_line>\d+)\.?\d+,(?<end_line>\d+)\.?\d+ ?\d+ (?<count>\d+)")
            .unwrap();

        // becuase the results are not sorted by line number, we need to keep track of the line numbers
        let mut line_count_map: BTreeMap<i64, i64> = BTreeMap::new();

        text.split('\n').skip(1).for_each(|line| {
            if line.is_empty() {
                return;
            }

            let tokens: Vec<_> = line.split(':').collect();
            let file_name = tokens[0];
            let line_info = tokens[1];

            if current_file_name != file_name {
                if !current_file_name.is_empty() {
                    self.push_line_hits(&line_count_map, current_file_name, &mut file_coverages);
                    line_count_map = BTreeMap::new();
                }

                current_file_name = file_name;
            }

            let regex_capture = re.captures(line_info).unwrap();
            let start_line = regex_capture
                .name("start_line")
                .unwrap()
                .as_str()
                .parse::<i64>()
                .unwrap();
            let end_line = regex_capture
                .name("end_line")
                .unwrap()
                .as_str()
                .parse::<i64>()
                .unwrap();
            let count = regex_capture
                .name("count")
                .unwrap()
                .as_str()
                .parse::<i64>()
                .unwrap();

            for x in start_line..(end_line + 1) {
                line_count_map.insert(x, count);
            }
        });

        self.push_line_hits(&line_count_map, current_file_name, &mut file_coverages);

        Ok(file_coverages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverprofile_results() {
        // Make sure that the <?xml version="1.0"?> tag is always right at the beginning of the string to avoid parsing errors
        let input = include_str!("../../tests/fixtures/coverprofile/sample.out");

        let parsed_results = Coverprofile::new().parse_text(input).unwrap();
        insta::assert_yaml_snapshot!(parsed_results, @r#"
    - path: github.com/codeclimate/test-reporter/formatters/report.go
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
        - "-1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "-1"
        - "-1"
        - "1"
        - "-1"
        - "-1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "0"
        - "0"
        - "0"
        - "-1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "-1"
        - "-1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "-1"
        - "-1"
        - "1"
        - "-1"
        - "-1"
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
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "-1"
        - "-1"
        - "0"
        - "0"
        - "0"
        - "0"
        - "0"
        - "0"
        - "0"
    - path: github.com/codeclimate/test-reporter/formatters/source_file.go
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
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "-1"
        - "-1"
        - "1"
        - "1"
        - "1"
        - "-1"
        - "-1"
        - "-1"
        - "1"
        - "1"
        - "1"
        - "-1"
        - "-1"
        - "1"
        - "1"
        - "-1"
        - "-1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "-1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "-1"
        - "1"
        - "-1"
        - "1"
        - "1"
        - "-1"
        - "-1"
        - "1"
        - "0"
        - "0"
        - "0"
        - "0"
        - "-1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "0"
        - "0"
        - "0"
        - "0"
        - "0"
        - "0"
        - "0"
        - "-1"
        - "-1"
        - "1"
        - "1"
        - "1"
        - "0"
        - "0"
        - "0"
        - "1"
        - "0"
        - "0"
        - "0"
        - "0"
        - "0"
        - "-1"
        - "1"
        - "-1"
        - "1"
    - path: github.com/codeclimate/test-reporter/formatters/coverage.go
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
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "0"
        - "0"
        - "0"
        - "0"
        - "0"
        - "0"
        - "0"
        - "0"
        - "0"
        - "0"
        - "0"
        - "-1"
        - "-1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "0"
        - "0"
        - "0"
        - "1"
        - "1"
    - path: github.com/codeclimate/test-reporter/formatters/line_counts.go
      hits:
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
    "#);
    }
}
