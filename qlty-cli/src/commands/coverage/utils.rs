use console::style;
use qlty_config::{version::LONG_VERSION, QltyConfig, Workspace};
use qlty_coverage::publish::{Plan, Settings};
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

    eprintln!();
}

pub fn print_metadata(plan: &Plan, quiet: bool) {
    if quiet {
        return;
    }

    if !plan.metadata.ci.is_empty() {
        eprintln!("    CI: {}", plan.metadata.ci);
    }

    eprintln!("    Commit: {}", plan.metadata.commit_sha);
    if !plan.metadata.pull_request_number.is_empty() {
        eprintln!("    Pull Request: #{}", plan.metadata.pull_request_number);
    }

    if !plan.metadata.branch.is_empty() {
        eprintln!("    Branch: {}", plan.metadata.branch);
    }

    if !plan.metadata.build_id.is_empty() {
        eprintln!("    Build ID: {}", plan.metadata.build_id);
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
