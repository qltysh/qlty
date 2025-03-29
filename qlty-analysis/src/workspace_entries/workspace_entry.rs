use anyhow::Result;
use path_absolutize::Absolutize;
use serde::Serialize;
use std::hash::Hash;
use std::sync::Arc;
use std::{path::PathBuf, time::SystemTime};

use crate::utils::fs::path_to_string;

#[derive(Default, Debug, Serialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WorkspaceEntryKind {
    #[default]
    File,
    Directory,
}

#[derive(Debug, Serialize, Eq, PartialEq, PartialOrd, Hash, Clone)]
pub struct WorkspaceEntry {
    pub path: PathBuf,
    pub kind: WorkspaceEntryKind,
    pub content_modified: SystemTime,
    pub contents_size: u64,
    pub language_name: Option<String>,
}

impl WorkspaceEntry {
    pub fn path_string(&self) -> String {
        path_to_string(&self.path)
    }

    pub fn full_path(&self, base_path: &PathBuf) -> Result<PathBuf> {
        self.path
            .absolutize_from(base_path)
            .map(|p| p.into_owned())
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to get the absolute path for {}: {}",
                    self.path.display(),
                    e
                )
            })
    }
}

pub trait WorkspaceEntrySource: std::fmt::Debug + Send + Sync {
    fn entries(&self) -> Result<Arc<Vec<WorkspaceEntry>>>;
}
