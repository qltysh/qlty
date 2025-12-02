use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::publish::Report;

const DEFAULT_THRESHOLD: f64 = 90.0;

/// Validator for validating coverage reports.
#[derive(Debug, Clone)]
pub struct Validator {
    threshold: f64,
    workspace_root: Option<PathBuf>,
}

impl Default for Validator {
    fn default() -> Self {
        Self {
            threshold: DEFAULT_THRESHOLD,
            workspace_root: None,
        }
    }
}

impl Validator {
    /// Creates a new validator with the given threshold and workspace root.
    pub fn new(threshold: Option<f64>, workspace_root: Option<PathBuf>) -> Self {
        Self {
            threshold: threshold.unwrap_or(DEFAULT_THRESHOLD),
            workspace_root,
        }
    }

    /// Validates a coverage report.
    pub fn validate(&self, report: &Report) -> Result<ValidationResult> {
        let mut validation_result = ValidationResult {
            threshold: self.threshold,
            ..Default::default()
        };

        report.file_coverages.iter().for_each(|file| {
            let path = PathBuf::from(&file.path);

            if !path.exists() {
                validation_result.files_missing += 1;
                return;
            }

            if !self.is_within_workspace(&path) {
                validation_result.files_outside_workspace += 1;
                return;
            }

            validation_result.files_present += 1;
        });

        validation_result.total_files = validation_result.files_present
            + validation_result.files_missing
            + validation_result.files_outside_workspace;

        if validation_result.total_files == 0 {
            validation_result.status = ValidationStatus::NoCoverageData;
            return Ok(validation_result);
        }

        validation_result.coverage_percentage =
            (validation_result.files_present as f64 / validation_result.total_files as f64) * 100.0;

        validation_result.status = if validation_result.coverage_percentage < self.threshold {
            ValidationStatus::Invalid
        } else {
            ValidationStatus::Valid
        };

        Ok(validation_result)
    }

    fn is_within_workspace(&self, file_path: &Path) -> bool {
        let Some(ref workspace_root) = self.workspace_root else {
            return true;
        };

        match (file_path.canonicalize(), workspace_root.canonicalize()) {
            (Ok(canonical_file), Ok(canonical_root)) => canonical_file.starts_with(&canonical_root),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub enum ValidationStatus {
    #[default]
    #[serde(rename = "valid")]
    Valid,
    #[serde(rename = "invalid")]
    Invalid,
    #[serde(rename = "no_coverage_data")]
    NoCoverageData,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ValidationResult {
    pub files_present: usize,
    pub files_missing: usize,
    pub files_outside_workspace: usize,
    pub total_files: usize,
    pub coverage_percentage: f64,
    pub threshold: f64,
    pub status: ValidationStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use qlty_analysis::utils::fs::path_to_string;
    use qlty_types::tests::v1::{CoverageMetadata, FileCoverage, ReportFile};
    use std::collections::HashSet;
    use std::fs::{self, File};
    use tempfile::tempdir;

    // Helper function to create a test Report instance
    fn create_test_report(file_coverages: Vec<FileCoverage>) -> Report {
        // Create a minimal valid Report
        Report {
            metadata: CoverageMetadata::default(),
            report_files: vec![ReportFile::default()],
            file_coverages,
            found_files: HashSet::new(),
            missing_files: HashSet::new(),
            outside_workspace_files: HashSet::new(),
            totals: Default::default(),
            excluded_files_count: 0,
            auto_path_fixing_enabled: false,
        }
    }

    fn create_dummy_file(path: &PathBuf) {
        let parent = path.parent().unwrap();
        fs::create_dir_all(parent).expect("Failed to create parent dirs");
        File::create(path).expect("Failed to create dummy file");
    }

    #[test]
    fn test_all_files_present() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("file1.rs");

        create_dummy_file(&file_path);

        let report = create_test_report(vec![FileCoverage {
            path: path_to_string(file_path),
            ..Default::default()
        }]);

        let validator = Validator::new(Some(90.0), None);
        let result = validator.validate(&report).unwrap();

        assert_eq!(result.files_present, 1);
        assert_eq!(result.files_missing, 0);
        assert_eq!(result.status, ValidationStatus::Valid);
    }

    #[test]
    fn test_some_files_missing() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let existing_path = temp_dir.path().join("file2.rs");

        create_dummy_file(&existing_path);

        let report = create_test_report(vec![
            FileCoverage {
                path: path_to_string(existing_path),
                ..Default::default()
            },
            FileCoverage {
                path: "target/test-data/src/missing.rs".to_string(),
                ..Default::default()
            },
        ]);

        let validator = Validator::new(Some(80.0), None);
        let result = validator.validate(&report).unwrap();

        assert_eq!(result.files_present, 1);
        assert_eq!(result.files_missing, 1);
        assert_eq!(result.status, ValidationStatus::Invalid);
    }

    #[test]
    fn test_no_coverage_data() {
        let report = create_test_report(vec![]);

        let validator = Validator::new(Some(80.0), None);
        let result = validator.validate(&report).unwrap();

        assert_eq!(result.total_files, 0);
        assert_eq!(result.status, ValidationStatus::NoCoverageData);
    }

    #[test]
    fn test_threshold_enforcement() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let existing_path = temp_dir.path().join("file3.rs");

        create_dummy_file(&existing_path);

        let report = create_test_report(vec![
            FileCoverage {
                path: path_to_string(existing_path),
                ..Default::default()
            },
            FileCoverage {
                path: "target/test-data/src/missing1.rs".to_string(),
                ..Default::default()
            },
            FileCoverage {
                path: "target/test-data/src/missing2.rs".to_string(),
                ..Default::default()
            },
        ]);

        // With 33.33% files present and threshold of 30%, should be valid
        let validator_lenient = Validator::new(Some(30.0), None);
        let result_lenient = validator_lenient.validate(&report).unwrap();
        assert_eq!(result_lenient.status, ValidationStatus::Valid);

        // With 33.33% files present and threshold of 50%, should be invalid
        let validator_strict = Validator::new(Some(50.0), None);
        let result_strict = validator_strict.validate(&report).unwrap();
        assert_eq!(result_strict.status, ValidationStatus::Invalid);
    }

    #[test]
    fn test_default_threshold() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let existing_path = temp_dir.path().join("file4.rs");

        create_dummy_file(&existing_path);

        let report = create_test_report(vec![
            FileCoverage {
                path: path_to_string(existing_path),
                ..Default::default()
            },
            FileCoverage {
                path: "target/test-data/src/missing.rs".to_string(),
                ..Default::default()
            },
        ]);

        // 50% files present with default threshold (90%)
        let validator = Validator::default();
        let result = validator.validate(&report).unwrap();

        assert_eq!(result.files_present, 1);
        assert_eq!(result.files_missing, 1);
        assert_eq!(result.threshold, DEFAULT_THRESHOLD);
        assert_eq!(result.status, ValidationStatus::Invalid);
    }

