use super::Tool;
use crate::utils::generate_random_id;
use anyhow::Result;
use chrono::Utc;
use qlty_config::version::QLTY_VERSION;
use qlty_types::analysis::v1::Installation;
use std::{fs::create_dir_all, path::PathBuf};
use tracing::{debug, error};

pub fn initialize_installation(tool: &dyn Tool) -> Result<Installation> {
    let env = tool.env()?;

    Ok(Installation {
        tool_name: tool.name(),
        version: tool.version().unwrap_or_default(),
        tool_type: format!("{:?}", tool.tool_type()),
        directory: tool.debug_files_directory(),
        runtime: tool.runtime().map_or("".to_string(), |r| r.name()),
        fingerprint: tool.fingerprint(),
        qlty_cli_version: QLTY_VERSION.to_string(),
        log_file_path: tool.install_log_path(),
        started_at: Some(Utc::now().into()),
        env,
        ..Default::default()
    })
}

pub fn write_to_file(installation: &Installation) {
    if let Err(err) = write_to_file_impl(installation) {
        error!("Error writing debug data: {}", err);
    }
}

fn write_to_file_impl(installation: &Installation) -> Result<()> {
    let installation_id = generate_random_id(6);
    let installation_files_directory = PathBuf::from(&installation.directory);
    if let Err(err) = create_dir_all(&installation_files_directory) {
        error!("Error creating installation directory: {}", err);
    }

    let path = installation_files_directory.join(format!("installation-{}.yaml", installation_id));

    debug!("Writing installation to {:?}", path);

    let yaml = serde_yaml::to_string(installation)?;
    std::fs::write(path, yaml)?;

    Ok(())
}
