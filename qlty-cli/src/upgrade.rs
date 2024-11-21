use anyhow::{bail, Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm};
use duct::cmd;
use indicatif::ProgressBar;
use itertools::Itertools;
use qlty_analysis::version::QLTY_VERSION;
use serde::Deserialize;

use std::time::SystemTime;

const USER_AGENT_PREFIX: &str = "qlty";
const VERSION_CHECK_INTERVAL: u64 = 24 * 60 * 60; // 24 hours

#[derive(Debug, Clone)]
pub struct QltyRelease {
    pub version: String,
    pub size: u64,
    pub filename: String,
    pub download_url: String,
}

pub enum ReleaseSpec {
    Latest,
    Tag(String),
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
            .default(false)
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

        if release.semver()? > qlty_analysis::version::qlty_semver() {
            return Ok(Some(release.version));
        }

        Ok(None)
    }

    fn load_latest() -> Result<Self> {
        Self::load(ReleaseSpec::Latest)
    }

    pub fn load(spec: ReleaseSpec) -> Result<Self> {
        let url = "http://qlty-releases.s3.amazonaws.com/?list-type=2&prefix=qlty/&delimiter=/";
        let response = ureq::get(url)
            .set(
                "User-Agent",
                &format!("{}/{}", USER_AGENT_PREFIX, QLTY_VERSION),
            )
            .call()
            .with_context(|| format!("Unable to get URL: {}", &url))?;

        if response.status() != 200 {
            bail!("GET {} returned {} status", &url, response.status());
        }

        let result: ListBucketResult = serde_xml_rs::from_str(&response.into_string()?)
            .with_context(|| "Failed to parse XML")?;

        let version = match spec {
            ReleaseSpec::Latest => {
                let semvers = result
                    .releases
                    .iter()
                    .filter_map(|release| {
                        let version = release
                            .prefix
                            .trim_start_matches("qlty/v")
                            .trim_end_matches('/');
                        semver::Version::parse(version).ok()
                    })
                    .sorted()
                    .collect::<Vec<_>>();

                semvers.last().unwrap().to_string()
            }
            ReleaseSpec::Tag(tag) => tag,
        };

        let url =
            format!("http://qlty-releases.s3.amazonaws.com/?list-type=2&prefix=qlty/v{version}/&delimiter=/");

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

        let result: ListBucketResult = serde_xml_rs::from_str(&response.into_string()?)
            .with_context(|| "Failed to parse XML")?;

        let (filename, download_url, size) = Self::find_asset(&result, &version)?;

        Ok(Self {
            version,
            download_url,
            size,
            filename,
        })
    }

    pub fn semver(&self) -> Result<semver::Version> {
        semver::Version::parse(&self.version).with_context(|| {
            format!(
                "Unable to parse version string as semver: {}",
                &self.version
            )
        })
    }

    pub fn download(&self) -> Result<Vec<u8>> {
        let response = ureq::get(&self.download_url)
            .set(
                "User-Agent",
                &format!("{}/{}", USER_AGENT_PREFIX, QLTY_VERSION),
            )
            .set("Accept", "application/octet-stream")
            .call()
            .with_context(|| format!("Unable to get URL: {}", &self.download_url))?;

        if response.status() != 200 {
            bail!(
                "GET {} returned {} status",
                &self.download_url,
                response.status()
            );
        }

        let bytes = Self::download_with_progress(response)?;

        if bytes.len() != self.size as usize {
            bail!(
                "GET {} returned {} bytes, expected {}",
                &self.download_url,
                bytes.len(),
                self.size
            );
        }

        Ok(bytes)
    }

    pub fn download_with_progress(response: ureq::Response) -> Result<Vec<u8>> {
        let content_length = response
            .header("Content-Length")
            .ok_or("Failed to get content length".to_string())
            .unwrap();

        let content_length_u64 = content_length
            .parse::<u64>()
            .with_context(|| format!("Failed to parse content length: {}", content_length))?;

        let progress = Self::build_progress_bar(content_length_u64);

        let mut bytes: Vec<u8> = vec![];
        let mut stream = response.into_reader();
        let mut buffer = [0; 1024];

        while let Ok(bytes_read) = stream.read(&mut buffer) {
            if bytes_read == 0 {
                break;
            }

            bytes.extend_from_slice(&buffer[..bytes_read]);
            progress.set_position(bytes.len() as u64);
        }

        progress.finish_and_clear();

        if bytes.is_empty() {
            bail!("GET returned empty response");
        }

        Ok(bytes)
    }

    fn build_progress_bar(total_bytes: u64) -> ProgressBar {
        let progress = ProgressBar::new(total_bytes);
        progress.set_style(Self::download_bar_style());
        progress.set_prefix("Downloading");
        progress
    }

    fn download_bar_style() -> indicatif::ProgressStyle {
        indicatif::ProgressStyle::with_template(
            "{prefix:.cyan.bold}  {percent}% [{wide_bar}]  {bytes}/{total_bytes}",
        )
        .unwrap()
        .progress_chars("=> ")
    }

    fn find_asset(release: &ListBucketResult, version: &str) -> Result<(String, String, u64)> {
        for asset in &release.files {
            if !asset.key.ends_with(".xz") {
                continue;
            }

            let name = asset.key.split('/').last().expect("key");
            let platform = name
                .strip_prefix("qlty-")
                .expect("name starts with qlty-")
                .strip_suffix(".tar.xz")
                .expect("name ends with .tar.xz");

            if platform == Self::current_platform() {
                let size = asset.size as u64;
                let url = format!("http://qlty-releases.s3.amazonaws.com/qlty/v{version}/{name}");
                return Ok((name.to_owned(), url.to_owned(), size));
            }
        }

        bail!(
            "qlty v{} is out, but not for this platform ({}) yet.",
            version,
            Self::current_platform()
        );
    }

    pub fn current_platform() -> String {
        let cpu_architecture = match std::env::consts::ARCH {
            "aarch64" => "aarch64",
            _ => "x86_64",
        };

        let platform_label = match std::env::consts::OS {
            "macos" => "apple-darwin",
            _ => "unknown-linux-gnu",
        };

        format!("{}-{}", cpu_architecture, platform_label)
    }
}

impl std::fmt::Display for QltyRelease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.version)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
struct ListBucketResult {
    #[serde(default, rename = "CommonPrefixes")]
    releases: Vec<Release>,

    #[serde(default, rename = "Contents")]
    files: Vec<Object>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Release {
    #[serde(rename = "Prefix")]
    prefix: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Object {
    #[serde(rename = "Key")]
    key: String,

    #[serde(rename = "Size")]
    size: usize,
}
