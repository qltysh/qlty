// @generated
// This file is @generated by prost-build.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Metadata {
    #[prost(string, tag="15")]
    pub workspace_id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub project_id: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub build_id: ::prost::alloc::string::String,
    #[prost(string, tag="13")]
    pub reference: ::prost::alloc::string::String,
    #[prost(string, optional, tag="10")]
    pub pull_request_number: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="11")]
    pub tracked_branch_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, tag="4")]
    pub revision_oid: ::prost::alloc::string::String,
    #[prost(enumeration="AnalysisResult", tag="14")]
    pub result: i32,
    #[prost(string, tag="3")]
    pub branch: ::prost::alloc::string::String,
    #[prost(bool, tag="23")]
    pub backfill: bool,
    #[prost(string, tag="5")]
    pub root_directory: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub repository_clone_url: ::prost::alloc::string::String,
    #[prost(uint32, optional, tag="12")]
    pub files_analyzed: ::core::option::Option<u32>,
    #[prost(message, optional, tag="8")]
    pub start_time: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(message, optional, tag="9")]
    pub finish_time: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(string, tag="17")]
    pub commit_message: ::prost::alloc::string::String,
    #[prost(message, optional, tag="16")]
    pub committed_at: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(string, tag="18")]
    pub committer_email: ::prost::alloc::string::String,
    #[prost(string, tag="19")]
    pub committer_name: ::prost::alloc::string::String,
    #[prost(string, tag="20")]
    pub author_email: ::prost::alloc::string::String,
    #[prost(string, tag="21")]
    pub author_name: ::prost::alloc::string::String,
    #[prost(message, optional, tag="22")]
    pub authored_at: ::core::option::Option<::pbjson_types::Timestamp>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    #[prost(string, tag="14")]
    pub workspace_id: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub project_id: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub reference: ::prost::alloc::string::String,
    #[prost(string, tag="10")]
    pub build_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="12")]
    pub build_timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(string, tag="11")]
    pub commit_sha: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(string, tag="6")]
    pub module: ::prost::alloc::string::String,
    /// Type
    #[prost(string, tag="5")]
    pub ty: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub message: ::prost::alloc::string::String,
    #[prost(string, tag="13")]
    pub details: ::prost::alloc::string::String,
    #[prost(enumeration="MessageLevel", tag="4")]
    pub level: i32,
    #[prost(map="string, string", tag="7")]
    pub tags: ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Stats {
    #[prost(string, tag="15")]
    pub workspace_id: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub project_id: ::prost::alloc::string::String,
    #[prost(string, tag="12")]
    pub reference: ::prost::alloc::string::String,
    #[prost(string, tag="13")]
    pub build_id: ::prost::alloc::string::String,
    #[prost(string, tag="14")]
    pub commit_sha: ::prost::alloc::string::String,
    #[prost(string, optional, tag="9")]
    pub pull_request_number: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="10")]
    pub tracked_branch_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag="8")]
    pub analyzed_at: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(string, tag="2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub fully_qualified_name: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub path: ::prost::alloc::string::String,
    #[prost(enumeration="ComponentType", tag="5")]
    pub kind: i32,
    #[prost(enumeration="Language", tag="6")]
    pub language: i32,
    #[prost(uint32, optional, tag="100")]
    pub files: ::core::option::Option<u32>,
    #[prost(uint32, optional, tag="101")]
    pub classes: ::core::option::Option<u32>,
    #[prost(uint32, optional, tag="102")]
    pub functions: ::core::option::Option<u32>,
    #[prost(uint32, optional, tag="103")]
    pub fields: ::core::option::Option<u32>,
    #[prost(uint32, optional, tag="104")]
    pub lines: ::core::option::Option<u32>,
    #[prost(uint32, optional, tag="105")]
    pub code_lines: ::core::option::Option<u32>,
    #[prost(uint32, optional, tag="106")]
    pub comment_lines: ::core::option::Option<u32>,
    #[prost(uint32, optional, tag="107")]
    pub blank_lines: ::core::option::Option<u32>,
    #[prost(uint32, optional, tag="108")]
    pub complexity: ::core::option::Option<u32>,
    #[prost(uint32, optional, tag="109")]
    pub cyclomatic: ::core::option::Option<u32>,
    #[prost(uint32, optional, tag="110")]
    pub lcom4: ::core::option::Option<u32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Invocation {
    #[prost(string, tag="31")]
    pub workspace_id: ::prost::alloc::string::String,
    #[prost(string, tag="26")]
    pub project_id: ::prost::alloc::string::String,
    #[prost(string, tag="27")]
    pub reference: ::prost::alloc::string::String,
    #[prost(string, tag="28")]
    pub build_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="30")]
    pub build_timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(string, tag="29")]
    pub commit_sha: ::prost::alloc::string::String,
    /// Metadata
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub qlty_cli_version: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub plugin_name: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub driver_name: ::prost::alloc::string::String,
    #[prost(string, tag="32")]
    pub prefix: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub plugin_version: ::prost::alloc::string::String,
    /// Inputs
    #[prost(enumeration="ExecutionVerb", tag="6")]
    pub verb: i32,
    #[prost(uint32, tag="7")]
    pub targets_count: u32,
    #[prost(string, repeated, tag="8")]
    pub target_paths: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag="9")]
    pub config_paths: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Execution
    #[prost(string, tag="10")]
    pub script: ::prost::alloc::string::String,
    #[prost(string, tag="11")]
    pub cwd: ::prost::alloc::string::String,
    #[prost(map="string, string", tag="12")]
    pub env: ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    /// Timing
    #[prost(message, optional, tag="13")]
    pub started_at: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(float, tag="14")]
    pub duration_secs: f32,
    /// Outputs
    #[prost(int64, optional, tag="15")]
    pub exit_code: ::core::option::Option<i64>,
    #[prost(string, tag="16")]
    pub stdout: ::prost::alloc::string::String,
    #[prost(string, tag="17")]
    pub stderr: ::prost::alloc::string::String,
    #[prost(string, optional, tag="18")]
    pub tmpfile_path: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="19")]
    pub tmpfile_contents: ::core::option::Option<::prost::alloc::string::String>,
    /// Results
    #[prost(enumeration="ExitResult", tag="21")]
    pub exit_result: i32,
    #[prost(string, optional, tag="23")]
    pub parser_error: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint32, tag="24")]
    pub issues_count: u32,
    #[prost(uint32, tag="25")]
    pub rewrites_count: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Issue {
    #[prost(string, tag="39")]
    pub workspace_id: ::prost::alloc::string::String,
    #[prost(string, tag="31")]
    pub project_id: ::prost::alloc::string::String,
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag="36")]
    pub reference: ::prost::alloc::string::String,
    #[prost(string, tag="37")]
    pub build_id: ::prost::alloc::string::String,
    #[prost(string, tag="38")]
    pub commit_sha: ::prost::alloc::string::String,
    #[prost(string, optional, tag="33")]
    pub pull_request_number: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="34")]
    pub tracked_branch_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag="32")]
    pub analyzed_at: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(string, tag="25")]
    pub tool: ::prost::alloc::string::String,
    #[prost(string, tag="26")]
    pub driver: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub rule_key: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub message: ::prost::alloc::string::String,
    #[prost(enumeration="Level", tag="24")]
    pub level: i32,
    #[prost(enumeration="Language", tag="7")]
    pub language: i32,
    #[prost(string, tag="6")]
    pub fingerprint: ::prost::alloc::string::String,
    #[prost(enumeration="Category", tag="19")]
    pub category: i32,
    #[prost(string, tag="20")]
    pub snippet: ::prost::alloc::string::String,
    #[prost(string, tag="21")]
    pub snippet_with_context: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub replacement: ::prost::alloc::string::String,
    #[prost(string, tag="30")]
    pub documentation_url: ::prost::alloc::string::String,
    #[prost(uint32, tag="10")]
    pub effort_minutes: u32,
    #[prost(uint32, tag="27")]
    pub value: u32,
    #[prost(uint32, tag="11")]
    pub value_delta: u32,
    #[prost(string, tag="12")]
    pub source_checksum: ::prost::alloc::string::String,
    #[prost(uint32, tag="13")]
    pub source_checksum_version: u32,
    #[prost(string, tag="14")]
    pub author: ::prost::alloc::string::String,
    #[prost(message, optional, tag="15")]
    pub author_time: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(string, repeated, tag="16")]
    pub tags: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, optional, tag="29")]
    pub location: ::core::option::Option<Location>,
    #[prost(message, repeated, tag="28")]
    pub other_locations: ::prost::alloc::vec::Vec<Location>,
    #[prost(message, repeated, tag="40")]
    pub suggestions: ::prost::alloc::vec::Vec<Suggestion>,
    #[prost(message, optional, tag="17")]
    pub properties: ::core::option::Option<::pbjson_types::Struct>,
    #[prost(map="string, string", tag="18")]
    pub partial_fingerprints: ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    #[prost(enumeration="Mode", tag="35")]
    pub mode: i32,
    #[prost(bool, tag="41")]
    pub on_added_line: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Suggestion {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub description: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub patch: ::prost::alloc::string::String,
    #[prost(bool, tag="8")]
    pub r#unsafe: bool,
    #[prost(enumeration="SuggestionSource", tag="7")]
    pub source: i32,
    #[prost(message, repeated, tag="6")]
    pub replacements: ::prost::alloc::vec::Vec<Replacement>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Replacement {
    #[prost(string, tag="1")]
    pub data: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub location: ::core::option::Option<Location>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Location {
    #[prost(string, tag="1")]
    pub path: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub range: ::core::option::Option<Range>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Range {
    #[prost(uint32, tag="1")]
    pub start_line: u32,
    #[prost(uint32, tag="2")]
    pub start_column: u32,
    #[prost(uint32, tag="3")]
    pub end_line: u32,
    #[prost(uint32, tag="4")]
    pub end_column: u32,
    #[prost(uint32, optional, tag="5")]
    pub start_byte: ::core::option::Option<u32>,
    #[prost(uint32, optional, tag="6")]
    pub end_byte: ::core::option::Option<u32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ExecutionVerb {
    Unspecified = 0,
    Check = 1,
    Fmt = 2,
    Validate = 3,
}
impl ExecutionVerb {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ExecutionVerb::Unspecified => "EXECUTION_VERB_UNSPECIFIED",
            ExecutionVerb::Check => "EXECUTION_VERB_CHECK",
            ExecutionVerb::Fmt => "EXECUTION_VERB_FMT",
            ExecutionVerb::Validate => "EXECUTION_VERB_VALIDATE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "EXECUTION_VERB_UNSPECIFIED" => Some(Self::Unspecified),
            "EXECUTION_VERB_CHECK" => Some(Self::Check),
            "EXECUTION_VERB_FMT" => Some(Self::Fmt),
            "EXECUTION_VERB_VALIDATE" => Some(Self::Validate),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Mode {
    Unspecified = 0,
    Block = 1,
    Comment = 2,
    Monitor = 3,
    Disabled = 4,
}
impl Mode {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Mode::Unspecified => "MODE_UNSPECIFIED",
            Mode::Block => "MODE_BLOCK",
            Mode::Comment => "MODE_COMMENT",
            Mode::Monitor => "MODE_MONITOR",
            Mode::Disabled => "MODE_DISABLED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "MODE_UNSPECIFIED" => Some(Self::Unspecified),
            "MODE_BLOCK" => Some(Self::Block),
            "MODE_COMMENT" => Some(Self::Comment),
            "MODE_MONITOR" => Some(Self::Monitor),
            "MODE_DISABLED" => Some(Self::Disabled),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SuggestionSource {
    Unspecified = 0,
    Tool = 1,
    Llm = 2,
}
impl SuggestionSource {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SuggestionSource::Unspecified => "SUGGESTION_SOURCE_UNSPECIFIED",
            SuggestionSource::Tool => "SUGGESTION_SOURCE_TOOL",
            SuggestionSource::Llm => "SUGGESTION_SOURCE_LLM",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SUGGESTION_SOURCE_UNSPECIFIED" => Some(Self::Unspecified),
            "SUGGESTION_SOURCE_TOOL" => Some(Self::Tool),
            "SUGGESTION_SOURCE_LLM" => Some(Self::Llm),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum MessageLevel {
    Unspecified = 0,
    Debug = 1,
    Info = 2,
    Warning = 3,
    Error = 4,
    Fatal = 5,
}
impl MessageLevel {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            MessageLevel::Unspecified => "MESSAGE_LEVEL_UNSPECIFIED",
            MessageLevel::Debug => "MESSAGE_LEVEL_DEBUG",
            MessageLevel::Info => "MESSAGE_LEVEL_INFO",
            MessageLevel::Warning => "MESSAGE_LEVEL_WARNING",
            MessageLevel::Error => "MESSAGE_LEVEL_ERROR",
            MessageLevel::Fatal => "MESSAGE_LEVEL_FATAL",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "MESSAGE_LEVEL_UNSPECIFIED" => Some(Self::Unspecified),
            "MESSAGE_LEVEL_DEBUG" => Some(Self::Debug),
            "MESSAGE_LEVEL_INFO" => Some(Self::Info),
            "MESSAGE_LEVEL_WARNING" => Some(Self::Warning),
            "MESSAGE_LEVEL_ERROR" => Some(Self::Error),
            "MESSAGE_LEVEL_FATAL" => Some(Self::Fatal),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Level {
    Unspecified = 0,
    Note = 10,
    Fmt = 20,
    Low = 30,
    Medium = 40,
    High = 50,
}
impl Level {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Level::Unspecified => "LEVEL_UNSPECIFIED",
            Level::Note => "LEVEL_NOTE",
            Level::Fmt => "LEVEL_FMT",
            Level::Low => "LEVEL_LOW",
            Level::Medium => "LEVEL_MEDIUM",
            Level::High => "LEVEL_HIGH",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "LEVEL_UNSPECIFIED" => Some(Self::Unspecified),
            "LEVEL_NOTE" => Some(Self::Note),
            "LEVEL_FMT" => Some(Self::Fmt),
            "LEVEL_LOW" => Some(Self::Low),
            "LEVEL_MEDIUM" => Some(Self::Medium),
            "LEVEL_HIGH" => Some(Self::High),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Category {
    Unspecified = 0,
    Bug = 1,
    Vulnerability = 2,
    Structure = 3,
    Duplication = 4,
    SecurityHotspot = 5,
    Performance = 6,
    Documentation = 7,
    TypeCheck = 8,
    Style = 9,
    AntiPattern = 10,
    Accessibility = 11,
    DeadCode = 12,
    Lint = 13,
    Secret = 14,
    DependencyAlert = 15,
}
impl Category {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Category::Unspecified => "CATEGORY_UNSPECIFIED",
            Category::Bug => "CATEGORY_BUG",
            Category::Vulnerability => "CATEGORY_VULNERABILITY",
            Category::Structure => "CATEGORY_STRUCTURE",
            Category::Duplication => "CATEGORY_DUPLICATION",
            Category::SecurityHotspot => "CATEGORY_SECURITY_HOTSPOT",
            Category::Performance => "CATEGORY_PERFORMANCE",
            Category::Documentation => "CATEGORY_DOCUMENTATION",
            Category::TypeCheck => "CATEGORY_TYPE_CHECK",
            Category::Style => "CATEGORY_STYLE",
            Category::AntiPattern => "CATEGORY_ANTI_PATTERN",
            Category::Accessibility => "CATEGORY_ACCESSIBILITY",
            Category::DeadCode => "CATEGORY_DEAD_CODE",
            Category::Lint => "CATEGORY_LINT",
            Category::Secret => "CATEGORY_SECRET",
            Category::DependencyAlert => "CATEGORY_DEPENDENCY_ALERT",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CATEGORY_UNSPECIFIED" => Some(Self::Unspecified),
            "CATEGORY_BUG" => Some(Self::Bug),
            "CATEGORY_VULNERABILITY" => Some(Self::Vulnerability),
            "CATEGORY_STRUCTURE" => Some(Self::Structure),
            "CATEGORY_DUPLICATION" => Some(Self::Duplication),
            "CATEGORY_SECURITY_HOTSPOT" => Some(Self::SecurityHotspot),
            "CATEGORY_PERFORMANCE" => Some(Self::Performance),
            "CATEGORY_DOCUMENTATION" => Some(Self::Documentation),
            "CATEGORY_TYPE_CHECK" => Some(Self::TypeCheck),
            "CATEGORY_STYLE" => Some(Self::Style),
            "CATEGORY_ANTI_PATTERN" => Some(Self::AntiPattern),
            "CATEGORY_ACCESSIBILITY" => Some(Self::Accessibility),
            "CATEGORY_DEAD_CODE" => Some(Self::DeadCode),
            "CATEGORY_LINT" => Some(Self::Lint),
            "CATEGORY_SECRET" => Some(Self::Secret),
            "CATEGORY_DEPENDENCY_ALERT" => Some(Self::DependencyAlert),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum AnalysisResult {
    Unspecified = 0,
    Success = 1,
    Error = 2,
}
impl AnalysisResult {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            AnalysisResult::Unspecified => "ANALYSIS_RESULT_UNSPECIFIED",
            AnalysisResult::Success => "ANALYSIS_RESULT_SUCCESS",
            AnalysisResult::Error => "ANALYSIS_RESULT_ERROR",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ANALYSIS_RESULT_UNSPECIFIED" => Some(Self::Unspecified),
            "ANALYSIS_RESULT_SUCCESS" => Some(Self::Success),
            "ANALYSIS_RESULT_ERROR" => Some(Self::Error),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ExitResult {
    Unspecified = 0,
    Success = 1,
    KnownError = 2,
    UnknownError = 3,
    NoIssues = 4,
}
impl ExitResult {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ExitResult::Unspecified => "EXIT_RESULT_UNSPECIFIED",
            ExitResult::Success => "EXIT_RESULT_SUCCESS",
            ExitResult::KnownError => "EXIT_RESULT_KNOWN_ERROR",
            ExitResult::UnknownError => "EXIT_RESULT_UNKNOWN_ERROR",
            ExitResult::NoIssues => "EXIT_RESULT_NO_ISSUES",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "EXIT_RESULT_UNSPECIFIED" => Some(Self::Unspecified),
            "EXIT_RESULT_SUCCESS" => Some(Self::Success),
            "EXIT_RESULT_KNOWN_ERROR" => Some(Self::KnownError),
            "EXIT_RESULT_UNKNOWN_ERROR" => Some(Self::UnknownError),
            "EXIT_RESULT_NO_ISSUES" => Some(Self::NoIssues),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ComponentType {
    Unspecified = 0,
    File = 1,
    Directory = 2,
    Project = 3,
    Module = 5,
    Class = 6,
    Function = 7,
}
impl ComponentType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ComponentType::Unspecified => "COMPONENT_TYPE_UNSPECIFIED",
            ComponentType::File => "COMPONENT_TYPE_FILE",
            ComponentType::Directory => "COMPONENT_TYPE_DIRECTORY",
            ComponentType::Project => "COMPONENT_TYPE_PROJECT",
            ComponentType::Module => "COMPONENT_TYPE_MODULE",
            ComponentType::Class => "COMPONENT_TYPE_CLASS",
            ComponentType::Function => "COMPONENT_TYPE_FUNCTION",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "COMPONENT_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "COMPONENT_TYPE_FILE" => Some(Self::File),
            "COMPONENT_TYPE_DIRECTORY" => Some(Self::Directory),
            "COMPONENT_TYPE_PROJECT" => Some(Self::Project),
            "COMPONENT_TYPE_MODULE" => Some(Self::Module),
            "COMPONENT_TYPE_CLASS" => Some(Self::Class),
            "COMPONENT_TYPE_FUNCTION" => Some(Self::Function),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Language {
    Unspecified = 0,
    Unknown = 1,
    Java = 2,
    Javascript = 3,
    Typescript = 4,
    Python = 5,
    Ruby = 6,
    Jsx = 7,
    Tsx = 8,
    Go = 9,
    Rust = 10,
    Kotlin = 11,
    Php = 12,
    CSharp = 13,
}
impl Language {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Language::Unspecified => "LANGUAGE_UNSPECIFIED",
            Language::Unknown => "LANGUAGE_UNKNOWN",
            Language::Java => "LANGUAGE_JAVA",
            Language::Javascript => "LANGUAGE_JAVASCRIPT",
            Language::Typescript => "LANGUAGE_TYPESCRIPT",
            Language::Python => "LANGUAGE_PYTHON",
            Language::Ruby => "LANGUAGE_RUBY",
            Language::Jsx => "LANGUAGE_JSX",
            Language::Tsx => "LANGUAGE_TSX",
            Language::Go => "LANGUAGE_GO",
            Language::Rust => "LANGUAGE_RUST",
            Language::Kotlin => "LANGUAGE_KOTLIN",
            Language::Php => "LANGUAGE_PHP",
            Language::CSharp => "LANGUAGE_C_SHARP",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "LANGUAGE_UNSPECIFIED" => Some(Self::Unspecified),
            "LANGUAGE_UNKNOWN" => Some(Self::Unknown),
            "LANGUAGE_JAVA" => Some(Self::Java),
            "LANGUAGE_JAVASCRIPT" => Some(Self::Javascript),
            "LANGUAGE_TYPESCRIPT" => Some(Self::Typescript),
            "LANGUAGE_PYTHON" => Some(Self::Python),
            "LANGUAGE_RUBY" => Some(Self::Ruby),
            "LANGUAGE_JSX" => Some(Self::Jsx),
            "LANGUAGE_TSX" => Some(Self::Tsx),
            "LANGUAGE_GO" => Some(Self::Go),
            "LANGUAGE_RUST" => Some(Self::Rust),
            "LANGUAGE_KOTLIN" => Some(Self::Kotlin),
            "LANGUAGE_PHP" => Some(Self::Php),
            "LANGUAGE_C_SHARP" => Some(Self::CSharp),
            _ => None,
        }
    }
}
include!("qlty.analysis.v1.serde.rs");
// @@protoc_insertion_point(module)