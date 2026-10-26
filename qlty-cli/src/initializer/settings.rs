use super::SourceSpec;
use qlty_config::Workspace;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Settings {
    pub workspace: Workspace,
    pub skip_plugins: bool,
    pub skip_default_source: bool,
    pub source: Option<SourceSpec>,
    pub plugins_to_enable: Option<HashMap<String, String>>,
}
