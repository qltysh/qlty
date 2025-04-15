use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FetchItem {
    pub url: String,
    pub path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Prepare {
    pub fetch: Vec<FetchItem>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CheckConfig {
    pub threshold: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Check {
    pub enabled: Option<bool>,
    pub config: Option<CheckConfig>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Checks {
    #[serde(rename = "return-statements")]
    pub return_statements: Option<Check>,

    #[serde(rename = "argument-count")]
    pub argument_count: Option<Check>,

    #[serde(rename = "nested-control-flow")]
    pub nested_control_flow: Option<Check>,

    #[serde(rename = "complex-logic")]
    pub complex_logic: Option<Check>,

    #[serde(rename = "file-lines")]
    pub file_lines: Option<Check>,

    #[serde(rename = "method-complexity")]
    pub method_complexity: Option<Check>,

    #[serde(rename = "identical-code")]
    pub identical_code: Option<Check>,

    #[serde(rename = "similar-code")]
    pub similar_code: Option<Check>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ClassicConfig {
    pub prepare: Option<Prepare>,
    pub checks: Option<Checks>,
    pub exclude_patterns: Option<Vec<String>>,
}

impl ClassicConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let classic_file = File::open(path)?;
        let classic_reader = BufReader::new(classic_file);
        let config = serde_yaml::from_reader(classic_reader)
            .with_context(|| "Error reading .codeclimate.yml")?;

        Ok(config)
    }

    pub fn fetch_items(&self) -> Vec<FetchItem> {
        self.prepare
            .as_ref()
            .map(|prepare| prepare.fetch.clone())
            .unwrap_or_default()
    }
}
