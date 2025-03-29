use crate::{walker::WalkerBuilder, WorkspaceEntry, WorkspaceEntryKind, WorkspaceEntrySource};
use anyhow::Result;
use core::fmt;
use ignore::WalkState;
use path_absolutize::Absolutize;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

// qlty-ignore(semgrep/derive-debug): manual Debug impl below
pub struct ArgsSource {
    pub root: PathBuf,
    pub paths: Vec<PathBuf>,
    pub entries: Result<Arc<Vec<WorkspaceEntry>>>,
}

impl fmt::Debug for ArgsSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArgsSource[{:?}, {:?}]", self.root, self.paths)
    }
}

impl ArgsSource {
    pub fn new(root: PathBuf, paths: Vec<PathBuf>) -> Self {
        Self {
            entries: Self::build(&root, &paths),
            root,
            paths,
        }
    }

    fn build(root: &PathBuf, paths: &[PathBuf]) -> Result<Arc<Vec<WorkspaceEntry>>> {
        let workspace_entries = Arc::new(Mutex::new(vec![]));

        WalkerBuilder::new().build(paths).run(|| {
            let entries = workspace_entries.clone();
            Box::new(move |entry| {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => return WalkState::Continue,
                };
                let path = entry.path();

                let workspace_entry_kind = if path.is_dir() {
                    WorkspaceEntryKind::Directory
                } else {
                    WorkspaceEntryKind::File
                };

                let clean_path = match path.absolutize() {
                    Ok(abs_path) => match abs_path.strip_prefix(root) {
                        Ok(rel_path) => rel_path.to_path_buf(),
                        Err(_) => path.to_path_buf(),
                    },
                    Err(_) => path.to_path_buf(),
                };

                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => return WalkState::Continue,
                };

                let content_modified = metadata
                    .modified()
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

                if let Ok(mut entries_guard) = entries.lock() {
                    entries_guard.push(WorkspaceEntry {
                        path: clean_path,
                        content_modified,
                        contents_size: metadata.len(),
                        kind: workspace_entry_kind,
                        language_name: None,
                    });
                }

                WalkState::Continue
            })
        });

        // Use a separate scope for the lock to ensure it's released before return
        let entries_vec = {
            let guard = workspace_entries
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock workspace entries: {}", e))?;
            guard.to_vec()
        };

        Ok(Arc::new(entries_vec))
    }
}

impl WorkspaceEntrySource for ArgsSource {
    fn entries(&self) -> Result<Arc<Vec<WorkspaceEntry>>> {
        match &self.entries {
            Ok(entries) => Ok(entries.clone()),
            Err(e) => Err(anyhow::anyhow!("Failed to get entries: {}", e)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use itertools::Itertools;
    use qlty_test_utilities::git::build_sample_project;

    #[test]
    fn test_args_source_next() {
        let root = build_sample_project();
        let args = vec![
            root.path().to_path_buf().join("lib/tasks/ops"),
            root.path().to_path_buf().join("greetings.rb"),
        ];
        let source = ArgsSource::new(root.path().to_path_buf(), args);

        let mut paths = vec![];

        for workspace_entry in source.entries().unwrap().iter() {
            let workspace_entry = workspace_entry.clone();
            paths.push((workspace_entry.path, workspace_entry.kind));
        }

        let expected_paths = build_expected_workspace_entries(vec![
            ("lib/tasks/ops", WorkspaceEntryKind::Directory),
            ("lib/tasks/ops/deploy.rb", WorkspaceEntryKind::File),
            ("lib/tasks/ops/setup.rb", WorkspaceEntryKind::File),
            ("greetings.rb", WorkspaceEntryKind::File),
        ]);

        assert_eq!(
            paths
                .iter()
                .cloned()
                .sorted()
                .collect::<Vec<(PathBuf, WorkspaceEntryKind)>>(),
            expected_paths
        );
    }

    #[test]
    fn test_args_source_includes_hidden_files() {
        let root = build_sample_project();
        std::fs::write(
            root.path().join("lib/tasks/ops/.hidden"),
            "This is a hidden file.",
        )
        .unwrap();
        let args = vec![root.path().to_path_buf().join("lib/tasks/ops")];
        let source = ArgsSource::new(root.path().to_path_buf(), args);

        let mut paths = vec![];

        for workspace_entry in source.entries().unwrap().iter() {
            paths.push(workspace_entry.clone().path);
        }

        assert!(
            paths.contains(&PathBuf::from("lib/tasks/ops/.hidden")),
            "Expected .hidden file to be included in the paths, but it wasn't."
        );
    }

    fn build_expected_workspace_entries(
        workspace_entries: Vec<(&str, WorkspaceEntryKind)>,
    ) -> Vec<(PathBuf, WorkspaceEntryKind)> {
        workspace_entries
            .into_iter()
            .map(|(s, tt)| (PathBuf::from(s), tt))
            .sorted()
            .collect()
    }
}
