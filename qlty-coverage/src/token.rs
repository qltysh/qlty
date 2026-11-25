use std::ffi::OsString;

use crate::ci::{GitHub, CI};
use anyhow::{bail, Result};
use git2::Repository;
use qlty_config::Workspace;
use tracing::debug;

const COVERAGE_TOKEN_WORKSPACE_PREFIX: &str = "qltcw_";
const REDACTED: &str = "[REDACTED]";

pub fn load_auth_token(token: &Option<String>, project: Option<&str>) -> Result<String> {
    expand_token(
        match token {
            Some(token) => Ok(token.to_owned()),
            None => std::env::var("QLTY_COVERAGE_TOKEN").map_err(|_| {
                anyhow::Error::msg("QLTY_COVERAGE_TOKEN environment variable is required.")
            }),
        }?
        .trim()
        .to_string(),
        project,
    )
}

/// Joins the command-line arguments with the value of `--token` or `-t` replaced.
/// Arguments that aren't valid Unicode are converted lossily.
pub fn redact_command(args: impl IntoIterator<Item = impl Into<OsString>>) -> String {
    let mut value_is_next = false;
    let mut redacted = vec![];

    for arg in args {
        let arg = arg.into().to_string_lossy().into_owned();
        redacted.push(if std::mem::take(&mut value_is_next) {
            REDACTED.to_string()
        } else if arg == "--token" {
            value_is_next = true;
            arg
        } else if arg.starts_with("--token=") {
            format!("--token={REDACTED}")
        } else if let Some(flag) = short_token_flag(&arg) {
            if flag.len() == arg.len() {
                value_is_next = true;
                arg
            } else {
                format!("{flag}{REDACTED}")
            }
        } else {
            arg
        });
    }

    redacted.join(" ")
}

// Clap reads everything after `t` in a group of short flags such as `-qtVALUE` as the value,
// or the next argument when `t` ends the group.
fn short_token_flag(arg: &str) -> Option<&str> {
    let flags = arg
        .strip_prefix('-')
        .filter(|flags| !flags.starts_with('-'))?;
    flags.find('t').map(|index| &arg[..index + 2])
}

fn expand_token(token: String, project: Option<&str>) -> Result<String> {
    if token.starts_with(COVERAGE_TOKEN_WORKSPACE_PREFIX) {
        if token.contains('/') {
            return Ok(token);
        }
        let project = if let Some(project) = project {
            project.to_string()
        } else if let Some(repository) = find_repository_name_from_env() {
            repository
        } else {
            match find_repository_name_from_repository() {
                Ok(repository) => repository,
                Err(err) => {
                    debug!("Find repository name: {}", err);
                    bail!(
                        "Could not infer project name from environment, please provide it using --project"
                    )
                }
            }
        };
        Ok(format!("{token}/{project}"))
    } else {
        Ok(token)
    }
}

fn find_repository_name_from_env() -> Option<String> {
    let repository = GitHub::default().repository_name();
    if repository.is_empty() {
        None
    } else {
        extract_repository_name(&repository)
    }
}

fn find_repository_name_from_repository() -> Result<String> {
    let root = Workspace::assert_within_git_directory()?;
    let repo = Repository::open(root)?;
    let remote = repo.find_remote("origin")?;
    repository_name_from_remote_url(remote.url().unwrap_or_default())
}

// The URL is left out of the error because an https remote can carry a credential.
fn repository_name_from_remote_url(url: &str) -> Result<String> {
    match extract_repository_name(url) {
        Some(name) => Ok(name),
        None => bail!("Could not find repository name from the origin remote URL"),
    }
}

