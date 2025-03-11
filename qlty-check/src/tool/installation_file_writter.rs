use crate::utils::generate_random_id;
use anyhow::Result;
use qlty_types::analysis::v1::Installation;
use std::path::PathBuf;
use tracing::debug;

pub struct InstallationFileWritter {}
impl InstallationFileWritter {
    pub fn write_to_file(installation: &Installation) -> Result<()> {
        let installation_id = generate_random_id(6);
        let path = PathBuf::from(&installation.directory)
            .join(format!("installation-{}.yaml", installation_id));

        debug!("Writing installation to {:?}", path);

        let yaml = serde_yaml::to_string(installation)?;
        std::fs::write(path, yaml)?;

        Ok(())
    }
}
