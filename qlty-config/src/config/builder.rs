use super::exclude::Exclude;
use crate::config::{Match, Set, Triage};
use crate::sources::SourcesList;
use crate::{warn_once, Library, QltyConfig};
use crate::{workspace::Workspace, TomlMerge};
use anyhow::{anyhow, bail, Context as _, Result};
use config::{Config, File, FileFormat};
use console::style;
use qlty_types::level_from_str;
use std::path::Path;
use toml::Value;
use tracing::{debug, trace};

const EXPECTED_CONFIG_VERSION: &str = "0";

const QLTY_TOML_PARSE_ERROR: &str = r#"There was an error reading your qlty.toml config file.

Please make sure you are using the latest version of the CLI with `qlty upgrade`.

For more information, please visit: https://qlty.io/docs/troubleshooting/qlty-toml-parse-error"#;

pub struct Builder;

impl Builder {
    pub fn default_config() -> Result<QltyConfig> {
        Self::toml_to_config(Self::defaults_toml())
    }

    pub fn project_config(workspace: &Workspace) -> Result<QltyConfig> {
        let mut toml = Self::defaults_toml();
        let qlty_toml = Self::qlty_config_toml(workspace)?;
        toml = Self::merge(toml, qlty_toml)?;
        Self::build_config(toml)
    }

    pub fn sources_config(workspace: &Workspace) -> Result<QltyConfig> {
        let mut toml = Self::defaults_toml();
        toml = Self::merge(toml, Self::qlty_config_toml(workspace)?)?;

        if let Ok(sources_config) = Self::extract_sources(toml) {
            Self::build_config(sources_config)
        } else {
            Ok(Self::default_config()?)
        }
    }

    pub fn full_config_for_workspace(workspace: &Workspace) -> Result<QltyConfig> {
        Self::full_config(
            workspace.sources_list()?.toml()?,
            Self::qlty_config_toml(workspace)?,
        )
    }

    pub fn validate_toml(path: &Path, toml: Value) -> Result<()> {
        Self::parse_toml_as_config(toml).with_context(|| {
            format!(
                "This TOML configuration file is not valid to Qlty: {}",
                path.display()
            )
        })?;
        Ok(())
    }

    fn defaults_toml() -> Value {
        include_str!("../../default.toml").parse::<Value>().unwrap()
    }

    fn extract_sources(mut toml: Value) -> Result<Value> {
        let mut new_toml = Value::Table(Default::default());

        {
            let source = toml.get_mut("source");
            if let Some(source) = source {
                if let Some(source_array) = source.as_array() {
                    if !source_array.is_empty() {
                        // should be a safe unwrap()
                        let new_table = new_toml.as_table_mut().unwrap();
                        let mut new_source_array = vec![];

                        for source in source_array {
                            new_source_array.push(source.clone());
                        }

                        new_table.insert("source".to_string(), Value::Array(new_source_array));
                    }
                }
            }
        }

        if new_toml.get("source").is_none() {
            bail!("No sources found");
        }

        Ok(new_toml)
    }

    fn merge(left: Value, right: Value) -> Result<Value> {
        if let Some(value) = left.get("config_version") {
            Self::validate_config_version(value)?;
        }

        if let Some(value) = right.get("config_version") {
            Self::validate_config_version(value)?;
        }

        Ok(TomlMerge::merge(left, right).unwrap())
    }

    fn validate_config_version(value: &Value) -> Result<()> {
        let config_version = value.as_str().expect("config_version is not a string");

        if config_version != EXPECTED_CONFIG_VERSION {
            bail!(
                "Config version mismatch. Expected {}, found {}",
                EXPECTED_CONFIG_VERSION,
                config_version
            );
        }

        Ok(())
    }

    fn build_config(toml: Value) -> Result<QltyConfig> {
        let config = Self::toml_to_config(toml)?;
        config.validate_cli_version()?;
        Ok(config)
    }

    fn qlty_config_toml(workspace: &Workspace) -> Result<Value> {
        let path = workspace.library().unwrap().qlty_config_path();
        let contents_string = Self::qlty_config_contents(workspace)?;
        let toml_value = contents_string
            .parse::<Value>()
            .with_context(|| format!("Failed to parse qlty config file at: {}", &path.display()))?;
        Self::validate_toml(&path, toml_value.clone()).with_context(|| QLTY_TOML_PARSE_ERROR)?;
        Ok(toml_value)
    }

    fn qlty_config_contents(workspace: &Workspace) -> Result<String> {
        let config_path = workspace.library()?.qlty_config_path();

        if !config_path.exists() {
            Err(anyhow!(
                "No qlty config file found. Try running `qlty init`"
            ))
        } else {
            Ok(std::fs::read_to_string(config_path)?)
        }
    }

