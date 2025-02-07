use super::{Language, ReleaseDef};
use crate::config::DownloadDef;
use crate::QltyConfig;
use anyhow::{Context, Result};
use qlty_types::analysis::v1::{Category, Level};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Write as _;
use std::path::PathBuf;

const SEMVER_REGEX: &str = r"(\d+\.\d+\.\d+)";
const ALL: &str = "ALL";

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
pub struct PluginsConfig {
    #[serde(default)]
    pub downloads: HashMap<String, DownloadDef>,

    #[serde(default)]
    pub releases: HashMap<String, ReleaseDef>,

    #[serde(default)]
    pub definitions: HashMap<String, PluginDef>,
}

fn semver_regex() -> String {
    SEMVER_REGEX.to_string()
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct DriverDef {
    #[serde(default)]
    pub script: String,

    #[serde(default)]
    pub output: OutputDestination,

    #[serde(default)]
    pub output_format: OutputFormat,

    #[serde(default)]
    pub output_missing: OutputMissing,

    pub output_regex: Option<String>,

    #[serde(default)]
    pub output_level: Option<OutputLevel>,

    #[serde(default)]
    pub output_category: Option<OutputCategory>,

    #[serde(default)]
    pub driver_type: DriverType,

    #[serde(default)]
    pub batch: bool,

    #[serde(default = "default_max_batch")]
    pub max_batch: usize,

    #[serde(default)]
    pub success_codes: Vec<i64>,

    #[serde(default)]
    pub no_issue_codes: Vec<i64>,

    #[serde(default)]
    pub error_codes: Vec<i64>,

    #[serde(default)]
    pub cache_results: bool,

    pub file_types: Option<Vec<String>>,

    #[serde(default)]
    pub target: TargetDef,

    #[serde(default)]
    #[serde(rename = "runs_from")]
    pub invocation_directory_def: InvocationDirectoryDef,

    #[serde(default)]
    pub prepare_script: Option<String>,

    #[serde(default)]
    pub skip_upstream: bool,

    #[serde(default)]
    pub version: Vec<DriverDef>,

    #[serde(default)]
    pub version_matcher: Option<String>,

    #[serde(default)]
    pub copy_configs_into_tool_install: bool,

    #[serde(default)]
    pub config_files: Vec<PathBuf>,

    #[serde(default)]
    pub suggested: SuggestionMode,

    /// The latest validated version of the driver
    #[serde(default)]
    pub known_good_version: Option<String>,

    #[serde(default)]
    pub batch_by: DriverBatchBy,

    #[serde(default = "default_driver_timeout")]
    pub timeout: u64,

    #[serde(default)]
    pub autoload_script: Option<String>,

    #[serde(default)]
    pub config_script: Option<String>,
}

fn default_driver_timeout() -> u64 {
    600
}

fn default_max_batch() -> usize {
    64
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default, JsonSchema)]
pub enum OutputMissing {
    /// Raise an error if the output is missing
    #[default]
    #[serde(rename = "error")]
    Error,

    /// Interpret no output as no issues
    #[serde(rename = "no_issues")]
    NoIssues,

    /// Hand the empty output to the parser for processing
    #[serde(rename = "parse")]
    Parse,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default, JsonSchema)]
pub enum DriverBatchBy {
    #[default]
    #[serde(rename = "none")]
    None,

    #[serde(rename = "invocation_directory")]
    InvocationDirectory,

    #[serde(rename = "config_file")]
    ConfigFile,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default, JsonSchema)]
pub enum InvocationDirectoryType {
    #[default]
    #[serde(rename = "root")]
    Root,

    #[serde(rename = "target_directory")]
    TargetDirectory,

    #[serde(rename = "root_or_parent_with_any_config")]
    RootOrParentWithAnyConfig,

    #[serde(rename = "root_or_parent_with")]
    RootOrParentWith,

    #[serde(rename = "tool_directory")]
    ToolDir,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default, JsonSchema)]
pub struct InvocationDirectoryDef {
    #[serde(default)]
    #[serde(rename = "type")]
    pub kind: InvocationDirectoryType,

    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default, JsonSchema)]
