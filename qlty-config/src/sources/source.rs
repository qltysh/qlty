use super::SourcesList;
use crate::config::Builder;
use crate::{QltyConfig, TomlMerge};
use anyhow::{Context, Result};
use config::File;
use globset::{Glob, GlobSetBuilder};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use toml::Value;
use tracing::trace;

const SOURCE_PARSE_ERROR: &str = r#"There was an error reading configuration from one of your declared Sources.

Please make sure you are using the latest version of the CLI with `qlty upgrade`.

Also, please make sure you are specifying the latest source tag in your qlty.toml file.

For more information, please visit: https://qlty.io/docs/troubleshooting/source-parse-error"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub path: PathBuf,
    pub contents: String,
}

impl SourceFile {
    pub fn write_to(&self, path: &Path) -> Result<()> {
        std::fs::write(path, &self.contents).with_context(|| {
            format!(
                "Could not write the plugin configuration to {}",
                path.display()
            )
        })
    }
}

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
    fn plugin_tomls(&self) -> Result<Vec<SourceFile>> {
        let mut globset_builder = GlobSetBuilder::new();

        for pattern in &[
            "*/linters/*/plugin.toml",
            "*/plugins/linters/*/plugin.toml",
            "linters/*/plugin.toml",
            "plugins/linters/*/plugin.toml",
        ] {
            globset_builder.add(Glob::new(pattern)?);
        }

        let globset = globset_builder.build()?;

        Ok(self
            .paths()?
            .into_iter()
            .filter(|path| globset.is_match(path))
            .map(|path| {
                self.get_file(&path)
                    .with_context(|| {
                        format!(
                            "Could not read the plugin configuration from {}",
                            path.display()
                        )
                    })
                    .unwrap()
                    .unwrap()
            })
            .collect::<Vec<SourceFile>>())
    }

    fn paths(&self) -> Result<Vec<PathBuf>>;

    fn get_config_file(&self, plugin_name: &str, config_file: &Path) -> Result<Option<SourceFile>> {
        let candidates = vec![
            PathBuf::from("plugins/linters")
                .join(plugin_name)
                .join(config_file),
            PathBuf::from("linters").join(plugin_name).join(config_file),
        ];

        for candidate in candidates {
            if let Some(file) = self.get_file(&candidate)? {
                return Ok(Some(file));
            }
        }

        Ok(None)
    }

    fn get_file(&self, file_name: &Path) -> Result<Option<SourceFile>>;

    fn toml(&self) -> Result<toml::Value> {
        let mut toml: toml::Value = toml::Value::Table(toml::value::Table::new());

        for plugin_toml in self.plugin_tomls()?.iter() {
            self.add_source_file_to_toml(&mut toml, plugin_toml)?;
        }

        if let Some(source_file) = self.get_file(Path::new("source.toml"))? {
            self.add_source_file_to_toml(&mut toml, &source_file)?;
        }

        Ok(toml)
    }

    fn add_source_file_to_toml(
        &self,
        toml: &mut toml::Value,
        source_file: &SourceFile,
    ) -> Result<()> {
        trace!("Loading config toml from {}", source_file.path.display());

        let mut contents_toml = source_file
            .contents
            .parse::<toml::Value>()
            .with_context(|| format!("Could not parse {}", source_file.path.display()))?;
        self.add_context_to_exported_config_files(&mut contents_toml, source_file);

        Builder::validate_toml(&source_file.path, contents_toml.clone())
            .with_context(|| SOURCE_PARSE_ERROR)?;

        *toml = TomlMerge::merge(toml.clone(), contents_toml).unwrap();

        Ok(())
    }

    fn add_context_to_exported_config_files(
        &self,
        toml: &mut toml::Value,
        source_file: &SourceFile,
    ) -> Option<()> {
        for (_, plugin) in toml
            .as_table_mut()?
            .get_mut("plugins")?
            .as_table_mut()?
            .get_mut("definitions")?
            .as_table_mut()?
            .iter_mut()
        {
            plugin
                .get_mut("exported_config_files")?
                .as_array_mut()
                .map(|values| {
                    for value in values.iter_mut() {
                        if let Some(value_str) = value.as_str() {
                            if let Some(parent) = source_file.path.parent() {
                                *value = Value::String(
                                    parent.join(value_str).to_string_lossy().to_string(),
                                );
                            }
                        }
                    }
                });
        }

        Some(())
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

    fn clone_box(&self) -> Box<dyn Source>;
}

impl Clone for Box<dyn Source> {
    fn clone(&self) -> Box<dyn Source> {
        Source::clone_box(self.as_ref())
    }
}
