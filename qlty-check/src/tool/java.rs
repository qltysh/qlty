use super::installations::initialize_installation;
use super::runnable_archive::RunnableArchive;
use super::Tool;
use super::ToolType;
use crate::tool::download::Download;
use crate::tool::finalize_installation_from_cmd_result;
use crate::tool::RuntimeTool;
use crate::ui::ProgressBar;
use crate::ui::ProgressTask;
use anyhow::{bail, Result};
use duct::cmd;
use qlty_analysis::join_path_string;
use qlty_config::config::OperatingSystem;
use qlty_config::config::PluginDef;
use qlty_config::config::{Cpu, DownloadDef, System};
use sha2::Digest;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use tracing::debug;

// Roots that the compromised eMudhra intermediates chain to, by SHA-256
// fingerprint. Removing the roots distrusts every encoding of those
// intermediates, which blocked.certs cannot guarantee.
const DISTRUSTED_ROOTS: &[(&str, &str)] = &[
    (
        "emSign ECC Root CA - G3",
        "86A1ECBA089C4A8D3BBE2734C612BA341D813E043CF9E8A862CD5C57A36BBE6B",
    ),
    (
        "AAA Certificate Services",
        "D7A7A0FB5D7E2731D771E9484EBCDEF71D5F0C3E0A2948782BC83EE0EA699EF4",
    ),
];

// Bump to reinstall cached runtimes when DISTRUSTED_ROOTS changes.
const TRUST_STORE_REVISION: &str = "distrust-emudhra-2026-10";

// The well-known password of the cacerts file that ships with the JDK.
const CACERTS_PASSWORD: &str = "changeit";

#[derive(Debug, Clone)]
pub struct Java {
    pub version: String,
}

impl Tool for Java {
    fn name(&self) -> String {
        "java".to_string()
    }

    fn tool_type(&self) -> ToolType {
        ToolType::Runtime
    }

    fn update_hash(&self, sha: &mut sha2::Sha256) -> Result<()> {
        self.download().update_hash(sha, &self.name())?;
        sha.update(TRUST_STORE_REVISION.as_bytes());
        Ok(())
    }

    fn version(&self) -> Option<String> {
        Some(self.version.clone())
    }

    fn install(&self, task: &ProgressTask) -> Result<()> {
        task.set_message(&format!("Installing Java v{}", self.version().unwrap()));
        self.download().install(self)?;
        Ok(())
    }

    fn post_install(&self, task: &ProgressTask) -> Result<()> {
        task.set_message("Removing distrusted root certificates from Java");
        self.remove_distrusted_roots()
    }

    fn version_command(&self) -> Option<String> {
        Some("java --version".to_string())
    }

    fn version_regex(&self) -> String {
        r"(\d+\.\d+\.\d+\+\d+)".to_string()
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    // So Java linux and macos releases have different directory structures
    fn extra_env_paths(&self) -> Result<Vec<String>> {
        Ok(vec![
            join_path_string!(self.directory(), "bin"),
            join_path_string!(self.directory(), "Contents", "Home", "bin"),
        ])
    }
}

impl Java {
    fn download(&self) -> Download {
        let major_version = self.version.split('.').next().unwrap();
        let url_version = self.version.replace('+', "_");

        Download::new(
            &DownloadDef {
                strip_components: 1,
                systems: vec![System {
                    url: format!("https://github.com/adoptium/temurin{}-binaries/releases/download/jdk-${{version}}/OpenJDK{}U-jdk_x64_mac_hotspot_{}.tar.gz", major_version, major_version, url_version),
                    cpu: Cpu::X86_64,
                    os: OperatingSystem::MacOS,
                },
                System {
                    url: format!("https://github.com/adoptium/temurin{}-binaries/releases/download/jdk-${{version}}/OpenJDK{}U-jdk_aarch64_mac_hotspot_{}.tar.gz", major_version, major_version, url_version),
                    cpu: Cpu::Aarch64,
                    os: OperatingSystem::MacOS,
                },
                System {
                    url: format!("https://github.com/adoptium/temurin{}-binaries/releases/download/jdk-${{version}}/OpenJDK{}U-jdk_x64_linux_hotspot_{}.tar.gz", major_version, major_version, url_version),
                    cpu: Cpu::X86_64,
                    os: OperatingSystem::Linux,
                },
                System {
                    url: format!("https://github.com/adoptium/temurin{}-binaries/releases/download/jdk-${{version}}/OpenJDK{}U-jdk_aarch64_linux_hotspot_{}.tar.gz", major_version, major_version, url_version),
                    cpu: Cpu::Aarch64,
                    os: OperatingSystem::Linux,
                },
                System {
                    url: format!("https://github.com/adoptium/temurin{}-binaries/releases/download/jdk-${{version}}/OpenJDK{}U-jdk_x64_windows_hotspot_{}.zip", major_version, major_version, url_version),
                    cpu: Cpu::X86_64,
                    os: OperatingSystem::Windows,
                }],
                ..Default::default()
            },
            &self.name(),
            &self.version,
        )
    }

