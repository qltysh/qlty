use crate::git::retrieve_commit_metadata;
use crate::publish::Plan;
use crate::publish::Settings;
use crate::transformer::AddPrefix;
use crate::transformer::AppendMetadata;
use crate::transformer::ComputeSummary;
use crate::transformer::IgnorePaths;
use crate::transformer::StripDotSlashPrefix;
use crate::transformer::StripPrefix;
use crate::utils::extract_path_and_format;
use crate::Transformer;
use anyhow::Result;
use pbjson_types::Timestamp;
use qlty_config::version::LONG_VERSION;
use qlty_config::QltyConfig;
use qlty_types::tests::v1::CoverageMetadata;
use qlty_types::tests::v1::ReferenceType;
use qlty_types::tests::v1::ReportFile;
use std::vec;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct Planner {
    config: QltyConfig,
    settings: Settings,
}

impl Planner {
    pub fn new(config: &QltyConfig, settings: &Settings) -> Self {
        Self {
            config: config.clone(),
            settings: settings.clone(),
        }
    }

    pub fn compute(&self) -> Result<Plan> {
        let metadata = self.compute_metadata()?;

        Ok(Plan {
            metadata: metadata.clone(),
            report_files: self.compute_report_files()?,
            transformers: self.compute_transformers(&metadata)?,
            skip_missing_files: self.settings.skip_missing_files,
        })
    }

    pub fn compute_metadata(&self) -> Result<CoverageMetadata> {
        let now = OffsetDateTime::now_utc();

        let mut metadata = if let Some(ci) = crate::ci::current() {
            ci.metadata()
        } else {
            CoverageMetadata {
                ci: "unknown".to_string(),
                publish_command: std::env::args().collect::<Vec<String>>().join(" "),
                ..CoverageMetadata::default()
            }
        };
        metadata.cli_version = LONG_VERSION.to_string();

        metadata.uploaded_at = Some(Timestamp {
            seconds: now.unix_timestamp(),
            nanos: now.nanosecond() as i32,
        });
        metadata.tag = self.settings.tag.clone();
        metadata.name = self.settings.name.clone();
        metadata.total_parts_count = self.settings.total_parts_count;
        metadata.incomplete = self.settings.incomplete;

        // Override metadata with command line arguments
        if let Some(build_id) = self.settings.override_build_id.clone() {
            metadata.build_id = build_id;
        }

        if let Some(commit_sha) = self.settings.override_commit_sha.clone() {
            metadata.commit_sha = commit_sha;
        }

        if let Some(branch) = self.settings.override_branch.clone() {
            metadata.branch = branch;
        }

        if let Some(pull_request_number) = self.settings.override_pull_request_number.clone() {
            metadata.pull_request_number = pull_request_number;
        }

        let commit_metadata = retrieve_commit_metadata()?;

        if let Some(commit_data) = commit_metadata {
            // Git is available, use git metadata
            metadata.commit_message = commit_data.commit_message;
            metadata.committer_email = commit_data.committer_email;
            metadata.committer_name = commit_data.committer_name;
            metadata.author_email = commit_data.author_email;
            metadata.author_name = commit_data.author_name;
            metadata.author_time = Some(Timestamp {
                seconds: commit_data.author_time.seconds(),
                nanos: 0,
            });

            // Use override commit time if provided, otherwise use git commit time
            if let Some(override_time) = &self.settings.override_commit_time {
                let parsed_timestamp = Self::parse_timestamp(override_time)?;
                metadata.commit_time = Some(Timestamp {
                    seconds: parsed_timestamp,
                    nanos: 0,
                });
            } else {
                metadata.commit_time = Some(Timestamp {
                    seconds: commit_data.commit_time.seconds(),
                    nanos: 0,
                });
            }
        } else {
            // Git is not available, use defaults and require override_commit_time
            metadata.commit_message = String::new();
            metadata.committer_email = String::new();
            metadata.committer_name = String::new();
            metadata.author_email = String::new();
            metadata.author_name = String::new();
            metadata.author_time = None;

            // When git is not available, override_commit_time is required
            if let Some(override_time) = &self.settings.override_commit_time {
                let parsed_timestamp = Self::parse_timestamp(override_time)?;
                metadata.commit_time = Some(Timestamp {
                    seconds: parsed_timestamp,
                    nanos: 0,
                });
            } else {
                return Err(anyhow::anyhow!(
                    "Git repository not found. When running without git, --override-commit-time must be provided."
                ));
            }
        }

        metadata.reference_type = if !metadata.pull_request_number.is_empty() {
            ReferenceType::PullRequest as i32
        } else if metadata.git_tag.is_some() && metadata.git_tag.as_ref().unwrap() != "" {
            ReferenceType::Tag as i32
        } else if !metadata.branch.is_empty() {
            ReferenceType::Branch as i32
        } else {
            ReferenceType::Unspecified as i32
        };

        Ok(metadata)
    }

