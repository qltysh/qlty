use super::{download::Download, installation_file_writter::InstallationFileWritter, Tool};
use anyhow::Result;
use chrono::Utc;
use qlty_config::version::QLTY_VERSION;
use qlty_types::analysis::v1::Installation;
use std::{io::Write, process::Output};
use tracing::error;

pub fn initialize_installation(tool: &dyn Tool) -> Installation {
    Installation {
        tool_name: tool.name(),
        version: tool.version().unwrap_or_default(),
        tool_type: format!("{:?}", tool.tool_type()),
        directory: format!("{}-installation-debug-files", tool.directory()),
        runtime: tool.runtime().map_or("".to_string(), |r| r.name()),
        fingerprint: tool.fingerprint(),
        qlty_cli_version: QLTY_VERSION.to_string(),
        log_file_path: tool.install_log_path(),
        started_at: Some(Utc::now().into()),
        env: tool.env(),
        ..Default::default()
    }
}

pub fn finalize_installation_from_assets_fetch(
    installation: &mut Installation,
    result: &Result<ureq::Response, ureq::Error>,
    url: &str,
) {
    installation.download_url = Some(url.to_string());

    if result.is_ok() {
        installation.download_success = Some(true);
    } else {
        installation.download_success = Some(false);
    }
    installation.finished_at = Some(Utc::now().into());

    if let Err(err) = InstallationFileWritter::write_to_file(&installation) {
        error!("Error writing debug data: {}", err);
    }
}

pub fn finalize_installation_from_cmd_result(
    tool: &dyn Tool,
    result: &std::io::Result<Output>,
    installation: &mut Installation,
    script: String,
) -> Result<()> {
    installation.script = Some(script);
    if let Ok(ref output) = result {
        installation.stdout = Some(String::from_utf8_lossy(&output.stdout).to_string());
        installation.stderr = Some(String::from_utf8_lossy(&output.stderr).to_string());
        installation.exit_code = Some(output.status.code().unwrap_or_default().into());
    } else {
        installation.stderr = Some(format!("{:?}", result));
    }

    let mut log_file = tool.install_log_file()?;
    log_file.write_all(installation.stdout.clone().unwrap_or_default().as_bytes())?;
    log_file.write_all(installation.stderr.clone().unwrap_or_default().as_bytes())?;

    installation.finished_at = Some(Utc::now().into());
    if let Err(err) = InstallationFileWritter::write_to_file(installation) {
        error!("Error writing debug data: {}", err);
    }

    Ok(())
}

pub fn finalize_installation_from_download_result(
    download: &Download,
    installation: &mut Installation,
    result: &Result<()>,
) -> Result<()> {
    installation.download_url = Some(download.url()?);
    installation.download_file_type = Some(download.file_type().to_string());
    installation.download_binary_name = download.binary_name();

    if result.is_ok() {
        installation.download_success = Some(true);
    } else {
        installation.download_success = Some(false);
    }
    installation.finished_at = Some(Utc::now().into());

    if let Err(err) = InstallationFileWritter::write_to_file(installation) {
        error!("Error writing debug data: {}", err);
    }

    Ok(())
}
