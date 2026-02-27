use anyhow::{Context as _, Result};
use git2::Repository;
use std::collections::HashSet;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub struct CommitMetadata {
    pub commit_time: git2::Time,
    pub author_time: git2::Time,
    pub committer_name: String,
    pub committer_email: String,
    pub author_name: String,
    pub author_email: String,
    pub commit_message: String,
}

pub fn retrieve_commit_metadata() -> Result<Option<CommitMetadata>> {
    if std::env::var("QLTY_COVERAGE_TESTING_WITHOUT_GIT").is_ok() {
        // If we're in testing for scenario without git, return None
        return Ok(None);
    }

    let repo = match Repository::discover(".") {
        Ok(repo) => repo,
        Err(err) => {
            if std::path::Path::new(".git").exists() {
                eprintln!(
                    "Failed to open Git repository to retrieve commit metadata: {:?}",
                    err
                );
                error!(
                    "Failed to open Git repository to retrieve commit metadata: {:?}",
                    err
                );
                return Err(err)
                    .context("Failed to open Git repository to retrieve commit metadata");
            } else {
                eprintln!("No Git repository found, skipping commit metadata retrieval");
                warn!("No Git repository found, skipping commit metadata retrieval");
                return Ok(None);
            }
        }
    };

    let head = repo.head()?;
    let oid = head.peel_to_commit()?.id();
    let commit = repo.find_commit(oid)?;

    let commit_time = commit.time();

    let committer = commit.committer();
    let committer_name = committer.name().unwrap_or("Unknown").to_string();
    let committer_email = committer.email().unwrap_or("Unknown").to_string();

    let author = commit.author();
    let author_name = author.name().unwrap_or("Unknown").to_string();
    let author_email = author.email().unwrap_or("Unknown").to_string();
    let author_time = author.when();

    let commit_message = commit.message().unwrap_or("").to_string();

    Ok(Some(CommitMetadata {
        commit_time,
        author_time,
        committer_name,
        committer_email,
        author_name,
        author_email,
        commit_message,
    }))
}

#[derive(Debug, Clone)]
pub struct GitTrackingInfo {
    pub repo_root: String,
    pub tracked_files: HashSet<String>,
}

impl GitTrackingInfo {
    pub fn is_tracked(&self, relative_path: &str) -> bool {
        let normalized = relative_path.replace('\\', "/");
        self.tracked_files.contains(&normalized)
    }
}

pub fn get_git_tracking_info() -> Option<GitTrackingInfo> {
    if std::env::var("QLTY_COVERAGE_TESTING_WITHOUT_GIT").is_ok() {
        return None;
    }

    let repo = match Repository::discover(".") {
        Ok(repo) => repo,
        Err(_) => {
            info!("No Git repository found");
            return None;
        }
    };

    let repo_root = match repo.workdir() {
        Some(path) => match path.to_str() {
            Some(s) => s.to_string(),
            None => {
                warn!("Git repository path is not valid UTF-8");
                return None;
            }
        },
        None => {
            warn!("Git repository has no working directory (bare repository)");
            return None;
        }
    };

    let index = match repo.index() {
        Ok(index) => index,
        Err(err) => {
            warn!("Failed to read Git index: {:?}", err);
            return None;
        }
    };

    let mut tracked_files = HashSet::new();
    for entry in index.iter() {
        if let Ok(path) = std::str::from_utf8(&entry.path) {
            tracked_files.insert(path.to_string());
        }
    }

    info!("Git repository found at: {}", repo_root);

    Some(GitTrackingInfo {
        repo_root,
        tracked_files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_tracked_returns_false_for_untracked() {
        let info = GitTrackingInfo {
            repo_root: "/tmp/test".to_string(),
            tracked_files: HashSet::from(["src/main.rs".to_string()]),
        };
        assert!(!info.is_tracked("src/untracked.rs"));
    }

    #[test]
    fn test_is_tracked_returns_true_for_tracked() {
        let info = GitTrackingInfo {
            repo_root: "/tmp/test".to_string(),
            tracked_files: HashSet::from(["src/main.rs".to_string()]),
        };
        assert!(info.is_tracked("src/main.rs"));
    }

    #[test]
    fn test_is_tracked_normalizes_windows_paths() {
        let info = GitTrackingInfo {
            repo_root: "/tmp/test".to_string(),
            tracked_files: HashSet::from(["src/main.rs".to_string()]),
        };
        assert!(info.is_tracked("src\\main.rs"));
    }
}