pub struct TargetDef {
    #[serde(default)]
    #[serde(rename = "type")]
    pub target_type: TargetType,

    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default, JsonSchema)]
pub enum TargetType {
    #[default]
    #[serde(rename = "file")]
    File,

    #[serde(rename = "parent_with")]
    ParentWith,

    #[serde(rename = "literal")]
    Literal,

    #[serde(rename = "parent")]
    Parent,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Default, JsonSchema)]
pub enum OutputLevel {
    #[serde(rename = "high")]
    High,

    #[default]
    #[serde(rename = "medium")]
    Medium,

    #[serde(rename = "low")]
    Low,

    #[serde(rename = "fmt")]
    Fmt,
}

impl Into<Level> for OutputLevel {
    fn into(self) -> Level {
        match self {
            OutputLevel::High => Level::High,
            OutputLevel::Medium => Level::Medium,
            OutputLevel::Low => Level::Low,
            OutputLevel::Fmt => Level::Fmt,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Default, JsonSchema)]
pub enum OutputCategory {
    #[default]
    #[serde(rename = "bug")]
    Bug,

    #[serde(rename = "vulnerability")]
    Vulnerability,

    #[serde(rename = "security_hotspot")]
    SecurityHotspot,

    #[serde(rename = "performance")]
    Performance,

    #[serde(rename = "style")]
    Style,

    #[serde(rename = "documentation")]
    Documentation,

    #[serde(rename = "anti-pattern")]
    AntiPattern,

    #[serde(rename = "type_check")]
    TypeCheck,

    #[serde(rename = "accessibility")]
    Accessibility,

    #[serde(rename = "structure")]
    Structure,

    #[serde(rename = "duplication")]
    Duplication,

    #[serde(rename = "dead_code")]
    DeadCode,

    #[serde(rename = "lint")]
    Lint,

    #[serde(rename = "secret")]
    Secret,

    #[serde(rename = "dependency_alert")]
    DependencyAlert,
}

impl Into<Category> for OutputCategory {
    fn into(self) -> Category {
        match self {
            OutputCategory::Bug => Category::Bug,
            OutputCategory::Vulnerability => Category::Vulnerability,
            OutputCategory::SecurityHotspot => Category::SecurityHotspot,
            OutputCategory::Performance => Category::Performance,
            OutputCategory::Style => Category::Style,
            OutputCategory::Documentation => Category::Documentation,
            OutputCategory::AntiPattern => Category::AntiPattern,
            OutputCategory::TypeCheck => Category::TypeCheck,
            OutputCategory::Accessibility => Category::Accessibility,
            OutputCategory::Structure => Category::Structure,
            OutputCategory::Duplication => Category::Duplication,
            OutputCategory::DeadCode => Category::DeadCode,
            OutputCategory::Lint => Category::Lint,
            OutputCategory::Secret => Category::Secret,
            OutputCategory::DependencyAlert => Category::DependencyAlert,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default, JsonSchema)]
pub struct PluginDef {
    pub runtime: Option<Runtime>,

    /// The version of the plugin to run
    pub version: Option<String>,

    /// The latest version of the upstream package (which may or may not be good)
    #[serde(default)]
    pub latest_version: Option<String>,

    /// The latest validated version of the plugin
    #[serde(default)]
    pub known_good_version: Option<String>,

    // Any known bad versions of the plugins
    #[serde(default)]
    pub known_bad_versions: Vec<String>,

    #[serde(default)]
    pub file_types: Vec<String>,

    #[serde(default)]
    pub config_files: Vec<PathBuf>,

    #[serde(default)]
    pub downloads: Vec<String>,

    #[serde(default)]
    pub releases: Vec<String>,

    #[serde(default)]
    pub package: Option<String>,

    #[serde(default)]
    pub extra_packages: Vec<ExtraPackage>,

    #[serde(default)]
    pub package_file: Option<String>,

    #[serde(default)]
    pub affects_cache: Vec<String>,

    #[serde(default)]
    pub drivers: HashMap<String, DriverDef>,

    #[serde(default)]
    pub version_command: Option<String>,

