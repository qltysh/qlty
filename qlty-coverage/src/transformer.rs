use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use qlty_config::Workspace;
use qlty_types::tests::v1::{CoverageMetadata, CoverageSummary, FileCoverage};
use regex::Regex;
use std::{fmt::Debug, path::PathBuf};

pub trait Transformer: Debug + Send + Sync + 'static {
    fn transform(&self, file_coverage: FileCoverage) -> Option<FileCoverage>;
    fn clone_box(&self) -> Box<dyn Transformer>;

    fn is_default_path_fixer(&self) -> bool {
        false
    }
}

impl Clone for Box<dyn Transformer> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct ComputeSummary {}

impl ComputeSummary {
    pub fn new() -> Self {
        Self {}
    }
}

impl Transformer for ComputeSummary {
    fn transform(&self, file_coverage: FileCoverage) -> Option<FileCoverage> {
        let mut covered = 0;
        let mut missed = 0;
        let mut omit = 0;

        for hit in &file_coverage.hits {
            match hit {
                -1 => omit += 1,
                0 => missed += 1,
                _ => covered += 1,
            }
        }

        let mut file_coverage = file_coverage;

        file_coverage.summary = Some(CoverageSummary {
            covered,
            missed,
            omit,
            total: covered + missed + omit,
        });

        Some(file_coverage)
    }

    fn clone_box(&self) -> Box<dyn Transformer> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct AppendMetadata {
    metadata: CoverageMetadata,
}

impl AppendMetadata {
    pub fn new(metadata: &CoverageMetadata) -> Self {
        Self {
            metadata: metadata.clone(),
        }
    }
}

impl Transformer for AppendMetadata {
    fn transform(&self, file_coverage: FileCoverage) -> Option<FileCoverage> {
        let mut file_coverage = file_coverage;
        file_coverage.build_id = self.metadata.build_id.clone();
        file_coverage.tag = self.metadata.tag.clone();
        file_coverage.branch = self.metadata.branch.clone();
        file_coverage.commit_sha = Some(self.metadata.commit_sha.clone());
        file_coverage.uploaded_at = self.metadata.uploaded_at;

        if self.metadata.pull_request_number != String::default() {
            file_coverage.pull_request_number = Some(self.metadata.pull_request_number.clone());
        }

        Some(file_coverage)
    }

    fn clone_box(&self) -> Box<dyn Transformer> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct IgnorePaths {
    glob_set: GlobSet,
}

impl IgnorePaths {
    pub fn new(paths: &[String]) -> Result<Self> {
        let mut builder = GlobSetBuilder::new();

        for glob in paths {
            builder.add(Glob::new(glob)?);
        }

        Ok(Self {
            glob_set: builder.build()?,
        })
    }
}

impl Transformer for IgnorePaths {
    fn transform(&self, file_coverage: FileCoverage) -> Option<FileCoverage> {
        if self.glob_set.is_match(&file_coverage.path) {
            None
        } else {
            Some(file_coverage)
        }
    }

    fn clone_box(&self) -> Box<dyn Transformer> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct AddPrefix {
    prefix: String,
}

impl AddPrefix {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_owned(),
        }
    }
}

impl Transformer for AddPrefix {
    fn transform(&self, file_coverage: FileCoverage) -> Option<FileCoverage> {
        let mut file_coverage = file_coverage;
        file_coverage.path = format!("{}{}", self.prefix, file_coverage.path);
        Some(file_coverage)
    }

    fn clone_box(&self) -> Box<dyn Transformer> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct StripPrefix {
    prefix: PathBuf,
}

impl StripPrefix {
    pub fn new(prefix: String) -> Self {
        Self {
            prefix: PathBuf::from(prefix),
        }
    }

    pub fn new_from_git_root() -> Result<Self> {
        Ok(Self {
            prefix: Workspace::assert_within_git_directory()?,
        })
    }
}

impl Transformer for StripPrefix {
    fn transform(&self, file_coverage: FileCoverage) -> Option<FileCoverage> {
        let mut file_coverage = file_coverage;
        let coverage_path = PathBuf::from(&file_coverage.path);

        if let Ok(sanitized_path) = coverage_path.strip_prefix(&self.prefix) {
            file_coverage.path = sanitized_path.to_string_lossy().to_string();
        }

        Some(file_coverage)
    }

    fn clone_box(&self) -> Box<dyn Transformer> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct StripDotSlashPrefix;

impl Transformer for StripDotSlashPrefix {
    fn transform(&self, file_coverage: FileCoverage) -> Option<FileCoverage> {
        let mut file_coverage = file_coverage;
        if file_coverage.path.starts_with("./") {
            file_coverage.path = file_coverage.path[2..].to_string();
        }
        Some(file_coverage)
    }