fn extract_repository_name(value: &str) -> Option<String> {
    value
        .split('/')
        .next_back()
        .map(|s| s.strip_suffix(".git").unwrap_or(s).to_string())
        .filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_repository_name() {
        assert_eq!(extract_repository_name(""), None);
        assert_eq!(extract_repository_name("a/"), None);
        assert_eq!(
            extract_repository_name("git@example.org:a/b"),
            Some("b".into())
        );
        assert_eq!(
            extract_repository_name("ssh://x@example.org:a/b"),
            Some("b".into())
        );
        assert_eq!(
            extract_repository_name("https://x:y@example.org/a/b"),
            Some("b".into())
        );
    }

    #[test]
    fn test_expand_token_project() -> Result<()> {
        let token = expand_token("qltcp_123".to_string(), None)?;
        assert_eq!(token, "qltcp_123");
        Ok(())
    }

    #[test]
    fn test_expand_token_workspace_with_project() -> Result<()> {
        let token = expand_token("qltcw_123".to_string(), Some("test"))?;
        assert_eq!(token, "qltcw_123/test");
        Ok(())
    }

    #[test]
    fn test_expand_token_workspace_with_env() -> Result<()> {
        let token = expand_token("qltcw_123".to_string(), None)?;
        assert!(token.starts_with("qltcw_123/"));

        std::env::set_var("GITHUB_REPOSITORY", "");
        let token = expand_token("qltcw_123".to_string(), None)?;
        assert!(token.starts_with("qltcw_123/"));

        std::env::set_var("GITHUB_REPOSITORY", "a/b.git");
        let token = expand_token("qltcw_123".to_string(), None)?;
        assert_eq!(token, "qltcw_123/b");

        std::env::set_var("GITHUB_REPOSITORY", "b/c");
        let token = expand_token("qltcw_123".to_string(), None)?;
        assert_eq!(token, "qltcw_123/c");

        Ok(())
    }

    #[test]
    fn test_expand_token_already_expanded() -> Result<()> {
        let token = expand_token("qltcw_123/abc".to_string(), Some("test"))?;
        assert_eq!(token, "qltcw_123/abc");
        Ok(())
    }

    #[test]
    fn test_load_auth_token_trims_line_endings() {
        let token = load_auth_token(&Some("qltcp_123\r\n".to_string()), None).unwrap();
        assert_eq!(token, "qltcp_123");
    }

    #[test]
    fn test_remote_url_error_omits_credential() {
        assert!(!repository_name_from_remote_url(
            "https://gitlab-ci-token:SECRET@gitlab.example.com/a/b/"
        )
        .unwrap_err()
        .to_string()
        .contains("SECRET"));
    }

    #[cfg(unix)]
    #[test]
    fn test_redact_command_non_unicode_argument() {
        use std::os::unix::ffi::OsStringExt;

        assert_eq!(
            redact_command([
                OsString::from("qlty"),
                OsString::from_vec(b"caf\xe9.rs".to_vec())
            ]),
            "qlty caf\u{FFFD}.rs"
        );
    }

    fn args(command: &str) -> Vec<String> {
        command.split(' ').map(String::from).collect()
    }

    #[test]
    fn test_redact_command_long_flag() {
        assert_eq!(
            redact_command(args("qlty coverage publish --token qltcp_SECRET lcov.info")),
            "qlty coverage publish --token [REDACTED] lcov.info"
        );
    }

    #[test]
    fn test_redact_command_long_flag_with_equals() {
        assert_eq!(
            redact_command(args("qlty coverage publish --token=qltcp_SECRET lcov.info")),
            "qlty coverage publish --token=[REDACTED] lcov.info"
        );
    }

    #[test]
    fn test_redact_command_short_flag() {
        assert_eq!(
            redact_command(args("qlty coverage publish -t qltcp_SECRET lcov.info")),
            "qlty coverage publish -t [REDACTED] lcov.info"
        );
    }

    #[test]
    fn test_redact_command_short_flag_with_attached_value() {
        assert_eq!(
            redact_command(args("qlty coverage publish -tqltcp_SECRET lcov.info")),
            "qlty coverage publish -t[REDACTED] lcov.info"
        );
    }

    #[test]
    fn test_redact_command_short_flag_with_equals() {
        assert_eq!(
            redact_command(args("qlty coverage publish -t=qltcp_SECRET lcov.info")),
            "qlty coverage publish -t[REDACTED] lcov.info"
        );
    }

    #[test]
    fn test_redact_command_grouped_short_flags() {
        assert_eq!(
            redact_command(args("qlty coverage publish -qt qltcp_SECRET lcov.info")),
            "qlty coverage publish -qt [REDACTED] lcov.info"
        );
    }

    #[test]
    fn test_redact_command_grouped_short_flags_with_attached_value() {
        assert_eq!(
            redact_command(args("qlty coverage publish -qtqltcp_SECRET lcov.info")),
            "qlty coverage publish -qt[REDACTED] lcov.info"
        );
    }

    #[test]
    fn test_redact_command_without_token() {
        assert_eq!(
            redact_command(args(
                "qlty coverage publish -q --tag tests --override-git-tag v1 lcov.info"
            )),
            "qlty coverage publish -q --tag tests --override-git-tag v1 lcov.info"
        );
    }
}
