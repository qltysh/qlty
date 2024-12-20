use super::{ActivePlugin, Planner};
use anyhow::bail;
use anyhow::{anyhow, Result};
use qlty_analysis::workspace_entries::TargetMode;
use qlty_config::config::{DriverDef, EnabledPlugin, IssueMode, Platform, PluginDef};
use semver::{Version, VersionReq};
use std::path::{Path, PathBuf};
use tracing::{debug, trace, warn};

const ALL: &str = "ALL";

pub fn enabled_plugins(planner: &Planner) -> Result<Vec<ActivePlugin>> {
    let active_plugins = configure_plugins(planner)?;

    if planner.settings.filters.is_empty() {
        Ok(active_plugins)
    } else {
        warn!("Filtering plugins: {:?}", planner.settings.filters);

        let mut filtered_plugins = vec![];

        for filter in planner.settings.filters.iter() {
            let plugin_name = &filter.plugin;

            let plugins: Vec<ActivePlugin> = active_plugins
                .iter()
                .filter(|p| p.name == *plugin_name)
                .cloned()
                .collect();

            if plugins.is_empty() {
                bail!("Plugin not found: {}", plugin_name);
            }

            filtered_plugins.extend(plugins);
        }

        Ok(filtered_plugins)
    }
}

fn configure_plugins(planner: &Planner) -> Result<Vec<ActivePlugin>> {
    trace!("Configuring plugins...");
    let mut enabled_plugins = vec![];

    for enabled_plugin in planner.config.plugin.iter() {
        if enabled_plugin.mode == IssueMode::Disabled {
            continue;
        }

        if let Some(plugin_def) = planner
            .config
            .plugins
            .definitions
            .get(enabled_plugin.name.as_str())
        {
            if !plugin_def.supported_platforms.is_empty()
                && !plugin_def
                    .supported_platforms
                    .contains(&Platform::current())
            {
                warn!(
                    "Plugin {} is not supported on this platform ({}), skipping. (Supported platforms are: {})",
                    enabled_plugin.name,
                    Platform::current(),
                    plugin_def.supported_platforms.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", ")
                );
                continue;
            }
        }

        if let Some(TargetMode::UpstreamDiff(_)) = &planner.target_mode {
            if enabled_plugin.skip_upstream.unwrap_or(false) {
                debug!(
                    "Enabled plugin {} skip_upstream is true, skipping plugin in upstream diff mode.",
                    enabled_plugin.name
                );
                continue;
            }
        }

        if !enabled_plugin.triggers.is_empty()
            && !enabled_plugin.triggers.contains(&planner.settings.trigger)
        {
            debug!(
                "Enabled triggers {:?}, detected trigger {:?}, skipping plugin {}.",
                enabled_plugin.triggers, planner.settings.trigger, enabled_plugin.name
            );
            continue;
        }

        let name = &enabled_plugin.name;
        let plugin = configure_plugin(planner, name, enabled_plugin)?;
        enabled_plugins.push(ActivePlugin {
            name: name.to_string(),
            plugin,
        });
    }

    Ok(enabled_plugins)
}

