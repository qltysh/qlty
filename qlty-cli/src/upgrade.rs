use crate::attestation;
use crate::get_exe_name;
use anyhow::{bail, Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm};
use duct::cmd;
use qlty_config::http;
use qlty_config::version::{qlty_semver, QLTY_VERSION};
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::time::SystemTime;
use tar::Archive;

const USER_AGENT_PREFIX: &str = "qlty";
const VERSION_CHECK_INTERVAL: u64 = 24 * 60 * 60; // 24 hours

const DEFAULT_MANIFEST_LOCATION_URL: &str =
    "https://qlty-releases.s3.amazonaws.com/qlty/latest/dist-manifest.json";
const DEFAULT_INSTALL_URL: &str = "https://qlty.sh";

#[derive(Debug, Clone)]
pub struct QltyRelease {
    pub version: String,
}

impl QltyRelease {
    pub fn upgrade_check() -> Result<()> {
        if let Some(new_version) = Self::check_upgrade_needed()? {
            println!();
            println!(
                "{} {} of qlty is available!",
                console::style("A new version").bold(),
                console::style(&new_version).cyan().bold()
            );

            if Self::ask_for_upgrade_confirmation()? {
                Self::run_upgrade(&new_version)?;
            }
        }

        Ok(())
    }

    pub fn run_upgrade(version: &str) -> Result<()> {
        println!();
        println!(
            "Running {} {} {} ...",
            console::style("qlty upgrade").bold(),
            console::style("--version").bold(),
            console::style(&version).cyan().bold(),
        );
        println!();

        cmd!(get_exe_name(), "upgrade", "--version", version, "--force").run()?;

        Ok(())
    }

    pub fn ask_for_upgrade_confirmation() -> Result<bool> {
        Ok(Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to upgrade qlty now?")
            .default(true)
            .show_default(true)
            .interact()?)
    }

    pub fn check_upgrade_needed() -> Result<Option<String>> {
        let mut user_data = qlty_config::UserData::create_or_load()?;

        if let Ok(elapsed) = SystemTime::now().duration_since(user_data.version_checked_at) {
            if elapsed.as_secs() < VERSION_CHECK_INTERVAL {
                return Ok(None);
            }
        }

        let release = Self::load_latest()?;
        user_data.touch_version_checked_at()?;

        if release.semver()? > qlty_semver() {
            return Ok(Some(release.version));
        }

        Ok(None)
    }

    pub fn load(tag: &Option<String>) -> Result<Self> {
        match tag {
            Some(tag) => Self::load_version(tag.clone()),
            None => Self::load_latest(),
        }
    }

    fn load_version(tag: String) -> Result<Self> {
        Ok(Self {
            version: tag.strip_prefix('v').unwrap_or(&tag).to_string(),
        })
    }

    fn load_latest() -> Result<Self> {
        let url = if let Ok(override_url) = std::env::var("QLTY_UPDATE_MANIFEST_URL") {
            override_url
        } else {
            DEFAULT_MANIFEST_LOCATION_URL.to_string()
        };

        let response = http::get(&url)?
            .set(
                "User-Agent",
                &format!("{}/{}", USER_AGENT_PREFIX, QLTY_VERSION),
            )
            .call()
            .with_context(|| format!("Unable to get URL: {}", &url))?;

        if response.status() != 200 {
            bail!("GET {} returned {} status", &url, response.status());
        }

        let result: DistManifest = serde_json::from_str(&response.into_string()?)
            .with_context(|| "Failed to parse JSON")?;

        let version = result
            .announcement_tag
            .strip_prefix('v')
            .unwrap_or(&result.announcement_tag)
            .to_string();
        Ok(Self { version })
    }

    pub fn semver(&self) -> Result<semver::Version> {
        semver::Version::parse(&self.version).with_context(|| {
            format!(
                "Unable to parse version string as semver: {}",
                &self.version
            )
        })
    }

    pub fn run_upgrade_command(&self, verify_attestations: bool) -> Result<()> {
        if verify_attestations {
            self.run_verified_upgrade()
        } else {
            self.run_installer_script()
        }
    }

