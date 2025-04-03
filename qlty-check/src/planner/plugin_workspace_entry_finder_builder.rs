use anyhow::Result;
use qlty_analysis::utils::fs::path_to_string;
use qlty_analysis::workspace_entries::{IgnoreGroupsMatcher, TargetMode};
use qlty_analysis::{
    git::GitDiff, workspace_entries::AndMatcher, FileMatcher, GlobsMatcher, PrefixMatcher,
    WorkspaceEntryFinder, WorkspaceEntryMatcher, WorkspaceEntrySource,
};
use qlty_analysis::{AllSource, ArgsSource, DiffSource};
use qlty_config::config::ignore_group::IgnoreGroup;
use qlty_config::config::issue_transformer::{IssueTransformer, NullIssueTransformer};
use qlty_config::config::Ignore;
use qlty_config::{FileType, Workspace};
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Default, Clone)]
pub struct PluginWorkspaceEntryFinderBuilder {
    pub mode: TargetMode,
    pub root: PathBuf,
    pub paths: Vec<PathBuf>,
    pub file_types: HashMap<String, FileType>,
    pub ignores: Vec<Ignore>,
    pub git_diff: Option<GitDiff>,
    pub source: Option<Arc<dyn WorkspaceEntrySource>>,
}

impl PluginWorkspaceEntryFinderBuilder {
    pub fn compute(&mut self) -> Result<()> {
        self.compute_git_diff()?;
        self.compute_source()?;
        Ok(())
    }

    pub fn build(
        &mut self,
        language_names: &[String],
        prefix: Option<String>,
    ) -> Result<WorkspaceEntryFinder> {
        Ok(WorkspaceEntryFinder::new(
            self.source()?,
            self.matcher(language_names, prefix)?,
        ))
    }

    pub fn diff_line_filter(&mut self) -> Result<Box<dyn IssueTransformer>> {
        match self.mode {
            TargetMode::HeadDiff | TargetMode::UpstreamDiff(_) => Ok(Box::new(
                self.git_diff.as_ref().unwrap().line_filter.clone(),
            )),
            _ => Ok(Box::new(NullIssueTransformer)),
        }
    }

    fn source(&mut self) -> Result<Arc<dyn WorkspaceEntrySource>> {
        Ok(self.source.as_ref().unwrap().clone())
    }

    fn matcher(
        &self,
        language_names: &[String],
        prefix: Option<String>,
    ) -> Result<Box<dyn WorkspaceEntryMatcher>> {
        let mut matchers: Vec<Box<dyn WorkspaceEntryMatcher>> = vec![];

        matchers.push(Box::new(FileMatcher));

        let file_types = self
            .file_types
            .iter()
            .filter_map(|(name, file_type)| {
                if language_names.contains(name) {
                    Some(file_type.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let file_matcher = GlobsMatcher::new_for_file_types(&file_types)?;

        matchers.push(Box::new(file_matcher));

        matchers.push(Box::new(PrefixMatcher::new(
            path_to_string(Workspace::current_dir()),
            self.root.to_owned(),
        )));

        if let Some(prefix) = prefix {
            matchers.push(Box::new(PrefixMatcher::new(
                path_to_string(self.root.join(prefix)),
                self.root.to_owned(),
            )));
        }

        let ignores_without_metadata = self
            .ignores
            .iter()
            .filter(|i| i.plugins.is_empty() && i.rules.is_empty() && i.levels.is_empty())
            .collect();

        let ignore_groups = IgnoreGroup::build_from_ignores(&ignores_without_metadata);

        matchers.push(Box::new(IgnoreGroupsMatcher::new(ignore_groups)));
        Ok(Box::new(AndMatcher::new(matchers)))
    }

    fn compute_source(&mut self) -> Result<()> {
        let cwd = Workspace::current_dir();

        self.source = Some(match self.mode {
            TargetMode::All | TargetMode::Sample(_) => Arc::new(AllSource::new(self.root.clone())),
            TargetMode::Paths(_) => Arc::new(ArgsSource::new(
                self.root.clone(),
                // Use absolute paths, so when running in a subdirectory, the paths are still correct
                self.paths.iter().map(|p| cwd.join(p)).collect(),
            )),
            TargetMode::UpstreamDiff(_)
            | TargetMode::HeadDiff
            | TargetMode::Index
            | TargetMode::IndexFile(_) => self.build_diff_source()?,
        });

        Ok(())
    }

    fn build_diff_source(&self) -> Result<Arc<dyn WorkspaceEntrySource>> {
        Ok(Arc::new(DiffSource::new(
            self.git_diff.as_ref().unwrap().changed_files.clone(),
            &self.root,
        )))
    }

    fn compute_git_diff(&mut self) -> Result<()> {
        if self.needs_git_diff() {
            self.git_diff = Some(GitDiff::compute(self.mode.diff_mode(), &self.root)?);
        }

        Ok(())
    }

    fn needs_git_diff(&self) -> bool {
        matches!(
            self.mode,
            TargetMode::HeadDiff
                | TargetMode::UpstreamDiff(_)
                | TargetMode::Index
                | TargetMode::IndexFile(_)
        )
    }
}
