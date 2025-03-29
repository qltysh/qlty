use super::workspace_entry::{WorkspaceEntry, WorkspaceEntryKind};
use crate::WorkspaceEntrySource;
use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};
use tracing::debug;

#[derive(Clone)]
pub struct DiffSource {
    entries: Arc<Vec<WorkspaceEntry>>,
}

impl DiffSource {
    pub fn new(changed_files: Vec<PathBuf>, path: &Path) -> Self {
        Self {
            entries: Arc::new(Self::build(&changed_files, path)),
        }
    }

    fn build(changed_files: &Vec<PathBuf>, path: &Path) -> Vec<WorkspaceEntry> {
        let mut entries = vec![];
        let full_path = path.to_path_buf();

        if !full_path.exists() {
            debug!("Changed path does not exist: {}", full_path.display());
            return entries;
        }

        for path in changed_files {
            if let Ok(metadata) = fs::metadata(full_path.join(path)) {
                let content_modified = if let Ok(modified) = metadata.modified() {
                    modified
                } else {
                    SystemTime::UNIX_EPOCH
                };
                entries.push(WorkspaceEntry {
                    path: path.to_owned(),
                    content_modified,
                    contents_size: metadata.len(),
                    kind: WorkspaceEntryKind::File,
                    language_name: None,
                });
            } else {
                debug!("Failed to get metadata from {}", path.display());
            }
        }

        entries
    }
}

impl WorkspaceEntrySource for DiffSource {
    fn entries(&self) -> Result<Arc<Vec<WorkspaceEntry>>> {
        Ok(self.entries.clone())
    }
}

impl std::fmt::Debug for DiffSource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DiffSource[{} entries]", self.entries.len())
    }
}