    fn run_installer_script(&self) -> Result<()> {
        let exe_path = std::env::current_exe()?;
        let bin_path = exe_path.parent().unwrap();
        self.upgrade_command()
            .env("QLTY_VERSION", &self.version)
            .env("QLTY_INSTALL_BIN_PATH", bin_path)
            .env("QLTY_NO_MODIFY_PATH", "1")
            .stdin_bytes(Self::download_installer()?.as_bytes())
            .run()
            .map(|_| ())
            .map_err(Into::into)
    }

    fn run_verified_upgrade(&self) -> Result<()> {
        let target = detect_target()?;
        let archive_name = format!("qlty-{}.tar.xz", target);
        let url = format!(
            "https://qlty-releases.s3.amazonaws.com/qlty/v{}/{}",
            self.version, archive_name
        );

        eprintln!("Downloading qlty v{}...", self.version);

        let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
        let archive_path = temp_dir.path().join(&archive_name);
        download_to_file(&url, &archive_path)?;

        attestation::verify_attestation(&archive_path)?;

        eprintln!("Installing...");
        let exe_path = std::env::current_exe()?;
        let bin_path = exe_path
            .parent()
            .context("Could not determine binary directory")?;
        extract_tarxz(&archive_path, bin_path, target)?;

        Ok(())
    }

    fn upgrade_command(&self) -> duct::Expression {
        if cfg!(windows) {
            cmd!("powershell", "-Command", "-")
        } else {
            cmd!("sh")
        }
    }

    fn install_url() -> String {
        std::env::var("QLTY_INSTALL_URL").unwrap_or_else(|_| DEFAULT_INSTALL_URL.to_string())
    }

    fn installer_user_agent() -> String {
        // emulate correct user-agent to retrieve install script
        let prefix = if cfg!(windows) {
            "WindowsPowerShell"
        } else {
            "curl"
        };

        format!("{}/{}-{}", prefix, USER_AGENT_PREFIX, QLTY_VERSION)
    }

    fn download_installer() -> Result<String> {
        http::get(&Self::install_url())?
            .set("User-Agent", &Self::installer_user_agent())
            .call()
            .with_context(|| format!("Failed to download installer from {}", &Self::install_url()))?
            .into_string()
            .map_err(Into::into)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
struct DistManifest {
    #[serde(default)]
    announcement_tag: String,
}

fn detect_target() -> Result<&'static str> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    match (os, arch) {
        ("macos", "x86_64") => Ok("x86_64-apple-darwin"),
        ("macos", "aarch64") => Ok("aarch64-apple-darwin"),
        ("linux", "x86_64") => Ok("x86_64-unknown-linux-gnu"),
        ("linux", "aarch64") => Ok("aarch64-unknown-linux-gnu"),
        _ => bail!("Unsupported platform: {}-{}", os, arch),
    }
}

fn download_to_file(url: &str, path: &Path) -> Result<()> {
    let response = http::get(url)?
        .call()
        .with_context(|| format!("Failed to download from {}", url))?;

    let mut file =
        File::create(path).with_context(|| format!("Failed to create {}", path.display()))?;
    let mut reader = response.into_reader();
    std::io::copy(&mut reader, &mut file).context("Failed to write downloaded content")?;

    Ok(())
}

fn extract_tarxz(archive_path: &Path, dest_dir: &Path, target: &str) -> Result<()> {
    let file = File::open(archive_path).context("Failed to open archive")?;
    let mut reader = BufReader::new(file);
    let mut tar_data: Vec<u8> = Vec::new();

    lzma_rs::xz_decompress(&mut reader, &mut tar_data)
        .context("Failed to decompress xz archive")?;

    let cursor = Cursor::new(tar_data);
    let mut archive = Archive::new(cursor);

    let expected_binary = format!("qlty-{}/qlty", target);

    for entry in archive
        .entries()
        .context("Failed to read archive entries")?
    {
        let mut entry = entry.context("Failed to read archive entry")?;
        let path = entry.path().context("Failed to get entry path")?;
        let path_str = path.to_string_lossy();

        if path_str == expected_binary {
            let dest_path = dest_dir.join("qlty");
            let mut dest_file = File::create(&dest_path)
                .with_context(|| format!("Failed to create {}", dest_path.display()))?;
            std::io::copy(&mut entry, &mut dest_file).context("Failed to extract binary")?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = dest_file.metadata()?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&dest_path, perms)?;
            }

            return Ok(());
        }
    }

    bail!("Binary not found in archive: {}", expected_binary)
}
