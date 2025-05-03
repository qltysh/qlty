use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;

use crate::publish::Report;

const DEFAULT_THRESHOLD: f64 = 90.0;

/// Validator for validating coverage reports.
#[derive(Debug, Clone)]
pub struct Validator {
    threshold: f64,
}

impl Default for Validator {
    fn default() -> Self {
        Self {
            threshold: DEFAULT_THRESHOLD,
        }
    }
}

impl Validator {
    /// Creates a new validator with the given threshold.
    pub fn new(threshold: Option<f64>) -> Self {
        Self {
            threshold: threshold.unwrap_or(DEFAULT_THRESHOLD),
        }
    }

    /// Validates a coverage report.
    pub fn validate(&self, report: &Report) -> Result<ValidationResult> {
        let mut validation_result = ValidationResult {
            threshold: self.threshold,
            ..Default::default()
        };

        report.file_coverages.iter().for_each(|file| {
            if PathBuf::from(&file.path).exists() {
                validation_result.files_present += 1;
            } else {
                validation_result.files_missing += 1;
            }
        });

        validation_result.total_files =
            validation_result.files_present + validation_result.files_missing;

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
            totals: Default::default(),
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

        let validator = Validator::new(Some(90.0));
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

        let validator = Validator::new(Some(80.0));
        let result = validator.validate(&report).unwrap();

        assert_eq!(result.files_present, 1);
        assert_eq!(result.files_missing, 1);
        assert_eq!(result.status, ValidationStatus::Invalid);
    }

    #[test]
    fn test_no_coverage_data() {
        let report = create_test_report(vec![]);

        let validator = Validator::new(Some(80.0));
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
        let validator_lenient = Validator::new(Some(30.0));
        let result_lenient = validator_lenient.validate(&report).unwrap();
        assert_eq!(result_lenient.status, ValidationStatus::Valid);

        // With 33.33% files present and threshold of 50%, should be invalid
        let validator_strict = Validator::new(Some(50.0));
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
}
