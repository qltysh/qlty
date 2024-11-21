use super::{MetricsMode, Plan, Settings};
use anyhow::Result;
use qlty_analysis::{
    code::{File, NodeFilterBuilder},
    lang,
};
use qlty_config::QltyConfig;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct Planner {
    config: QltyConfig,
    settings: Settings,
    files: Vec<Arc<File>>,
}

impl Planner {
    pub fn new(config: &QltyConfig, settings: &Settings, files: Vec<Arc<File>>) -> Self {
        Self {
            config: config.clone(),
            settings: settings.clone(),
            files,
        }
    }

    pub fn compute(&self) -> Result<Plan> {
        let mut node_filter_builders = HashMap::new();

        if self.settings.exclude_tests {
            for (language_name, language_settings) in &self.config.language {
                let language = lang::from_str(language_name).unwrap();

                let builder = NodeFilterBuilder::for_patterns(
                    language,
                    language_settings.test_syntax_patterns.to_vec(),
                );

                node_filter_builders.insert(language_name.to_string(), builder);
            }
        }

        Ok(Plan {
            mode: self.compute_mode(),
            target_mode: self.settings.target_mode.clone(),
            node_filter_builders,
            source_files: self.files.clone(),
        })
    }

    fn compute_mode(&self) -> MetricsMode {
        if self.settings.functions {
            MetricsMode::Functions
        } else {
            MetricsMode::Files
        }
    }
}
