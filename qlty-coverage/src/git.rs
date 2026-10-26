use anyhow::{Context as _, Result};
use git2::Repository;
use tracing::{error, warn};

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
