use crate::Tool;
use anyhow::{Context, Result};
use qlty_analysis::{utils::fs::path_to_native_string, WorkspaceEntry};
use qlty_config::{
    config::{DriverDef, InvocationDirectoryType, PluginDef},
    Library,
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct InvocationDirectoryPlanner {
    pub driver: DriverDef,
    pub plugin: PluginDef,
    pub tool: Box<dyn Tool>,
    pub target_root: PathBuf,
    pub workspace_root: PathBuf,
}

impl InvocationDirectoryPlanner {
    pub fn compute(&self, target: &WorkspaceEntry) -> Result<PathBuf> {
        match self.driver.invocation_directory_def.kind {
            InvocationDirectoryType::Root => Ok(self.target_root.clone()),
            InvocationDirectoryType::TargetDirectory => self.target_directory(target),
            InvocationDirectoryType::RootOrParentWithAnyConfig => {
                self.root_or_parent_with_any_config(target)
            }
            InvocationDirectoryType::RootOrParentWith => {
                let target_directory = self.target_directory(target)?;
                let search_target = self.driver.invocation_directory_def.path.as_ref().unwrap();

                self.find_parent_folder_with(&target_directory, search_target)
            }
            InvocationDirectoryType::RootOrOutermostConfig => self.root_or_outermost_config(target),
            InvocationDirectoryType::ToolDir => Ok(PathBuf::from(self.tool.directory())),
        }
    }

    fn find_parent_folder_with(
        &self,
        starting_directory: &Path,
        search_target: &String,
    ) -> Result<PathBuf> {
        let mut current_path = starting_directory.to_path_buf();

        loop {
            let potential_target = current_path.join(search_target);

            if std::fs::metadata(&potential_target).is_ok() {
                return Ok(current_path);
            }

            // loop until root is reached
            if current_path == self.target_root || !current_path.pop() {
                break;
            }
        }

        Ok(self.target_root.clone())
    }

    fn target_directory(&self, target: &WorkspaceEntry) -> Result<PathBuf> {
        let potential_dir = self.workspace_root.join(target.path.clone());

        if potential_dir.exists() {
            let metadata = std::fs::metadata(potential_dir.clone()).with_context(|| {
                format!(
                    "Failed to get metadata for potential target directory: {}",
                    path_to_native_string(&potential_dir)
                )
            })?;

            let potential_target_dir = if metadata.is_dir() {
                potential_dir
            } else {
                potential_dir.parent().unwrap().to_path_buf()
            };

            if self.target_root == self.workspace_root {
                return Ok(potential_target_dir);
            }

            // Handle case for formatters where files haven't been staged yet
            if let Ok(relative_path) = potential_target_dir.strip_prefix(&self.workspace_root) {
                let target_dir = self.target_root.join(relative_path);
                Ok(target_dir)
            } else {
                Ok(potential_target_dir)
            }
        } else {
            Ok(self.target_root.clone())
        }
    }

    fn root_or_parent_with_any_config(&self, target: &WorkspaceEntry) -> Result<PathBuf> {
        let target_directory = self.target_directory(target)?;

        for plugin_config in &self.plugin.config_files {
            let search_target = plugin_config
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            let path = self.find_parent_folder_with(&target_directory, &search_target)?;

            if path == self.target_root {
                continue;
            } else {
                return Ok(path);
            }
        }

        Ok(self.target_root.clone())
    }

    fn root_or_outermost_config(&self, target: &WorkspaceEntry) -> Result<PathBuf> {
        let config_file_names = self.config_file_names();

        if self.root_has_config_added_at_run_time(&config_file_names)? {
            return Ok(self.target_root.clone());
        }

        let mut current_path = self.target_directory(target)?;
        let mut outermost_path = self.target_root.clone();

        loop {
            if config_file_names
                .iter()
                .any(|name| std::fs::metadata(current_path.join(name)).is_ok())
            {
                outermost_path = current_path.clone();
            }

            if current_path == self.target_root || !current_path.pop() {
                break;
            }
        }

        Ok(outermost_path)
    }

    // Qlty places these at the root only when it runs, after planning, so the
    // directory walk can't see them.
    fn root_has_config_added_at_run_time(&self, config_file_names: &[&str]) -> Result<bool> {
        let configs_dir = Library::new(&self.workspace_root)?.configs_dir();

        Ok(config_file_names.iter().any(|name| {
            configs_dir.join(name).exists()
                || self
                    .plugin
                    .exported_config_paths
                    .iter()
                    .any(|path| path.file_name().and_then(|n| n.to_str()) == Some(*name))
                || self.plugin.fetch.iter().any(|fetch| {
                    Path::new(&fetch.path)
                        .strip_prefix(".")
                        .unwrap_or(Path::new(&fetch.path))
                        == Path::new(name)
                })
        }))
    }

    fn config_file_names(&self) -> Vec<&str> {
        self.plugin
            .config_files
            .iter()
            .filter_map(|config_file| config_file.file_name().and_then(|name| name.to_str()))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{planner::target::Target, tool::null_tool::NullTool};
    use qlty_analysis::{utils::fs::path_to_string, WorkspaceEntryKind};
    use qlty_config::config::{InvocationDirectoryDef, PluginFetch};
    use qlty_test_utilities::git::sample_repo;
    use std::{fs::File, time::SystemTime};
    use tempfile::tempdir;

    fn target_files(path: &str) -> Target {
        Target {
            path: PathBuf::from(path),
            kind: WorkspaceEntryKind::File,
            content_modified: SystemTime::now(),
            contents_size: 0,
            language_name: None,
        }
    }

    fn build_planner(
        temp_dir: &Path,
        invocation_directory_def: InvocationDirectoryDef,
    ) -> InvocationDirectoryPlanner {
        InvocationDirectoryPlanner {
            driver: DriverDef {
                invocation_directory_def,
                ..Default::default()
            },
            plugin: PluginDef {
                config_files: vec!["config_file.json".into()],
                ..Default::default()
            },
            tool: Box::new(NullTool {
                parent_directory: temp_dir
                    .to_path_buf()
                    .join(".qlty")
                    .join("cache")
                    .join("tools")
                    .join("null_tool"),
                plugin_name: "mock_plugin".to_string(),
                plugin: Default::default(),
            }),
            target_root: temp_dir.to_path_buf(),
            workspace_root: temp_dir.to_path_buf(),
        }
    }

    #[test]
    fn test_compute_root_invocation_directory() {
        let (temp_dir, _) = sample_repo();
        let planner = build_planner(
            temp_dir.path(),
            InvocationDirectoryDef {
                kind: InvocationDirectoryType::Root,
                path: None,
            },
        );

        let invocation_directory = planner.compute(&target_files("lib/hello.rb")).unwrap();
        assert_eq!(invocation_directory, temp_dir.path());
    }

    #[test]
    fn test_compute_root_or_parent_with_invocation_directory() {
        let (temp_dir, _) = sample_repo();
        let planner = build_planner(
            temp_dir.path(),
            InvocationDirectoryDef {
                kind: InvocationDirectoryType::RootOrParentWith,
                path: Some("config_file.json".into()),
            },
        );

        File::create(temp_dir.path().join("lib/config_file.json")).unwrap();

        let targets_results = vec![
            (target_files("lib/hello.rb"), temp_dir.path().join("lib")),
            (target_files("greetings.rb"), temp_dir.path().to_path_buf()),
            (
                target_files("lib/tasks/some.rb"),
                temp_dir.path().join("lib"),
            ),
        ];

        for (target, result) in targets_results {
            let invocation_directory = planner.compute(&target).unwrap();
            assert_eq!(invocation_directory, result);
        }
    }

    #[test]
    fn test_compute_root_or_parent_with_any_config_invocation_directory() {
        let (temp_dir, _) = sample_repo();
        let planner = build_planner(
            temp_dir.path(),
            InvocationDirectoryDef {
                kind: InvocationDirectoryType::RootOrParentWithAnyConfig,
                path: None,
            },
        );

        File::create(temp_dir.path().join("lib/config_file.json")).unwrap();

        let targets_results = vec![
            (target_files("lib/hello.rb"), temp_dir.path().join("lib")),
            (target_files("greetings.rb"), temp_dir.path().to_path_buf()),
            (
                target_files("lib/tasks/some.rb"),
                temp_dir.path().join("lib"),
            ),
        ];

        for (target, result) in targets_results {
            let invocation_directory = planner.compute(&target).unwrap();
            assert_eq!(invocation_directory, result);
        }
    }

    #[test]
    fn test_compute_target_directory_invocation_directory() {
        let (temp_dir, _) = sample_repo();
        let planner = build_planner(
            temp_dir.path(),
            InvocationDirectoryDef {
                kind: InvocationDirectoryType::TargetDirectory,
                path: None,
            },
        );

        let targets_results = vec![
            (target_files("lib/hello.rb"), temp_dir.path().join("lib")),
            (target_files("greetings.rb"), temp_dir.path().to_path_buf()),
            (
                target_files("lib/tasks/some.rb"),
                temp_dir.path().join("lib/tasks"),
            ),
        ];

        for (target, result) in targets_results {
            let invocation_directory = planner.compute(&target).unwrap();
            assert_eq!(invocation_directory, result);
        }
    }

    #[test]
    fn test_compute_tool_directory_invocation_directory() {
        let (temp_dir, _) = sample_repo();
        let planner = build_planner(
            temp_dir.path(),
            InvocationDirectoryDef {
                kind: InvocationDirectoryType::ToolDir,
                path: None,
            },
        );

        let tool_directory = temp_dir.path().join(planner.tool.directory());

        let targets_results = vec![
            (target_files("lib/hello.rb"), tool_directory.clone()),
            (target_files("greetings.rb"), tool_directory.clone()),
            (target_files("lib/tasks/some.rb"), tool_directory),
        ];

        for (target, result) in targets_results {
            let invocation_directory = planner.compute(&target).unwrap();
            let result_str = path_to_string(result);
            let result = result_str.split("/./").last().unwrap_or(&result_str);
            assert!(invocation_directory.ends_with(result));
        }
    }

    #[test]
    fn test_target_directory_when_different_target_root_from_workspace_root() {
        let (temp_dir, _) = sample_repo();
        let temp_dir_path = temp_dir.path();
        let temp_dir_staging = tempdir().unwrap();
        let temp_dir_staging_path = temp_dir_staging.path();

        let invocation_directory_def = InvocationDirectoryDef {
            kind: InvocationDirectoryType::TargetDirectory,
            path: None,
        };

        let planner = InvocationDirectoryPlanner {
            driver: DriverDef {
                invocation_directory_def,
                ..Default::default()
            },
            plugin: PluginDef {
                config_files: vec!["config_file.json".into()],
                ..Default::default()
            },
            tool: Box::new(NullTool {
                parent_directory: temp_dir_path
                    .to_path_buf()
                    .join(".qlty")
                    .join("cache")
                    .join("tools")
                    .join("null_tool"),
                plugin_name: "mock_plugin".to_string(),
                plugin: Default::default(),
            }),
            target_root: temp_dir_staging_path.to_path_buf(),
            workspace_root: temp_dir_path.to_path_buf(),
        };

        let targets_results = vec![
            (
                target_files("lib/hello.rb"),
                temp_dir_staging_path.join("lib"),
            ),
            (
                target_files("greetings.rb"),
                temp_dir_staging_path.to_path_buf(),
            ),
            (
                target_files("lib/tasks/some.rb"),
                temp_dir_staging_path.join("lib/tasks"),
            ),
        ];

        for (target, result) in targets_results {
            let invocation_directory = planner.compute(&target).unwrap();
            assert_eq!(invocation_directory, result);
        }
    }

    fn outermost_config_planner(temp_dir: &Path) -> InvocationDirectoryPlanner {
        let mut planner = build_planner(
            temp_dir,
            InvocationDirectoryDef {
                kind: InvocationDirectoryType::RootOrOutermostConfig,
                path: None,
            },
        );
        planner.plugin.config_files = vec!["biome.json".into(), "biome.jsonc".into()];
        planner
    }

    #[test]
    fn test_root_or_outermost_config_uses_config_directory() {
        let (temp_dir, _) = sample_repo();
        let planner = outermost_config_planner(temp_dir.path());
        File::create(temp_dir.path().join("lib/tasks/biome.jsonc")).unwrap();

        let invocation_directory = planner.compute(&target_files("lib/tasks/some.rb")).unwrap();
        assert_eq!(invocation_directory, temp_dir.path().join("lib/tasks"));
    }

    #[test]
    fn test_root_or_outermost_config_uses_highest_config_directory() {
        let (temp_dir, _) = sample_repo();
        let planner = outermost_config_planner(temp_dir.path());
        File::create(temp_dir.path().join("lib/biome.json")).unwrap();
        File::create(temp_dir.path().join("lib/tasks/biome.jsonc")).unwrap();

        let invocation_directory = planner.compute(&target_files("lib/tasks/some.rb")).unwrap();
        assert_eq!(invocation_directory, temp_dir.path().join("lib"));
    }

    #[test]
    fn test_root_or_outermost_config_uses_root_with_root_config() {
        let (temp_dir, _) = sample_repo();
        let planner = outermost_config_planner(temp_dir.path());
        File::create(temp_dir.path().join("biome.json")).unwrap();
        File::create(temp_dir.path().join("lib/biome.json")).unwrap();

        let invocation_directory = planner.compute(&target_files("lib/tasks/some.rb")).unwrap();
        assert_eq!(invocation_directory, temp_dir.path());
    }

    #[test]
    fn test_root_or_outermost_config_uses_root_with_qlty_configs_config() {
        let (temp_dir, _) = sample_repo();
        let planner = outermost_config_planner(temp_dir.path());
        std::fs::create_dir_all(temp_dir.path().join(".qlty/configs")).unwrap();
        File::create(temp_dir.path().join(".qlty/configs/biome.json")).unwrap();
        File::create(temp_dir.path().join("lib/biome.json")).unwrap();

        let invocation_directory = planner.compute(&target_files("lib/tasks/some.rb")).unwrap();
        assert_eq!(invocation_directory, temp_dir.path());
    }

    #[test]
    fn test_root_or_outermost_config_uses_root_with_exported_config() {
        let (temp_dir, _) = sample_repo();
        let mut planner = outermost_config_planner(temp_dir.path());
        planner.plugin.exported_config_paths = vec!["/sources/default/biome.json".into()];
        File::create(temp_dir.path().join("lib/biome.json")).unwrap();

        let invocation_directory = planner.compute(&target_files("lib/tasks/some.rb")).unwrap();
        assert_eq!(invocation_directory, temp_dir.path());
    }

    #[test]
    fn test_root_or_outermost_config_falls_back_to_root() {
        let (temp_dir, _) = sample_repo();
        let planner = outermost_config_planner(temp_dir.path());

        let invocation_directory = planner.compute(&target_files("lib/tasks/some.rb")).unwrap();
        assert_eq!(invocation_directory, temp_dir.path());
    }

    #[test]
    fn test_root_or_outermost_config_uses_root_with_fetched_config() {
        let (temp_dir, _) = sample_repo();
        let mut planner = outermost_config_planner(temp_dir.path());
        planner.plugin.fetch = vec![PluginFetch {
            url: "https://example.com/biome.json".to_string(),
            path: "./biome.json".to_string(),
        }];
        File::create(temp_dir.path().join("lib/biome.json")).unwrap();

        let invocation_directory = planner.compute(&target_files("lib/tasks/some.rb")).unwrap();
        assert_eq!(invocation_directory, temp_dir.path());
    }
}
