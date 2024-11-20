use std::{cmp::Reverse, collections::HashMap, path::PathBuf};

use anyhow::Result;
use qlty_config::{
    config::{DriverBatchBy, DriverDef},
    Library,
};
use tracing::{debug, warn};

use super::{
    config_files::PluginConfigFile, invocation_directory::InvocationDirectoryPlanner,
    target::Target,
};

const MAX_MAX_BATCH: usize = 512;

#[derive(Debug)]
pub struct DriverTargetBatch {
    pub targets: Vec<Target>,
    pub invocation_directory: Option<PathBuf>,
    pub config_file: Option<PluginConfigFile>,
}

#[derive(Debug)]
pub struct TargetBatcher {
    pub driver: DriverDef,
    pub driver_name: String,
    pub invocation_directory_planner: InvocationDirectoryPlanner,
    pub plugin_configs: Vec<PluginConfigFile>,
    pub current_prefix: Option<String>,
    pub all_prefixes: Vec<String>,
    pub workspace_root: PathBuf,
}

impl TargetBatcher {
    pub fn compute(&self, targets: &[Target]) -> Result<Vec<DriverTargetBatch>> {
        let mut driver_target_batches = vec![];
        let targets = self.filter_prefix(targets);

        match self.driver.batch_by {
            DriverBatchBy::None => {
                for targets in targets.chunks(self.compute_chunk_size()) {
                    driver_target_batches.push(DriverTargetBatch {
                        targets: targets.to_vec(),
                        invocation_directory: None,
                        config_file: None,
                    });
                }
            }
            DriverBatchBy::InvocationDirectory => {
                self.batch_by_invocation_directory(&mut driver_target_batches, &targets)?;
            }
            DriverBatchBy::ConfigFile => {
                self.batch_by_config_files(&mut driver_target_batches, &targets)?;
            }
        }

        Ok(driver_target_batches)
    }

    fn filter_prefix(&self, targets: &[Target]) -> Vec<Target> {
        if let Some(current_prefix) = &self.current_prefix {
            let current_prefix_path = PathBuf::from(&current_prefix);

            // sort prefixes by length in descending order
            // deeper prefixes should be processed first
            let mut sorted_prefixes: Vec<PathBuf> =
                self.all_prefixes.iter().map(PathBuf::from).collect();
            sorted_prefixes.sort_by_key(|p| Reverse(p.components().count()));

            targets
                .iter()
                .filter(|target| {
                    for prefix in &sorted_prefixes {
                        if *prefix == current_prefix_path {
                            return true;
                        }

                        if !prefix.as_os_str().is_empty()
                            && current_prefix_path.join(&target.path).starts_with(prefix)
                        {
                            return false;
                        }
                    }
                    true
                })
                .map(|target| {
                    let mut target = target.clone();
                    target.path = target
                        .path
                        .strip_prefix(&current_prefix_path)
                        .unwrap_or(target.path.as_path())
                        .to_path_buf();
                    target
                })
                .collect()
        } else {
            targets.to_vec()
        }
    }

    fn batch_by_invocation_directory(
        &self,
        driver_target_batches: &mut Vec<DriverTargetBatch>,
        targets: &[Target],
    ) -> Result<()> {
        let mut invocation_dir_targets_map = HashMap::new();

        for target in targets {
            let invocation_directory = self.invocation_directory_planner.compute(target)?;

            invocation_dir_targets_map
                .entry(invocation_directory)
                .or_insert_with(Vec::new)
                .push(target.clone());
        }

        for (invocation_directory, targets) in invocation_dir_targets_map {
            for targets in targets.chunks(self.compute_chunk_size()) {
                driver_target_batches.push(DriverTargetBatch {
                    targets: targets.to_vec(),
                    invocation_directory: Some(invocation_directory.clone()),
                    config_file: None,
                });
            }
        }

        Ok(())
    }

