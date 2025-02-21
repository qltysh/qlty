use crate::planner::config_files::PluginConfigFile;
use crate::planner::target::Target;
use crate::tool::Tool;
use anyhow::{Context, Result};
use git2::{Repository, Status};
use itertools::Itertools;
use prost::Message;
use qlty_analysis::cache::{Cache, CacheKey, HashDigest};
use qlty_config::config::PluginDef;
use qlty_config::version::QLTY_VERSION;
use qlty_config::Workspace;
use qlty_types::analysis::v1::Issue;
use std::sync::Arc;
use std::{collections::HashMap, fmt::Debug, path::PathBuf};
use tracing::trace;

#[derive(Debug, Clone)]
pub struct IssueCache {
    pub cache: Box<dyn Cache>,
}

impl IssueCache {
    pub fn new(cache: Box<dyn Cache>) -> Self {
        Self { cache }
    }

    pub fn read(&self, cache_key: &IssuesCacheKey) -> Result<Option<IssuesCacheHit>> {
        trace!(
            "IssueCache read for {}: {:?}",
            cache_key.hexdigest(),
            cache_key
        );
        let result = self.cache.read(cache_key)?;

        match result {
            Some(contents) => Ok(Some(IssuesCacheHit {
                cache_key: cache_key.clone(),
                issues: self.contents_to_issues(&contents),
            })),
            None => Ok(None),
        }
    }

    pub fn write(&self, cache_key: &IssuesCacheKey, issues: &[Issue]) -> Result<()> {
        trace!(
            "IssueCache write for {} ({} issues): {:?}",
            cache_key.hexdigest(),
            issues.len(),
            cache_key
        );
        let contents = self.issues_to_contents(issues);
        self.cache.write(cache_key, &contents)?;
        Ok(())
    }

    fn issues_to_contents(&self, issues: &[Issue]) -> Vec<u8> {
        if issues.is_empty() {
            return vec![];
        }

        let mut contents = vec![];

        for issue in issues {
            let mut buffer = Vec::new();
            issue.encode_length_delimited(&mut buffer).unwrap();
            contents.extend(buffer);
        }

        contents
    }

    fn contents_to_issues(&self, contents: &[u8]) -> Vec<Issue> {
        if contents.is_empty() {
            return vec![];
        }

        let mut issues = vec![];
        let mut reader = std::io::Cursor::new(contents);

        while let Ok(issue) = Issue::decode_length_delimited(&mut reader) {
            issues.push(issue)
        }

        issues
    }
}

#[derive(Debug, Clone)]
pub struct IssuesCacheHit {
    pub cache_key: IssuesCacheKey,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Clone)]
pub struct IssuesCacheKey {
    repository_tree_sha: Option<String>,
    dirty_paths: Vec<PathBuf>,
    pub digest: HashDigest,
}

#[derive(Debug)]
struct InvocationCacheKey {
    qlty_version: String,
    tool: Box<dyn Tool>,
    plugin: Arc<PluginDef>,
    driver_name: String,
    affects_cache: HashMap<PathBuf, String>,
    configs: Arc<Vec<PluginConfigFile>>,
}