    #[test]
    fn test_file_inside_workspace() {
        let workspace_dir = tempdir().expect("Failed to create workspace dir");
        let file_path = workspace_dir.path().join("src/file.rs");

        create_dummy_file(&file_path);

        let report = create_test_report(vec![FileCoverage {
            path: path_to_string(&file_path),
            ..Default::default()
        }]);

        let validator = Validator::new(Some(90.0), Some(workspace_dir.path().to_path_buf()));
        let result = validator.validate(&report).unwrap();

        assert_eq!(result.files_present, 1);
        assert_eq!(result.files_missing, 0);
        assert_eq!(result.files_outside_workspace, 0);
        assert_eq!(result.status, ValidationStatus::Valid);
    }

    #[test]
    fn test_file_outside_workspace() {
        let workspace_dir = tempdir().expect("Failed to create workspace dir");
        let outside_dir = tempdir().expect("Failed to create outside dir");
        let outside_file = outside_dir.path().join("external.rs");

        create_dummy_file(&outside_file);

        let report = create_test_report(vec![FileCoverage {
            path: path_to_string(&outside_file),
            ..Default::default()
        }]);

        let validator = Validator::new(Some(90.0), Some(workspace_dir.path().to_path_buf()));
        let result = validator.validate(&report).unwrap();

        assert_eq!(result.files_present, 0);
        assert_eq!(result.files_missing, 0);
        assert_eq!(result.files_outside_workspace, 1);
        assert_eq!(result.status, ValidationStatus::Invalid);
    }

    #[test]
    fn test_mixed_inside_outside_missing() {
        let workspace_dir = tempdir().expect("Failed to create workspace dir");
        let outside_dir = tempdir().expect("Failed to create outside dir");

        let inside_file = workspace_dir.path().join("src/inside.rs");
        let outside_file = outside_dir.path().join("external.rs");

        create_dummy_file(&inside_file);
        create_dummy_file(&outside_file);

        let report = create_test_report(vec![
            FileCoverage {
                path: path_to_string(&inside_file),
                ..Default::default()
            },
            FileCoverage {
                path: path_to_string(&outside_file),
                ..Default::default()
            },
            FileCoverage {
                path: "nonexistent/missing.rs".to_string(),
                ..Default::default()
            },
        ]);

        let validator = Validator::new(Some(30.0), Some(workspace_dir.path().to_path_buf()));
        let result = validator.validate(&report).unwrap();

        assert_eq!(result.files_present, 1);
        assert_eq!(result.files_missing, 1);
        assert_eq!(result.files_outside_workspace, 1);
        assert_eq!(result.total_files, 3);
        assert!((result.coverage_percentage - 33.33).abs() < 0.1);
        assert_eq!(result.status, ValidationStatus::Valid);
    }

    #[test]
    fn test_no_workspace_root_allows_all_existing_files() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("file.rs");

        create_dummy_file(&file_path);

        let report = create_test_report(vec![FileCoverage {
            path: path_to_string(&file_path),
            ..Default::default()
        }]);

        let validator = Validator::new(Some(90.0), None);
        let result = validator.validate(&report).unwrap();

        assert_eq!(result.files_present, 1);
        assert_eq!(result.files_outside_workspace, 0);
        assert_eq!(result.status, ValidationStatus::Valid);
    }
}
