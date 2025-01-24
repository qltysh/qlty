use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
pub struct FileType {
    #[serde(default)]
    pub globs: Vec<String>,

    #[serde(default)]
    pub interpreters: Vec<String>,
}
