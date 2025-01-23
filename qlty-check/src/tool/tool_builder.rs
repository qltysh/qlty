use anyhow::{Context, Result};
use qlty_config::{
    config::{PluginDef, Runtime},
    QltyConfig,
};

use crate::Tool;

use super::{
    download::{Download, DownloadTool},
    github::{GitHubRelease, GitHubReleaseTool},
    go, java, node,
    null_tool::NullTool,
    php, python, ruby, rust, RuntimeTool,
};

#[derive(Debug, Clone, Copy)]
pub struct ToolBuilder<'a> {
    config: &'a QltyConfig,
    plugin_name: &'a str,
    plugin: &'a PluginDef,
}

impl ToolBuilder<'_> {
    pub fn new<'a>(
        config: &'a QltyConfig,
        plugin_name: &'a str,
        plugin: &'a PluginDef,
    ) -> ToolBuilder<'a> {
        ToolBuilder {
            config,
            plugin_name,
            plugin,
        }
    }

    fn build_runtime_tool(&self, runtime: &Runtime) -> Result<Box<dyn Tool>> {
        let runtime_version = self
            .config
            .runtimes
            .enabled
            .get(runtime)
            .cloned()
            .unwrap_or(
                Self::latest_runtime_version(runtime)
                    .with_context(|| format!("Runtime not found: {}", runtime))?,
            );

        let runtime = Self::runtime_tool(runtime.to_owned(), &runtime_version);
        let package = runtime.package_tool(self.plugin_name, self.plugin);

        Ok(package)
    }

    fn build_release_tool(
        &self,
        release_name: &str,
        plugin_version: &str,
    ) -> Result<Box<dyn Tool>> {
        let release_def = self
            .config
            .plugins
            .releases
            .get(release_name)
            .with_context(|| {
                format!(
                    "No release definition for plugin {:?} with name {:?}",
                    self.plugin_name, release_name
                )
            })?;

        let runtime = if let Some(runtime) = &self.plugin.runtime {
            let runtime_version = self
                .config
                .runtimes
                .enabled
                .get(runtime)
                .cloned()
                .unwrap_or(
                    Self::latest_runtime_version(runtime)
                        .with_context(|| format!("Runtime not found: {}", runtime))?,
                );

            Some(Self::release_runtime_tool(
                runtime.to_owned(),
                &runtime_version,
            ))
        } else {
            None
        };

        Ok(Box::new(GitHubReleaseTool {
            plugin_name: self.plugin_name.to_string(),
            release: GitHubRelease::new(plugin_version.to_string(), release_def.clone()),
            plugin: self.plugin.clone(),
            runtime,
            ..Default::default()
        }))
    }

    fn build_download_tool(
        &self,
        download_name: &str,
        plugin_version: &str,
    ) -> Result<Box<dyn Tool>> {
        let download_def = self
            .config
            .plugins
            .downloads
            .get(download_name)
            .with_context(|| {
                format!(
                    "No download definition for plugin {:?} with name {:?}",
                    self.plugin_name, download_name
                )
            })?;

        Ok(Box::new(DownloadTool {
            plugin_name: self.plugin_name.to_string(),
            download: Download::new(download_def, download_name, plugin_version),
            plugin: self.plugin.clone(),
        }))
    }
    pub fn build_tool(&self) -> Result<Box<dyn Tool>> {
        let plugin_version = self
            .plugin
            .version
            .clone()
            .with_context(|| format!("No version for plugin {:?}", self.plugin_name))?;

        if let Some(release_name) = self.plugin.releases.first() {
            self.build_release_tool(release_name, &plugin_version)
        } else if let Some(runtime) = &self.plugin.runtime {
            self.build_runtime_tool(runtime)
        } else if let Some(download_name) = self.plugin.downloads.first() {
            self.build_download_tool(download_name, &plugin_version)
        } else {
            Ok(Box::new(NullTool {
                plugin_name: self.plugin_name.to_string(),
                plugin: self.plugin.clone(),
            }))
        }
    }

    fn latest_runtime_version(runtime: &Runtime) -> Option<String> {
        match runtime {
            Runtime::Go => Some("1.22.0".to_owned()),
            Runtime::Node => Some("21.7.3".to_owned()),
            Runtime::Python => Some("3.11.7".to_owned()),
            Runtime::Ruby => Some("3.3.7".to_owned()),
            Runtime::Rust => Some("1.77.2".to_owned()),
            Runtime::Java => Some("22.0.1+8".to_owned()),
            Runtime::Php => Some("8.3.7".to_owned()),
        }
    }

    fn runtime_tool(runtime: Runtime, version: &str) -> Box<dyn RuntimeTool> {
        match runtime {
            Runtime::Node => Box::new(node::NodeJS {
                version: version.to_string(),
            }),
            Runtime::Python => Box::new(python::Python {
                version: version.to_string(),
            }),
            Runtime::Ruby => ruby::Ruby::new_runtime(version),
            Runtime::Go => Box::new(go::Go {
                version: version.to_string(),
            }),
            Runtime::Rust => Box::new(rust::Rust {
                version: version.to_string(),
            }),
            Runtime::Java => Box::new(java::Java {
                version: version.to_string(),
            }),
            Runtime::Php => Box::new(php::Php {
                version: version.to_string(),
            }),
        }
    }

    // Since can't cast Box<dyn RuntimeTool> into Box<dyn Tool> directly, we need to
    fn release_runtime_tool(runtime: Runtime, version: &str) -> Box<dyn Tool> {
        match runtime {
            Runtime::Node => Box::new(node::NodeJS {
                version: version.to_string(),
            }),
            Runtime::Python => Box::new(python::Python {
                version: version.to_string(),
            }),
            Runtime::Ruby => ruby::Ruby::new_tool(version),
            Runtime::Go => Box::new(go::Go {
                version: version.to_string(),
            }),
            Runtime::Rust => Box::new(rust::Rust {
                version: version.to_string(),
            }),
            Runtime::Java => Box::new(java::Java {
                version: version.to_string(),
            }),
            Runtime::Php => Box::new(php::Php {
                version: version.to_string(),
            }),
        }
    }
}
