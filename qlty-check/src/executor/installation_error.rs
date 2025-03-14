#[derive(Debug, Clone)]
pub struct InstallationError {
    pub tool_name: String,
    pub error_message: Option<String>,
    pub directory: Option<String>,
}
