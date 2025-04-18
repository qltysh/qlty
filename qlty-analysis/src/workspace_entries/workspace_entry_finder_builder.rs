use super::{LanguagesShebangMatcher, OrMatcher, TargetMode};
use crate::{
    git::GitDiff,
    workspace_entries::{matchers::ExcludeGroupsMatcher, AndMatcher, LanguageGlobsMatcher},
    AllSource, ArgsSource, DiffSource, FileMatcher, WorkspaceEntryFinder, WorkspaceEntryMatcher,
    WorkspaceEntrySource,
};
use anyhow::{bail, Result};
use qlty_config::{
    config::{exclude_group::ExcludeGroup, Exclude},
    issue_transformer::{IssueTransformer, NullIssueTransformer},
    QltyConfig,
};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct WorkspaceEntryFinderBuilder {
    pub mode: TargetMode,
    pub root: PathBuf,
    pub paths: Vec<PathBuf>,
    pub config: QltyConfig,
    pub exclude_tests: bool,
    pub cached_git_diff: Option<GitDiff>,
}

impl Default for WorkspaceEntryFinderBuilder {
    fn default() -> Self {
        Self {
            mode: TargetMode::All,
            root: std::env::current_dir().unwrap(),
            paths: Vec::new(),
            config: QltyConfig::default(),
            exclude_tests: true,
            cached_git_diff: None,
        }
    }
}

impl WorkspaceEntryFinderBuilder {
    pub fn build(&mut self) -> Result<WorkspaceEntryFinder> {
        Ok(WorkspaceEntryFinder::new(self.source()?, self.matcher()?))
    }

    pub fn diff_line_filter(&mut self) -> Result<Box<dyn IssueTransformer>> {
        match self.mode {
            TargetMode::HeadDiff | TargetMode::UpstreamDiff(_) => {
                Ok(Box::new(self.git_diff()?.line_filter))
            }
            _ => Ok(Box::new(NullIssueTransformer)),
        }
    }

    fn source(&mut self) -> Result<Arc<dyn WorkspaceEntrySource>> {
        match self.mode {
            TargetMode::All => Ok(Arc::new(AllSource::new(self.root.clone()))),
            TargetMode::Paths(_) => Ok(Arc::new(ArgsSource::new(
                self.root.clone(),
                // Use absolute paths, so when running in a subdirectory, the paths are still correct
                self.paths.iter().map(|p| self.root.join(p)).collect(),
            ))),
            TargetMode::UpstreamDiff(_) => Ok(Arc::new(DiffSource::new(
                self.git_diff()?.changed_files,
                &self.root,
            ))),
            _ => bail!("Unsupported workspace entry mode: {:?}", self.mode),
        }
    }

    fn matcher(&self) -> Result<Box<dyn WorkspaceEntryMatcher>> {
        let mut matcher = AndMatcher::default();

        // Files only
        matcher.push(Box::new(FileMatcher));

        // Exclude explicit excludes and tests
        let mut excludes = self.config.exclude.clone();
        debug!("Ignoring globs: {:?}", excludes);

        if self.exclude_tests {
            if !self.config.test_patterns.is_empty() {
                debug!("Ignoring test patterns: {:?}", self.config.test_patterns);

                excludes.push(Exclude {
                    file_patterns: self.config.test_patterns.clone(),
                    ..Default::default()
                });
            } else {
                debug!("Ignoring test patterns: none");
            }
        }

        let exclude_groups = ExcludeGroup::build_from_excludes(&excludes.iter().collect());

        matcher.push(Box::new(ExcludeGroupsMatcher::new(exclude_groups)));

        // Must match a language
        matcher.push(self.languages_matcher()?);

        Ok(Box::new(matcher))
    }

    fn languages_matcher(&self) -> Result<Box<dyn WorkspaceEntryMatcher>> {
        let mut languages = OrMatcher::default();
        let mut interpreters = HashMap::new();

        for language_name in self.config.language.keys() {
            let language = self.config.language.get(language_name).unwrap();

            debug!(
                "Matching {} with globs: {:?}",
                language_name, &language.globs
            );
            let matcher = LanguageGlobsMatcher::new(language_name, &language.globs)?;

            languages.push(Box::new(matcher));

            if !language.interpreters.is_empty() {
                debug!(
                    "Matching {} with interpreters: {:?}",
                    language_name, language.interpreters
                );
                interpreters.insert(language_name.to_string(), language.interpreters.to_owned());
            }
        }

        // If none of the globs match, we fallback to checking the shebang
        // This is done last, so if a glob matches we never check the shebang
        let shebangs = LanguagesShebangMatcher::new(interpreters);
        languages.push(Box::new(shebangs));

        Ok(Box::new(languages))
    }

    fn git_diff(&mut self) -> Result<GitDiff> {
        GitDiff::compute(self.mode.diff_mode(), &self.root)
    }
}
