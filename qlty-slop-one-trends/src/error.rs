use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Not a Git repository: {0}")]
    NotARepository(PathBuf),
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("Git command failed: {0}")]
    GitCommand(String),
    #[error("Could not read historical git attributes: {0}")]
    GitAttributes(String),
    #[error("Invalid run configuration: {0}")]
    Configuration(String),
    #[error("The run is already frozen with different settings: {0}")]
    Frozen(String),
    #[error("At least two snapshots are needed for a trend report")]
    TooFewSnapshots,
    #[error("Unknown timezone: {0}")]
    Timezone(String),
    #[error("Score error: {0}")]
    Score(#[from] qlty_slop_one::Error),
    #[error("{0} measurements still pending; no partial report exported")]
    Pending(usize),
    #[error("Report data mismatch: {0}")]
    Mismatch(String),
    #[error("Explanation does not reconcile: {0}")]
    Unreconciled(String),
    #[error("Render error: {0}")]
    Render(String),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
