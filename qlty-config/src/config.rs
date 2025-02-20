mod builder;
mod coverage;
mod download;
mod file_type;
mod ignore;
pub mod ignore_group;
pub mod issue_transformer;
mod language;
mod overrides;
mod plugin;
mod release;
pub mod smells;
mod source;

pub use self::ignore::{Ignore, ALL_WILDCARD};
pub use self::overrides::Override;
use self::smells::Smells;
pub use builder::Builder;
use console::style;
pub use coverage::Coverage;
pub use download::{Cpu, DownloadDef, DownloadFileType, OperatingSystem, System};
pub use file_type::FileType;
pub use language::Language;
pub use plugin::{
    CheckTrigger, DriverBatchBy, DriverDef, DriverType, EnabledPlugin, ExtraPackage,
    InvocationDirectoryDef, InvocationDirectoryType, IssueMode, OutputDestination, OutputFormat,
    OutputMissing, PackageFileCandidate, Platform, PluginDef, PluginEnvironment, PluginFetch,
    Runtime, SuggestionMode, TargetDef, TargetType,
};
pub use release::ReleaseDef;
pub use source::SourceDef;

use crate::config::plugin::EnabledRuntimes;
pub use crate::config::plugin::PluginsConfig;
use crate::sources::SourcesList;
use crate::version::QLTY_VERSION;
use crate::Library;
use anyhow::{bail, Result};
use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
pub struct QltyConfig {
    pub config_version: Option<String>,
    pub cli_version: Option<String>,

    pub project_id: Option<String>,

    #[serde(default)]
    pub ignore: Vec<Ignore>,

    #[serde(default)]
    #[serde(rename = "override")] // Since `override` is a reserved keyword
    pub overrides: Vec<Override>,

    #[serde(default)]
    pub file_types: HashMap<String, FileType>,

    #[serde(default)]
    pub test_patterns: Vec<String>,

    #[serde(default)]
    pub coverage: Coverage,

    #[serde(default)]
    pub runtimes: EnabledRuntimes,

    #[serde(default)]
    pub plugins: PluginsConfig,

    #[serde(default)]
    pub language: HashMap<String, Language>,

    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    #[serde(default, skip_serializing)]
    pub ignore_patterns: Vec<String>,

    #[serde(default)]
    pub plugin: Vec<EnabledPlugin>,

    pub smells: Option<Smells>,

    #[serde(default)]
    pub source: Vec<SourceDef>,
}

const OLD_DEFAULT_SOURCE_REPOSITORY: &str = "https://github.com/qltysh/qlty";

impl QltyConfig {
    pub fn validate_cli_version(&self) -> Result<()> {
        if self.cli_version.is_none() {
            return Ok(());
        }

        let expected_version = Version::parse(self.cli_version.as_ref().unwrap())?;
        let actual_version = Version::parse(QLTY_VERSION)?;

        if !self.is_version_compatible(&expected_version, &actual_version) {
            if cfg!(debug_assertions) {
                debug!("qlty v{} is incompatible with the cli_version from qlty.toml ({}). Proceeding because qlty is a debug build.", actual_version, expected_version);
            } else {
                bail!("qlty v{} is incompatible with the cli_version from qlty.toml ({}). Please update qlty.", actual_version, expected_version);
            }
        }

        Ok(())
    }

    fn is_version_compatible(&self, expected: &Version, actual: &Version) -> bool {
        // Major version differences are always incompatible
        if expected.major != actual.major {
            return false;
        }

        // Prior to v1, consider minor changes to be incompatible if the expected version is greater than the actual version
        if expected.major == 0 && expected.minor > actual.minor {
            return false;
        }

        true
    }

    pub fn sources_list(&self, library: &Library) -> Result<SourcesList> {
        let mut sources_list = SourcesList::new();

        for source_def in self.source.iter() {
            sources_list.sources.push(source_def.source(library)?);
        }

        Ok(sources_list)
    }

    pub fn language_map<T>(&self, f: impl Fn(&Language) -> T) -> HashMap<String, T> {
        self.language
            .iter()
            .map(|(name, settings)| (name.clone(), f(settings)))
            .collect::<HashMap<_, _>>()
    }

    pub fn default_source(&self) -> Option<&SourceDef> {
        self.source
            .iter()
            .find(|s| s.name.as_deref() == Some("default"))
    }

    pub fn print_deprecation_warnings(&self) {
        match self.default_source() {
            Some(source) => {
                if source.repository.is_some()
                    && source
                        .repository
                        .as_ref()
                        .unwrap()
                        .starts_with(OLD_DEFAULT_SOURCE_REPOSITORY)
                {
                    warn!("qlty.toml default source is a repository-style reference to qltysh.");
                    eprintln!(
                        r#"
{} Warning: qlty.toml is using a deprecated, repository-based, default source.

Please change the default source in your qlty.toml to:

[[source]]
name = "default"
default = true
"#,
                        style("⚠").yellow()
                    );
                }
            }
            None => {
                warn!("No default source defined in qlty.toml.");
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Workspace;

    #[test]
    fn default() {
        let workspace = Workspace::new().unwrap();
        workspace.fetch_sources().unwrap();
        workspace.config().unwrap();
    }
}
