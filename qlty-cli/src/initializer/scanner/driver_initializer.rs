use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use qlty_analysis::{WorkspaceEntry, WorkspaceEntryKind};
use qlty_check::{
    executor::staging_area::{Mode, StagingArea},
    planner::target::TargetFinder,
};
use qlty_config::config::{DriverDef, PluginDef, TargetType};
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    time::SystemTime,
};

use super::{DriverCandidate, Scanner};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnableReason {
    None,
    Config,
    Target,
}

pub trait DriverInitializer: Debug + Send + Sync {
    fn is_enabler(&self, path: &str) -> EnableReason;
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

    fn is_enabler(&self, path: &str) -> EnableReason {
        if self.config_globset.is_match(path) && Path::new(path).is_file() {
            EnableReason::Config
        } else {
            EnableReason::None
        }
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
    pub driver_def: DriverDef,
    pub workspace_root: PathBuf,
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
            driver_def: candidate.driver,
            workspace_root: scanner.settings.workspace.root.clone(),
        })
    }

    fn matches_target_def(&self, path: &str) -> bool {
        let workspace_entry = WorkspaceEntry {
            path: PathBuf::from(path),
            kind: WorkspaceEntryKind::File,
            // attrs below don't matter in this flow
            content_modified: SystemTime::now(),
            contents_size: 0,
            language_name: None,
        };

        let staging_area = StagingArea::new(
            self.workspace_root.clone(),
            self.workspace_root.clone(),
            Mode::ReadOnly,
        );

        let target = TargetFinder::new(staging_area, self.driver_def.target.clone())
            .resolve_target_for_entry(&workspace_entry);

        matches!(target, Ok(Some(_)))
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

    fn is_enabler(&self, path: &str) -> EnableReason {
        let is_enabled = match self.driver_def.target.target_type {
            TargetType::File => self.is_workspace_entry(path),
            _ => self.matches_target_def(path),
        };

        if is_enabled {
            EnableReason::Target
        } else {
            EnableReason::None
        }
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