    fn batch_by_config_files(
        &self,
        driver_target_batches: &mut Vec<DriverTargetBatch>,
        targets: &[Target],
    ) -> Result<()> {
        if self.plugin_configs.is_empty() {
            for targets in targets.chunks(self.compute_chunk_size()) {
                driver_target_batches.push(DriverTargetBatch {
                    targets: targets.to_vec(),
                    invocation_directory: None,
                    config_file: None,
                });
            }

            return Ok(());
        }

        let mut sorted_configs = self.plugin_configs.clone();

        // If a config is in .qlty/configs directory
        // Batch as if it is in the root of the workspace
        // It is going to be moved there by the executor
        // After batching is resolved revert the path back to the original to ensure cache hits
        let mut original_path_map = HashMap::new();
        let library = Library::new(&self.workspace_root)?;
        let workspace_config_path = library.configs_dir();
        for config in &mut sorted_configs {
            if config.path.parent() == Some(&workspace_config_path) {
                let new_path = self.workspace_root.join(config.path.file_name().unwrap());
                original_path_map.insert(new_path.clone(), config.path.clone());
                debug!(
                    "Changing config file path from {:?} to {:?}",
                    config.path, new_path
                );

                config.path = self.workspace_root.join(config.path.file_name().unwrap());
            }
        }

        sorted_configs.sort_by(|a, b| {
            b.path
                .components()
                .count()
                .cmp(&a.path.components().count())
        });

        let mut config_targets_map = HashMap::new();

        let prefixed_workspace_root = self
            .workspace_root
            .join(self.current_prefix.as_deref().unwrap_or(""));

        for target in targets {
            for config in &sorted_configs {
                let config_path = config
                    .path
                    .parent()
                    .expect("Config path should have a parent");

                let full_target_path = target.full_path(&prefixed_workspace_root)?;

                if full_target_path.starts_with(config_path) {
                    config_targets_map
                        .entry(config)
                        .or_insert_with(Vec::new)
                        .push(target.clone());
                    break;
                }
            }
        }

        for (config_file, targets) in config_targets_map {
            for targets in targets.chunks(self.compute_chunk_size()) {
                // Revert the path back to the original
                let config_file_path = original_path_map
                    .get(&config_file.path)
                    .unwrap_or(&config_file.path);
                let config_file = PluginConfigFile {
                    path: config_file_path.clone(),
                    contents: config_file.contents.clone(),
                };

                driver_target_batches.push(DriverTargetBatch {
                    targets: targets.to_vec(),
                    invocation_directory: None,
                    config_file: Some(config_file),
                });
            }
        }

        Ok(())
    }

    pub fn compute_chunk_size(&self) -> usize {
        if self.driver.batch {
            let max_batch = self.driver.max_batch.clamp(1, MAX_MAX_BATCH);
            if self.driver.max_batch > MAX_MAX_BATCH {
                warn!(
                    "Driver {} has max_batch {} which is greater than the maximum allowed value of {}",
                    self.driver_name, self.driver.max_batch, MAX_MAX_BATCH
                );
            }
            max_batch
        } else {
            1
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tool::null_tool::NullTool;
    use qlty_analysis::WorkspaceEntryKind;
    use qlty_config::config::{InvocationDirectoryDef, InvocationDirectoryType, PluginDef};
    use qlty_test_utilities::git::sample_repo;
    use std::{fs::File, time::SystemTime};

    fn target_files(path: &str) -> Target {
        Target {
            path: PathBuf::from(path),
            kind: WorkspaceEntryKind::File,
            content_modified: SystemTime::now(),
            contents_size: 0,
            language_name: None,
        }
    }

    fn plugin_config_file(path: &str) -> PluginConfigFile {
        PluginConfigFile {
            path: PathBuf::from(path),
            contents: "".to_string(),
        }
    }

    fn setup_target_batcher(
        batch_by: DriverBatchBy,
        max_batch: usize,
        invocation_directory_def: InvocationDirectoryDef,
    ) -> TargetBatcher {
        let driver = DriverDef {
            batch_by,
            batch: max_batch > 1,
            max_batch,
            invocation_directory_def,
            ..Default::default()
        };

        TargetBatcher {
            driver: driver.clone(),
            driver_name: "test_driver".to_string(),
            invocation_directory_planner: InvocationDirectoryPlanner {
                driver: driver.clone(),
                plugin: PluginDef {
                    config_files: vec!["config1".into(), "config2".into()],
                    ..Default::default()
                },
                tool: Box::new(NullTool {
                    plugin_name: "mock_plugin".to_string(),
                    plugin: Default::default(),
                }),
                target_root: PathBuf::from("/User/test/project_root/"),
            },
            plugin_configs: vec![
                plugin_config_file("/User/test/project_root/config1"),
                plugin_config_file("/User/test/project_root/sub/config2"),
            ],
            current_prefix: None,
            all_prefixes: vec![],
            workspace_root: PathBuf::from("/User/test/project_root/"),
        }
    }

    #[test]
    fn test_compute_no_batching() {
        let target_batcher =
            setup_target_batcher(DriverBatchBy::None, 1, InvocationDirectoryDef::default());
        let targets = vec![target_files("/file1"), target_files("/sub/file2")];

        let batches = target_batcher.compute(&targets).unwrap();

        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].targets.len(), 1);
        assert_eq!(batches[1].targets.len(), 1);
        assert!(batches[0].invocation_directory.is_none());
        assert!(batches[0].config_file.is_none());
    }

