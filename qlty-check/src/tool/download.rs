use super::installations::initialize_installation;
use super::installations::write_to_file;
use super::Tool;
use super::ToolType;
use crate::ui::{ProgressBar, ProgressTask};
use anyhow::{anyhow, bail, Result};
use chrono::Utc;
use flate2::read::GzDecoder;
use itertools::Itertools;
use qlty_analysis::utils::fs::path_to_string;
use qlty_config::config::PluginDef;
use qlty_config::config::{Cpu, DownloadDef, DownloadFileType, OperatingSystem};
use qlty_types::analysis::v1::Installation;
use sha2::Digest;
use sha2::Sha256;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tar::Archive;
use tempfile::tempfile;
use tracing::{info, trace, warn};
use zip::ZipArchive;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[derive(Debug, Clone)]
pub struct Download {
    pub tool_name: String,
    pub version: String,
    def: DownloadDef,
}

impl Download {
    pub fn new(def: &DownloadDef, tool_name: &str, version: &str) -> Self {
        Self {
            def: def.to_owned(),
            tool_name: tool_name.to_string(),
            version: version.to_string(),
        }
    }

    pub fn url(&self) -> Result<String> {
        Ok(self
            .def
            .systems
            .iter()
            .find(|system| system.cpu == system_arch() && system.os == system_os())
            .map(|system| system.url.clone())
            .ok_or_else(|| {
                anyhow!(
                    "No download URL found for {}@{} on {:?}/{:?}",
                    self.tool_name,
                    self.version,
                    system_os(),
                    system_arch(),
                )
            })?
            .replace("${version}", &self.version))
    }

    pub fn binary_name(&self) -> Option<String> {
        self.def.binary_name.clone()
    }

    pub fn file_type(&self) -> DownloadFileType {
        if let Ok(url) = self.url() {
            if url.to_lowercase().ends_with(".tar.gz") {
                DownloadFileType::Targz
            } else if url.ends_with(".tar.xz") {
                DownloadFileType::Tarxz
            } else if url.ends_with(".gz") {
                DownloadFileType::Gz
            } else if url.ends_with(".zip") {
                DownloadFileType::Zip
            } else {
                DownloadFileType::Executable
            }
        } else {
            DownloadFileType::Executable
        }
    }

    pub fn update_hash(&self, hasher: &mut Sha256, tool_name: &str) -> Result<()> {
        hasher.update(tool_name);
        hasher.update(&self.url()?);
        hasher.update(format!("{:?}", self.file_type()));
        Ok(())
    }

    pub fn install(&self, tool: &dyn Tool) -> Result<()> {
        let directory = PathBuf::from(tool.directory());
        let tool_name = tool.name();
        let mut installation = initialize_installation(tool);

        let result = match self.file_type() {
            DownloadFileType::Executable => self.install_executable(&directory, &tool_name),
            DownloadFileType::Targz => self.install_targz(&directory),
            DownloadFileType::Tarxz => self.install_tarxz(&directory),
            DownloadFileType::Gz => self.install_gz(&directory, &tool_name),
            DownloadFileType::Zip => self.install_zip(&directory),
        };

        finalize_installation_from_download_result(self, &mut installation, &result)?;

        result
    }

    fn install_executable(&self, directory: &Path, tool_name: &str) -> Result<()> {
        let mut binary_name = self.binary_name().unwrap_or(tool_name.to_string());

        if cfg!(windows) && self.url()?.ends_with(".exe") {
            let mut binary_name_path = PathBuf::from(binary_name);
            binary_name_path.set_extension("exe");
            binary_name = path_to_string(binary_name_path);
        }

        let binary_path = directory.join(&binary_name);

        info!(
            "Downloading (binary) {} to {}",
            self.url()?,
            binary_path.display()
        );
        match ureq::get(&self.url()?).call() {
            Ok(response) => {
                let mut reader = response.into_reader();
                let mut file = File::create(binary_path)?;
                std::io::copy(&mut reader, &mut file)?;

                #[cfg(unix)]
                {
                    let mut perms = file.metadata()?.permissions();
                    perms.set_mode(0o755);
                    file.set_permissions(perms)?;
                }

                Ok(())
            }
            Err(_) => bail!("Error downloading file"),
        }
    }

    fn install_gz(&self, directory: &Path, tool_name: &str) -> Result<()> {
        let binary_name = self.binary_name().unwrap_or(tool_name.to_string());
        let binary_path = directory.join(binary_name);

        info!(
            "Downloading (gz) {} to {}",
            self.url()?,
            binary_path.display()
        );
        match ureq::get(&self.url()?).call() {
            Ok(response) => {
                let reader = response.into_reader();
                let mut decoder = GzDecoder::new(reader);

                let mut file = File::create(binary_path)?;
                std::io::copy(&mut decoder, &mut file)?;

                #[cfg(unix)]
                {
                    let mut perms = file.metadata()?.permissions();
                    perms.set_mode(0o755);
                    file.set_permissions(perms)?;
                }

                Ok(())
            }
            Err(_) => bail!("Error downloading file"),
        }
    }

    fn install_targz(&self, directory: &Path) -> Result<()> {
        info!("Downloading (tar.gz) {}", self.url()?);
        match ureq::get(&self.url()?).call() {
            Ok(response) => {
                let reader = response.into_reader();
                let tar = GzDecoder::new(reader);
                let mut archive = Archive::new(tar);
                self.extract_archive(&mut archive, directory)?;
                Ok(())
            }
            Err(_) => bail!("Error downloading file"),
        }
    }

