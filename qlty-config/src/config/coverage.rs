use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
pub struct Coverage {
    pub paths: Option<Vec<String>>,
    pub ignores: Option<Vec<String>>,
}
