use super::workspace_entry::{WorkspaceEntry, WorkspaceEntrySource};
use crate::ArgsSource;
use anyhow::Result;
use core::fmt;
use std::{path::PathBuf, sync::Arc};

// qlty-ignore(semgrep/derive-debug): manual Debug impl below
pub struct AllSource {
    iter: ArgsSource,
}

impl fmt::Debug for AllSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AllSource[{:?}]", self.iter.root)
    }
}

impl Default for AllSource {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

impl AllSource {
    pub fn new(root: PathBuf) -> Self {
        Self {
            iter: ArgsSource::new(root.clone(), vec![root]),
        }
    }
}

impl WorkspaceEntrySource for AllSource {
    fn entries(&self) -> Result<Arc<Vec<WorkspaceEntry>>> {
        self.iter.entries()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::WorkspaceEntryKind;
    use itertools::Itertools;
    use qlty_test_utilities::git::build_sample_project;

    #[test]
    fn test_all_source_next() {
        let root = build_sample_project();
        let source = AllSource::new(root.path().to_path_buf());
        let mut paths = vec![];

        for workspace_entry in source.entries().unwrap().iter() {
            let workspace_entry = workspace_entry.clone();
            paths.push((workspace_entry.path, workspace_entry.kind));
        }

        let expected_paths = build_expected_workspace_entries(vec![
            ("", WorkspaceEntryKind::Directory),
            (".gitignore", WorkspaceEntryKind::File),
            ("lib", WorkspaceEntryKind::Directory),
            ("lib/hello.rb", WorkspaceEntryKind::File),
            ("lib/tasks", WorkspaceEntryKind::Directory),
            ("lib/tasks/ops", WorkspaceEntryKind::Directory),
            ("lib/tasks/ops/deploy.rb", WorkspaceEntryKind::File),
            ("lib/tasks/ops/setup.rb", WorkspaceEntryKind::File),
            ("lib/tasks/some.rb", WorkspaceEntryKind::File),
            ("greetings.rb", WorkspaceEntryKind::File),
            ("README.md", WorkspaceEntryKind::File),
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
    fn test_all_source_includes_hidden_files() {
        let root = build_sample_project();
        // Add a .hidden file to the sample project
        std::fs::write(root.path().join(".hidden"), "This is a hidden file.").unwrap();
        let source = AllSource::new(root.path().to_path_buf());
        let mut paths = vec![];

        for workspace_entry in source.entries().unwrap().iter() {
            paths.push(workspace_entry.clone().path);
        }

        assert!(
            paths.contains(&PathBuf::from(".hidden")),
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
