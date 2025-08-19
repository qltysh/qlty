use crate::env::{EnvSource, SystemEnv};
use crate::Parser;
use anyhow::{Context, Result};
use qlty_types::tests::v1::FileCoverage;
use serde::Deserialize;
use serde_xml_rs;
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

pub struct Jacoco {
    env: Box<dyn EnvSource>,
}

impl Default for Jacoco {
    fn default() -> Self {
        Self::new()
    }
}

impl Jacoco {
    pub fn new() -> Self {
        Self {
            env: Box::new(SystemEnv::default()),
        }
    }

    pub fn with_env(env: Box<dyn EnvSource>) -> Self {
        Self { env }
    }

    fn get_source_paths(&self) -> Vec<String> {
        self.env
            .var("JACOCO_SOURCE_PATH")
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
            let full_path = Path::new(source_path).join(relative_path);
            if full_path.exists() {
                // Return the full path with source path prepended
                return full_path.to_string_lossy().to_string();
            }
        }

        // If file not found in any source path, use the first source path
        Path::new(&source_paths[0])
            .join(relative_path)
            .to_string_lossy()
            .to_string()
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
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[derive(Debug)]
    struct HashMapEnv {
        inner: HashMap<String, String>,
    }

    impl HashMapEnv {
        fn new(inner: HashMap<String, String>) -> Self {
            Self { inner }
        }
    }

    impl EnvSource for HashMapEnv {
        fn var(&self, name: &str) -> Option<String> {
            self.inner.get(name).cloned()
        }
    }

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
    fn jacoco_with_single_source_path_no_file_exists() {
        let input = include_str!("../../tests/fixtures/jacoco/sample.xml");

        // Create a mock environment with JACOCO_SOURCE_PATH set to /app/src
        let mut env = HashMap::new();
        env.insert("JACOCO_SOURCE_PATH".to_string(), "/app/src".to_string());

        let jacoco = Jacoco::with_env(Box::new(HashMapEnv::new(env)));
        let parsed_results = jacoco.parse_text(input).unwrap();

        // When the file doesn't exist, it should still prepend the source path
        // Use Path::join to create platform-appropriate expected paths
        assert_eq!(
            parsed_results[0].path,
            Path::new("/app/src")
                .join("be/apo/basic/Application.java")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(
            parsed_results[1].path,
            Path::new("/app/src")
                .join("be/apo/basic/rest/EchoService.java")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(
            parsed_results[2].path,
            Path::new("/app/src")
                .join("be/apo/basic/rest/model/Poney.java")
                .to_string_lossy()
                .to_string()
        );
    }

    #[test]
    fn jacoco_with_multiple_source_paths_no_files_exist() {
        let input = include_str!("../../tests/fixtures/jacoco/sample.xml");

        // Create a mock environment with multiple source paths
        let mut env = HashMap::new();
        env.insert(
            "JACOCO_SOURCE_PATH".to_string(),
            "/project/src /workspace/src /app/src".to_string(),
        );

        let jacoco = Jacoco::with_env(Box::new(HashMapEnv::new(env)));
        let parsed_results = jacoco.parse_text(input).unwrap();

        // When files don't exist, should use the first source path
        // Use Path::join to create platform-appropriate expected paths
        assert_eq!(
            parsed_results[0].path,
            Path::new("/project/src")
                .join("be/apo/basic/Application.java")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(
            parsed_results[1].path,
            Path::new("/project/src")
                .join("be/apo/basic/rest/EchoService.java")
                .to_string_lossy()
                .to_string()
        );
    }

    #[test]
    fn jacoco_without_source_path() {
        let input = include_str!("../../tests/fixtures/jacoco/sample.xml");

        // Create a mock environment without JACOCO_SOURCE_PATH
        let env = HashMap::new();
        let jacoco = Jacoco::with_env(Box::new(HashMapEnv::new(env)));
        let parsed_results = jacoco.parse_text(input).unwrap();

        // Should use the relative path as-is when no source path is set
        assert_eq!(parsed_results[0].path, "be/apo/basic/Application.java");
        assert_eq!(parsed_results[1].path, "be/apo/basic/rest/EchoService.java");
    }

    #[test]
    fn jacoco_with_source_path_and_existing_file() {
        let input = include_str!("../../tests/fixtures/jacoco/sample.xml");

        // Create a temporary directory with actual files
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path();

        // Create the expected file structure - only create the first file
        let java_path = source_path.join("be/apo/basic");
        fs::create_dir_all(&java_path).unwrap();
        fs::write(java_path.join("Application.java"), "// test file").unwrap();

        // Create a mock environment with JACOCO_SOURCE_PATH
        let mut env = HashMap::new();
        env.insert(
            "JACOCO_SOURCE_PATH".to_string(),
            source_path.to_str().unwrap().to_string(),
        );

        let jacoco = Jacoco::with_env(Box::new(HashMapEnv::new(env)));
        let parsed_results = jacoco.parse_text(input).unwrap();

        // The first file exists, so it should have the full path
        assert_eq!(
            parsed_results[0].path,
            source_path
                .join("be/apo/basic/Application.java")
                .to_string_lossy()
        );

        // The second file doesn't exist, so it should still get the source path prepended
        assert_eq!(
            parsed_results[1].path,
            source_path
                .join("be/apo/basic/rest/EchoService.java")
                .to_string_lossy()
        );
    }

    #[test]
    fn jacoco_with_multiple_source_paths_file_in_second_path() {
        let input = include_str!("../../tests/fixtures/jacoco/sample.xml");

        // Create two temporary directories
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();
        let source_path1 = temp_dir1.path();
        let source_path2 = temp_dir2.path();

        // Create file only in the second source path
        let java_path = source_path2.join("be/apo/basic");
        fs::create_dir_all(&java_path).unwrap();
        fs::write(java_path.join("Application.java"), "// test file").unwrap();

        // Create a mock environment with multiple paths
        let mut env = HashMap::new();
        let paths = format!(
            "{} {}",
            source_path1.to_str().unwrap(),
            source_path2.to_str().unwrap()
        );
        env.insert("JACOCO_SOURCE_PATH".to_string(), paths);

        let jacoco = Jacoco::with_env(Box::new(HashMapEnv::new(env)));
        let parsed_results = jacoco.parse_text(input).unwrap();

        // File exists in second path, so should use that path
        assert_eq!(
            parsed_results[0].path,
            source_path2
                .join("be/apo/basic/Application.java")
                .to_string_lossy()
        );

        // Other files don't exist, so should use first source path
        assert_eq!(
            parsed_results[1].path,
            source_path1
                .join("be/apo/basic/rest/EchoService.java")
                .to_string_lossy()
        );
    }
}