    fn parse_timestamp(timestamp_str: &str) -> Result<i64> {
        // Try parsing as Unix timestamp (seconds since epoch) first
        if let Ok(timestamp) = timestamp_str.parse::<i64>() {
            return Ok(timestamp);
        }

        // Try parsing as RFC3339/ISO8601 format
        if let Ok(datetime) = time::OffsetDateTime::parse(
            timestamp_str,
            &time::format_description::well_known::Rfc3339,
        ) {
            return Ok(datetime.unix_timestamp());
        }

        // Try parsing as ISO8601 with a basic format
        if let Ok(datetime) = time::OffsetDateTime::parse(
            timestamp_str,
            &time::format_description::well_known::Iso8601::DEFAULT,
        ) {
            return Ok(datetime.unix_timestamp());
        }

        anyhow::bail!(
            "Failed to parse timestamp '{}'. Expected Unix timestamp (seconds since epoch) or RFC3339/ISO8601 format (e.g., '2023-01-01T12:00:00Z')",
            timestamp_str
        )
    }

    fn compute_report_files(&self) -> Result<Vec<ReportFile>> {
        let paths = if self.settings.paths.is_empty() {
            self.config.coverage.paths.clone().unwrap_or_default()
        } else {
            self.settings.paths.clone()
        };

        let mut report_files: Vec<ReportFile> = vec![];

        for path in paths {
            let (path, format) = extract_path_and_format(&path, self.settings.report_format)?;

            report_files.push(ReportFile {
                path: path.to_string_lossy().into_owned(),
                format: format.to_string(),
                ..Default::default()
            })
        }

        Ok(report_files)
    }

