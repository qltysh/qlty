use super::installations::{initialize_installation, write_to_file};
use super::ToolType;
use crate::tool::Download;
use crate::{
    ui::{ProgressBar as _, ProgressTask},
    Tool,
};
use anyhow::Result;
use chrono::Utc;
use once_cell::sync::OnceCell;
use qlty_config::config::{Cpu, DownloadDef, OperatingSystem, PluginDef, ReleaseDef, System};
use qlty_config::version::QLTY_VERSION;
use qlty_types::analysis::v1::Installation;
use sha2::Digest;
use tracing::{debug, info, trace};

const GITHUB_API_VERSION: &str = "2022-11-28";
const USER_AGENT_PREFIX: &str = "qlty-check";

#[derive(Debug, Clone, Default)]
pub struct GitHubRelease {
    pub version: String,
    pub def: ReleaseDef,
}

impl GitHubRelease {
    pub fn new(version: String, def: ReleaseDef) -> Self {
        Self { version, def }
    }

    fn update_hash(&self, sha: &mut sha2::Sha256) -> Result<()> {
        sha.update(self.version.as_bytes());
        sha.update(
            self.def
                .binary_name
                .as_ref()
                .unwrap_or(&"".to_string())
                .as_bytes(),
        );
        sha.update(self.def.download_type.to_string().as_bytes());
        sha.update(self.def.strip_components.to_string().as_bytes());
        Ok(())
    }

    fn download(&self, assets: &[GitHubReleaseAsset]) -> Result<DownloadDef> {
        let systems = self.systems(assets);

        Ok(DownloadDef {
            binary_name: self.def.binary_name.clone(),
            strip_components: self.def.strip_components,
            systems,
        })
    }

    fn systems(&self, assets: &[GitHubReleaseAsset]) -> Vec<System> {
        let candidates = self.candidate_assets(assets);
        let mut systems = vec![];

        if let Some(system) = self.linux_x86_64_system(&candidates) {
            debug!("Found Linux x86_64 system: {}", system.url);
            systems.push(system);
        }

        if let Some(system) = self.linux_aarch64_system(&candidates) {
            debug!("Found Linux aarch64 system: {}", system.url);
            systems.push(system);
        }

        if let Some(system) = self.macos_x86_64_system(&candidates) {
            debug!("Found MacOS x86_64 system: {}", system.url);
            systems.push(system);
        }

        if let Some(system) = self.macos_aarch64_system(&candidates) {
            debug!("Found MacOS aarch64 system: {}", system.url);
            systems.push(system);
        }

        if let Some(system) = self.windows_x86_64_system(&candidates) {
            debug!("Found Windows x86_64 system: {}", system.url);
            systems.push(system);
        }

        if let Some(system) = self.windows_aarch64_system(&candidates) {
            debug!("Found Windows aarch64 system: {}", system.url);
            systems.push(system);
        }

        systems
    }

    fn candidate_assets(&self, assets: &[GitHubReleaseAsset]) -> Vec<GitHubReleaseAsset> {
        assets
            .iter()
            .filter(|a| self.allowed_content_types().contains(&a.content_type))
            .cloned()
            .collect::<Vec<_>>()
    }

    fn linux_x86_64_system(&self, candidates: &[GitHubReleaseAsset]) -> Option<System> {
        Some(System {
            url: self
                .linux_x86_64_asset(candidates)?
                .browser_download_url
                .clone(),
            cpu: Cpu::X86_64,
            os: OperatingSystem::Linux,
        })
    }

    fn linux_aarch64_system(&self, candidates: &[GitHubReleaseAsset]) -> Option<System> {
        Some(System {
            url: self
                .linux_aarch64_asset(candidates)?
                .browser_download_url
                .clone(),
            cpu: Cpu::Aarch64,
            os: OperatingSystem::Linux,
        })
    }

    fn macos_x86_64_system(&self, candidates: &[GitHubReleaseAsset]) -> Option<System> {
        Some(System {
            url: self
                .macos_x86_64_asset(candidates)?
                .browser_download_url
                .clone(),
            cpu: Cpu::X86_64,
            os: OperatingSystem::MacOS,
        })
    }

    fn macos_aarch64_system(&self, candidates: &[GitHubReleaseAsset]) -> Option<System> {
        Some(System {
            url: self
                .macos_aarch64_asset(candidates)?
                .browser_download_url
                .clone(),
            cpu: Cpu::Aarch64,
            os: OperatingSystem::MacOS,
        })
    }

    fn windows_x86_64_system(&self, candidates: &[GitHubReleaseAsset]) -> Option<System> {
        Some(System {
            url: self
                .windows_x86_64_asset(candidates)?
                .browser_download_url
                .clone(),
            cpu: Cpu::X86_64,
            os: OperatingSystem::Windows,
        })
    }

