use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use console::style;
use qlty_config::{version::LONG_VERSION, QltyConfig, Workspace};
use qlty_coverage::publish::Settings;
use qlty_types::tests::v1::{CoverageMetadata, ReferenceType};
use regex::Regex;
use std::path::PathBuf;

const COVERAGE_TOKEN_WORKSPACE_PREFIX: &str = "qltcw_";
const COVERAGE_TOKEN_PROJECT_PREFIX: &str = "qltcp_";
const OIDC_REGEX: &str = r"^([a-zA-Z0-9\-_]+)\.([a-zA-Z0-9\-_]+)\.([a-zA-Z0-9\-_]+)$";

pub fn load_config() -> QltyConfig {
    Workspace::new()
        .and_then(|workspace| workspace.config())
        .unwrap_or_default()
}

pub fn print_initial_messages(quiet: bool) {
    if !quiet {
        eprintln!("qlty {}", LONG_VERSION.as_str());
        eprintln!("{}", Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ"));
        eprintln!("{}", style("https://qlty.sh/d/coverage").dim());
        eprintln!();
    }
}

pub fn print_settings(settings: &Settings) {
    if settings.quiet {
        return;
    }

    eprintln!(
        "    cwd: {}",
        std::env::current_dir()
            .unwrap_or(PathBuf::from("ERROR"))
            .to_string_lossy()
    );

    if settings.dry_run {
        eprintln!("    dry-run: {}", settings.dry_run);
    }
    if let Some(format) = &settings.report_format {
        eprintln!("    format: {format}");
    }
    if let Some(name) = &settings.name {
        eprintln!("    name: {name}");
    }
    if let Some(output_dir) = &settings.output_dir {
        eprintln!("    output-dir: {}", output_dir.to_string_lossy());
    }
    if let Some(tag) = &settings.tag {
        eprintln!("    tag: {tag}");
    }
    if let Some(override_build_id) = &settings.override_build_id {
        eprintln!("    override-build-id: {override_build_id}");
    }
    if let Some(override_branch) = &settings.override_branch {
        eprintln!("    override-branch: {override_branch}");
    }
    if let Some(override_commit_sha) = &settings.override_commit_sha {
        eprintln!("    override-commit-sha: {override_commit_sha}");
    }
    if let Some(override_pr_number) = &settings.override_pull_request_number {
        eprintln!("    override-pr-number: {override_pr_number}");
    }
    if let Some(override_commit_time) = &settings.override_commit_time {
        eprintln!("    override-commit-time: {override_commit_time}");
    }
    if let Some(override_git_tag) = &settings.override_git_tag {
        eprintln!("    override-git-tag: {override_git_tag}");
    }
    if let Some(add_prefix) = &settings.add_prefix {
        eprintln!("    add-prefix: {add_prefix}");
    }
    if let Some(strip_prefix) = &settings.strip_prefix {
        eprintln!("    strip-prefix: {strip_prefix}");
    }
    if let Some(project) = &settings.project {
        eprintln!("    project: {project}");
    }

    if settings.skip_missing_files {
        eprintln!("    skip-missing-files: {}", settings.skip_missing_files);
    }

    if let Some(total_parts_count) = settings.total_parts_count {
        eprintln!("    total-parts-count: {total_parts_count}");
    }

    if settings.incomplete {
        eprintln!("    incomplete: {}", settings.incomplete);
    }

    // Print JACOCO_SOURCE_PATH if defined
    if let Ok(jacoco_source_path) = std::env::var("JACOCO_SOURCE_PATH") {
        if !jacoco_source_path.is_empty() {
            let paths: Vec<&str> = jacoco_source_path.split_whitespace().collect();
            if !paths.is_empty() {
                eprintln!("    JACOCO_SOURCE_PATH (from environment):");
                for path in paths {
                    eprintln!("      {}", path);
                }
            }
        }
    }

    eprintln!();
}

pub fn print_metadata(metadata: &CoverageMetadata, quiet: bool) {
    if quiet {
        return;
    }

    if !metadata.ci.is_empty() {
        eprintln!("    CI: {}", metadata.ci);
    }

    let reference_type = ReferenceType::try_from(metadata.reference_type)
        .map(|rt| format!("{rt:?}"))
        .unwrap_or_else(|_| "Unknown".to_string());
    eprintln!("    Reference Type: {}", reference_type);

    eprintln!("    Commit: {}", metadata.commit_sha);
    if !metadata.pull_request_number.is_empty() {
        eprintln!("    Pull Request: #{}", metadata.pull_request_number);
    }

    if !metadata.branch.is_empty() {
        eprintln!("    Branch: {}", metadata.branch);
    }

    if !metadata.build_id.is_empty() {
        eprintln!("    Build ID: {}", metadata.build_id);
    }

    if metadata.commit_time.is_some() {
        let commit_time = metadata.commit_time.unwrap();
        let date_time =
            DateTime::from_timestamp(commit_time.seconds, commit_time.nanos as u32).unwrap();
        eprintln!("    Commit Time: {}", date_time);
    }

    if let Some(git_tag) = &metadata.git_tag {
        eprintln!("    Git Tag: {}", git_tag);
    }

    eprintln!();
}

pub fn print_authentication_info(token: &str, quiet: bool) {
    if quiet {
        return;
    }

    let token_type = if token.starts_with(COVERAGE_TOKEN_WORKSPACE_PREFIX) {
        "Workspace Token"
    } else if token.starts_with(COVERAGE_TOKEN_PROJECT_PREFIX) {
        "Project Token"
    } else if let Ok(oidc_regex) = Regex::new(OIDC_REGEX) {
        if oidc_regex.is_match(token) {
            "OIDC"
        } else {
            "Unknown"
        }
    } else {
        "ERROR"
    };
    eprintln!("    Auth Method: {token_type}");
    eprintln!("    Token: {token}");
    eprintln!();
}

pub fn validate_metadata(metadata: &CoverageMetadata) -> Result<()> {
    if metadata.commit_sha.is_empty() {
        bail!(
            "Unable to determine commit SHA from the environment.\nPlease provide it using --override-commit-sha"
        )
    }

    if metadata.reference_type == ReferenceType::Unspecified as i32 {
        bail!(
            "A branch, tag, or pull request must be specified.\nPlease provide it using a supported CI provider or with one of --override-branch, --override-git-tag, or --override-pr-number"
        )
    }

    if metadata.commit_time.is_none() {
        bail!(
            "Unable to determine commit time from the environment.\nPlease provide it using --override-commit-time"
        )
    }

    Ok(())
}

pub fn validate_minimal_metadata(metadata: &CoverageMetadata) -> Result<()> {
    if metadata.commit_sha.is_empty() {
        bail!(
            "Unable to determine commit SHA from the environment.\nPlease provide it using --override-commit-sha"
        )
    }

    Ok(())
}

pub fn print_minimal_metadata(metadata: &CoverageMetadata, quiet: bool) {
    if quiet {
        return;
    }

    eprintln!("    Commit: {}", metadata.commit_sha);

    if let Some(tag) = &metadata.tag {
        eprintln!("    Tag: {}", tag);
    }

    eprintln!();
}