    #[test]
    fn test_compute_basic_numerical_batching() {
        let target_batcher =
            setup_target_batcher(DriverBatchBy::None, 5, InvocationDirectoryDef::default());
        let targets = vec![target_files("/file1"), target_files("/sub/file2")];

        let batches = target_batcher.compute(&targets).unwrap();

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].targets.len(), 2);
        assert!(batches[0].invocation_directory.is_none());
        assert!(batches[0].config_file.is_none());
    }

    #[test]
    fn test_compute_with_config_file_batching() {
        let target_batcher = setup_target_batcher(
            DriverBatchBy::ConfigFile,
            2,
            InvocationDirectoryDef::default(),
        );
        let targets = vec![
            target_files("file1"),
            target_files("sub/file2"),
            target_files("file3"),
        ];

        let mut batches = target_batcher.compute(&targets).unwrap();

        // sort for easier comparison
        batches.sort_by(|a, b| b.targets.len().cmp(&a.targets.len()));

        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].targets.len(), 2);
        assert_eq!(
            batches[0].config_file,
            Some(plugin_config_file("/User/test/project_root/config1"))
        );
        assert_eq!(batches[1].targets.len(), 1);
        assert_eq!(
            batches[1].config_file,
            Some(plugin_config_file("/User/test/project_root/sub/config2"))
        );
    }

    #[test]
    fn test_compute_with_config_file_batching_with_no_config_files() {
        let mut target_batcher = setup_target_batcher(
            DriverBatchBy::ConfigFile,
            5,
            InvocationDirectoryDef::default(),
        );
        target_batcher.plugin_configs = vec![];

        let targets = vec![
            target_files("file1"),
            target_files("sub/file2"),
            target_files("file3"),
        ];

        let batches = target_batcher.compute(&targets).unwrap();

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].targets.len(), 3);
        assert_eq!(batches[0].config_file, None);
    }

    #[test]
    fn test_compute_with_config_file_batching_with_no_config_files_into_chunks() {
        let mut target_batcher = setup_target_batcher(
            DriverBatchBy::ConfigFile,
            2,
            InvocationDirectoryDef::default(),
        );
        target_batcher.plugin_configs = vec![];

        let targets = vec![
            target_files("file1"),
            target_files("sub/file2"),
            target_files("file3"),
        ];

        let mut batches = target_batcher.compute(&targets).unwrap();

        // sort for easier comparison
        batches.sort_by(|a, b| b.targets.len().cmp(&a.targets.len()));

        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].targets.len(), 2);
        assert_eq!(batches[0].config_file, None);

        assert_eq!(batches[1].targets.len(), 1);
        assert_eq!(batches[1].config_file, None);
    }

    #[test]
    fn test_compute_with_config_file_in_qlty_configs_dir() {
        let mut target_batcher = setup_target_batcher(
            DriverBatchBy::ConfigFile,
            2,
            InvocationDirectoryDef::default(),
        );
        target_batcher.plugin_configs = vec![plugin_config_file(
            "/User/test/project_root/.qlty/configs/config1",
        )];

        let targets = vec![target_files("file1"), target_files("sub/file2")];

        let batches = target_batcher.compute(&targets).unwrap();

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].targets.len(), 2);
        assert_eq!(
            batches[0].config_file,
            Some(plugin_config_file(
                "/User/test/project_root/.qlty/configs/config1"
            ))
        );
    }

    #[test]
    fn test_compute_chunk_size() {
        let target_batcher =
            setup_target_batcher(DriverBatchBy::None, 1024, InvocationDirectoryDef::default());
        assert_eq!(target_batcher.compute_chunk_size(), MAX_MAX_BATCH);

        let target_batcher =
            setup_target_batcher(DriverBatchBy::None, 256, InvocationDirectoryDef::default());
        assert_eq!(target_batcher.compute_chunk_size(), 256);

        let target_batcher =
            setup_target_batcher(DriverBatchBy::None, 0, InvocationDirectoryDef::default());
        assert_eq!(target_batcher.compute_chunk_size(), 1);
    }

    #[test]
    fn test_compute_chunk_size_no_batching() {
        let target_batcher =
            setup_target_batcher(DriverBatchBy::None, 1, InvocationDirectoryDef::default());
        assert_eq!(target_batcher.compute_chunk_size(), 1);
    }

    #[test]
    fn test_compute_with_invocation_directory_batching() {
        let (temp_dir, _) = sample_repo();
        let temp_path = temp_dir.path().to_path_buf();
        File::create(temp_path.join("lib/config_file.json")).unwrap();
        File::create(temp_path.join("lib/tasks/ops/config_file.json")).unwrap();

        let driver = DriverDef {
            batch_by: DriverBatchBy::InvocationDirectory,
            batch: true,
            max_batch: 5,
            invocation_directory_def: InvocationDirectoryDef {
                kind: InvocationDirectoryType::RootOrParentWithAnyConfig,
                path: None,
            },
            ..Default::default()
        };

        let target_batcher = TargetBatcher {
            driver: driver.clone(),
            driver_name: "test_driver".to_string(),
            invocation_directory_planner: InvocationDirectoryPlanner {
                driver: driver.clone(),
                plugin: PluginDef {
                    config_files: vec!["config_file.json".into()],
                    ..Default::default()
                },
                tool: Box::new(NullTool {
                    plugin_name: "mock_plugin".to_string(),
                    plugin: Default::default(),
                }),
                target_root: temp_path.clone(),
            },
            plugin_configs: vec![],
            current_prefix: None,
            all_prefixes: vec![],
            workspace_root: temp_path.clone(),
        };

        let targets = vec![
            target_files("lib/hello.rb"),
            target_files("lib/tasks/ops/deploy.rb"),
            target_files("lib/tasks/ops/setup.rb"),
        ];

        let mut batches = target_batcher.compute(&targets).unwrap();

        batches.sort_by(|a, b| b.targets.len().cmp(&a.targets.len()));

        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].targets.len(), 2);
        assert_eq!(batches[1].targets.len(), 1);
        assert_eq!(
            batches[0].invocation_directory,
            Some(temp_path.join("lib/tasks/ops"))
        );
        assert_eq!(
            batches[1].invocation_directory,
            Some(temp_path.join("lib/"))
        );
        assert!(batches[0].config_file.is_none());
        assert!(batches[1].config_file.is_none());
    }

    #[test]
    fn test_filter_prefix() {
        let target_batcher =
            setup_target_batcher(DriverBatchBy::None, 1, InvocationDirectoryDef::default());

        let targets = vec![
            target_files("lib/hello.rb"),
            target_files("lib/tasks/ops/deploy.rb"),
            target_files("lib/tasks/ops/setup.rb"),
        ];

        let filtered_targets = target_batcher.filter_prefix(&targets);

        assert_eq!(filtered_targets.len(), 3);

        let mut target_batcher =
            setup_target_batcher(DriverBatchBy::None, 1, InvocationDirectoryDef::default());
        target_batcher.current_prefix = Some("".to_string());
        target_batcher.all_prefixes = vec!["lib/tasks".to_string(), "".to_string()];

        let filtered_targets = target_batcher.filter_prefix(&targets);

        assert_eq!(filtered_targets.len(), 1);
        assert_eq!(filtered_targets[0].path, PathBuf::from("lib/hello.rb"));
    }
}
