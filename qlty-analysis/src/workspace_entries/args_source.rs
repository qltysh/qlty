use crate::{walker::WalkerBuilder, WorkspaceEntry, WorkspaceEntryKind, WorkspaceEntrySource};
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
    pub entries: Arc<Vec<WorkspaceEntry>>,
}

impl fmt::Debug for ArgsSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArgsSource[{:?}, {:?}]", self.root, self.paths)
    }
}

impl ArgsSource {
    pub fn new(root: PathBuf, paths: Vec<PathBuf>) -> Self {
        Self {
            entries: Arc::new(Self::build(&root, &paths)),
            root,
            paths,
        }
    }

    fn build(root: &PathBuf, paths: &Vec<PathBuf>) -> Vec<WorkspaceEntry> {
        let workspace_entries = Arc::new(Mutex::new(vec![]));

        WalkerBuilder::new().build(paths).run(|| {
            let entries = workspace_entries.clone();
            Box::new(move |entry| {
                let entry = entry.unwrap();
                let path = entry.path();

                let workspace_entry_kind = if path.is_dir() {
                    WorkspaceEntryKind::Directory
                } else {
                    WorkspaceEntryKind::File
                };

                let clean_path = path
                    .absolutize()
                    .ok()
                    .unwrap()
                    .strip_prefix(root)
                    .ok()
                    .unwrap_or(path)
                    .to_path_buf();
                let metadata = entry.metadata().ok().unwrap();

                entries.lock().unwrap().push(WorkspaceEntry {
                    path: clean_path,
                    content_modified: metadata.modified().ok().unwrap(),
                    contents_size: metadata.len(),
                    kind: workspace_entry_kind,
                    language_name: None,
                });

                WalkState::Continue
            })
        });

        let guard = workspace_entries.lock().unwrap();
        guard.to_vec()
    }
}

impl WorkspaceEntrySource for ArgsSource {
    fn entries(&self) -> Arc<Vec<WorkspaceEntry>> {
        self.entries.clone()
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

        for workspace_entry in source.entries().iter() {
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

        for workspace_entry in source.entries().iter() {
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