impl InvocationCacheKey {
    fn build(&self) -> HashDigest {
        let mut digest = HashDigest::new();

        if let Some(runtime) = &self.plugin.runtime {
            digest.add("plugin.runtime", &runtime.to_string());
        }

        if let Some(version) = &self.plugin.version {
            digest.add("plugin.version", version);
        }

        digest.add(
            "plugin.downloads",
            &serde_yaml::to_string(&self.plugin.downloads.iter().sorted().collect_vec()).unwrap(),
        );

        digest.add(
            "plugin.releases",
            &serde_yaml::to_string(&self.plugin.releases.iter().sorted().collect_vec()).unwrap(),
        );

        digest.add(
            "plugin.extra_packages",
            &serde_yaml::to_string(&self.plugin.extra_packages).unwrap(),
        );

        digest.add(
            "plugin.releases",
            &serde_yaml::to_string(&self.plugin.releases.iter().sorted().collect_vec()).unwrap(),
        );

        if let Some(package) = &self.plugin.package {
            digest.add("plugin.package", package);
        }

        if let Some(download_type) = &self.plugin.download_type {
            digest.add("plugin.download_type", download_type);
        }

        for environment in self.plugin.environment.iter().sorted() {
            if environment.list.is_empty() {
                digest.add(
                    &format!("plugin.environment.{}", environment.name),
                    &environment.value,
                );
            } else {
                digest.add(
                    &format!("plugin.environment.{}", environment.name),
                    &serde_yaml::to_string(&environment.list).unwrap(),
                );
            }
        }

        if let Some(prefix) = &self.plugin.prefix {
            digest.add("plugin.prefix", prefix);
        }

        let driver = self.plugin.drivers.get(&self.driver_name).unwrap();

        digest.add("plugin.driver.script", &driver.script);

        digest.add("plugin.driver.output", &driver.output.to_string());

        digest.add(
            "plugin.driver.output_format",
            &driver.output_format.to_string(),
        );

        if let Some(output_regex) = &driver.output_regex {
            digest.add("plugin.driver.output_regex", output_regex);
        }

        if let Some(output_level) = &driver.output_level {
            digest.add(
                "plugin.driver.output_level",
                &serde_yaml::to_string(output_level).unwrap(),
            );
        }

        if let Some(output_category) = &driver.output_category {
            digest.add(
                "plugin.driver.output_category",
                &serde_yaml::to_string(output_category).unwrap(),
            );
        }

        digest.add(
            "plugin.driver.driver_type",
            &serde_yaml::to_string(&driver.driver_type).unwrap(),
        );

        digest.add("plugin.driver.batch", &driver.batch.to_string());

        digest.add("plugin.driver.max_batch", &driver.max_batch.to_string());

        digest.add(
            "plugin.driver.success_codes",
            &serde_yaml::to_string(&driver.success_codes).unwrap(),
        );

        digest.add(
            "plugin.driver.cache_results",
            &driver.cache_results.to_string(),
        );

        digest.add(
            "plugin.driver.target",
            &serde_yaml::to_string(&driver.target).unwrap(),
        );

        digest.add(
            "plugin.driver.invocation_directory_def",
            &serde_yaml::to_string(&driver.invocation_directory_def).unwrap(),
        );

        if let Some(prepare_script) = &driver.prepare_script {
            digest.add(
                "plugin.driver.prepare_script",
                &serde_yaml::to_string(&prepare_script).unwrap(),
            );
        }

        if let Some(version_matcher) = &driver.version_matcher {
            digest.add(
                "plugin.driver.version_matcher",
                &serde_yaml::to_string(&version_matcher).unwrap(),
            );
        }

        digest.add(
            "plugin.driver.copy_configs_into_tool_install",
            &driver.copy_configs_into_tool_install.to_string(),
        );

        digest.add("qlty_version", &self.qlty_version);
        digest.add("tool", &self.tool.directory());
        digest.add("driver_name", &self.driver_name);

        for config in self.configs.clone().iter().sorted() {
            digest.add(&config.path.to_string_lossy(), &config.contents);
        }

        for (path, contents) in self.affects_cache.iter().sorted() {
            digest.add(&path.to_string_lossy(), contents);
        }

        digest
    }
}

impl IssuesCacheKey {
    pub fn new(
        tool: Box<dyn Tool>,
        plugin: Arc<PluginDef>,
        driver_name: String,
        configs: Arc<Vec<PluginConfigFile>>,
        affects_cache: Vec<String>,
    ) -> Self {
        let mut cache_busters = HashMap::new();

        for affect_cache in affects_cache.iter().sorted() {
            let path = PathBuf::from(affect_cache);
            let contents = std::fs::read_to_string(&path).unwrap_or("".to_string());
            cache_busters.insert(path, contents);
        }

        let mut repository_sha: Option<String> = None;
        let mut dirty_paths = Vec::new();
        if let Ok(repository) = Workspace::new().and_then(|w| w.repo()) {
            repository_sha = Self::repository_sha(&repository).ok();
            dirty_paths = Self::collect_dirty_paths(&repository);
        }

        Self {
            repository_tree_sha: repository_sha,
            dirty_paths,
            digest: InvocationCacheKey {
                qlty_version: QLTY_VERSION.to_string(),
                tool: tool.clone(),
                plugin: plugin.clone(),
                driver_name,
                affects_cache: cache_busters,
                configs,
            }
            .build(),
        }
    }

    pub fn finalize(&mut self, target: &Target) {
        self.digest.add("target_path", &target.path_string());

        self.digest
            .add("target_contents_size", &target.contents_size.to_string());

        if !self.add_target_sha(target) {
            self.digest.add(
                "target_content_modified",
                &target
                    .content_modified
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string(),
            );
        }

        self.digest.finalize();
    }

    fn add_target_sha(&mut self, target: &Target) -> bool {
        if let Some(sha) = &self.repository_tree_sha {
            if !self
                .dirty_paths
                .iter()
                .any(|path| target.path.starts_with(path))
            {
                self.digest.add("target_sha", sha);
                return true;
            }
        }

        false
    }

    fn repository_sha(repo: &Repository) -> Result<String> {
        Ok(repo
            .head()?
            .resolve()?
            .target()
            .context("missing target")?
            .to_string())
    }

    fn collect_dirty_paths(repo: &Repository) -> Vec<PathBuf> {
        if let Ok(statuses) = repo.statuses(None) {
            statuses
                .iter()
                .filter(|entry| entry.status() != Status::CURRENT && entry.path().is_some())
                .flat_map(|entry| entry.path().map(PathBuf::from))
                .collect::<Vec<PathBuf>>()
        } else {
            vec![]
        }
    }
}

impl CacheKey for IssuesCacheKey {
    fn hexdigest(&self) -> String {
        self.digest.hexdigest()
    }
}
