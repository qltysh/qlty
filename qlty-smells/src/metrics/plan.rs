use qlty_analysis::code::{File, NodeFilter, NodeFilterBuilder};
use qlty_analysis::workspace_entries::TargetMode;
use std::{collections::HashMap, sync::Arc};

use super::MetricsMode;

#[derive(Debug, Clone)]
pub struct Plan {
    pub mode: MetricsMode,
    pub target_mode: TargetMode,
    pub source_files: Vec<Arc<File>>,
    pub node_filter_builders: HashMap<String, NodeFilterBuilder>,
}

impl Plan {
    pub fn node_filter_for(&self, source_file: &File, tree: &tree_sitter::Tree) -> NodeFilter {
        if let Some(builder) = self.node_filter_builders.get(&source_file.language_name) {
            builder.build(source_file, tree)
        } else {
            NodeFilter::empty()
        }
    }

    pub fn description(&self) -> String {
        match self.target_mode {
            TargetMode::All => format!("{} over all targets", self.mode),
            TargetMode::Sample(ref samples) => format!("{} over {} samples", self.mode, samples),
            TargetMode::Paths(ref paths) => format!("{} over {} paths", self.mode, paths),
            TargetMode::UpstreamDiff(ref upstream) => format!("{} vs. {}", self.mode, upstream),
            TargetMode::HeadDiff => format!("{} vs. HEAD", self.mode),
            _ => "".to_string(),
        }
    }
}
