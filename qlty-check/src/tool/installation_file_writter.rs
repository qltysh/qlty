use anyhow::Result;
use qlty_types::analysis::v1::Installation;

pub struct InstallationFileWritter {}
impl InstallationFileWritter {
    pub fn write_to_file(installation: &Installation) -> Result<()> {
        let path = format!("{}-install-debug.yaml", installation.directory);
        let yaml = serde_yaml::to_string(installation)?;
        std::fs::write(path, yaml)?;

        Ok(())
    }
}