    fn insert_ignores_from_exclude_patterns(config: &mut QltyConfig) {
        let mut all_exclude_patterns = config.exclude_patterns.clone();

        if !config.ignore_patterns.is_empty() {
            warn_once(&format!(
                "{} The `{}` field in qlty.toml is deprecated. Please use `{}` instead.",
                style("WARNING:").bold().yellow(),
                style("ignore_patterns").bold(),
                style("exclude_patterns").bold()
            ));
            all_exclude_patterns.extend(config.ignore_patterns.clone());
        }

        if !config.ignore.is_empty() {
            warn_once(&format!(
                "{} The `{}` field in qlty.toml is deprecated. Please use `{}` or `{}` instead.",
                style("WARNING:").bold().yellow(),
                style("[[ignore]]").bold(),
                style("[[exclude]]").bold(),
                style("exclude_patterns").bold()
            ));

            for ignore in &config.ignore {
                if ignore.file_patterns.is_empty() {
                    warn_once(&format!(
                        "{} The use of `{}` field in qlty.toml without {} is no longer supported. Skipping ignore without file_patterns.",
                        style("WARNING:").bold().yellow(),
                        style("[[ignore]]").bold(),
                        style("file_patterns").bold()
                    ));
                    continue;
                }

                if !ignore.file_patterns.is_empty()
                    && ignore.plugins.is_empty()
                    && ignore.rules.is_empty()
                    && ignore.levels.is_empty()
                {
                    debug!(
                        "Adding ignore with only file patterns to exclude patterns, ignore: {:#?}",
                        ignore
                    );
                    all_exclude_patterns.extend(ignore.file_patterns.clone());
                } else if !ignore.file_patterns.is_empty()
                    && !ignore.plugins.is_empty()
                    && ignore.rules.is_empty()
                    && ignore.levels.is_empty()
                {
                    debug!(
                        "Adding ignore with only file patterns and plugins to exclude, ignore: {:#?}",
                        ignore
                    );

                    config.exclude.push(Exclude {
                        file_patterns: ignore.file_patterns.clone(),
                        plugins: ignore.plugins.clone(),
                        ..Default::default()
                    });
                } else {
                    debug!(
                        "Adding ignore with more than file patterns and plugins to triage, ignore: {:#?}",
                        ignore
                    );

                    config.triage.push(Triage {
                        r#match: Match {
                            file_patterns: ignore.file_patterns.clone(),
                            plugins: ignore.plugins.clone(),
                            rules: ignore.rules.clone(),
                            levels: ignore
                                .levels
                                .iter()
                                .map(|level| level_from_str(level))
                                .collect(),
                            ..Default::default()
                        },
                        set: Set {
                            ignored: true,
                            ..Default::default()
                        },
                    })
                }
            }
        }

