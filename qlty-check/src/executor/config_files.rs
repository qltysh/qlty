use crate::{
    fs::{create_symlink, ensure_parent_exists},
    planner::config_files::{ConfigCopyMode, ConfigOperation, ConfigSource},
};
use anyhow::{bail, Context, Result};
use qlty_analysis::utils::fs::path_to_string;
use qlty_config::config::PluginFetch;
use qlty_config::http;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write as _;
use std::path::Path;
use tracing::debug;

pub fn perform_config_operation(
    operation: &ConfigOperation,
    download_cache: &mut HashMap<String, Vec<u8>>,
) -> Result<Option<String>> {
    match &operation.source {
        ConfigSource::File(source_path) => {
            stage_file(source_path, &operation.destination, operation.mode.clone())
        }
        ConfigSource::Download(fetch) => {
            download_file(fetch, &operation.destination, download_cache)
        }
    }
}

fn stage_file(
    source_path: &Path,
    destination_path: &Path,
    mode: ConfigCopyMode,
) -> Result<Option<String>> {
    if !source_path.exists() {
        return Ok(None);
    }

    ensure_parent_exists(destination_path)?;

    if destination_path.exists() {
        return Ok(Some(path_to_string(destination_path)));
    }

    match mode {
        ConfigCopyMode::Symlink => {
            debug!(
                "Symlinking {} to {}",
                source_path.display(),
                destination_path.display()
            );
            create_symlink(source_path, destination_path).with_context(|| {
                format!(
                    "Failed to symlink config file {} to {}",
                    source_path.display(),
                    destination_path.display()
                )
            })?;
        }
        ConfigCopyMode::Copy => {
            debug!(
                "Copying {} to {}",
                source_path.display(),
                destination_path.display()
            );
            std::fs::copy(source_path, destination_path).with_context(|| {
                format!(
                    "Failed to copy config file {} to {}",
                    source_path.display(),
                    destination_path.display()
                )
            })?;
        }
    }

    Ok(Some(path_to_string(destination_path)))
}

fn download_file(
    fetch: &PluginFetch,
    destination_path: &Path,
    download_cache: &mut HashMap<String, Vec<u8>>,
) -> Result<Option<String>> {
    ensure_parent_exists(destination_path)?;

    let data = if let Some(cached) = download_cache.get(&fetch.url) {
        cached.clone()
    } else {
        let response = http::get(&fetch.url)
            .call()
            .with_context(|| format!("Failed to get url: {}", fetch.url))?;

        if response.status() != 200 {
            bail!(
                "Failed to download file: {}, status: {}",
                fetch.url,
                response.status()
            );
        }

        let data = response
            .into_string()
            .with_context(|| {
                format!(
                    "Failed to get contents of {} to download to {}",
                    fetch.url, fetch.path
                )
            })?
            .into_bytes();

        download_cache.insert(fetch.url.clone(), data.clone());
        data
    };

    if let Some(parent) = destination_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
    }

    let mut file = File::create(destination_path).with_context(|| {
        format!(
            "Failed to create file for fetched config: {}",
            destination_path.display()
        )
    })?;

    file.write_all(&data).with_context(|| {
        format!(
            "Failed to write fetched config to {}",
            destination_path.display()
        )
    })?;

    Ok(Some(path_to_string(destination_path)))
}
