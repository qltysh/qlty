use super::{config::enabled_plugins, ActivePlugin, Planner};
use anyhow::{Context, Result};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use itertools::Itertools;
use qlty_config::{config::PluginFetch, warn_once};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};
use tracing::{debug, error, warn};

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct PluginConfigFile {
    pub path: PathBuf,
    pub contents: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigOperation {
    pub source: ConfigSource,
    pub destination: PathBuf,
    pub mode: ConfigCopyMode,
}

#[derive(Debug, Clone, Serialize)]
pub enum ConfigSource {
    File(PathBuf),
    Download(PluginFetch),
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum ConfigCopyMode {
    Symlink,
    Copy,
}

impl Ord for PluginConfigFile {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}

impl PartialOrd for PluginConfigFile {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PluginConfigFile {
    pub fn from_path(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file from path {}", path.display()))?;

        Ok(Self {
            path: path.to_path_buf(),
            contents,
        })
    }
}

#[derive(Debug, Clone)]
struct PluginConfig {
    plugin_name: String,
    config_globset: GlobSet,
}

pub fn config_globset(config_files: &Vec<PathBuf>) -> Result<GlobSet> {
    let mut globset = GlobSetBuilder::new();

    for config_file in config_files {
        let glob = GlobBuilder::new(
            config_file
                .to_str()
                .ok_or(anyhow::anyhow!("Invalid path: {:?}", config_file))?,
        )
        .literal_separator(true)
        .build()?;

        globset.add(glob);
    }

    Ok(globset.build()?)
}

fn exclude_globset(exclude_patterns: &Vec<String>) -> Result<GlobSet> {
    let mut globset = GlobSetBuilder::new();

    for pattern in exclude_patterns {
        let glob = GlobBuilder::new(pattern)
            .literal_separator(true)
            .build()
            .with_context(|| format!("Failed to build glob for pattern: {pattern}"))?;

        globset.add(glob);
    }

    Ok(globset.build()?)
}

pub fn plugin_configs(planner: &Planner) -> Result<HashMap<String, Vec<PluginConfigFile>>> {
    let plugins = enabled_plugins(planner)?;
    let mut plugins_configs = vec![];
    let mut configs: HashMap<String, Vec<PluginConfigFile>> = HashMap::new();

    for active_plugin in &plugins {
        plugins_configs.push(PluginConfig {
            plugin_name: active_plugin.name.clone(),
            config_globset: config_globset(&active_plugin.plugin.config_files)?,
        });

        for exported_config_path in &active_plugin.plugin.exported_config_paths {
            debug!(
                "Adding exported config path ({:?}) to plugin config {}",
                exported_config_path, &active_plugin.name,
            );
            let file_name = exported_config_path.file_name().ok_or(anyhow::anyhow!(
                "Invalid exported config path: {:?}",
                exported_config_path
            ))?;

            let exported_path = planner.workspace.root.join(file_name);

            // Create an empty config file entry for exported config paths.
            let config_file = PluginConfigFile {
                path: exported_path.clone(),
                contents: String::new(),
            };

            configs
                .entry(active_plugin.name.clone())
                .or_default()
                .push(config_file);
        }
    }

    let exclude_globset = exclude_globset(&planner.config.exclude_patterns)?;

    for entry in planner.workspace.walker() {
        let entry = entry?;
        if let Some(os_str) = entry.path().file_name() {
            let file_name = os_str.to_os_string();
            for plugin_config in &plugins_configs {
                if plugin_config.config_globset.is_match(&file_name) {
                    if exclude_globset.is_match(entry.path()) {
                        warn!(
                            "Excluding config file {:?} due to exclude patterns",
                            entry.path()
                        );
                        warn_once(&format!(
                            "Excluding config file {:?} due to exclude patterns",
                            entry.path()
                        ));
                    } else {
                        let entry_path = entry.path();
                        let config_file = match PluginConfigFile::from_path(entry_path) {
                            Ok(config_file) => config_file,
                            _ => {
                                error!("Failed to read config file from path {:?}", entry_path);
                                continue;
                            }
                        };

                        debug!(
                            "Found config file for plugin {}: {:?}",
                            &plugin_config.plugin_name, &config_file.path
                        );
                        configs
                            .entry(plugin_config.plugin_name.clone())
                            .or_default()
                            .push(config_file);
                    }
                }
            }
        }
    }

    Ok(configs)
}

pub fn compute_config_staging_operations(planner: &Planner) -> Result<Vec<ConfigOperation>> {
    let plugins = enabled_plugins(planner)?;
    let all_config_paths = collect_all_config_paths(&plugins);

    let mut operations = Vec::new();
    // Operations for config files in the repository
    operations.extend(repository_config_operations(planner, &all_config_paths)?);
    // Operations for any exported config files in the sources
    operations.extend(exported_config_operations(planner, &plugins)?);
    // Operations for config files in the .qlty/configs directory
    operations.extend(qlty_config_operations(planner, &plugins)?);
    // Operations for any fetch directives
    operations.extend(fetch_operations(planner, &plugins));
    // Operations for any copying config files into tool installations
    operations.extend(tool_install_config_operations(planner)?);

    Ok(operations)
}

fn collect_all_config_paths(plugins: &[ActivePlugin]) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    for active_plugin in plugins {
        paths.extend(active_plugin.plugin.config_files.clone());
        for affects_cache in &active_plugin.plugin.affects_cache {
            paths.push(PathBuf::from(affects_cache));
        }
    }

    paths
}

fn repository_config_operations(
    planner: &Planner,
    all_config_paths: &[PathBuf],
) -> Result<Vec<ConfigOperation>> {
    if all_config_paths.is_empty() {
        return Ok(Vec::new());
    }

    let config_globset = config_globset(&all_config_paths.to_vec())?;
    let exclude_globset = exclude_globset(&planner.config.exclude_patterns)?;

    let mut operations = Vec::new();
    for entry in planner.workspace.walker() {
        let entry = entry?;
        if let Some(os_str) = entry.path().file_name() {
            let file_name = os_str.to_os_string();
            if config_globset.is_match(&file_name) && !exclude_globset.is_match(entry.path()) {
                if let Ok(library) = planner.workspace.library() {
                    if entry.path().starts_with(library.configs_dir()) {
                        continue;
                    }
                }

                let destination_path = entry
                    .path()
                    .strip_prefix(&planner.workspace.root)
                    .map(|relative_path| {
                        planner
                            .staging_area
                            .destination_directory
                            .join(relative_path)
                    })
                    .unwrap_or_else(|_| {
                        planner
                            .staging_area
                            .destination_directory
                            .join(entry.path().file_name().unwrap())
                    });

                operations.push(ConfigOperation {
                    source: ConfigSource::File(entry.path().to_path_buf()),
                    destination: destination_path,
                    mode: ConfigCopyMode::Symlink,
                });
            }
        }
    }

    Ok(operations)
}

fn exported_config_operations(
    planner: &Planner,
    plugins: &[ActivePlugin],
) -> Result<Vec<ConfigOperation>> {
    let mut operations = Vec::new();
    let exported_config_paths: Vec<_> = plugins
        .iter()
        .flat_map(|plugin| &plugin.plugin.exported_config_paths)
        .unique()
        .collect();

    for exported_config_path in exported_config_paths {
        let file_name = exported_config_path.file_name().ok_or(anyhow::anyhow!(
            "Invalid exported config path: {:?}",
            exported_config_path
        ))?;

        if planner.workspace.root != planner.staging_area.destination_directory {
            operations.push(ConfigOperation {
                source: ConfigSource::File(exported_config_path.clone()),
                destination: planner.staging_area.destination_directory.join(file_name),
                mode: ConfigCopyMode::Symlink,
            });
        }

        operations.push(ConfigOperation {
            source: ConfigSource::File(exported_config_path.clone()),
            destination: planner.workspace.root.join(file_name),
            mode: ConfigCopyMode::Symlink,
        });
    }

    Ok(operations)
}

fn qlty_config_operations(
    planner: &Planner,
    plugins: &[ActivePlugin],
) -> Result<Vec<ConfigOperation>> {
    let mut operations = Vec::new();
    let library = planner.workspace.library().ok();

    let config_file_names: Vec<_> = plugins
        .iter()
        .flat_map(|plugin| &plugin.plugin.config_files)
        .map(|path| path.to_string_lossy().to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    for config_file_name in config_file_names {
        let source_path = if let Some(library) = &library {
            library.configs_dir().join(&config_file_name)
        } else {
            PathBuf::from(&config_file_name)
        };

        if planner.workspace.root != planner.staging_area.destination_directory {
            operations.push(ConfigOperation {
                source: ConfigSource::File(source_path.clone()),
                destination: planner
                    .staging_area
                    .destination_directory
                    .join(&config_file_name),
                mode: ConfigCopyMode::Symlink,
            });
        }

        operations.push(ConfigOperation {
            source: ConfigSource::File(source_path.clone()),
            destination: planner.workspace.root.join(&config_file_name),
            mode: ConfigCopyMode::Symlink,
        });
    }

    Ok(operations)
}

fn fetch_operations(planner: &Planner, plugins: &[ActivePlugin]) -> Vec<ConfigOperation> {
    let mut operations = Vec::new();

    for active_plugin in plugins {
        for fetch in &active_plugin.plugin.fetch {
            operations.push(ConfigOperation {
                source: ConfigSource::Download(fetch.clone()),
                destination: planner.workspace.root.join(&fetch.path),
                mode: ConfigCopyMode::Copy,
            });

            if planner.workspace.root != planner.staging_area.destination_directory {
                operations.push(ConfigOperation {
                    source: ConfigSource::Download(fetch.clone()),
                    destination: planner.staging_area.destination_directory.join(&fetch.path),
                    mode: ConfigCopyMode::Copy,
                });
            }
        }
    }

    operations
}

fn tool_install_config_operations(planner: &Planner) -> Result<Vec<ConfigOperation>> {
    let mut operations = Vec::new();
    let mut seen_destinations = HashSet::new();
    let plugin_configs = plugin_configs(planner)?;

    for invocation in &planner.invocations {
        if invocation.driver.copy_configs_into_tool_install {
            let plugin_name = &invocation.plugin_name;
            if let Some(configs) = plugin_configs.get(plugin_name) {
                for config in configs {
                    let tool_dir = PathBuf::from(invocation.tool.directory());
                    let file_name = config
                        .path
                        .file_name()
                        .ok_or(anyhow::anyhow!("Invalid config path: {:?}", config.path))?;

                    let destination = tool_dir.join(file_name);

                    if seen_destinations.insert(destination.clone()) {
                        operations.push(ConfigOperation {
                            source: ConfigSource::File(config.path.clone()),
                            destination,
                            mode: ConfigCopyMode::Copy,
                        });
                    }
                }
            }
        }
    }

    Ok(operations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::driver::test::build_driver;
    use crate::executor::Driver;
    use crate::planner::target::Target;
    use crate::planner::{InvocationPlan, Planner};
    use crate::tool::null_tool::NullTool;
    use crate::Settings;
    use qlty_analysis::{WorkspaceEntry, WorkspaceEntryKind};
    use qlty_config::config::InvocationDirectoryDef;
    use qlty_config::{QltyConfig, Workspace};
    use qlty_types::analysis::v1::ExecutionVerb;
    use std::fs;
    use std::sync::Arc;
    use std::time::SystemTime;
    use tempfile::TempDir;

    fn create_test_planner() -> (Planner, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::for_root(temp_dir.path()).unwrap();
        let settings = Settings {
            root: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let config = QltyConfig::default();

        let cache = Planner::build_cache(&workspace, &settings).unwrap();

        let planner = Planner {
            verb: ExecutionVerb::Check,
            settings,
            workspace,
            config,
            staging_area: crate::executor::staging_area::StagingArea::generate(
                crate::executor::staging_area::Mode::Source,
                temp_dir.path().to_path_buf(),
                None,
            ),
            issue_cache: crate::cache::IssueCache::new(cache),
            target_mode: None,
            workspace_entry_finder_builder: None,
            cache_hits: vec![],
            active_plugins: vec![],
            plugin_configs: HashMap::new(),
            invocations: vec![],
            transformers: vec![],
            config_staging_operations: vec![],
        };

        (planner, temp_dir)
    }

    #[test]
    fn test_config_globset_with_config_files_only() {
        let config_files = vec![PathBuf::from(".eslintrc.js"), PathBuf::from("package.json")];
        let globset = config_globset(&config_files).unwrap();

        assert!(globset.is_match(".eslintrc.js"));
        assert!(globset.is_match("package.json"));
        assert!(!globset.is_match("other.txt"));
    }

    #[test]
    fn test_config_globset_with_affects_cache() {
        let config_files = vec![
            PathBuf::from(".eslintrc.js"),
            PathBuf::from("package.json"),
            PathBuf::from("yarn.lock"), // affects_cache file
        ];
        let globset = config_globset(&config_files).unwrap();

        assert!(globset.is_match(".eslintrc.js"));
        assert!(globset.is_match("package.json"));
        assert!(globset.is_match("yarn.lock"));
        assert!(!globset.is_match("other.txt"));
    }

    #[test]
    fn test_compute_config_staging_operations_empty_plugins() {
        let (planner, _temp_dir) = create_test_planner();
        let operations = compute_config_staging_operations(&planner).unwrap();
        assert!(operations.is_empty());
    }

    #[test]
    fn test_compute_config_staging_operations_with_repository_configs() {
        let (mut planner, temp_dir) = create_test_planner();

        // Create a test config file in the repository
        fs::write(temp_dir.path().join(".eslintrc.js"), "module.exports = {};").unwrap();

        // Set up the plugin definition with config files
        let mut eslint_plugin_def = qlty_config::config::PluginDef::default();
        eslint_plugin_def.config_files = vec![PathBuf::from(".eslintrc.js")];

        let mut plugin_definitions = HashMap::new();
        plugin_definitions.insert("eslint".to_string(), eslint_plugin_def);

        // Set up the complete config with both enabled plugins and definitions
        let config = qlty_config::QltyConfig {
            plugin: vec![qlty_config::config::EnabledPlugin {
                name: "eslint".to_string(),
                ..Default::default()
            }],
            plugins: qlty_config::config::PluginsConfig {
                definitions: plugin_definitions,
                downloads: HashMap::new(),
                releases: HashMap::new(),
            },
            ..Default::default()
        };

        planner.config = config;

        // Call the actual function being tested
        let operations = compute_config_staging_operations(&planner).unwrap();

        let staging_dir = planner.staging_area.destination_directory.clone();
        let workspace_root = planner.workspace.root.clone();

        // Should find the repository config file
        let staging_ops: Vec<_> = operations
            .iter()
            .filter(|op| op.destination.starts_with(&staging_dir))
            .collect();

        // Should also have qlty directory operations (workspace root symlinks)
        let qlty_ops: Vec<_> = operations
            .iter()
            .filter(|op| op.destination.starts_with(&workspace_root))
            .collect();

        // We should have found the .eslintrc.js file in the repository
        let found_eslintrc = staging_ops.iter().any(|op| match &op.source {
            ConfigSource::File(path) => path.file_name().unwrap() == ".eslintrc.js",
            _ => false,
        });

        assert!(found_eslintrc, "Should find .eslintrc.js in repository");
        assert!(
            qlty_ops
                .iter()
                .any(|op| matches!(op.mode, ConfigCopyMode::Symlink)),
            "Should have qlty directory operations"
        );
    }

    #[test]
    fn test_tool_install_config_operations_empty_invocations() {
        let (planner, _temp_dir) = create_test_planner();
        let operations = tool_install_config_operations(&planner).unwrap();
        assert!(operations.is_empty());
    }

    #[test]
    fn test_exclude_globset() {
        let patterns = vec!["node_modules/**".to_string(), "*.tmp".to_string()];
        let globset = exclude_globset(&patterns).unwrap();

        assert!(globset.is_match(&PathBuf::from("node_modules/package/index.js")));
        assert!(globset.is_match(&PathBuf::from("test.tmp")));
        assert!(!globset.is_match(&PathBuf::from("src/main.js")));
    }

    #[test]
    fn test_config_globset_invalid_patterns() {
        // Test error handling for invalid glob patterns
        let invalid_patterns = vec![PathBuf::from("[invalid-glob")];
        let result = config_globset(&invalid_patterns);
        assert!(result.is_err());
    }

    #[test]
    fn test_exclude_globset_invalid_patterns() {
        // Test error handling for invalid exclusion patterns
        let invalid_patterns = vec!["[invalid-glob".to_string()];
        let result = exclude_globset(&invalid_patterns);
        assert!(result.is_err());
    }

    #[test]
    fn test_compute_config_staging_operations_with_exported_configs() {
        let (mut planner, _temp_dir) = create_test_planner();

        // Set up plugin with exported config paths
        let mut plugin_def = qlty_config::config::PluginDef::default();
        plugin_def.exported_config_paths = vec![PathBuf::from("custom.config")];

        let mut plugin_definitions = HashMap::new();
        plugin_definitions.insert("custom_plugin".to_string(), plugin_def);

        let config = qlty_config::QltyConfig {
            plugin: vec![qlty_config::config::EnabledPlugin {
                name: "custom_plugin".to_string(),
                ..Default::default()
            }],
            plugins: qlty_config::config::PluginsConfig {
                definitions: plugin_definitions,
                downloads: HashMap::new(),
                releases: HashMap::new(),
            },
            ..Default::default()
        };

        planner.config = config;
        let operations = compute_config_staging_operations(&planner).unwrap();

        // Should create operations for exported config paths
        let workspace_root = planner.workspace.root.clone();
        let workspace_ops: Vec<_> = operations
            .iter()
            .filter(|op| op.destination.starts_with(&workspace_root))
            .collect();

        assert!(
            workspace_ops
                .iter()
                .any(|op| matches!(op.mode, ConfigCopyMode::Symlink)),
            "Should have workspace copy operations for exported configs"
        );
    }

    #[test]
    fn test_compute_config_staging_operations_mixed_config_types() {
        let (mut planner, temp_dir) = create_test_planner();

        // Create files in repository
        fs::write(temp_dir.path().join(".eslintrc.js"), "module.exports = {};").unwrap();
        fs::write(temp_dir.path().join("package-lock.json"), "{}").unwrap();

        // Set up plugin with both config_files AND affects_cache
        let mut plugin_def = qlty_config::config::PluginDef::default();
        plugin_def.config_files = vec![PathBuf::from(".eslintrc.js")];
        plugin_def.affects_cache = vec!["package-lock.json".to_string()];

        let mut plugin_definitions = HashMap::new();
        plugin_definitions.insert("eslint".to_string(), plugin_def);

        let config = qlty_config::QltyConfig {
            plugin: vec![qlty_config::config::EnabledPlugin {
                name: "eslint".to_string(),
                ..Default::default()
            }],
            plugins: qlty_config::config::PluginsConfig {
                definitions: plugin_definitions,
                downloads: HashMap::new(),
                releases: HashMap::new(),
            },
            ..Default::default()
        };

        planner.config = config;
        let operations = compute_config_staging_operations(&planner).unwrap();

        let staging_dir = planner.staging_area.destination_directory.clone();
        let staging_ops: Vec<_> = operations
            .iter()
            .filter(|op| op.destination.starts_with(&staging_dir))
            .collect();

        // Should find both config files AND affects_cache files
        let found_files: Vec<_> = staging_ops
            .iter()
            .filter_map(|op| match &op.source {
                ConfigSource::File(path) => path.file_name().and_then(|os| os.to_str()),
                _ => None,
            })
            .collect();

        assert!(
            found_files.contains(&".eslintrc.js"),
            "Should find config file"
        );
        assert!(
            found_files.contains(&"package-lock.json"),
            "Should find affects_cache file"
        );
    }

    #[test]
    fn test_tool_install_config_operations_with_copy_enabled() {
        let (mut planner, temp_dir) = create_test_planner();

        // Create actual config files in the workspace directory where they'll be discovered
        let config_file_path = temp_dir.path().join("test_config.yml");
        fs::write(&config_file_path, "test: config").unwrap();

        // Enable the test plugin in the planner config so it gets discovered
        planner.config.plugins.definitions.insert(
            "test".to_string(),
            qlty_config::config::PluginDef {
                config_files: vec![PathBuf::from("test_config.yml")],
                ..Default::default()
            },
        );

        // Also add the plugin to the enabled plugins list
        planner
            .config
            .plugin
            .push(qlty_config::config::EnabledPlugin {
                name: "test".to_string(),
                ..Default::default()
            });

        // Create a mock invocation using NullTool (like the existing test)

        let plugin = qlty_config::config::PluginDef {
            config_files: vec![PathBuf::from("test_config.yml")],
            ..Default::default()
        };

        let tool_dir = temp_dir.path().join(".qlty/tools/test");
        let mut driver = build_driver(vec![], vec![]);
        driver.copy_configs_into_tool_install = true; // Key setting!

        let mock_file_path = temp_dir.path().join("lib/hello.rb");
        let invocation = InvocationPlan {
            invocation_id: "test".to_string(),
            verb: ExecutionVerb::Check,
            settings: Settings::default(),
            workspace: Workspace::new().unwrap(),
            runtime: None,
            runtime_version: None,
            plugin_name: "test".to_string(),
            plugin: plugin.clone(),
            tool: Box::new(NullTool {
                plugin_name: "test".to_string(),
                parent_directory: tool_dir.clone(),
                plugin: plugin.clone(),
            }),
            driver_name: "test".to_string(),
            driver: Driver::from(driver),
            plugin_configs: vec![], // Not used by the function we're testing
            target_root: temp_dir.path().to_path_buf(),
            workspace_entries: Arc::new(vec![WorkspaceEntry {
                path: mock_file_path.clone(),
                content_modified: SystemTime::now(),
                language_name: None,
                contents_size: 0,
                kind: WorkspaceEntryKind::File,
            }]),
            targets: vec![Target {
                path: mock_file_path,
                content_modified: SystemTime::now(),
                language_name: None,
                contents_size: 0,
                kind: WorkspaceEntryKind::File,
            }],
            invocation_directory: temp_dir.path().to_path_buf(),
            invocation_directory_def: InvocationDirectoryDef::default(),
        };
        planner.invocations.push(invocation);

        let operations = tool_install_config_operations(&planner).unwrap();

        // Should create tool install operations for the config file
        let tool_install_ops: Vec<_> = operations
            .iter()
            .filter(|op| {
                matches!(
                    (&op.source, op.mode.clone()),
                    (ConfigSource::File(_), ConfigCopyMode::Copy)
                )
            })
            .collect();

        assert!(
            !tool_install_ops.is_empty(),
            "Should have tool install operations when copy_configs_into_tool_install=true"
        );
        assert_eq!(
            tool_install_ops.len(),
            1,
            "Should have one operation for the config file"
        );

        let op = &tool_install_ops[0];
        assert!(
            matches!(&op.source, ConfigSource::File(path) if path == &config_file_path),
            "Should copy from the config file"
        );
        assert!(
            op.destination.to_string_lossy().contains("test_config.yml"),
            "Should copy config file to tool directory"
        );
        assert!(
            op.destination
                .to_string_lossy()
                .contains(".qlty/tools/test"),
            "Should copy to tool installation directory"
        );
        assert!(matches!(op.mode, ConfigCopyMode::Copy));
    }

    #[test]
    fn test_plugin_configs_excludes_affects_cache() {
        let (mut planner, temp_dir) = create_test_planner();

        fs::write(temp_dir.path().join(".eslintrc.js"), "module.exports = {};").unwrap();
        fs::write(temp_dir.path().join("package-lock.json"), "{}").unwrap();

        let mut plugin_def = qlty_config::config::PluginDef::default();
        plugin_def.config_files = vec![PathBuf::from(".eslintrc.js")];
        plugin_def.affects_cache = vec!["package-lock.json".to_string()];

        let mut plugin_definitions = HashMap::new();
        plugin_definitions.insert("eslint".to_string(), plugin_def);

        let config = qlty_config::QltyConfig {
            plugin: vec![qlty_config::config::EnabledPlugin {
                name: "eslint".to_string(),
                ..Default::default()
            }],
            plugins: qlty_config::config::PluginsConfig {
                definitions: plugin_definitions,
                downloads: HashMap::new(),
                releases: HashMap::new(),
            },
            ..Default::default()
        };

        planner.config = config;

        let configs = plugin_configs(&planner).unwrap();
        let eslint_configs = configs.get("eslint").unwrap();

        let config_paths: Vec<_> = eslint_configs
            .iter()
            .map(|c| c.path.file_name().unwrap().to_str().unwrap())
            .collect();

        assert!(
            config_paths.contains(&".eslintrc.js"),
            "Should include config file"
        );
        assert!(
            !config_paths.contains(&"package-lock.json"),
            "Should not include affects_cache file as a config"
        );
    }
}
