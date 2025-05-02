use crate::ci::{GitHub, CI};
use anyhow::{bail, Result};
use git2::Repository;
use qlty_config::Workspace;
use tracing::debug;

const COVERAGE_TOKEN_WORKSPACE_PREFIX: &str = "qltcw_";

pub fn load_auth_token(token: &Option<String>, project: Option<&str>) -> Result<String> {
    expand_token(
        match token {
            Some(token) => Ok(token.to_owned()),
            None => std::env::var("QLTY_COVERAGE_TOKEN").map_err(|_| {
                anyhow::Error::msg("QLTY_COVERAGE_TOKEN environment variable is required.")
            }),
        }?,
        project,
    )
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
    if let Some(name) = extract_repository_name(remote.url().unwrap_or_default()) {
        Ok(name)
    } else {
        bail!(
            "Could not find repository name from git remote: {:?}",
            remote.url()
        )
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
}
