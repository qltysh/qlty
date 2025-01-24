use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DownloadFileType {
    #[default]
    Executable,
    Targz,
    Tarxz,
    Gz,
    Zip,
}

impl FromStr for DownloadFileType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "executable" => Ok(DownloadFileType::Executable),
            "targz" => Ok(DownloadFileType::Targz),
            "tarxz" => Ok(DownloadFileType::Tarxz),
            "gz" => Ok(DownloadFileType::Gz),
            "zip" => Ok(DownloadFileType::Zip),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for DownloadFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default, JsonSchema)]
pub enum OperatingSystem {
    #[default]
    #[serde(rename = "linux")]
    Linux,
    #[serde(rename = "macos")]
    MacOS,
    #[serde(rename = "windows")]
    Windows,
}

impl OperatingSystem {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::MacOS => "macos",
            Self::Windows => "windows",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default, JsonSchema)]
pub enum Cpu {
    #[default]
    #[serde(rename = "x86_64")]
    X86_64,
    #[serde(rename = "aarch64")]
    Aarch64,
}

impl Cpu {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "aarch64",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default, JsonSchema)]
pub struct System {
    pub url: String,
    pub cpu: Cpu,
    pub os: OperatingSystem,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, JsonSchema)]
pub struct DownloadDef {
    pub binary_name: Option<String>,

    #[serde(default = "default_strip_components")]
    pub strip_components: usize,

    #[serde(rename = "system")]
    pub systems: Vec<System>,
}

impl Default for DownloadDef {
    fn default() -> Self {
        Self {
            binary_name: None,
            strip_components: default_strip_components(),
            systems: vec![],
        }
    }
}

fn default_strip_components() -> usize {
    1
}
