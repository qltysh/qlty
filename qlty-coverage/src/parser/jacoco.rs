use crate::Parser;
use anyhow::{Context, Result};
use qlty_types::tests::v1::FileCoverage;
use serde::Deserialize;
use serde_xml_rs;
use std::env;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(rename = "report")]
struct JacocoSource {
    package: Vec<Package>,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    sourcefile: Vec<Sourcefile>,
}

#[derive(Debug, Deserialize)]
struct Sourcefile {
    name: String,
    line: Option<Vec<Line>>,
}

#[derive(Debug, Deserialize)]
struct Line {
    nr: i64,
    ci: i64,
}

pub struct Jacoco {}

impl Default for Jacoco {
    fn default() -> Self {
        Self::new()
    }
}

impl Jacoco {
    pub fn new() -> Self {
        Self {}
    }

    fn get_source_paths(&self) -> Vec<String> {
        env::var("JACOCO_SOURCE_PATH")
            .unwrap_or_default()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }

    fn resolve_file_path(&self, relative_path: &str) -> String {
        let source_paths = self.get_source_paths();

        // If no source paths are provided, return the relative path as-is
        if source_paths.is_empty() {
            return relative_path.to_string();
        }

        // Try each source path to find the file
        for source_path in &source_paths {
            let absolute_path = Path::new(source_path).join(relative_path);
            if absolute_path.exists() {
                // Return the relative path that was found
                // We still return the relative path, not the absolute one,
                // to maintain consistency with the original behavior
                return relative_path.to_string();
            }
        }

        // If file not found in any source path, return the relative path
        relative_path.to_string()
    }
}

impl Parser for Jacoco {
    fn parse_text(&self, text: &str) -> Result<Vec<FileCoverage>> {
        let source: JacocoSource =
            serde_xml_rs::from_str(text).with_context(|| "Failed to parse XML text")?;
        let mut file_coverages: Vec<FileCoverage> = vec![];

        for package in source.package.iter() {
            for sourcefile in package.sourcefile.iter() {
                let mut line_hits = Vec::new();
                if let Some(lines) = sourcefile.line.as_ref() {
                    for line in lines {
                        // Fill in any missing lines with -1 to indicate that are omitted
                        for _x in (line_hits.len() as i64)..(line.nr - 1) {
                            line_hits.push(-1)
                        }

                        line_hits.push(line.ci);
                    }
                }

                let relative_path = format!("{}/{}", package.name, sourcefile.name);
                let path = self.resolve_file_path(&relative_path);

                let file_coverage = FileCoverage {
                    path,
                    hits: line_hits,
                    ..Default::default()
                };

                file_coverages.push(file_coverage);
            }
        }

        Ok(file_coverages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn jacoco_results() {
        // Make sure that the <?xml version="1.0"?> tag is always right at the beginning of the string to avoid parsing errors
        let input = include_str!("../../tests/fixtures/jacoco/sample.xml");

        let parsed_results = Jacoco::new().parse_text(input).unwrap();
        insta::assert_yaml_snapshot!(parsed_results, @r###"
        - path: be/apo/basic/Application.java
          hits:
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "3"
            - "-1"
            - "-1"
            - "0"
            - "0"
        - path: be/apo/basic/rest/EchoService.java
          hits:
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "3"
            - "-1"
            - "-1"
            - "-1"
            - "0"
        - path: be/apo/basic/rest/model/Poney.java
          hits:
            - "-1"
            - "-1"
            - "0"
            - "-1"
            - "-1"
            - "0"
            - "-1"
            - "-1"
            - "0"
            - "-1"
            - "-1"
            - "-1"
            - "0"
            - "0"
            - "-1"
            - "-1"
            - "0"
        - path: be/apo/basic/rest/model/Empty.java
        "###);
    }

    #[test]
    fn jacoco_with_source_path() {
        let input = include_str!("../../tests/fixtures/jacoco/sample.xml");

        // Create a temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("src");
        fs::create_dir_all(&source_path).unwrap();

        // Create the expected file structure
        let java_path = source_path.join("be/apo/basic");
        fs::create_dir_all(&java_path).unwrap();
        fs::write(java_path.join("Application.java"), "// test file").unwrap();

        // Set the JACOCO_SOURCE_PATH environment variable
        env::set_var("JACOCO_SOURCE_PATH", source_path.to_str().unwrap());

        let parsed_results = Jacoco::new().parse_text(input).unwrap();

        // The path should still be the relative path, but it should have been validated
        assert_eq!(parsed_results[0].path, "be/apo/basic/Application.java");

        // Clean up
        env::remove_var("JACOCO_SOURCE_PATH");
    }

    #[test]
    fn jacoco_with_multiple_source_paths() {
        let input = include_str!("../../tests/fixtures/jacoco/sample.xml");

        // Create multiple temporary directories
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();
        let source_path1 = temp_dir1.path().join("src1");
        let source_path2 = temp_dir2.path().join("src2");
        fs::create_dir_all(&source_path1).unwrap();
        fs::create_dir_all(&source_path2).unwrap();

        // Create file in the second source path
        let java_path = source_path2.join("be/apo/basic");
        fs::create_dir_all(&java_path).unwrap();
        fs::write(java_path.join("Application.java"), "// test file").unwrap();

        // Set multiple paths in JACOCO_SOURCE_PATH
        let paths = format!(
            "{} {}",
            source_path1.to_str().unwrap(),
            source_path2.to_str().unwrap()
        );
        env::set_var("JACOCO_SOURCE_PATH", paths);

        let parsed_results = Jacoco::new().parse_text(input).unwrap();

        // The path should still be the relative path
        assert_eq!(parsed_results[0].path, "be/apo/basic/Application.java");

        // Clean up
        env::remove_var("JACOCO_SOURCE_PATH");
    }

    #[test]
    fn jacoco_without_source_path() {
        let input = include_str!("../../tests/fixtures/jacoco/sample.xml");

        // Ensure JACOCO_SOURCE_PATH is not set
        env::remove_var("JACOCO_SOURCE_PATH");

        let parsed_results = Jacoco::new().parse_text(input).unwrap();

        // Should use the default behavior
        assert_eq!(parsed_results[0].path, "be/apo/basic/Application.java");
    }
}