    #[serde(default = "semver_regex")]
    pub version_regex: String,

    pub issue_url_format: Option<String>,

    // three download attrs for java/php
    pub runnable_archive_url: Option<String>,
    pub download_type: Option<String>,
    pub strip_components: Option<usize>,

    #[serde(default)]
    pub environment: Vec<PluginEnvironment>,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub security: bool,

    #[serde(default = "default_idempotent")]
    pub idempotent: bool,

    #[serde(default)]
    pub hidden: bool,

    #[serde(default)]
    pub fetch: Vec<PluginFetch>,

    #[serde(default)]
    pub package_filters: Vec<String>,

    #[serde(default)]
    pub package_file_candidate: Option<PackageFileCandidate>,

    #[serde(default)]
    pub package_file_candidate_filters: Vec<String>,

    #[serde(default)]
    pub prefix: Option<String>,

    #[serde(default)]
    pub supported_platforms: Vec<Platform>,
}

fn default_idempotent() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Default, JsonSchema)]
pub enum PackageFileCandidate {
    #[default]
    #[serde(rename = "package.json")]
    PackageJson,
    #[serde(rename = "Gemfile")]
    Gemfile,
}

impl std::fmt::Display for PackageFileCandidate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PackageFileCandidate::PackageJson => write!(f, "package.json"),
            PackageFileCandidate::Gemfile => write!(f, "Gemfile"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default, Eq, JsonSchema)]
pub struct PluginEnvironment {
    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub list: Vec<String>,

    #[serde(default)]
    pub value: String,
}

impl Ord for PluginEnvironment {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for PluginEnvironment {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Default, JsonSchema)]
pub enum SuggestionMode {
    #[default]
    #[serde(rename = "never")]
    Never,
    #[serde(rename = "config")]
    Config,
    #[serde(rename = "targets")]
    Targets,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Default, JsonSchema)]
pub enum OutputDestination {
    #[default]
    #[serde(rename = "stdout")]
    Stdout,
    #[serde(rename = "stderr")]
    Stderr,
    #[serde(rename = "tmpfile")]
    Tmpfile,
    #[serde(rename = "rewrite")]
    Rewrite,
    #[serde(rename = "pass_fail")]
    PassFail,
}

impl std::fmt::Display for OutputDestination {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OutputDestination::Stdout => write!(f, "stdout"),
            OutputDestination::Stderr => write!(f, "stderr"),
            OutputDestination::Tmpfile => write!(f, "tmpfile"),
            OutputDestination::Rewrite => write!(f, "rewrite"),
            OutputDestination::PassFail => write!(f, "pass_fail"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Default, JsonSchema)]
