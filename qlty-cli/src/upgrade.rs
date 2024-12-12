use anyhow::{bail, Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm};
use duct::cmd;
use qlty_config::version::{qlty_semver, QLTY_VERSION};
use serde::Deserialize;

use std::time::SystemTime;

const USER_AGENT_PREFIX: &str = "qlty";
const VERSION_CHECK_INTERVAL: u64 = 24 * 60 * 60; // 24 hours

const DEFAULT_MANIFEST_LOCATION_URL: &str =
    "http://qlty-releases.s3.amazonaws.com/qlty/latest/dist-manifest.json";
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

        let exe = std::env::current_exe().context("Unable to get current executable path")?;
        cmd!(exe, "upgrade", "--version", version, "--force").run()?;

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

        let response = ureq::get(&url)
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

    pub fn run_upgrade_command(&self) -> Result<()> {
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
        ureq::get(&Self::install_url())
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
