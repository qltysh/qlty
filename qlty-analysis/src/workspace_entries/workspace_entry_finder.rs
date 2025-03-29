use std::sync::Arc;

use crate::{code::File, WorkspaceEntry, WorkspaceEntrySource};

use super::{
    all_source::AllSource,
    matchers::{AnyMatcher, WorkspaceEntryMatcher},
};
use anyhow::Result;
use rand::seq::SliceRandom;
use rand::thread_rng;
use tracing::{debug, trace, warn};

pub struct WorkspaceEntryFinder {
    source: Arc<dyn WorkspaceEntrySource>,
    matcher: Box<dyn WorkspaceEntryMatcher>,
    results: Option<Vec<WorkspaceEntry>>,
}

impl Default for WorkspaceEntryFinder {
    fn default() -> Self {
        Self::new(Arc::new(AllSource::default()), Box::new(AnyMatcher))
    }
}

impl WorkspaceEntryFinder {
    pub fn new(
        source: Arc<dyn WorkspaceEntrySource>,
        matcher: Box<dyn WorkspaceEntryMatcher>,
    ) -> Self {
        debug!("Creating workspace entry finder from source: {:?}", source);

        Self {
            source,
            matcher,
            results: None,
        }
    }

    pub fn files(&mut self) -> Result<Vec<Arc<File>>> {
        let workspace_entries = self.workspace_entries()?;
        let mut files = Vec::new();

        for workspace_entry in workspace_entries {
            match File::from_workspace_entry(&workspace_entry) {
                Ok(file) => {
                    trace!("{:?}", workspace_entry);
                    files.push(file)
                }
                Err(e) => warn!("Unable to process workspace entry: {}", e),
            }
        }

        Ok(files)
    }

    pub fn workspace_entries(&mut self) -> Result<Vec<WorkspaceEntry>> {
        self.compute_workspace_entries()?;
        Ok(self.results.as_ref().unwrap().to_owned())
    }

    pub fn sample(&mut self, sample: usize) -> Result<Vec<WorkspaceEntry>> {
        self.compute_workspace_entries()?;
        let mut workspace_entries = self.results.as_ref().unwrap().to_owned();
        workspace_entries.shuffle(&mut thread_rng());
        workspace_entries.truncate(sample);
        Ok(workspace_entries)
    }

    fn compute_workspace_entries(&mut self) -> Result<()> {
        if self.results.is_some() {
            return Ok(());
        }

        let mut workspace_entries = vec![];

        let entries = self.source.entries()?;
        for workspace_entry in entries.iter() {
            if let Some(workspace_entry) = self.matcher.matches(workspace_entry.clone()) {
                trace!("Adding workspace entry: {:?}", &workspace_entry);
                workspace_entries.push(workspace_entry);
            } else {
                trace!("Skipping workspace entry: {:?}", &workspace_entry);
            }
        }

        self.results = Some(workspace_entries);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        workspace_entries::{
            matchers::{FileMatcher, GlobsMatcher},
            AndMatcher,
        },
        WorkspaceEntryKind,
    };
    use itertools::Itertools;
    use qlty_config::config::Builder;
    use qlty_test_utilities::git::build_sample_project;
    use std::path::PathBuf;

    #[test]
    fn test_find_without_matchers() -> Result<()> {
        let root = build_sample_project();
        let source = AllSource::new(root.path().to_path_buf());
        let mut workspace_entry_finder =
            WorkspaceEntryFinder::new(Arc::new(source), Box::new(AnyMatcher));
        let workspace_entries = workspace_entry_finder.workspace_entries().unwrap();
        let mut paths = vec![];
        for workspace_entry in workspace_entries {
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

        Ok(())
    }

    #[test]
    fn test_find_with_matchers() -> Result<()> {
        let root = build_sample_project();
        let source = AllSource::new(root.path().to_path_buf());
        let file_matcher = Box::new(FileMatcher) as Box<dyn WorkspaceEntryMatcher>;
        let file_type_matcher = build_file_types_workspace_entry_matcher()?;
        let file_type_matcher = Box::new(file_type_matcher) as Box<dyn WorkspaceEntryMatcher>;

        let mut workspace_entry_finder = WorkspaceEntryFinder::new(
            Arc::new(source),
            Box::new(AndMatcher::new(vec![file_matcher, file_type_matcher])),
        );

        let workspace_entries = workspace_entry_finder.workspace_entries().unwrap();
        let mut paths = vec![];
        for workspace_entry in workspace_entries {
            paths.push((workspace_entry.path, workspace_entry.kind));
        }

        let expected_paths = build_expected_workspace_entries(vec![
            ("lib/hello.rb", WorkspaceEntryKind::File),
            ("lib/tasks/ops/deploy.rb", WorkspaceEntryKind::File),
            ("lib/tasks/ops/setup.rb", WorkspaceEntryKind::File),
            ("lib/tasks/some.rb", WorkspaceEntryKind::File),
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

        Ok(())
    }

    fn build_file_types_workspace_entry_matcher() -> Result<GlobsMatcher> {
        let project_config = Builder::default_config().unwrap().to_owned();
        let all_file_types = project_config.file_types.to_owned();
        let file_types_names = vec!["ruby".to_owned()];
        let file_types = all_file_types
            .iter()
            .filter_map(|(name, file_type)| {
                if file_types_names.contains(&name) {
                    Some(file_type.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        GlobsMatcher::new_for_file_types(&file_types)
    }

    #[test]
    fn test_sample() -> Result<()> {
        let root = build_sample_project();
        let source = AllSource::new(root.path().to_path_buf());
        let mut workspace_entry_finder =
            WorkspaceEntryFinder::new(Arc::new(source), Box::new(AnyMatcher));
        let workspace_entries = workspace_entry_finder.sample(3).unwrap();
        let mut paths = vec![];
        for workspace_entry in workspace_entries {
            paths.push(workspace_entry.path);
        }

        let possible_paths = vec![
            PathBuf::from(""),
            PathBuf::from(".gitignore"),
            PathBuf::from("lib"),
            PathBuf::from("lib/hello.rb"),
            PathBuf::from("lib/tasks"),
            PathBuf::from("lib/tasks/ops"),
            PathBuf::from("lib/tasks/ops/deploy.rb"),
            PathBuf::from("lib/tasks/ops/setup.rb"),
            PathBuf::from("lib/tasks/some.rb"),
            PathBuf::from("greetings.rb"),
            PathBuf::from("README.md"),
        ];

        assert!(possible_paths.contains(&paths[0]));
        assert!(possible_paths.contains(&paths[1]));
        assert!(possible_paths.contains(&paths[2]));
        assert!(paths.len() == 3);

        Ok(())
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