pub enum OutputFormat {
    #[default]
    #[serde(rename = "sarif")]
    Sarif,
    #[serde(rename = "actionlint")]
    Actionlint,
    #[serde(rename = "bandit")]
    Bandit,
    #[serde(rename = "biome")]
    Biome,
    #[serde(rename = "clippy")]
    Clippy,
    #[serde(rename = "coffeelint")]
    Coffeelint,
    #[serde(rename = "eslint")]
    Eslint,
    #[serde(rename = "golangci_lint")]
    GolangciLint,
    #[serde(rename = "hadolint")]
    Hadolint,
    #[serde(rename = "knip")]
    Knip,
    #[serde(rename = "markdownlint")]
    Markdownlint,
    #[serde(rename = "mypy")]
    Mypy,
    #[serde(rename = "pylint")]
    Pylint,
    #[serde(rename = "php_codesniffer")]
    PhpCodesniffer,
    #[serde(rename = "phpstan")]
    Phpstan,
    #[serde(rename = "radarlint")]
    Radarlint,
    #[serde(rename = "reek")]
    Reek,
    #[serde(rename = "regex")]
    Regex,
    #[serde(rename = "ripgrep")]
    Ripgrep,
    #[serde(rename = "rubocop")]
    Rubocop,
    #[serde(rename = "ruff")]
    Ruff,
    #[serde(rename = "shellcheck")]
    Shellcheck,
    #[serde(rename = "sqlfluff")]
    Sqlfluff,
    #[serde(rename = "stylelint")]
    Stylelint,
    #[serde(rename = "taplo")]
    Taplo,
    #[serde(rename = "trivy_sarif")]
    TrivySarif,
    #[serde(rename = "trufflehog")]
    Trufflehog,
    #[serde(rename = "tsc")]
    Tsc,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OutputFormat::Actionlint => write!(f, "actionlint"),
            OutputFormat::Bandit => write!(f, "bandit"),
            OutputFormat::Biome => write!(f, "biome"),
            OutputFormat::Clippy => write!(f, "clippy"),
            OutputFormat::Coffeelint => write!(f, "coffeelint"),
            OutputFormat::Eslint => write!(f, "eslint"),
            OutputFormat::GolangciLint => write!(f, "golangci_lint"),
            OutputFormat::Hadolint => write!(f, "hadolint"),
            OutputFormat::Knip => write!(f, "knip"),
            OutputFormat::Markdownlint => write!(f, "markdownlint"),
            OutputFormat::Mypy => write!(f, "mypy"),
            OutputFormat::PhpCodesniffer => write!(f, "php_codesniffer"),
            OutputFormat::Phpstan => write!(f, "phpstan"),
            OutputFormat::Pylint => write!(f, "pylint"),
            OutputFormat::Radarlint => write!(f, "radarlint"),
            OutputFormat::Reek => write!(f, "reek"),
            OutputFormat::Regex => write!(f, "regex"),
            OutputFormat::Ripgrep => write!(f, "ripgrep"),
            OutputFormat::Rubocop => write!(f, "rubocop"),
            OutputFormat::Ruff => write!(f, "ruff"),
            OutputFormat::Sarif => write!(f, "sarif"),
            OutputFormat::Shellcheck => write!(f, "shellcheck"),
            OutputFormat::Sqlfluff => write!(f, "sqlfluff"),
            OutputFormat::Stylelint => write!(f, "stylelint"),
            OutputFormat::Taplo => write!(f, "taplo"),
            OutputFormat::TrivySarif => write!(f, "trivy_sarif"),
            OutputFormat::Trufflehog => write!(f, "trufflehog"),
            OutputFormat::Tsc => write!(f, "tsc"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Default, JsonSchema)]
pub enum DriverType {
    #[default]
    #[serde(rename = "linter")]
    Linter,
    #[serde(rename = "formatter")]
    Formatter,
    #[serde(rename = "validator")]
    Validator,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Default, JsonSchema)]
pub enum Runtime {
    #[default]
    #[serde(rename = "go")]
    Go,
    #[serde(rename = "ruby")]
    Ruby,
    #[serde(rename = "python")]
    Python,
    #[serde(rename = "node")]
    Node,
    #[serde(rename = "rust")]
    Rust,
    #[serde(rename = "java")]
    Java,
    #[serde(rename = "php")]
    Php,
}

impl std::fmt::Display for Runtime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Runtime::Go => write!(f, "go"),
            Runtime::Ruby => write!(f, "ruby"),
            Runtime::Python => write!(f, "python"),
            Runtime::Node => write!(f, "node"),
            Runtime::Rust => write!(f, "rust"),
            Runtime::Java => write!(f, "java"),
            Runtime::Php => write!(f, "php"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[allow(unused)]
pub struct EnabledRuntimes {
    #[serde(default)]
    pub enabled: HashMap<Runtime, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, JsonSchema)]
pub struct EnabledPlugin {
    pub name: String,

    #[serde(default = "default_plugin_version")]
    pub version: String,

    #[serde(default)]
    pub package_file: Option<String>,

    #[serde(default)]
    pub extra_packages: Vec<ExtraPackage>,

    #[serde(default)]
    pub config_files: Vec<PathBuf>,

    #[serde(default)]
    pub affects_cache: Vec<String>,

    #[serde(default = "default_plugin_drivers")]
    pub drivers: Vec<String>,

    #[serde(default)]
    pub mode: IssueMode,

    #[serde(default)]
    pub skip_upstream: Option<bool>,

    #[serde(default)]
    pub triggers: Vec<CheckTrigger>,

