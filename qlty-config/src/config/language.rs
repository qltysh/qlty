use super::smells::Smells;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[allow(unused)]
pub struct Language {
    #[serde(default = "_default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub test_syntax_patterns: Vec<String>,

    pub smells: Option<Smells>,

    #[serde(default)]
    pub globs: Vec<String>,

    #[serde(default)]
    pub interpreters: Vec<String>,
}

const fn _default_true() -> bool {
    true
}
