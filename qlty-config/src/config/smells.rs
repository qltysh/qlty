use super::IssueMode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, JsonSchema)]
pub struct BooleanLogic {
    #[serde(default = "_default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub threshold: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, JsonSchema)]
pub struct NestedControlFlow {
    #[serde(default = "_default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub threshold: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, JsonSchema)]
pub struct FunctionParameters {
    #[serde(default = "_default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub threshold: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, JsonSchema)]
pub struct ReturnStatements {
    #[serde(default = "_default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub threshold: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, JsonSchema)]
pub struct FileComplexity {
    #[serde(default = "_default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub threshold: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, JsonSchema)]
pub struct FunctionComplexity {
    #[serde(default = "_default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub threshold: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
pub struct IdenticalCode {
    #[serde(default = "_default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub threshold: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
pub struct SimilarCode {
    #[serde(default = "_default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub threshold: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
pub struct Duplication {
    #[serde(default)]
    pub filter_patterns: Vec<String>,

    #[serde(default)]
    pub nodes_threshold: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
pub struct Smells {
    #[serde(default)]
    pub mode: Option<IssueMode>,

    #[serde(default)]
    pub duplication: Option<Duplication>,

    #[serde(default)]
    pub boolean_logic: Option<BooleanLogic>,

    #[serde(default)]
    pub nested_control_flow: Option<NestedControlFlow>,

    #[serde(default)]
    pub function_parameters: Option<FunctionParameters>,

    #[serde(default)]
    pub return_statements: Option<ReturnStatements>,

    #[serde(default)]
    pub file_complexity: Option<FileComplexity>,

    #[serde(default)]
    pub function_complexity: Option<FunctionComplexity>,

    #[serde(default)]
    pub identical_code: Option<IdenticalCode>,

    #[serde(default)]
    pub similar_code: Option<SimilarCode>,
}

const fn _default_true() -> bool {
    true
}
