mod checks;
mod classic;
mod prepare;
use crate::QltyConfig;
use anyhow::Result;
use checks::CheckMigration;
use classic::ClassicConfig;
use prepare::get_plugins_fetch_items;
use std::{
    fs::{self},
    path::{Path, PathBuf},
};
use toml_edit::{array, table, value, DocumentMut, Item, Table};

#[derive(Default, Debug, Clone)]
pub struct MigrationSettings {
    pub root_path: PathBuf,
    pub qlty_config: QltyConfig,
    pub qlty_config_path: PathBuf,
    pub classic_config_path: PathBuf,
    pub dry_run: bool,
}

impl MigrationSettings {
    pub fn new(
        root_path: &Path,
        qlty_config: QltyConfig,
        qlty_config_path: &Path,
        classic_config_path: &Path,
        dry_run: bool,
    ) -> Result<Self> {
        Ok(Self {
            root_path: root_path.to_path_buf(),
            qlty_config,
            qlty_config_path: qlty_config_path.to_path_buf(),
            classic_config_path: classic_config_path.to_path_buf(),
            dry_run,
        })
    }
}

#[derive(Default, Debug, Clone)]
pub struct MigrateConfig {
    settings: MigrationSettings,
    document: DocumentMut,
}

impl MigrateConfig {
    pub fn new(settings: MigrationSettings) -> Result<Self> {
        Ok(Self {
            settings: settings.clone(),
            ..Default::default()
        })
    }

    pub fn migrate(&mut self) -> Result<()> {
        self.init()?;
        self.apply_migrations()?;
        self.finish()?;

        Ok(())
    }

    fn load_document_with_qlty_toml(&mut self) -> Result<()> {
        let base_contents = fs::read_to_string(self.settings.qlty_config_path.clone())?;
        self.document = base_contents
            .parse::<DocumentMut>()
            .expect("Invalid config doc");
        Ok(())
    }

    fn init(&mut self) -> Result<()> {
        self.load_document_with_qlty_toml()
    }

    fn write_to_output(&self) -> Result<()> {
        if self.settings.dry_run {
            println!("{}", self.document);
            return Ok(());
        } else {
            fs::write(
                self.settings.qlty_config_path.clone(),
                self.document.to_string(),
            )?;
        }

        Ok(())
    }

    fn finish(&self) -> Result<()> {
        self.write_to_output()
    }

    fn migrate_prepare_statement(&mut self, classic_config: &ClassicConfig) -> Result<()> {
        if self.document.get("plugin").is_none() {
            return Ok(());
        }

        let fetch_items_map = get_plugins_fetch_items(classic_config, &self.settings.qlty_config)?;

        for (plugin_name, fetch_items) in fetch_items_map {
            let plugin_table = self
                .document
                .get_mut("plugin")
                .unwrap()
                .as_array_of_tables_mut()
                .unwrap()
                .iter_mut()
                .find(|table| table.get("name").unwrap().as_str().unwrap() == plugin_name);

            if plugin_table.is_none() {
                continue;
            }
            let plugin_table: &mut toml_edit::Table = plugin_table.unwrap();

            if plugin_table.get("fetch").is_none() {
                plugin_table["fetch"] = array();
            }

            let plugin_table = plugin_table["fetch"].as_array_of_tables_mut().unwrap();

            for fetch_item in fetch_items {
                let mut plugin_fetch = table();
                plugin_fetch["url"] = value(fetch_item.url);
                plugin_fetch["path"] = value(fetch_item.path.to_str().unwrap());

                plugin_table.push(plugin_fetch.as_table().unwrap().clone());
            }
        }

        Ok(())
    }