    #[serde(default)]
    pub fetch: Vec<PluginFetch>,

    #[serde(default)]
    pub package_filters: Vec<String>,

    #[serde(default)]
    pub prefix: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct PluginFetch {
    pub url: String,
    pub path: String,
}

impl PluginFetch {
    pub fn download_file_to(&self, directories: &[PathBuf]) -> Result<()> {
        let response = ureq::get(&self.url)
            .call()
            .with_context(|| format!("Failed to get url: {}", &self.url))?;

        if response.status() != 200 {
            return Err(anyhow::anyhow!(
                "Failed to download file: {}, status: {}",
                &self.url,
                response.status()
            ));
        }

        let data = response.into_string().with_context(|| {
            format!(
                "Failed to get contents of {} to download to {}",
                &self.url, &self.path
            )
        })?;

        for directory in directories {
            if !directory.exists() {
                std::fs::create_dir_all(directory)
                    .with_context(|| format!("Failed to create directory: {:?}", directory))?;
            }

            if !directory.is_dir() {
                return Err(anyhow::anyhow!(
                    "Failed to create directory: {:?}, it is not a directory",
                    directory
                ));
            }

            let path = directory.join(&self.path);
            let mut file = File::create(path)
                .with_context(|| format!("Failed to create file: {}", &self.path))?;

            file.write_all(data.as_bytes())
                .with_context(|| format!("Failed to write contents to {}", &self.path))?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, JsonSchema)]
pub enum IssueMode {
    #[default]
    #[serde(rename = "block")]
    Block = 1,
    #[serde(rename = "comment")]
    Comment = 2,
    #[serde(rename = "monitor")]
    Monitor = 3,
    #[serde(rename = "disabled")]
    Disabled = 4,
}

impl IssueMode {
    pub fn to_str(&self) -> &'static str {
        match self {
            IssueMode::Block => "block",
            IssueMode::Comment => "comment",
            IssueMode::Monitor => "monitor",
            IssueMode::Disabled => "disabled",
        }
    }

    pub fn extract_issue_mode_from_smells(
        language: &Language,
        qlty_config: &QltyConfig,
    ) -> IssueMode {
        if let Some(smells) = &language.smells {
            if let Some(mode) = &smells.mode {
                return mode.clone();
            }
        }

        if let Some(smells) = &qlty_config.smells {
            if let Some(mode) = &smells.mode {
                return mode.clone();
            }
        }

        IssueMode::Block
    }
}

#[derive(Debug, Serialize, Clone, Default, PartialEq, JsonSchema)]
pub struct ExtraPackage {
    pub name: String,
    pub version: String,
}

impl<'de> Deserialize<'de> for ExtraPackage {
    fn deserialize<D>(deserializer: D) -> std::result::Result<ExtraPackage, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        let (name, version) = string.rsplit_once('@').unwrap();

        Ok(ExtraPackage {
            name: name.to_string(),
            version: version.to_string(),
        })
    }
}

fn default_plugin_version() -> String {
    "latest".to_string()
}

fn default_plugin_drivers() -> Vec<String> {
    vec![ALL.to_string()]
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum CheckTrigger {
    #[serde(rename = "manual")]
    Manual,
    #[serde(rename = "pre-commit")]
    PreCommit,
    #[serde(rename = "pre-push")]
    PrePush,
    #[serde(rename = "build")]
    Build,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, JsonSchema)]
pub enum Platform {
    #[serde(rename = "linux")]
    Linux,
    #[serde(rename = "macos")]
    MacOS,
    #[serde(rename = "windows")]
    Windows,
}

impl Platform {
    pub fn all_values() -> Vec<Platform> {
        vec![Platform::Linux, Platform::MacOS, Platform::Windows]
    }

    pub fn current() -> Platform {
        if cfg!(target_os = "linux") {
            Platform::Linux
        } else if cfg!(target_os = "macos") {
            Platform::MacOS
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else {
            panic!("Unsupported platform")
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Platform::Windows => "windows",
            Platform::MacOS => "macos",
            Platform::Linux => "linux",
        };
        write!(f, "{}", name)
    }
}