    fn windows_aarch64_system(&self, candidates: &[GitHubReleaseAsset]) -> Option<System> {
        Some(System {
            url: self
                .windows_aarch64_asset(candidates)?
                .browser_download_url
                .clone(),
            cpu: Cpu::Aarch64,
            os: OperatingSystem::Windows,
        })
    }

    fn linux_x86_64_asset(&self, candidates: &[GitHubReleaseAsset]) -> Option<GitHubReleaseAsset> {
        candidates
            .iter()
            .find(|a| self.is_x86_64(&a.name) && self.is_linux(&a.name))
            .cloned()
    }

    fn linux_aarch64_asset(&self, candidates: &[GitHubReleaseAsset]) -> Option<GitHubReleaseAsset> {
        candidates
            .iter()
            .find(|a| self.is_aarch64(&a.name) && self.is_linux(&a.name))
            .cloned()
    }

    fn macos_x86_64_asset(&self, candidates: &[GitHubReleaseAsset]) -> Option<GitHubReleaseAsset> {
        candidates
            .iter()
            .find(|a| self.is_x86_64(&a.name) && self.is_macos(&a.name))
            .cloned()
    }

    fn macos_aarch64_asset(&self, candidates: &[GitHubReleaseAsset]) -> Option<GitHubReleaseAsset> {
        candidates
            .iter()
            .find(|a| self.is_aarch64(&a.name) && self.is_macos(&a.name))
            .cloned()
            .or(self.macos_x86_64_asset(candidates))
    }

    fn windows_x86_64_asset(
        &self,
        candidates: &[GitHubReleaseAsset],
    ) -> Option<GitHubReleaseAsset> {
        candidates
            .iter()
            .find(|a| !self.is_aarch64(&a.name) && !self.is_32bit(&a.name) && self.is_windows(a))
            .cloned()
    }

    fn windows_aarch64_asset(
        &self,
        candidates: &[GitHubReleaseAsset],
    ) -> Option<GitHubReleaseAsset> {
        candidates
            .iter()
            .find(|a| self.is_aarch64(&a.name) && self.is_windows(a))
            .cloned()
    }

    fn is_x86_64(&self, filename: &str) -> bool {
        let lower_case_filename = filename.to_lowercase();
        ["x86_64", "amd64", "x64", "64bit", "64-bit"]
            .iter()
            .any(|s| lower_case_filename.contains(s))
    }

    fn is_aarch64(&self, filename: &str) -> bool {
        let lower_case_filename = filename.to_lowercase();
        ["aarch64", "arm64", "armv8", "arm64e", "armv7", "armv6"]
            .iter()
            .any(|s| lower_case_filename.contains(s))
    }

    fn is_32bit(&self, filename: &str) -> bool {
        let lower_case_filename = filename.to_lowercase();
        lower_case_filename
            .split(&['-', '.', '_'])
            .any(|part| ["386", "i386", "32bit", "32-bit"].contains(&part))
    }

    fn is_linux(&self, filename: &str) -> bool {
        filename.to_lowercase().contains("linux")
    }

    fn is_macos(&self, filename: &str) -> bool {
        filename.to_lowercase().contains("macos") || filename.to_lowercase().contains("darwin")
    }

    fn is_windows(&self, candidate: &GitHubReleaseAsset) -> bool {
        candidate.name.to_lowercase().contains("windows")
            || self
                .allowed_content_types_windows()
                .contains(&candidate.content_type)
            // FIXME(loren): hacky solution to find non-Windows suffixed zip files. This works for shellcheck,
            // but may break apart for other tools.
            || (candidate.name.ends_with(".zip")
                && !self.is_macos(&candidate.name)
                && !self.is_linux(&candidate.name))
    }

    fn allowed_content_types(&self) -> Vec<String> {
        [
            "application/octet-stream",
            "application/gzip",
            "application/x-gtar",
            "application/x-xz",
            "application/zip",
        ]
        .iter()
        .map(|s| s.to_string())
        .chain(self.allowed_content_types_windows())
        .collect::<Vec<_>>()
    }

    fn allowed_content_types_windows(&self) -> Vec<String> {
        [
            "application/x-ms-dos-executable",
            "application/x-msdownload",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, Default)]
pub struct GitHubReleaseTool {
    pub plugin_name: String,
    pub release: GitHubRelease,
    pub plugin: PluginDef,
    pub download: OnceCell<Download>,
    pub runtime: Option<Box<dyn Tool>>,
}

impl Tool for GitHubReleaseTool {
    fn name(&self) -> String {
        self.plugin_name.clone()
    }

    fn tool_type(&self) -> ToolType {
        ToolType::GitHubRelease
    }

    fn version(&self) -> Option<String> {
        Some(self.release.version.clone())
    }

    fn version_command(&self) -> Option<String> {
        self.plugin.version_command.clone()
    }

    fn version_regex(&self) -> String {
        self.plugin.version_regex.clone()
    }