    fn compute_transformers(
        &self,
        metadata: &CoverageMetadata,
    ) -> Result<Vec<Box<dyn Transformer>>> {
        let mut transformers: Vec<Box<dyn Transformer>> = vec![];

        transformers.push(Box::new(ComputeSummary::new()));

        if let Some(prefix) = self.settings.strip_prefix.clone() {
            transformers.push(Box::new(StripPrefix::new(prefix)));
        } else if let Ok(strip_prefix) = StripPrefix::new_from_git_root() {
            transformers.push(Box::new(strip_prefix));
        }

        transformers.push(Box::new(StripDotSlashPrefix));

        if self.config.coverage.ignores.is_some() {
            transformers.push(Box::new(IgnorePaths::new(
                self.config.coverage.ignores.as_ref().unwrap(),
            )?));
        }

        if let Some(prefix) = self.settings.add_prefix.clone() {
            transformers.push(Box::new(AddPrefix::new(&prefix)));
        }

        transformers.push(Box::new(AppendMetadata::new(metadata)));
        Ok(transformers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planner_override_commit_time_tests() {
        let config = QltyConfig::default();
        let settings = Settings {
            override_commit_time: Some("2025-05-30T05:00:00+00:00".to_string()),
            ..Default::default()
        };
        let planner = Planner::new(&config, &settings);
        let metadata = planner.compute_metadata().unwrap();
        assert_eq!(
            metadata.commit_time,
            Some(Timestamp {
                seconds: 1748581200,
                nanos: 0
            })
        );
    }

    #[test]
    fn test_parse_unix_timestamp() {
        let input = "1729100000";
        let parsed = Planner::parse_timestamp(input).unwrap();
        assert_eq!(parsed, 1729100000);
    }

    #[test]
    fn test_parse_rfc3339() {
        let input = "2023-01-01T12:00:00Z";
        let parsed = Planner::parse_timestamp(input).unwrap();
        assert_eq!(parsed, 1672574400);
    }

    #[test]
    fn test_parse_iso8601_basic() {
        let input = "2023-01-01T12:00:00+00:00"; // valid ISO8601::DEFAULT format
        let parsed = Planner::parse_timestamp(input).unwrap();
        assert_eq!(parsed, 1672574400);
    }

    #[test]
    fn test_parse_invalid_format() {
        let input = "not-a-valid-timestamp";
        let result = Planner::parse_timestamp(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata_with_all_overrides() {
        let config = QltyConfig::default();
        let settings = Settings {
            override_build_id: Some("build-123".to_string()),
            override_commit_sha: Some("sha-abc".to_string()),
            override_branch: Some("main".to_string()),
            override_pull_request_number: Some("42".to_string()),
            override_commit_time: Some("2023-01-01T12:00:00Z".to_string()),
            tag: Some("tag".to_string()),
            name: Some("test-report".to_string()),
            total_parts_count: Some(2),
            incomplete: true,
            ..Default::default()
        };
        let planner = Planner::new(&config, &settings);
        let metadata = planner.compute_metadata().unwrap();
        assert_eq!(metadata.build_id, "build-123");
        assert_eq!(metadata.commit_sha, "sha-abc");
        assert_eq!(metadata.branch, "main");
        assert_eq!(metadata.pull_request_number, "42");
        assert_eq!(metadata.tag, Some("tag".to_string()));
        assert_eq!(metadata.name, Some("test-report".to_string()));
        assert_eq!(metadata.total_parts_count, Some(2));
        assert_eq!(metadata.incomplete, true);
        assert_eq!(
            metadata.commit_time,
            Some(Timestamp {
                seconds: 1672574400,
                nanos: 0
            })
        );
    }

    #[test]
    fn test_metadata_with_only_commit_time_override() {
        let config = QltyConfig::default();
        let settings = Settings {
            override_commit_time: Some("1729100000".to_string()),
            ..Default::default()
        };
        let planner = Planner::new(&config, &settings);
        let metadata = planner.compute_metadata().unwrap();
        assert_eq!(
            metadata.commit_time,
            Some(Timestamp {
                seconds: 1729100000,
                nanos: 0
            })
        );
    }

    #[test]
    fn test_reference_type_pull_request() {
        let config = QltyConfig::default();
        let settings = Settings {
            override_commit_time: Some("1729100000".to_string()),
            override_pull_request_number: Some("123".to_string()),
            override_branch: Some("feature-branch".to_string()),
            ..Default::default()
        };
        let planner = Planner::new(&config, &settings);
        let metadata = planner.compute_metadata().unwrap();
        assert_eq!(metadata.reference_type, ReferenceType::PullRequest as i32);
    }

    #[test]
    fn test_reference_type_branch() {
        let config = QltyConfig::default();
        let settings = Settings {
            override_commit_time: Some("1729100000".to_string()),
            override_branch: Some("main".to_string()),
            ..Default::default()
        };
        let planner = Planner::new(&config, &settings);
        let metadata = planner.compute_metadata().unwrap();
        assert_eq!(metadata.reference_type, ReferenceType::Branch as i32);
    }

    #[test]
    fn test_reference_type_unspecified() {
        let config = QltyConfig::default();
        let settings = Settings {
            override_commit_time: Some("1729100000".to_string()),
            ..Default::default()
        };
        let planner = Planner::new(&config, &settings);
        let metadata = planner.compute_metadata().unwrap();
        assert_eq!(metadata.reference_type, ReferenceType::Unspecified as i32);
    }
}