    fn migrate_checks(&mut self, classic_config: &ClassicConfig) -> Result<()> {
        let smells_table = if let Some(smells) = self.document.get_mut("smells") {
            smells.as_table_mut().unwrap()
        } else {
            let mut empty_smells_table = Table::new();
            empty_smells_table.set_implicit(true);
            self.document["smells"] = Item::Table(empty_smells_table);
            self.document
                .get_mut("smells")
                .unwrap()
                .as_table_mut()
                .unwrap()
        };

        CheckMigration::migrate_maintainability_checks(classic_config, smells_table)?;

        Ok(())
    }

    fn migrate_exclude_patterns(&mut self, classic_config: &ClassicConfig) -> Result<()> {
        if let Some(classic_exclude_patterns) = &classic_config.exclude_patterns {
            let target_array = self
                .document
                .get_mut("exclude_patterns")
                .and_then(|item| item.as_array_mut());

            match target_array {
                Some(existing_array) => {
                    for pattern in classic_exclude_patterns {
                        existing_array.push(pattern);
                    }
                }
                None => {
                    let mut new_array = toml_edit::Array::new();
                    for pattern in classic_exclude_patterns {
                        new_array.push(pattern);
                    }
                    self.document["exclude_patterns"] =
                        toml_edit::Item::Value(toml_edit::Value::Array(new_array));
                }
            }
        }

        Ok(())
    }

    fn apply_migrations(&mut self) -> Result<()> {
        let classic_config = ClassicConfig::load(self.settings.classic_config_path.as_path())?;
        self.migrate_prepare_statement(&classic_config)?;
        self.migrate_checks(&classic_config)?;
        self.migrate_exclude_patterns(&classic_config)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml_edit::DocumentMut;

    fn basic_settings() -> MigrationSettings {
        MigrationSettings {
            root_path: PathBuf::from("."),
            qlty_config: QltyConfig::default(),
            qlty_config_path: PathBuf::from("qlty.toml"),
            classic_config_path: PathBuf::from("classic.toml"),
            dry_run: true,
        }
    }

    #[test]
    fn test_migrate_with_no_existing_excludes() {
        let doc = "[settings]\n".parse::<DocumentMut>().unwrap();

        let config = ClassicConfig {
            exclude_patterns: Some(vec!["target/".to_string(), "node_modules/".to_string()]),
            ..Default::default()
        };

        let settings = basic_settings();
        let mut migrator = MigrateConfig {
            settings,
            document: doc,
        };

        migrator.migrate_exclude_patterns(&config).unwrap();

        let arr = migrator
            .document
            .get("exclude_patterns")
            .and_then(|item| item.as_array())
            .unwrap();

        let values: Vec<_> = arr.iter().map(|v| v.as_str().unwrap()).collect();
        assert_eq!(values, vec!["target/", "node_modules/"]);
    }

    #[test]
    fn test_migrate_appends_to_existing_excludes() {
        let doc = r#"
exclude_patterns = ["dist/", "build/"]
"#
        .parse::<DocumentMut>()
        .unwrap();

        let config = ClassicConfig {
            exclude_patterns: Some(vec!["target/".to_string()]),
            ..Default::default()
        };

        let settings = basic_settings();
        let mut migrator = MigrateConfig {
            settings,
            document: doc,
        };

        migrator.migrate_exclude_patterns(&config).unwrap();

        let arr = migrator
            .document
            .get("exclude_patterns")
            .and_then(|item| item.as_array())
            .unwrap();

        let values: Vec<_> = arr.iter().map(|v| v.as_str().unwrap()).collect();
        assert_eq!(values, vec!["dist/", "build/", "target/"]);
    }

    #[test]
    fn test_migrate_with_no_patterns_does_nothing() {
        let doc = "[settings]\n".parse::<DocumentMut>().unwrap();

        let config = ClassicConfig {
            exclude_patterns: None,
            ..Default::default()
        };

        let settings = basic_settings();
        let mut migrator = MigrateConfig {
            settings,
            document: doc.clone(),
        };

        migrator.migrate_exclude_patterns(&config).unwrap();

        assert_eq!(migrator.document.to_string(), doc.to_string());
    }
}
