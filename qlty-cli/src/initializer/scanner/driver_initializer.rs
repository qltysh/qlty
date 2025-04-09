use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use qlty_config::config::{DriverDef, PluginDef};
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use super::{DriverCandidate, Scanner};

pub trait DriverInitializer: Debug + Send + Sync {
    fn is_enabler(&self, path: &str) -> bool;
    fn is_workspace_entry(&self, path: &str) -> bool;
    fn clone_box(&self) -> Box<dyn DriverInitializer>;
    fn driver_name(&self) -> String;
    fn version(&self) -> String;
    fn key(&self) -> String;
}

impl Clone for Box<dyn DriverInitializer> {
    fn clone(&self) -> Box<dyn DriverInitializer> {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct ConfigDriver {
    pub workspace_entry_globset: GlobSet,
    pub config_globset: GlobSet,
    pub key: String,
    pub driver_name: String,
    pub version: String,
}

impl ConfigDriver {
    pub fn new(
        candidate: DriverCandidate,
        plugin_def: &PluginDef,
        scanner: &Scanner,
    ) -> Result<Self> {
        let workspace_entry_globset =
            build_workspace_entries_globset(scanner, plugin_def, &candidate.driver)?;

        Ok(ConfigDriver {
            workspace_entry_globset,
            config_globset: Self::build_configs_globset(plugin_def, &candidate.driver).unwrap(),
            key: candidate.key,
            driver_name: candidate.name,
            version: candidate.version,
        })
    }

    fn build_configs_globset(plugin_def: &PluginDef, driver_def: &DriverDef) -> Result<GlobSet> {
        let driver_config_files = if !driver_def.config_files.is_empty() {
            driver_def.config_files.clone()
        } else {
            plugin_def.config_files.clone()
        };

        let mut globset_builder = GlobSetBuilder::new();

        for config_file in &driver_config_files {
            let glob = PathBuf::from("**").join(config_file);
            globset_builder.add(Glob::new(&glob.to_string_lossy()).unwrap());
        }

        globset_builder.build().map_err(Into::into)
    }
}

impl DriverInitializer for ConfigDriver {
    fn driver_name(&self) -> String {
        self.driver_name.to_string()
    }

    fn version(&self) -> String {
        self.version.to_string()
    }

    fn key(&self) -> String {
        self.key.to_string()
    }

    fn is_enabler(&self, path: &str) -> bool {
        self.config_globset.is_match(path) && Path::new(path).is_file()
    }

    fn is_workspace_entry(&self, path: &str) -> bool {
        self.workspace_entry_globset.is_match(path)
    }

    fn clone_box(&self) -> Box<dyn DriverInitializer> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct TargetDriver {
    pub workspace_entries_globset: GlobSet,
    pub key: String,
    pub driver_name: String,
    pub version: String,
}

impl TargetDriver {
    pub fn new(
        candidate: DriverCandidate,
        plugin_def: &PluginDef,
        scanner: &Scanner,
    ) -> Result<Self> {
        let workspace_entries_globset =
            build_workspace_entries_globset(scanner, plugin_def, &candidate.driver)?;

        Ok(TargetDriver {
            workspace_entries_globset,
            key: candidate.key,
            driver_name: candidate.name,
            version: candidate.version,
        })
    }
}

impl DriverInitializer for TargetDriver {
    fn driver_name(&self) -> String {
        self.driver_name.to_string()
    }

    fn version(&self) -> String {
        self.version.to_string()
    }

    fn key(&self) -> String {
        self.key.to_string()
    }

    fn is_enabler(&self, path: &str) -> bool {
        self.is_workspace_entry(path)
    }

    fn is_workspace_entry(&self, path: &str) -> bool {
        self.workspace_entries_globset.is_match(path)
    }

    fn clone_box(&self) -> Box<dyn DriverInitializer> {
        Box::new(self.clone())
    }
}

fn build_workspace_entries_globset(
    scanner: &Scanner,
    plugin_def: &PluginDef,
    driver_def: &DriverDef,
) -> Result<GlobSet> {
    let mut workspace_entry_globset_builder = GlobSetBuilder::new();

    let driver_file_types = driver_def
        .file_types
        .clone()
        .unwrap_or_else(|| plugin_def.file_types.clone());

    for file_type_name in &driver_file_types {
        if let Some(file_type) = scanner.sources_only_config.file_types.get(file_type_name) {
            for glob in &file_type.globs {
                workspace_entry_globset_builder.add(Glob::new(glob).unwrap());
            }
        } else if let Some(file_type) = scanner.default_config.file_types.get(file_type_name) {
            for glob in &file_type.globs {
                workspace_entry_globset_builder.add(Glob::new(glob).unwrap());
            }
        }
    }

    workspace_entry_globset_builder.build().map_err(Into::into)
}
