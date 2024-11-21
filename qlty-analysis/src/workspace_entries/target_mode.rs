use crate::git::DiffMode;

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
pub enum TargetMode {
    #[default]
    All,
    Sample(usize),
    Paths(usize),
    UpstreamDiff(String),
    HeadDiff,
}



impl TargetMode {
    pub fn diff_mode(&self) -> DiffMode {
        match self {
            TargetMode::UpstreamDiff(upstream) => DiffMode::UpstreamToWorkdir(upstream.to_string()),
            TargetMode::HeadDiff => DiffMode::HeadToWorkdir,
            _ => panic!("diff_mode() called on {:?}", self),
        }
    }
}
