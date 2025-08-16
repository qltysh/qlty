use crate::ci::CI;
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

pub struct MetadataPlanner {
    settings: Settings,
    ci: Option<Box<dyn CI>>,
}

impl std::fmt::Debug for MetadataPlanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetadataPlanner")
            .field("settings", &self.settings)
            .field("ci", &self.ci.as_ref().map(|c| c.ci_name()))
            .finish()
    }
}

impl Planner {
    pub fn new(config: &QltyConfig, settings: &Settings) -> Self {
        Self {
            config: config.clone(),
            settings: settings.clone(),
        }
    }

    pub fn compute(&self) -> Result<Plan> {
        let metadata_planner = MetadataPlanner::new(&self.settings, crate::ci::current());
        let metadata = metadata_planner.compute()?;

        Ok(Plan {
            metadata: metadata.clone(),
            report_files: self.compute_report_files()?,
            transformers: self.compute_transformers(&metadata)?,
            skip_missing_files: self.settings.skip_missing_files,
        })
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

impl MetadataPlanner {
    pub fn new(settings: &Settings, ci: Option<Box<dyn CI>>) -> Self {
        Self {
            settings: settings.clone(),
            ci,
        }
    }

    pub fn compute_minimal(&self) -> Result<CoverageMetadata> {
        let now = OffsetDateTime::now_utc();

        let mut metadata = CoverageMetadata::default();

        // Try to get commit SHA from CI first, then override
        if let Some(ref ci) = self.ci {
            let ci_metadata = ci.metadata();
            metadata.commit_sha = ci_metadata.commit_sha;
        }

        // Override with explicit value if provided
        if let Some(ref commit_sha) = self.settings.override_commit_sha {
            metadata.commit_sha = commit_sha.clone();
        }

        // Set minimal required fields
        metadata.cli_version = LONG_VERSION.to_string();
        metadata.uploaded_at = Some(Timestamp {
            seconds: now.unix_timestamp(),
            nanos: now.nanosecond() as i32,
        });
        metadata.tag = self.settings.tag.clone();

        Ok(metadata)
    }

    pub fn compute(&self) -> Result<CoverageMetadata> {
        let now = OffsetDateTime::now_utc();

        let is_merge_group = self
            .ci
            .as_ref()
            .is_some_and(|ci| ci.is_merge_group_branch());

        let mut metadata = if let Some(ref ci) = self.ci {
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

        if let Some(git_tag) = self.settings.override_git_tag.as_ref() {
            metadata.git_tag = Some(git_tag.to_string());
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

        metadata.reference_type = if is_merge_group {
            ReferenceType::MergeGroup as i32
        } else if !metadata.pull_request_number.is_empty() {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestCI {
        build_id: String,
        commit_sha: String,
        branch: String,
        pull_request_number: String,
        git_tag: Option<String>,
    }

    impl TestCI {
        fn new() -> Self {
            Self {
                build_id: "test-build-456".to_string(),
                commit_sha: "test-sha-789".to_string(),
                branch: "test-branch".to_string(),
                pull_request_number: "99".to_string(),
                git_tag: Some("v0.1.0".to_string()),
            }
        }
    }

    impl CI for TestCI {
        fn detect(&self) -> bool {
            true
        }

        fn ci_name(&self) -> String {
            "TestCI".to_string()
        }

        fn ci_url(&self) -> String {
            "https://test-ci.example.com".to_string()
        }

        fn repository_name(&self) -> String {
            "test-org/test-repo".to_string()
        }

        fn repository_url(&self) -> String {
            "https://github.com/test-org/test-repo".to_string()
        }

        fn branch(&self) -> String {
            self.branch.clone()
        }

        fn pull_number(&self) -> String {
            self.pull_request_number.clone()
        }

        fn pull_url(&self) -> String {
            format!(
                "https://github.com/test-org/test-repo/pull/{}",
                self.pull_request_number
            )
        }

        fn commit_sha(&self) -> String {
            self.commit_sha.clone()
        }

        fn git_tag(&self) -> Option<String> {
            self.git_tag.clone()
        }

        fn workflow(&self) -> String {
            "test-workflow".to_string()
        }

        fn job(&self) -> String {
            "test-job".to_string()
        }

        fn build_id(&self) -> String {
            self.build_id.clone()
        }

        fn build_url(&self) -> String {
            format!("https://test-ci.example.com/builds/{}", self.build_id)
        }
    }

    #[test]
    fn planner_override_commit_time_tests() {
        let settings = Settings {
            override_commit_time: Some("2025-05-30T05:00:00+00:00".to_string()),
            ..Default::default()
        };
        let metadata_planner = MetadataPlanner::new(&settings, None);
        let metadata = metadata_planner.compute().unwrap();
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
        let parsed = MetadataPlanner::parse_timestamp(input).unwrap();
        assert_eq!(parsed, 1729100000);
    }

    #[test]
    fn test_parse_rfc3339() {
        let input = "2023-01-01T12:00:00Z";
        let parsed = MetadataPlanner::parse_timestamp(input).unwrap();
        assert_eq!(parsed, 1672574400);
    }

    #[test]
    fn test_parse_iso8601_basic() {
        let input = "2023-01-01T12:00:00+00:00"; // valid ISO8601::DEFAULT format
        let parsed = MetadataPlanner::parse_timestamp(input).unwrap();
        assert_eq!(parsed, 1672574400);
    }

    #[test]
    fn test_parse_invalid_format() {
        let input = "not-a-valid-timestamp";
        let result = MetadataPlanner::parse_timestamp(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata_with_ci_without_overrides() {
        let settings = Settings {
            tag: Some("tag".to_string()),
            name: Some("test-report".to_string()),
            total_parts_count: Some(1),
            incomplete: false,
            ..Default::default()
        };
        let test_ci = Box::new(TestCI::new());
        let metadata_planner = MetadataPlanner::new(&settings, Some(test_ci));
        let metadata = metadata_planner.compute().unwrap();

        // Verify that CI values are used when no overrides are present
        assert_eq!(metadata.ci, "TestCI");
        assert_eq!(metadata.build_id, "test-build-456"); // From CI
        assert_eq!(metadata.commit_sha, "test-sha-789"); // From CI
        assert_eq!(metadata.branch, "test-branch"); // From CI
        assert_eq!(metadata.pull_request_number, "99"); // From CI
        assert_eq!(metadata.git_tag, Some("v0.1.0".to_string())); // From CI

        // Verify settings-based fields
        assert_eq!(metadata.tag, Some("tag".to_string()));
        assert_eq!(metadata.name, Some("test-report".to_string()));
        assert_eq!(metadata.total_parts_count, Some(1));
        assert_eq!(metadata.incomplete, false);
    }

    #[test]
    fn test_metadata_with_build_id_override() {
        let settings = Settings {
            override_build_id: Some("build-123".to_string()),
            ..Default::default()
        };
        let test_ci = Box::new(TestCI::new());
        let metadata_planner = MetadataPlanner::new(&settings, Some(test_ci));
        let metadata = metadata_planner.compute().unwrap();

        assert_eq!(metadata.build_id, "build-123"); // Override wins over CI's "test-build-456"
    }

    #[test]
    fn test_metadata_with_commit_sha_override() {
        let settings = Settings {
            override_commit_sha: Some("sha-abc".to_string()),
            ..Default::default()
        };
        let test_ci = Box::new(TestCI::new());
        let metadata_planner = MetadataPlanner::new(&settings, Some(test_ci));
        let metadata = metadata_planner.compute().unwrap();

        assert_eq!(metadata.commit_sha, "sha-abc"); // Override wins over CI's "test-sha-789"
    }

    #[test]
    fn test_metadata_with_branch_override() {
        let settings = Settings {
            override_branch: Some("main".to_string()),
            ..Default::default()
        };
        let test_ci = Box::new(TestCI::new());
        let metadata_planner = MetadataPlanner::new(&settings, Some(test_ci));
        let metadata = metadata_planner.compute().unwrap();

        assert_eq!(metadata.branch, "main"); // Override wins over CI's "test-branch"
    }

    #[test]
    fn test_metadata_with_pull_request_number_override() {
        let settings = Settings {
            override_pull_request_number: Some("42".to_string()),
            ..Default::default()
        };
        let test_ci = Box::new(TestCI::new());
        let metadata_planner = MetadataPlanner::new(&settings, Some(test_ci));
        let metadata = metadata_planner.compute().unwrap();

        assert_eq!(metadata.pull_request_number, "42"); // Override wins over CI's "99"
    }

    #[test]
    fn test_metadata_with_commit_time_override() {
        let settings = Settings {
            override_commit_time: Some("1729100000".to_string()),
            ..Default::default()
        };
        let metadata_planner = MetadataPlanner::new(&settings, None);
        let metadata = metadata_planner.compute().unwrap();
        assert_eq!(
            metadata.commit_time,
            Some(Timestamp {
                seconds: 1729100000,
                nanos: 0
            })
        );
    }

    #[test]
    fn test_metadata_with_git_tag_override() {
        let settings = Settings {
            override_git_tag: Some("v2.0.0".to_string()),
            ..Default::default()
        };
        let test_ci = Box::new(TestCI::new());
        let metadata_planner = MetadataPlanner::new(&settings, Some(test_ci));
        let metadata = metadata_planner.compute().unwrap();

        assert_eq!(metadata.git_tag, Some("v2.0.0".to_string())); // Override wins over CI's "v0.1.0"
    }

    #[test]
    fn test_reference_type_pull_request() {
        let settings = Settings {
            override_commit_time: Some("1729100000".to_string()),
            override_pull_request_number: Some("123".to_string()),
            override_branch: Some("feature-branch".to_string()),
            ..Default::default()
        };
        let metadata_planner = MetadataPlanner::new(&settings, None);
        let metadata = metadata_planner.compute().unwrap();
        assert_eq!(metadata.reference_type, ReferenceType::PullRequest as i32);
    }

    #[test]
    fn test_reference_type_branch() {
        let settings = Settings {
            override_commit_time: Some("1729100000".to_string()),
            override_branch: Some("main".to_string()),
            ..Default::default()
        };
        let metadata_planner = MetadataPlanner::new(&settings, None);
        let metadata = metadata_planner.compute().unwrap();
        assert_eq!(metadata.reference_type, ReferenceType::Branch as i32);
    }

    #[test]
    fn test_reference_type_tag() {
        let settings = Settings {
            override_git_tag: Some("1729100000".to_string()),
            override_branch: Some("main".to_string()),
            ..Default::default()
        };
        let metadata_planner = MetadataPlanner::new(&settings, None);
        let metadata = metadata_planner.compute().unwrap();
        assert_eq!(metadata.reference_type, ReferenceType::Tag as i32);
    }

    #[test]
    fn test_reference_type_unspecified() {
        let settings = Settings {
            override_commit_time: Some("1729100000".to_string()),
            ..Default::default()
        };
        let metadata_planner = MetadataPlanner::new(&settings, None);
        let metadata = metadata_planner.compute().unwrap();
        assert_eq!(metadata.reference_type, ReferenceType::Unspecified as i32);
    }

    #[test]
    fn test_reference_type_merge_group() {
        #[derive(Debug)]
        struct MergeGroupTestCI;

        impl CI for MergeGroupTestCI {
            fn detect(&self) -> bool {
                true
            }

            fn ci_name(&self) -> String {
                "TestCI".to_string()
            }

            fn ci_url(&self) -> String {
                "https://test-ci.example.com".to_string()
            }

            fn repository_name(&self) -> String {
                "test-org/test-repo".to_string()
            }

            fn repository_url(&self) -> String {
                "https://github.com/test-org/test-repo".to_string()
            }

            fn branch(&self) -> String {
                "gh-readonly-queue/main/pr-123".to_string()
            }

            fn pull_number(&self) -> String {
                String::new()
            }

            fn pull_url(&self) -> String {
                String::new()
            }

            fn commit_sha(&self) -> String {
                "test-sha-789".to_string()
            }

            fn workflow(&self) -> String {
                "test-workflow".to_string()
            }

            fn job(&self) -> String {
                "test-job".to_string()
            }

            fn build_id(&self) -> String {
                "test-build-456".to_string()
            }

            fn build_url(&self) -> String {
                "https://test-ci.example.com/builds/test-build-456".to_string()
            }

            fn is_merge_group_branch(&self) -> bool {
                true
            }
        }

        let settings = Settings {
            override_commit_time: Some("1729100000".to_string()),
            override_branch: Some("gh-readonly-queue/main/pr-123".to_string()),
            ..Default::default()
        };

        let test_ci = Box::new(MergeGroupTestCI);
        let metadata_planner = MetadataPlanner::new(&settings, Some(test_ci));
        let metadata = metadata_planner.compute().unwrap();

        assert_eq!(metadata.reference_type, ReferenceType::MergeGroup as i32);
    }
}