    fn install_tarxz(&self, directory: &Path) -> Result<()> {
        info!("Downloading (tar.xz) {}", self.url()?);
        match ureq::get(&self.url()?).call() {
            Ok(response) => {
                let response_reader = response.into_reader();
                let mut reader = BufReader::new(response_reader);
                let mut tar: Vec<u8> = Vec::new();
                lzma_rs::xz_decompress(&mut reader, &mut tar)
                    .map_err(|e| anyhow!("Failed to decompress xz file: {:?}", e))?;
                let cursor = Cursor::new(tar);
                let mut archive = Archive::new(cursor);
                self.extract_archive(&mut archive, directory)?;
                Ok(())
            }
            Err(_) => bail!("Error downloading file"),
        }
    }

    fn extract_archive<R: std::io::Read>(
        &self,
        archive: &mut Archive<R>,
        destination: &Path,
    ) -> Result<()> {
        info!("Extracting to {}", destination.display());
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            let stripped_path = strip_components(&path, self.def.strip_components);
            let full_path = destination.join(stripped_path);

            // Ensure the full path is still within the destination directory
            if !full_path.starts_with(destination) {
                warn!("Skipping {}", full_path.display());
                continue; // Skip entries with path traversal issues
            }

            if full_path.is_dir() {
                trace!("Creating directory {}", full_path.display());
                std::fs::create_dir_all(&full_path)?;
            } else {
                if let Some(parent) = full_path.parent() {
                    if parent.is_dir() {
                        trace!("Parent exists {}", parent.display());
                    } else {
                        trace!("Creating directory {}", parent.display());
                        std::fs::create_dir_all(parent)?;
                    }
                }

                trace!("Extracting {}", full_path.display());
                entry.unpack(&full_path)?;
            }
        }
        Ok(())
    }

    fn install_zip(&self, directory: &Path) -> Result<()> {
        match ureq::get(&self.url()?).call() {
            Ok(response) => {
                let mut reader = response.into_reader();
                let mut file = tempfile()?;
                std::io::copy(&mut reader, &mut file)?;
                self.extract_zip(file, directory)?;
                Ok(())
            }
            Err(_) => bail!("Error downloading file: {}", self.url()?),
        }
    }

    fn extract_zip(&self, file: File, directory: &Path) -> Result<()> {
        info!("Extracting zip to {}", directory.display());
        let mut archive =
            ZipArchive::new(file).map_err(|e| anyhow!("Failed to open zip archive: {}", e))?;

        let mut strip_component_count = 0;
        let strip_component = archive.file_names().count() > 1
            && archive
                .file_names()
                .map(|f| Path::new(f).components().next())
                .all_equal();

        if strip_component {
            strip_component_count = 1;
            let first_entry = archive.by_index(0)?;
            let enclosed_name = first_entry
                .enclosed_name()
                .ok_or_else(|| anyhow!("Invalid path in zip archive"))?;
            let first_component = enclosed_name
                .components()
                .next()
                .ok_or_else(|| anyhow!("Empty path component in zip archive"))?;

            trace!(
                "Detected shared root directory in zip, stripping: {}",
                path_to_string(first_component)
            );
        }

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => {
                    let stripped_path = strip_components(&path, strip_component_count);
                    directory.join(stripped_path)
                }
                None => continue,
            };

            trace!(
                "extract_zip: Extracting file {:?} -> {:?}",
                file.name(),
                outpath
            );
            if file.is_dir() {
                std::fs::create_dir_all(&outpath)?;
                trace!("extract_zip: Creating directory {}", outpath.display());
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                        trace!("extract_zip: Creating directory {}", p.display());
                    }
                }

                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
                trace!("extract_zip: Extracting file {}", outpath.display());

                #[cfg(unix)]
                {
                    if let Some(mode) = file.unix_mode() {
                        std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DownloadTool {
    pub plugin_name: String,
    pub download: Download,
    pub plugin: PluginDef,
}

impl Tool for DownloadTool {
    fn name(&self) -> String {
        match self.download.binary_name() {
            Some(name) => name,
            None => self.plugin_name.clone(),
        }
    }

    fn tool_type(&self) -> ToolType {
        ToolType::Download
    }

    fn version(&self) -> Option<String> {
        Some(self.download.version.clone())
    }

    fn version_command(&self) -> Option<String> {
        self.plugin.version_command.clone()
    }

    fn version_regex(&self) -> String {
        self.plugin.version_regex.clone()
    }

    fn update_hash(&self, sha: &mut sha2::Sha256) -> Result<()> {
        self.download.update_hash(sha, &self.name())?;
        Ok(())
    }

    fn install(&self, task: &ProgressTask) -> Result<()> {
        task.set_message(&format!("Installing {}", self.name()));
        self.download.install(self)?;

        Ok(())
    }

    fn extra_env_paths(&self) -> Vec<String> {
        vec![self.directory()]
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn plugin(&self) -> Option<PluginDef> {
        Some(self.plugin.clone())
    }
}

pub fn system_arch() -> Cpu {
    match std::env::consts::ARCH {
        "x86_64" => Cpu::X86_64,
        "aarch64" => Cpu::Aarch64,
        _ => Cpu::X86_64,
    }
}

pub fn system_os() -> OperatingSystem {
    match std::env::consts::OS {
        "linux" => OperatingSystem::Linux,
        "macos" => OperatingSystem::MacOS,
        "windows" => OperatingSystem::Windows,
        _ => OperatingSystem::Linux,
    }
}

fn strip_components(path: &Path, n: usize) -> PathBuf {
    path.components()
        .skip(n)
        .fold(PathBuf::new(), |mut acc, comp| {
            acc.push(comp);
            acc
        })
        .to_path_buf()
}

fn finalize_installation_from_download_result(
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
    write_to_file(installation);

    Ok(())
}
