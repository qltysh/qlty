use crate::formats::{parser_for, Formats};
use anyhow::{bail, Result};
use qlty_types::tests::v1::FileCoverage;
use serde::Serialize;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use zip::ZipArchive;

const DEFAULT_THRESHOLD: f64 = 90.0;

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

impl ValidationResult {
    pub fn compute(path: &str, threshold: Option<f64>) -> Result<ValidationResult> {
        let threshold = threshold.unwrap_or(DEFAULT_THRESHOLD);
        let file_coverages = Self::read_zip(path)?;

        let mut validation_result = ValidationResult {
            threshold,
            ..Default::default()
        };

        file_coverages.iter().for_each(|file| {
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

        validation_result.status = if validation_result.coverage_percentage < threshold {
            ValidationStatus::Invalid
        } else {
            ValidationStatus::Valid
        };

        Ok(validation_result)
    }

    fn read_zip(path: &str) -> Result<Vec<FileCoverage>> {
        let path_buf = PathBuf::from(path);

        if !path_buf.exists() {
            bail!("Coverage report file not found: {}", path_buf.display());
        }

        if path_buf.extension().is_some_and(|ext| ext == "zip") {
            let zip_file = File::open(&path_buf)?;
            let mut archive = ZipArchive::new(BufReader::new(zip_file))?;
            let coverage_zip_result = archive.by_name("file_coverages.jsonl");

            if let Ok(coverage_zip) = coverage_zip_result {
                let mut reader = BufReader::new(coverage_zip);
                let mut file_contents = String::new();
                reader.read_to_string(&mut file_contents)?;

                let parser = parser_for(&Formats::Qlty);
                let coverages = parser.parse_text(&file_contents)?;

                Ok(coverages)
            } else {
                bail!("file_coverages.jsonl not found in the zip file")
            }
        } else {
            bail!("Expected zip file got {}", path_buf.display())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qlty_analysis::utils::fs::path_to_string;
    use qlty_types::tests::v1::FileCoverage;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;

    fn create_test_zip(temp_dir_path: &Path, file_coverages: &[FileCoverage]) -> PathBuf {
        let zip_path = temp_dir_path.join("test.zip");

        let file = File::create(&zip_path).expect("Failed to create zip file");
        let mut zip = zip::ZipWriter::new(file);

        let mut jsonl = String::new();
        for fc in file_coverages {
            let line = serde_json::to_string(fc).expect("Failed to serialize FileCoverage");
            jsonl.push_str(&line);
            jsonl.push('\n');
        }

        zip.start_file("file_coverages.jsonl", SimpleFileOptions::default())
            .expect("Failed to start file in zip");
        zip.write_all(jsonl.as_bytes())
            .expect("Failed to write JSONL");
        zip.finish().expect("Failed to finish zip");

        zip_path
    }

    fn create_dummy_file(path: &PathBuf) {
        let parent = path.parent().unwrap();
        fs::create_dir_all(parent).expect("Failed to create parent dirs");
        File::create(path).expect("Failed to create dummy file");
    }

    #[test]
    fn test_all_files_present() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_dir_path = temp_dir.path();
        let file_path = temp_dir.path().join("file1.rs");

        create_dummy_file(&file_path);

        let zip_path = create_test_zip(
            temp_dir_path,
            &[FileCoverage {
                path: path_to_string(file_path),
                ..Default::default()
            }],
        );

        let result = ValidationResult::compute(zip_path.to_str().unwrap(), Some(90.0)).unwrap();
        assert_eq!(result.files_present, 1);
        assert_eq!(result.files_missing, 0);
        assert_eq!(result.status, ValidationStatus::Valid);
    }

    #[test]
    fn test_some_files_missing() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_dir_path = temp_dir.path();
        let existing_path = temp_dir.path().join("file2.rs");

        create_dummy_file(&existing_path);

        let zip_path = create_test_zip(
            temp_dir_path,
            &[
                FileCoverage {
                    path: path_to_string(existing_path),
                    ..Default::default()
                },
                FileCoverage {
                    path: "target/test-data/src/missing.rs".to_string(),
                    ..Default::default()
                },
            ],
        );

        let result = ValidationResult::compute(zip_path.to_str().unwrap(), Some(80.0)).unwrap();
        assert_eq!(result.files_present, 1);
        assert_eq!(result.files_missing, 1);
        assert_eq!(result.status, ValidationStatus::Invalid);
    }

    #[test]
    fn test_no_coverage_data() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_dir_path = temp_dir.path();

        let zip_path = create_test_zip(temp_dir_path, &[]);
        let result = ValidationResult::compute(zip_path.to_str().unwrap(), Some(80.0)).unwrap();
        assert_eq!(result.total_files, 0);
        assert_eq!(result.status, ValidationStatus::NoCoverageData);
    }

    #[test]
    fn test_non_zip_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("not_a_zip.txt");
        File::create(&path).unwrap();

        let result = ValidationResult::compute(path.to_str().unwrap(), Some(50.0));
        assert!(result.is_err());

        let err = result.unwrap_err();
        let msg = err.to_string();

        assert!(msg.contains("Expected zip file got"));
    }

    #[test]
    fn test_zip_missing_jsonl() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bad.zip");

        let file = File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("some_other_file.txt", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"Hello").unwrap();
        zip.finish().unwrap();

        let result = ValidationResult::compute(path.to_str().unwrap(), Some(50.0));
        assert!(result.is_err());

        let err = result.unwrap_err();
        let msg = err.to_string();

        assert!(msg.contains("file_coverages.jsonl not found in the zip file"));
    }
}
