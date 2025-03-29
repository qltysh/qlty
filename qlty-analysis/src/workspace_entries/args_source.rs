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
        // Using a channel to communicate potential errors from the walker callback
        let (error_sender, error_receiver) = std::sync::mpsc::channel();
        let entries = Arc::new(Mutex::new(vec![]));

        let walker = WalkerBuilder::new().build(paths);
        let should_quit = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        walker.run(|| {
            let entries_clone = entries.clone();
            let error_sender_clone = error_sender.clone();
            let should_quit_clone = should_quit.clone();

            Box::new(move |entry| {
                // Check if we should quit due to a previous error
                if should_quit_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    return WalkState::Quit;
                }

                // Process the entry, handling any errors
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        let err = anyhow::anyhow!("Failed to access entry: {}", e);
                        let _ = error_sender_clone.send(err);
                        should_quit_clone.store(true, std::sync::atomic::Ordering::Relaxed);
                        return WalkState::Quit;
                    }
                };

                let path = entry.path();

                let workspace_entry_kind = if path.is_dir() {
                    WorkspaceEntryKind::Directory
                } else {
                    WorkspaceEntryKind::File
                };

                // Handle absolutize errors
                let abs_path = match path.absolutize() {
                    Ok(p) => p,
                    Err(e) => {
                        let err =
                            anyhow::anyhow!("Failed to absolutize path {}: {}", path.display(), e);
                        let _ = error_sender_clone.send(err);
                        should_quit_clone.store(true, std::sync::atomic::Ordering::Relaxed);
                        return WalkState::Quit;
                    }
                };

                // Handle strip_prefix errors
                let clean_path = match abs_path.strip_prefix(root) {
                    Ok(rel_path) => rel_path.to_path_buf(),
                    Err(e) => {
                        let err = anyhow::anyhow!(
                            "Failed to strip prefix from path {}: {}",
                            abs_path.display(),
                            e
                        );
                        let _ = error_sender_clone.send(err);
                        should_quit_clone.store(true, std::sync::atomic::Ordering::Relaxed);
                        return WalkState::Quit;
                    }
                };

                // Handle metadata errors
                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(e) => {
                        let err = anyhow::anyhow!(
                            "Failed to get metadata for path {}: {}",
                            path.display(),
                            e
                        );
                        let _ = error_sender_clone.send(err);
                        should_quit_clone.store(true, std::sync::atomic::Ordering::Relaxed);
                        return WalkState::Quit;
                    }
                };

                // Handle modified time errors
                let content_modified = match metadata.modified() {
                    Ok(m) => m,
                    Err(e) => {
                        let err = anyhow::anyhow!(
                            "Failed to get modification time for path {}: {}",
                            path.display(),
                            e
                        );
                        let _ = error_sender_clone.send(err);
                        should_quit_clone.store(true, std::sync::atomic::Ordering::Relaxed);
                        return WalkState::Quit;
                    }
                };

                // Handle lock errors
                match entries_clone.lock() {
                    Ok(mut entries_guard) => {
                        entries_guard.push(WorkspaceEntry {
                            path: clean_path,
                            content_modified,
                            contents_size: metadata.len(),
                            kind: workspace_entry_kind,
                            language_name: None,
                        });
                    }
                    Err(e) => {
                        let err = anyhow::anyhow!(
                            "Failed to lock entries when processing path {}: {}",
                            path.display(),
                            e
                        );
                        let _ = error_sender_clone.send(err);
                        should_quit_clone.store(true, std::sync::atomic::Ordering::Relaxed);
                        return WalkState::Quit;
                    }
                }

                WalkState::Continue
            })
        });

        // Check if there was an error during the walk
        drop(error_sender); // Drop the sender to close the channel
        if let Ok(err) = error_receiver.try_recv() {
            return Err(err);
        }

        // Get the final entries
        let entries_vec = {
            let guard = entries.lock().map_err(|e| {
                anyhow::anyhow!(
                    "Failed to lock workspace entries at final collection: {}",
                    e
                )
            })?;
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
