use anyhow::{Context, Result};
use git2::Repository;

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

pub fn retrieve_commit_metadata() -> Result<CommitMetadata> {
    let repo = Repository::discover(".")
        .with_context(|| "Error opening git repository for retrieving commit metadata")?;

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

    Ok(CommitMetadata {
        commit_time,
        author_time,
        committer_name,
        committer_email,
        author_name,
        author_email,
        commit_message,
    })
}