    fn update_hash(&self, sha: &mut sha2::Sha256) -> Result<()> {
        sha.update(self.name().as_bytes());
        self.release.update_hash(sha)?;
        Ok(())
    }

    fn install(&self, task: &ProgressTask) -> Result<()> {
        task.set_message(&format!("Installing {}", self.name()));
        self.download()?.install(self)?;
        Ok(())
    }

    fn extra_env_paths(&self) -> Result<Vec<String>> {
        Ok(vec![self.directory()])
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn plugin(&self) -> Option<PluginDef> {
        Some(self.plugin.clone())
    }

    fn runtime(&self) -> Option<Box<dyn Tool>> {
        if let Some(runtime) = &self.runtime {
            Some(runtime.clone_box())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
struct GitHubReleaseAsset {
    name: String,
    content_type: String,
    browser_download_url: String,
}

impl GitHubReleaseTool {
    fn download(&self) -> Result<Download> {
        self.download
            .get_or_try_init(|| self.compute_download())
            .cloned()
    }

    fn compute_download(&self) -> Result<Download> {
        let assets = self.release_assets()?;
        trace!("Release assets: {:?}", assets);
        let download = Download::new(
            &self.release.download(&assets)?,
            &self.plugin_name,
            &self.release.version,
        );
        Ok(download)
    }

    fn release_assets(&self) -> Result<Vec<GitHubReleaseAsset>> {
        let mut asset_values = vec![];

        info!(
            "Fetching release assets from {} from 'v{}' tag",
            self.release.def.github, self.release.version
        );
        if let Ok(assets) = self.get_release_assets(&format!(
            "https://api.github.com/repos/{}/releases/tags/v{}",
            self.release.def.github, self.release.version
        )) {
            asset_values = assets;
        }

        if asset_values.is_empty() {
            info!(
                "Fetching release assets from {} from '{}' tag",
                self.release.def.github, self.release.version
            );
            asset_values = self.get_release_assets(&format!(
                "https://api.github.com/repos/{}/releases/tags/{}",
                self.release.def.github, self.release.version
            ))?;
        }

        Ok(asset_values
            .iter()
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect())
    }

    fn get_release_assets(&self, url: &str) -> Result<Vec<serde_json::Value>> {
        let mut request = ureq::get(url)
            .set(
                "User-Agent",
                &format!("{}/{}", USER_AGENT_PREFIX, QLTY_VERSION),
            )
            .set("X-GitHub-Api-Version", GITHUB_API_VERSION);

        if let Ok(auth_token) = std::env::var("QLTY_GITHUB_TOKEN") {
            request = request.set("Authorization", &format!("Bearer {}", auth_token));
        }

        let mut installation = initialize_installation(self)?;
        let result = request.call();
        finalize_installation_from_assets_fetch(&mut installation, &result, url);

        let json = result?.into_json::<serde_json::Value>()?;
        json["assets"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No assets found"))
            .cloned()
    }
}

fn finalize_installation_from_assets_fetch(
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

    write_to_file(installation);
}

#[cfg(test)]
mod test {
    use super::{GitHubRelease, GitHubReleaseAsset};
    use qlty_config::config::{DownloadFileType, ReleaseDef};

    #[test]
    fn test_windows_x86_64_asset() {
        let release = GitHubRelease::new(
            "v0.7.0".into(),
            ReleaseDef {
                binary_name: Some("tool".into()),
                github: "repo/tool".into(),
                download_type: DownloadFileType::Zip,
                strip_components: 0,
            },
        );

        let tests = vec![
            ("tool-v0.7.0.windows.x86_64.zip", "application/zip", true),
            ("tool-v0.7.0_windows_armv6.zip", "application/zip", false),
            ("tool-v0.7.0_windows_armv7.zip", "application/zip", false),
            ("tool-v0.7.0_windows_arm64.zip", "application/zip", false),
            ("tool-v0.7.0_windows_386.zip", "application/zip", false),
            ("any_filename.ext", "application/x-msdownload", true),
            ("any_filename.ext", "application/x-ms-dos-executable", true),
            ("tool-v0.7.0.zip", "application/zip", true),
            ("tool-v0.7.0.tar.gz", "application/gzip", false),
            ("tool-v0.7.0.linux.zip", "application/zip", false),
            ("tool-v0.7.0.linux.x86_64.zip", "application/zip", false),
            ("tool-v0.7.0.linux.x86_64.tar.gz", "application/gzip", false),
            ("tool-v0.7.0.macos.x86_64.zip", "application/zip", false),
            ("tool-v0.7.0.aarch64.tar.gz", "application/gzip", false),
        ];

        for (name, content_type, matches) in tests {
            let asset = GitHubReleaseAsset {
                name: name.into(),
                content_type: content_type.into(),
                browser_download_url: "https://example.org".into(),
            };

            let result = if matches { Some(asset.clone()) } else { None };
            assert_eq!(release.windows_x86_64_asset(&[asset.clone()]), result);
        }
    }
}