    fn clone_box(&self) -> Box<dyn Transformer> {
        Box::new(Self)
    }
}

#[derive(Debug, Clone)]
pub struct DefaultPathFixer {
    patterns: Vec<Regex>,
}

impl DefaultPathFixer {
    pub fn new() -> Result<Self> {
        let pattern_strings = vec![
            r"^/?home/circleci/project/",
            r"^/?(home|Users)/runner/work/[^/]+/[^/]+/",
            r"^/?github.com/[^/]+/[^/]+/",
            r"^/?(home|Users)/travis/build/[^/]+/[^/]+/",
            r"^/?(home|Users)/jenkins/jobs/[^/]+/workspace/",
            r"^/?Users/distiller/[^/]+/",
            r"^/?(home|Users)/[^/]+/workspace/[^/]+/[^/]+/",
        ];

        let mut patterns = Vec::new();
        for pattern in pattern_strings {
            patterns.push(Regex::new(pattern)?);
        }

        Ok(Self { patterns })
    }

    fn normalize_path(&self, path: &str) -> String {
        // Convert Windows backslashes to forward slashes
        if cfg!(windows) || path.contains('\\') {
            path.replace('\\', "/")
        } else {
            path.to_string()
        }
    }

    fn clean_windows_prefix(&self, path: &str) -> String {
        if path.len() >= 3 && path.chars().nth(1) == Some(':') && path.chars().nth(2) == Some('/') {
            path[3..].to_string()
        } else {
            path.to_string()
        }
    }
}

impl Transformer for DefaultPathFixer {
    fn transform(&self, file_coverage: FileCoverage) -> Option<FileCoverage> {
        let mut file_coverage = file_coverage;

        // First normalize the path (convert backslashes to forward slashes)
        let normalized_path = self.normalize_path(&file_coverage.path);

        // Remove Windows drive prefix if present
        let path_without_drive = self.clean_windows_prefix(&normalized_path);

        // Try to match and remove any of the CI/CD patterns
        let mut cleaned_path = path_without_drive.clone();
        for pattern in &self.patterns {
            if let Some(captures) = pattern.find(&path_without_drive) {
                cleaned_path = path_without_drive[captures.end()..].to_string();
                break; // Apply only the first matching pattern
            }
        }

        file_coverage.path = cleaned_path;
        Some(file_coverage)
    }

    fn clone_box(&self) -> Box<dyn Transformer> {
        Box::new(self.clone())
    }

