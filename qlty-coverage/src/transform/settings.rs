use crate::formats::Formats;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub path: String,
    pub report_format: Option<Formats>,
    pub add_prefix: Option<String>,
    pub strip_prefix: Option<String>,
}
