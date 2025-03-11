use crate::utils::generate_random_id;
use anyhow::Result;
use qlty_types::analysis::v1::Installation;
use std::{fs::create_dir_all, path::PathBuf};
use tracing::{debug, error};

pub struct InstallationFileWritter {}
impl InstallationFileWritter {
    pub fn write_to_file(installation: &Installation) -> Result<()> {
        let installation_id = generate_random_id(6);
        let installation_files_directory = PathBuf::from(&installation.directory);
        if let Err(err) = create_dir_all(&installation_files_directory) {
            error!("Error creating installation directory: {}", err);
        }

        let path =
            installation_files_directory.join(format!("installation-{}.yaml", installation_id));

        debug!("Writing installation to {:?}", path);

        let yaml = serde_yaml::to_string(installation)?;
        std::fs::write(path, yaml)?;

        Ok(())
    }
}
