use anyhow::Result;
use qlty_analysis::utils::fs::path_to_string;
use qlty_analysis::workspace_entries::{
    ExcludeGroupsMatcher, PluginSpecificExcludeMatcher, TargetMode,
};
use qlty_analysis::{
    git::GitDiff, workspace_entries::AndMatcher, FileMatcher, GlobsMatcher, PrefixMatcher,
    WorkspaceEntryFinder, WorkspaceEntryMatcher, WorkspaceEntrySource,
};
use qlty_analysis::{AllSource, ArgsSource, DiffSource};
use qlty_config::config::exclude_group::ExcludeGroup;
use qlty_config::config::issue_transformer::{IssueTransformer, NullIssueTransformer};
use qlty_config::config::Exclude;
use qlty_config::{FileType, Workspace};
use std::collections::HashSet;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Default, Clone)]
pub struct PluginWorkspaceEntryFinderBuilder {
    pub mode: TargetMode,
    pub root: PathBuf,
    pub paths: Vec<PathBuf>,
    pub file_types: HashMap<String, FileType>,
    pub excludes: Vec<Exclude>,
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

        let exclude_groups = ExcludeGroup::build_from_excludes(&self.excludes.iter().collect());

        matchers.push(Box::new(ExcludeGroupsMatcher::new(exclude_groups)));

        let all_excluded_plugins: HashSet<String> = self
            .excludes
            .iter()
            .flat_map(|i| i.plugins.clone())
            .collect();

        for plugin_name in all_excluded_plugins {
            let plugin_specific_excludes = self
                .excludes
                .iter()
                .filter(|i| i.plugins.contains(&plugin_name))
                .cloned()
                .collect::<Vec<_>>();

            if !plugin_specific_excludes.is_empty() {
                matchers.push(Box::new(PluginSpecificExcludeMatcher::new(
                    plugin_name.clone(),
                    plugin_specific_excludes,
                    self.root.clone(),
                )));
            }
        }

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