fn configure_plugin(
    planner: &Planner,
    name: &str,
    enabled_plugin: &EnabledPlugin,
) -> Result<PluginDef> {
    if let Some(plugin_def) = planner.config.plugins.definitions.get(name) {
        let mut plugin_def = plugin_def.clone();

        plugin_def.version = Some(enabled_plugin.version.clone());

        for (driver_name, driver) in plugin_def.drivers.iter_mut() {
            // if driver has multiple versions, we need to find the one that matches the active plugin version
            if !driver.version.is_empty() {
                let enabled_version = plugin_def.version.clone().unwrap();

                if let Some(enabled_driver) =
                    version_match_driver(&enabled_version, &driver.version)?
                {
                    debug!(
                        "Plugin {} driver {} version {} matched with driver: {:?}",
                        name, driver_name, enabled_version, enabled_driver
                    );
                    *driver = enabled_driver;
                } else {
                    bail!(
                        "No matching driver version found for plugin {} driver {} version {}",
                        name,
                        driver_name,
                        enabled_version
                    );
                }
            }
        }

        if let Some(package_file) = &enabled_plugin.package_file {
            let mut package_file = Path::new(&package_file);
            let root = planner.workspace.root.clone();

            package_file = package_file.strip_prefix("/").unwrap_or(package_file);
            package_file = package_file.strip_prefix("\\").unwrap_or(package_file);

            let prefixed_root = root.join(enabled_plugin.prefix.clone().unwrap_or_default());
            let package_file = prefixed_root.join(package_file);

            if !package_file.exists() {
                bail!("Missing package file for {}: {:?}", name, package_file);
            }

            plugin_def.package_file = Some(package_file.to_str().unwrap_or_default().to_string());
        }

        // This is becoming a weird pattern, we should probably refactor this?
        plugin_def.fetch = enabled_plugin.fetch.clone();
        plugin_def.package_filters = enabled_plugin.package_filters.clone();
        plugin_def.prefix = enabled_plugin.prefix.clone();

        plugin_def
            .extra_packages
            .extend(enabled_plugin.extra_packages.clone());

        for driver in plugin_def.drivers.values() {
            plugin_def.config_files.extend(driver.config_files.clone());
        }

        plugin_def
            .config_files
            .extend(enabled_plugin.config_files.clone());

        plugin_def.config_files.extend(
            enabled_plugin
                .fetch
                .iter()
                .map(|fetch| PathBuf::from(fetch.path.clone()))
                .collect::<Vec<PathBuf>>(),
        );

        plugin_def
            .affects_cache
            .extend(enabled_plugin.affects_cache.clone());

        if !enabled_plugin.drivers.contains(&ALL.to_string()) {
            plugin_def
                .drivers
                .retain(|driver_name, _| enabled_plugin.drivers.contains(driver_name));
        }
        if let Some(TargetMode::UpstreamDiff(_)) = &planner.target_mode {
            if enabled_plugin.skip_upstream.is_none() || enabled_plugin.skip_upstream.unwrap() {
                plugin_def.drivers.retain(|driver_name, driver| {
                    if !driver.skip_upstream {
                        debug!(
                            "Enabled plugin {} driver {} skip_upstream is true, skipping driver in upstream diff mode.",
                            enabled_plugin.name,
                            driver_name
                        );
                        true
                    } else {
                        false
                    }
                });
            }
        }

        Ok(plugin_def)
    } else {
        bail!("Unknown plugin {}", name)
    }
}