    fn is_default_path_fixer(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_path_fixer_circleci() {
        let transformer = DefaultPathFixer::new().unwrap();

        // Test with leading slash
        let file_coverage = FileCoverage {
            path: "/home/circleci/project/src/main.rs".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "src/main.rs");

        // Test without leading slash
        let file_coverage = FileCoverage {
            path: "home/circleci/project/lib/test.js".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "lib/test.js");
    }

    #[test]
    fn test_default_path_fixer_github_actions() {
        let transformer = DefaultPathFixer::new().unwrap();

        // Test with Users prefix
        let file_coverage = FileCoverage {
            path: "/Users/runner/work/repo/repo/lib/test.js".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "lib/test.js");

        // Test with home prefix
        let file_coverage = FileCoverage {
            path: "/home/runner/work/myproject/myproject/src/app.py".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "src/app.py");
    }

    #[test]
    fn test_default_path_fixer_github_dot_com() {
        let transformer = DefaultPathFixer::new().unwrap();

        // Test with Users prefix
        let file_coverage = FileCoverage {
            path: "github.com/repo/repo/lib/test.js".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "lib/test.js");

        // Test with home prefix
        let file_coverage = FileCoverage {
            path: "/github.com/myproject/myproject/src/app.py".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "src/app.py");
    }

    #[test]
    fn test_default_path_fixer_travis() {
        let transformer = DefaultPathFixer::new().unwrap();

        // Test Travis with home
        let file_coverage = FileCoverage {
            path: "/home/travis/build/org/repo/app.py".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "app.py");

        // Test Travis with Users
        let file_coverage = FileCoverage {
            path: "Users/travis/build/myorg/myrepo/index.html".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "index.html");
    }

    #[test]
    fn test_default_path_fixer_jenkins() {
        let transformer = DefaultPathFixer::new().unwrap();

        let file_coverage = FileCoverage {
            path: "/home/jenkins/jobs/job1/workspace/src/main.java".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "src/main.java");

        // Test with Users prefix
        let file_coverage = FileCoverage {
            path: "Users/jenkins/jobs/build-job/workspace/test.go".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "test.go");
    }

    #[test]
    fn test_default_path_fixer_distiller() {
        let transformer = DefaultPathFixer::new().unwrap();

        let file_coverage = FileCoverage {
            path: "/Users/distiller/project/src/component.tsx".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "src/component.tsx");
    }

    #[test]
    fn test_default_path_fixer_generic_workspace() {
        let transformer = DefaultPathFixer::new().unwrap();

        // Test with home prefix
        let file_coverage = FileCoverage {
            path: "/home/builder/workspace/frontend/app/main.js".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "main.js");

        // Test with Users prefix
        let file_coverage = FileCoverage {
            path: "Users/dev/workspace/backend/api/server.py".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "server.py");
    }

    #[test]
    fn test_default_path_fixer_windows_paths() {
        let transformer = DefaultPathFixer::new().unwrap();

        // Test Windows path with backslashes and GitHub Actions pattern
        let file_coverage = FileCoverage {
            path: r"C:\Users\runner\work\repo\repo\src\main.cs".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "src/main.cs");

        // Test Windows path with D: drive
        let file_coverage = FileCoverage {
            path: r"D:\home\jenkins\jobs\test\workspace\app.rb".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "app.rb");

        // Test Windows path with mixed slashes
        let file_coverage = FileCoverage {
            path: r"C:/Users/travis\build/org\repo\test.py".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "test.py");
    }

    #[test]
    fn test_default_path_fixer_non_matching() {
        let transformer = DefaultPathFixer::new().unwrap();

        // Test path that doesn't match any pattern
        let file_coverage = FileCoverage {
            path: "src/components/Header.tsx".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "src/components/Header.tsx");

        // Test absolute path that doesn't match
        let file_coverage = FileCoverage {
            path: "/opt/app/src/main.go".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "/opt/app/src/main.go");

        // Test relative path
        let file_coverage = FileCoverage {
            path: "./lib/utils.js".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "./lib/utils.js");
    }

    #[test]
    fn test_default_path_fixer_edge_cases() {
        let transformer = DefaultPathFixer::new().unwrap();

        // Test empty path
        let file_coverage = FileCoverage {
            path: "".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "");

        // Test path that exactly matches pattern (no file after)
        let file_coverage = FileCoverage {
            path: "/home/circleci/project/".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(result.path, "");

        // Test very long path
        let file_coverage = FileCoverage {
            path: "/home/runner/work/repo/repo/very/deep/nested/directory/structure/with/many/levels/file.txt".to_string(),
            ..Default::default()
        };
        let result = transformer.transform(file_coverage).unwrap();
        assert_eq!(
            result.path,
            "very/deep/nested/directory/structure/with/many/levels/file.txt"
        );
    }

    #[test]
    fn test_strip_prefix_transformer_no_trailing_slash() {
        let transformer = StripPrefix::new("/home/circleci/project".to_string());
        let file_coverage = FileCoverage {
            path: "/home/circleci/project/app/deep/nested/file.rb".to_string(),
            ..Default::default()
        };
        let file_coverage = transformer.transform(file_coverage).unwrap();
        assert_eq!(file_coverage.path, "app/deep/nested/file.rb".to_string());
    }

    #[test]
    fn test_strip_prefix_transformer_trailing_slash() {
        let transformer = StripPrefix::new("/home/circleci/project/".to_string());
        let file_coverage = FileCoverage {
            path: "/home/circleci/project/app/deep/nested/file.rb".to_string(),
            ..Default::default()
        };
        let file_coverage = transformer.transform(file_coverage).unwrap();
        assert_eq!(file_coverage.path, "app/deep/nested/file.rb".to_string());
    }

    #[test]
    fn test_add_prefix_transformer_trailing_slash() {
        let transformer = AddPrefix::new("project/");
        let file_coverage = FileCoverage {
            path: "src/main.rs".to_string(),
            ..Default::default()
        };
        let transformed = transformer.transform(file_coverage).unwrap();
        assert_eq!(transformed.path, "project/src/main.rs");
    }

    // Documenting current behavior, not necessarily desired behavior
    #[test]
    fn test_add_prefix_transformer_no_trailing_slash() {
        let transformer = AddPrefix::new("project");
        let file_coverage = FileCoverage {
            path: "src/main.rs".to_string(),
            ..Default::default()
        };
        let transformed = transformer.transform(file_coverage).unwrap();
        assert_eq!(transformed.path, "projectsrc/main.rs");
    }
}
