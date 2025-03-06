use anyhow::Result;
use qlty_types::analysis::v1::ToolInstallSummary;

pub struct ToolInstallSummaryWritter {}
impl ToolInstallSummaryWritter {
    pub fn write_to_file(tool_install_summary: &ToolInstallSummary) -> Result<()> {
        let path = format!("{}-install-debug.yaml", tool_install_summary.directory);
        println!("Writing install debug file to {}", path);
        let yaml = serde_yaml::to_string(tool_install_summary)?;
        std::fs::write(path, yaml)?;

        Ok(())
    }
}
