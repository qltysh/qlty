use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::DownloadFileType;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, JsonSchema)]
pub struct ReleaseDef {
    pub github: String,

    pub binary_name: Option<String>,

    #[serde(default = "default_download_type")]
    pub download_type: DownloadFileType,

    #[serde(default = "default_strip_components")]
    pub strip_components: usize,
}

impl Default for ReleaseDef {
    fn default() -> Self {
        Self {
            github: String::default(),
            binary_name: None,
            download_type: default_download_type(),
            strip_components: default_strip_components(),
        }
    }
}

fn default_download_type() -> DownloadFileType {
    DownloadFileType::Targz
}

fn default_strip_components() -> usize {
    1
}
