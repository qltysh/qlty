use qlty_config::{config::EnabledPlugin, issue_transformer::IssueTransformer};
use qlty_types::analysis::v1::Issue;

#[derive(Debug, Clone)]
pub struct PluginModeTransformer {
    pub plugins: Vec<EnabledPlugin>,
}

impl PluginModeTransformer {}

impl IssueTransformer for PluginModeTransformer {
    fn transform(&self, mut issue: Issue) -> Option<Issue> {
        if let Some(plugin) = self.plugins.iter().find(|p| p.name == issue.tool) {
            issue.mode = plugin.mode.unwrap_or_default() as i32;
        }

        Some(issue)
    }

    fn clone_box(&self) -> Box<dyn IssueTransformer> {
        Box::new(self.clone())
    }
}
