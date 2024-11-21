use crate::config::Builder;
use crate::{QltyConfig, TomlMerge};
use anyhow::{bail, Context, Result};
use config::File;
use glob::glob;
use std::fmt::Debug;
use std::path::PathBuf;
use tracing::trace;

use super::SourcesList;

const SOURCE_PARSE_ERROR: &str = r#"There was an error reading configuration from one of your declared Sources.

Please make sure you are using the latest version of the CLI with `qlty upgrade`.

Also, please make sure you are specifying the latest source tag in your qlty.toml file.

For more information, please visit: https://qlty.io/docs/troubleshooting/source-parse-error"#;

pub trait SourceFetch: Debug + Send + Sync {
    fn fetch(&self) -> Result<()>;
    fn clone_box(&self) -> Box<dyn SourceFetch>;
    fn sources(&self) -> Vec<Box<dyn Source>> {
        vec![]
    }
}

impl Clone for Box<dyn SourceFetch> {
    fn clone(&self) -> Box<dyn SourceFetch> {
        SourceFetch::clone_box(self.as_ref())
    }
}

impl Default for Box<dyn SourceFetch> {
    fn default() -> Box<dyn SourceFetch> {
        Box::<SourcesList>::default()
    }
}

pub trait Source: SourceFetch {
    fn local_root(&self) -> PathBuf;

    fn toml(&self) -> Result<toml::Value> {
        let mut toml: toml::Value = toml::Value::Table(toml::value::Table::new());

        for path in self.paths_glob()?.iter() {
            trace!("Loading plugin config from {}", path.display());

            let contents = std::fs::read_to_string(path)
                .with_context(|| format!("Could not read {}", path.display()))?;

            let contents_toml = contents
                .parse::<toml::Value>()
                .with_context(|| format!("Could not parse {}", path.display()))?;

            Builder::validate_toml(path, contents_toml.clone())
                .with_context(|| SOURCE_PARSE_ERROR)?;
            toml = TomlMerge::merge(toml, contents_toml).unwrap();
        }

        Ok(toml)
    }

    fn build_config(&self) -> Result<QltyConfig> {
        let toml_string = toml::to_string(&self.toml()?).unwrap();
        let file = File::from_str(&toml_string, config::FileFormat::Toml);
        let builder = config::Config::builder().add_source(file);
        builder
            .build()?
            .try_deserialize()
            .context("Could not process the plugin configuration")
    }

    fn paths_glob(&self) -> Result<Vec<PathBuf>> {
        if !self.local_root().exists() {
            bail!(
                "The source directory does not exist: {}",
                self.local_root().display()
            );
        }

        Ok(glob(
            &self
                .local_root()
                .join("linters/*/plugin.toml")
                .to_string_lossy(),
        )?
        .chain(glob(
            &self
                .local_root()
                .join("plugins/linters/*/plugin.toml")
                .to_string_lossy(),
        )?)
        .flat_map(Result::ok)
        .collect::<Vec<_>>())
    }

    fn config_path_with_prefix(
        &self,
        plugin_name: &str,
        config_file: &PathBuf,
        prefix: &str,
    ) -> PathBuf {
        self.local_root()
            .join(prefix)
            .join(plugin_name)
            .join(config_file)
    }

    fn config_path(&self, plugin_name: &str, config_file: PathBuf) -> PathBuf {
        let path = self.config_path_with_prefix(plugin_name, &config_file, "plugins/linters");
        if path.exists() {
            return path;
        }
        self.config_path_with_prefix(plugin_name, &config_file, "linters")
    }

    fn clone_box(&self) -> Box<dyn Source>;
}

impl Clone for Box<dyn Source> {
    fn clone(&self) -> Box<dyn Source> {
        Source::clone_box(self.as_ref())
    }
}
