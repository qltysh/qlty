use crate::git::DiffMode;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TargetMode {
    #[default]
    All,
    Sample(usize),
    Paths(usize),
    UpstreamDiff(String),
    HeadDiff,
    Index,
    IndexFile(PathBuf),
}

impl TargetMode {
    pub fn diff_mode(&self) -> DiffMode {
        match self {
            TargetMode::Index => DiffMode::HeadToIndex,
            TargetMode::IndexFile(index_path) => DiffMode::HeadToIndexFile(index_path.clone()),
            TargetMode::HeadDiff => DiffMode::HeadToWorkdir,
            TargetMode::UpstreamDiff(upstream) => DiffMode::UpstreamToWorkdir(upstream.to_string()),
            _ => panic!("diff_mode() called on {:?}", self),
        }
    }
}