    // Linux and Windows archives unpack to the JDK home; macOS archives
    // unpack to a bundle with the JDK home under Contents/Home.
    fn java_home(&self) -> Result<PathBuf> {
        let directory = PathBuf::from(self.directory());
        for home in [directory.clone(), directory.join("Contents").join("Home")] {
            if home.join("lib").join("security").join("cacerts").exists() {
                return Ok(home);
            }
        }
        bail!(
            "No lib/security/cacerts found under {}",
            directory.display()
        )
    }

    fn remove_distrusted_roots(&self) -> Result<()> {
        let home = self.java_home()?;
        let keytool = home
            .join("bin")
            .join(format!("keytool{}", std::env::consts::EXE_SUFFIX));
        let cacerts = home.join("lib").join("security").join("cacerts");

        for alias in distrusted_aliases(&self.keytool(&keytool, &cacerts, &["-list", "-v"])?) {
            self.keytool(&keytool, &cacerts, &["-delete", "-alias", &alias])?;
        }

        let remaining = distrusted_aliases(&self.keytool(&keytool, &cacerts, &["-list", "-v"])?);
        if !remaining.is_empty() {
            bail!(
                "Failed to remove distrusted roots from Java: {:?}",
                remaining
            );
        }

        Ok(())
    }

    fn keytool(&self, keytool: &Path, cacerts: &Path, args: &[&str]) -> Result<String> {
        let mut all_args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
        all_args.extend([
            "-keystore".to_string(),
            cacerts.to_string_lossy().to_string(),
            "-storepass".to_string(),
            CACERTS_PASSWORD.to_string(),
            // keytool localizes its labels, which distrusted_aliases parses
            "-J-Duser.language=en".to_string(),
            "-J-Duser.country=US".to_string(),
        ]);

        let cmd = cmd(keytool, all_args)
            .full_env(self.env()?)
            .unchecked()
            .stderr_capture()
            .stdout_capture();

        let script = format!("{:?}", cmd);
        debug!(script);

        let mut installation = initialize_installation(self)?;
        let result = cmd.run();
        finalize_installation_from_cmd_result(self, &result, &mut installation, script).ok();

        let output = result?;
        if !output.status.success() {
            bail!(
                "keytool {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

// Returns the aliases in `keytool -list -v` output whose SHA-256
// fingerprint matches a distrusted root.
fn distrusted_aliases(listing: &str) -> Vec<String> {
    let mut aliases = vec![];
    let mut alias = None;

    for line in listing.lines() {
        let line = line.trim();
        if let Some(name) = line.strip_prefix("Alias name:") {
            alias = Some(name.trim().to_string());
        } else if let Some(fingerprint) = line.strip_prefix("SHA256:") {
            let fingerprint = fingerprint.trim().replace(':', "").to_uppercase();
            if DISTRUSTED_ROOTS.iter().any(|(_, fp)| *fp == fingerprint) {
                if let Some(name) = alias.take() {
                    aliases.push(name);
                }
            }
        }
    }

    aliases
}

impl RuntimeTool for Java {
    fn package_tool(&self, name: &str, plugin: &PluginDef) -> Box<dyn Tool> {
        Box::new(JavaPackage {
            name: name.to_owned(),
            plugin: plugin.clone(),
            runtime: self.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct JavaPackage {
    pub name: String,
    pub plugin: PluginDef,
    pub runtime: Java,
}

impl Tool for JavaPackage {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn tool_type(&self) -> ToolType {
        ToolType::RuntimePackage
    }

    fn runtime(&self) -> Option<Box<dyn Tool>> {
        Some(Box::new(self.runtime.clone()))
    }

    fn version(&self) -> Option<String> {
        self.plugin.version.clone()
    }

    fn update_hash(&self, sha: &mut sha2::Sha256) -> Result<()> {
        sha.update(self.name().as_bytes());

        Ok(())
    }

    fn version_command(&self) -> Option<String> {
        self.plugin.version_command.clone()
    }

    fn version_regex(&self) -> String {
        self.plugin.version_regex.clone()
    }

    fn install(&self, _task: &ProgressTask) -> Result<()> {
        self.download().install(self)?;
        Ok(())
    }

    fn extra_env_paths(&self) -> Result<Vec<String>> {
        let mut paths = vec![self.directory(), join_path_string!(self.directory(), "bin")];
        paths.extend(self.runtime.extra_env_paths()?);

        Ok(paths)
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn plugin(&self) -> Option<PluginDef> {
        Some(self.plugin.clone())
    }
}

impl RunnableArchive for JavaPackage {}

#[cfg(test)]
mod test {
    use super::*;

    const LISTING: &str = "Keystore type: PKCS12
Keystore provider: SUN

Your keystore contains 3 entries

Alias name: cn_aaa_certificate_services,o_comodo_ca_limited [jdk]
Creation date: Apr 19, 2024
Entry type: trustedCertEntry

Owner: CN=AAA Certificate Services, O=Comodo CA Limited
Certificate fingerprints:
\t SHA1: D1:EB:23:A4:6D:17:D6:8F:D9:25:64:C2:F1:F1:60:17:64:D8:E3:49
\t SHA256: D7:A7:A0:FB:5D:7E:27:31:D7:71:E9:48:4E:BC:DE:F7:1D:5F:0C:3E:0A:29:48:78:2B:C8:3E:E0:EA:69:9E:F4

Alias name: cn_emsign_ecc_root_ca__g3,o_emudhra_technologies_limited [jdk]
Certificate fingerprints:
\t SHA256: 86:A1:EC:BA:08:9C:4A:8D:3B:BE:27:34:C6:12:BA:34:1D:81:3E:04:3C:F9:E8:A8:62:CD:5C:57:A3:6B:BE:6B

Alias name: cn_usertrust_rsa_certification_authority [jdk]
Certificate fingerprints:
\t SHA256: E7:93:C9:B0:2F:D8:AA:13:E2:1C:31:22:8A:CC:B0:81:19:64:3B:74:9C:89:89:64:B1:74:6D:46:C3:D4:CB:D2
";

    #[test]
    fn distrusted_aliases_finds_both_roots() {
        assert_eq!(
            distrusted_aliases(LISTING),
            vec![
                "cn_aaa_certificate_services,o_comodo_ca_limited [jdk]".to_string(),
                "cn_emsign_ecc_root_ca__g3,o_emudhra_technologies_limited [jdk]".to_string(),
            ]
        );
    }

    #[test]
    fn distrusted_aliases_empty_when_roots_absent() {
        let listing = "Alias name: cn_usertrust_rsa_certification_authority [jdk]
Certificate fingerprints:
\t SHA256: E7:93:C9:B0:2F:D8:AA:13:E2:1C:31:22:8A:CC:B0:81:19:64:3B:74:9C:89:89:64:B1:74:6D:46:C3:D4:CB:D2
";
        assert!(distrusted_aliases(listing).is_empty());
    }

    #[test]
    fn update_hash_includes_trust_store_revision() {
        let java = Java {
            version: "22.0.1+8".to_string(),
        };
        let mut with_revision = sha2::Sha256::new();
        java.update_hash(&mut with_revision).unwrap();

        let mut without_revision = sha2::Sha256::new();
        java.download()
            .update_hash(&mut without_revision, &java.name())
            .unwrap();

        assert_ne!(
            format!("{:x}", with_revision.finalize()),
            format!("{:x}", without_revision.finalize())
        );
    }
}