fn version_match_driver(
    enabled_version: &str,
    drivers: &Vec<DriverDef>,
) -> Result<Option<DriverDef>> {
    let version = Version::parse(enabled_version)?;

    for driver in drivers {
        let version_matcher = driver
            .version_matcher
            .as_ref()
            .ok_or_else(|| anyhow!("version_matcher is None"))?;

        if VersionReq::parse(version_matcher)?.matches(&version) {
            return Ok(Some(driver.clone()));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        cache::IssueCache,
        executor::staging_area::{Mode, StagingArea},
        CheckFilter, Settings,
    };
    use qlty_config::{
        config::{DriverDef, PluginFetch, PluginsConfig},
        QltyConfig, Workspace,
    };
    use qlty_types::analysis::v1::ExecutionVerb;
    use std::{collections::HashMap, fs, path::PathBuf};
    use tempfile::tempdir;

    fn build_planner(config: QltyConfig) -> Planner {
        let workspace = Workspace::default();
        let settings = Settings::default();
        let cache = Planner::build_cache(&workspace, &settings).unwrap();

        Planner {
            config,
            verb: ExecutionVerb::Check,
            target_mode: Some(TargetMode::UpstreamDiff("main".to_string())),
            settings,
            workspace,
            staging_area: StagingArea::new(PathBuf::new(), PathBuf::new(), Mode::Source),
            issue_cache: IssueCache::new(cache),
            workspace_entry_finder_builder: None,
            cache_hits: vec![],
            active_plugins: vec![],
            plugin_configs: HashMap::new(),
            invocations: vec![],
            transformers: vec![],
        }
    }

    #[test]
    fn test_skip_upstream_plugins_from_enabled_plugins() {
        let mut plugin_defs = HashMap::new();
        plugin_defs.insert("enabled".to_string(), PluginDef::default());
        plugin_defs.insert("test_in_upstream".to_string(), PluginDef::default());

        let mut planner = build_planner(QltyConfig {
            plugin: vec![
                EnabledPlugin {
                    name: "test_in_upstream".to_string(),
                    skip_upstream: Some(true),
                    ..Default::default()
                },
                EnabledPlugin {
                    name: "enabled".to_string(),
                    ..Default::default()
                },
            ],
            plugins: PluginsConfig {
                downloads: HashMap::new(),
                releases: HashMap::new(),
                definitions: plugin_defs,
            },
            ..Default::default()
        });

        let plugins = enabled_plugins(&planner).unwrap();

        assert!(plugins.iter().find(|p| p.name == "enabled").is_some());
        assert!(plugins
            .iter()
            .find(|p| p.name == "test_in_upstream")
            .is_none());

        // Check if test_in_upstream shows when TargetMode is All
        planner.target_mode = Some(TargetMode::All);

        let plugins = enabled_plugins(&planner).unwrap();

        assert!(plugins.iter().find(|p| p.name == "enabled").is_some());
        assert!(plugins
            .iter()
            .find(|p| p.name == "test_in_upstream")
            .is_some());
    }

    #[test]
    fn test_skip_upstream_plugins_from_driver_def() {
        let mut plugin_defs = HashMap::new();
        plugin_defs.insert(
            "enabled".to_string(),
            PluginDef {
                drivers: vec![("test".to_string(), DriverDef::default())]
                    .into_iter()
                    .collect(),
                ..Default::default()
            },
        );
        plugin_defs.insert(
            "test_in_upstream".to_string(),
            PluginDef {
                drivers: vec![(
                    "test".to_string(),
                    DriverDef {
                        skip_upstream: true,
                        ..Default::default()
                    },
                )]
                .into_iter()
                .collect(),
                ..Default::default()
            },
        );

        let mut planner = build_planner(QltyConfig {
            plugin: vec![
                EnabledPlugin {
                    name: "test_in_upstream".to_string(),
                    drivers: vec![ALL.to_string()],
                    ..Default::default()
                },
                EnabledPlugin {
                    name: "enabled".to_string(),
                    drivers: vec![ALL.to_string()],
                    ..Default::default()
                },
            ],
            plugins: PluginsConfig {
                downloads: HashMap::new(),
                releases: HashMap::new(),
                definitions: plugin_defs,
            },
            ..Default::default()
        });

        let plugins = enabled_plugins(&planner).unwrap();

        assert_eq!(
            plugins
                .iter()
                .find(|p| p.name == "enabled")
                .unwrap()
                .plugin
                .drivers
                .len(),
            1
        );

        assert!(plugins
            .iter()
            .find(|p| p.name == "test_in_upstream")
            .unwrap()
            .plugin
            .drivers
            .is_empty());

        // test_in_upstream should show up when enabled plugin's
        // skip_upstream is set to false (override behavior)
        planner.config.plugin[0].skip_upstream = Some(false);
        let plugins = enabled_plugins(&planner).unwrap();

        assert_eq!(
            plugins
                .iter()
                .find(|p| p.name == "test_in_upstream")
                .unwrap()
                .plugin
                .drivers
                .len(),
            1
        );

        // Check if test_in_upstream shows when TargetMode is All
        planner.target_mode = Some(TargetMode::All);

        let plugins = enabled_plugins(&planner).unwrap();

        assert!(plugins.iter().find(|p| p.name == "enabled").is_some());
        assert!(plugins
            .iter()
            .find(|p| p.name == "test_in_upstream")
            .is_some());

        // Check if test_in_upstream shows up when driver_def does not contain skip_upstream
        planner.target_mode = Some(TargetMode::UpstreamDiff("test".to_string()));
        planner.config.plugins.definitions.insert(
            "test_in_upstream".to_string(),
            PluginDef {
                drivers: vec![("test".to_string(), DriverDef::default())]
                    .into_iter()
                    .collect(),
                ..Default::default()
            },
        );

        let plugins = enabled_plugins(&planner).unwrap();

        assert_eq!(
            plugins
                .iter()
                .find(|p| p.name == "enabled")
                .unwrap()
                .plugin
                .drivers
                .len(),
            1
        );

        assert_eq!(
            plugins
                .iter()
                .find(|p| p.name == "test_in_upstream")
                .unwrap()
                .plugin
                .drivers
                .len(),
            1
        );
    }

    #[test]
    fn test_fetch_paths_in_config_files() {
        let mut plugin_defs = HashMap::new();
        plugin_defs.insert(
            "enabled".to_string(),
            PluginDef {
                config_files: vec![PathBuf::from("path1")],
                ..Default::default()
            },
        );

        let planner = build_planner(QltyConfig {
            plugin: vec![EnabledPlugin {
                name: "enabled".to_string(),
                config_files: vec![PathBuf::from("path2")],
                fetch: vec![
                    PluginFetch {
                        url: "someurl".to_string(),
                        path: "path3".to_string(),
                    },
                    PluginFetch {
                        url: "someurl".to_string(),
                        path: "path4".to_string(),
                    },
                ],
                ..Default::default()
            }],
            plugins: PluginsConfig {
                downloads: HashMap::new(),
                releases: HashMap::new(),
                definitions: plugin_defs,
            },
            ..Default::default()
        });

        let plugins = enabled_plugins(&planner).unwrap();

        let enabled_plugin = plugins.iter().find(|p| p.name == "enabled").unwrap();
        let config_files = enabled_plugin.plugin.config_files.clone();
        assert_eq!(config_files.len(), 4);
        assert!(config_files.contains(&PathBuf::from("path1")));
        assert!(config_files.contains(&PathBuf::from("path2")));
        assert!(config_files.contains(&PathBuf::from("path3")));
        assert!(config_files.contains(&PathBuf::from("path4")));
    }

    #[test]
    fn test_filter_multiple_active_plugins() {
        let mut plugin_defs = HashMap::new();
        plugin_defs.insert(
            "enabled".to_string(),
            PluginDef {
                drivers: vec![("test".to_string(), DriverDef::default())]
                    .into_iter()
                    .collect(),
                ..Default::default()
            },
        );

        let mut planner = build_planner(QltyConfig {
            plugin: vec![
                EnabledPlugin {
                    name: "enabled".to_string(),
                    prefix: Some("prefix".to_string()),
                    drivers: vec![ALL.to_string()],
                    ..Default::default()
                },
                EnabledPlugin {
                    name: "enabled".to_string(),
                    prefix: Some("".to_string()),
                    drivers: vec![ALL.to_string()],
                    ..Default::default()
                },
            ],
            plugins: PluginsConfig {
                downloads: HashMap::new(),
                releases: HashMap::new(),
                definitions: plugin_defs,
            },
            ..Default::default()
        });

        let plugins = enabled_plugins(&planner).unwrap();

        assert_eq!(plugins.len(), 2);

        planner.settings.filters = vec![CheckFilter {
            plugin: "enabled".to_string(),
            rule_key: None,
        }];

        let plugins = enabled_plugins(&planner).unwrap();

        assert_eq!(plugins.len(), 2);
        assert_eq!(plugins[0].name, "enabled");
        assert_eq!(plugins[0].plugin.prefix, Some("prefix".to_string()));
        assert_eq!(plugins[1].name, "enabled");
        assert_eq!(plugins[1].plugin.prefix, Some("".to_string()));
    }

    #[test]
    fn test_disable_plugin_if_unsupported_platform() {
        let supported_platforms = Platform::all_values()
            .into_iter()
            .filter(|&x| x != Platform::current())
            .collect();

        let mut plugin_defs = HashMap::new();
        plugin_defs.insert(
            "enabled".to_string(),
            PluginDef {
                drivers: vec![("test".to_string(), DriverDef::default())]
                    .into_iter()
                    .collect(),
                supported_platforms: supported_platforms,
                ..Default::default()
            },
        );

        let planner = build_planner(QltyConfig {
            plugin: vec![EnabledPlugin {
                name: "enabled".to_string(),
                prefix: Some("prefix".to_string()),
                drivers: vec![ALL.to_string()],
                ..Default::default()
            }],
            plugins: PluginsConfig {
                downloads: HashMap::new(),
                releases: HashMap::new(),
                definitions: plugin_defs,
            },
            ..Default::default()
        });

        let plugins: Vec<ActivePlugin> = enabled_plugins(&planner).unwrap();

        assert!(plugins.is_empty());
    }

    #[test]
    fn test_config_plugin_prefix_package_file() {
        let mut plugin_defs = HashMap::new();
        plugin_defs.insert(
            "enabled".to_string(),
            PluginDef {
                drivers: vec![("test".to_string(), DriverDef::default())]
                    .into_iter()
                    .collect(),
                ..Default::default()
            },
        );

        let mut planner = build_planner(QltyConfig {
            plugin: vec![EnabledPlugin {
                name: "enabled".to_string(),
                prefix: Some("prefix".to_string()),
                package_file: Some("package_file".to_string()),
                ..Default::default()
            }],
            plugins: PluginsConfig {
                downloads: HashMap::new(),
                releases: HashMap::new(),
                definitions: plugin_defs,
            },
            ..Default::default()
        });

        let tempdir = tempdir().unwrap();
        let temp_path = tempdir.path().to_path_buf();
        planner.workspace.root = temp_path.clone();
        fs::create_dir(temp_path.join("prefix")).unwrap();

        let package_file_path = temp_path.join("prefix").join("package_file");
        fs::File::create(&package_file_path).unwrap();

        let plugins = enabled_plugins(&planner).unwrap();

        let enabled_plugin = plugins.iter().find(|p| p.name == "enabled").unwrap();
        assert_eq!(
            enabled_plugin.plugin.package_file,
            Some(package_file_path.to_str().unwrap().to_string())
        );
    }
}