        if !all_exclude_patterns.is_empty() {
            config.exclude_patterns = all_exclude_patterns.clone();

            match config.coverage.ignores {
                Some(_) => {
                    config
                        .coverage
                        .ignores
                        .as_mut()
                        .unwrap()
                        .extend(all_exclude_patterns);
                }
                None => {
                    config.coverage.ignores = Some(all_exclude_patterns);
                }
            }
        }
    }

    pub fn toml_to_config(toml: Value) -> Result<QltyConfig> {
        let mut config: QltyConfig = Self::parse_toml_as_config(toml)?;
        Self::insert_ignores_from_exclude_patterns(&mut config);
        let config = Self::post_process_config(config);

        trace!("Config: {:#?}", config);
        config
    }

    fn parse_toml_as_config(toml: Value) -> Result<QltyConfig> {
        let yaml = serde_yaml::to_string(&toml).unwrap();
        let file = File::from_str(&yaml, FileFormat::Yaml);
        let builder = Config::builder().add_source(file);
        let config = builder.build()?;
        config
            .try_deserialize()
            .context("Invalid TOML configuration")
    }

    fn post_process_config(config: QltyConfig) -> Result<QltyConfig> {
        let mut config = config.clone();

        for enabled_plugin in &mut config.plugin {
            enabled_plugin.validate().with_context(|| {
                format!("Configuration error for plugin '{}'", enabled_plugin.name)
            })?;
            let plugin_definition =
                config
                    .plugins
                    .definitions
                    .get(&enabled_plugin.name)
                    .ok_or(anyhow!(
                        "Plugin definition not found for {}",
                        &enabled_plugin.name
                    ))?;

            if enabled_plugin.version == "latest" {
                let latest_version = plugin_definition.latest_version.as_ref().ok_or(anyhow!(
                    "The enabled plugin version is \"latest\", but the latest version is unknown: {}",
                    &enabled_plugin.name
                ))?;

                enabled_plugin.version = latest_version.clone();
            } else if enabled_plugin.version == "known_good" {
                let known_good_version =
                    plugin_definition
                        .known_good_version
                        .as_ref()
                        .ok_or(anyhow!(
                            "The enabled plugin version is \"known_good\", but the known good version is unknown: {}",
                            &enabled_plugin.name
                        ))?;

                enabled_plugin.version = known_good_version.clone();
            }
        }

        Ok(config)
    }

    pub fn full_config_from_toml_str(
        qlty_toml_str: &String,
        library: &Library,
    ) -> Result<QltyConfig> {
        let sources = Self::sources_list_from_qlty_toml(qlty_toml_str, library)?.toml()?;
        let qlty_config = Self::qlty_config_from_toml_string(qlty_toml_str)?;
        Self::full_config(sources, qlty_config)
    }

    fn full_config(sources: Value, qlty_config: Value) -> Result<QltyConfig> {
        let mut toml = Self::defaults_toml();
        toml = Self::merge(toml, sources)?;
        toml = Self::merge(toml, qlty_config)?;
        Self::build_config(toml)
    }

    pub fn sources_list_from_qlty_toml(
        qlty_toml_str: &String,
        library: &Library,
    ) -> Result<SourcesList> {
        Builder::sources_config_from_toml(qlty_toml_str)?.sources_list(library)
    }

    fn sources_config_from_toml(qlty_toml_str: &String) -> Result<QltyConfig> {
        let mut toml = Self::defaults_toml();
        toml = Self::merge(toml, Self::qlty_config_from_toml_string(qlty_toml_str)?)?;

        if let Ok(sources_config) = Self::extract_sources(toml) {
            Self::build_config(sources_config)
        } else {
            Ok(Self::default_config()?)
        }
    }

    fn qlty_config_from_toml_string(toml: &String) -> Result<Value> {
        let toml_value = toml
            .parse::<Value>()
            .with_context(|| format!("Failed to parse qlty config from input string: {}", &toml))?;

        Ok(toml_value)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use toml::{toml, Value::Table};

    #[test]
    fn test_extract_sources_with_only_source() {
        let input = toml! {
            random_key = "random value to be filtered out"

            [[source]]
            key2 = "value2"
            key3 = "value3"
        };

        let expected_output = toml! {
            [[source]]
            key2 = "value2"
            key3 = "value3"
        };

        let result = Builder::extract_sources(Table(input)).unwrap();
        assert_eq!(result, Table(expected_output));
    }

    #[test]
    fn test_extract_sources_with_no_sources() {
        let input = toml! {
            random_key = "random value to be filtered out"
        };

        let result = Builder::extract_sources(Table(input));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_plugin_with_mutually_exclusive_options() {
        let invalid_config = toml! {
            config_version = "0"

            [[plugin]]
            name = "rubocop"
            version = "1.56.3"
            extra_packages = ["rubocop-factory_bot@2.25.1"]
            package_file = "Gemfile"

            [plugins.definitions.rubocop]
            runtime = "ruby"
        };

        let result = Builder::toml_to_config(Table(invalid_config));
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_chain = error
            .chain()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        assert!(error_chain.contains("rubocop"));
        assert!(error_chain.contains("package_file"));
        assert!(error_chain.contains("extra_packages"));
        assert!(error_chain.contains("mutually exclusive"));
    }

    #[test]
    fn test_validate_plugin_with_package_file_only() {
        let valid_config = toml! {
            config_version = "0"

            [[plugin]]
            name = "rubocop"
            version = "1.56.3"
            package_file = "Gemfile"

            [plugins.definitions.rubocop]
            runtime = "ruby"
        };

        let result = Builder::toml_to_config(Table(valid_config));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_plugin_with_extra_packages_only() {
        let valid_config = toml! {
            config_version = "0"

            [[plugin]]
            name = "rubocop"
            version = "1.56.3"
            extra_packages = ["rubocop-factory_bot@2.25.1"]

            [plugins.definitions.rubocop]
            runtime = "ruby"
        };

        let result = Builder::toml_to_config(Table(valid_config));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_plugin_with_package_filters_but_no_package_file() {
        let invalid_config = toml! {
            config_version = "0"

            [[plugin]]
            name = "rubocop"
            version = "1.56.3"
            package_filters = ["some-filter"]

            [plugins.definitions.rubocop]
            runtime = "ruby"
        };

        let result = Builder::toml_to_config(Table(invalid_config));
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_chain = error
            .chain()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        assert!(error_chain.contains("rubocop"));
        assert!(error_chain.contains("package_filters"));
        assert!(error_chain.contains("package_file"));
        assert!(error_chain.contains("requires"));
    }

    #[test]
    fn test_validate_plugin_with_package_filters_and_package_file() {
        let valid_config = toml! {
            config_version = "0"

            [[plugin]]
            name = "rubocop"
            version = "1.56.3"
            package_file = "Gemfile"
            package_filters = ["some-filter"]

            [plugins.definitions.rubocop]
            runtime = "ruby"
        };

        let result = Builder::toml_to_config(Table(valid_config));
        assert!(result.is_ok());
    }
}
