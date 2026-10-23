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
use duct::{cmd, Expression};
use qlty_analysis::join_path_string;
use qlty_config::config::OperatingSystem;
use qlty_config::config::PluginDef;
use qlty_config::config::{Cpu, DownloadDef, System};
use sha2::Digest;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::Output;
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

        for alias in distrusted_aliases(&self.list_certificates(&keytool, &cacerts)?) {
            self.delete_certificate(&keytool, &cacerts, &alias)?;
        }

        verify_distrusted_roots_absent(&self.list_certificates(&keytool, &cacerts)?)
    }

    // The listing stays out of the install log, where it would bury any
    // error under thousands of lines of certificate details.
    fn list_certificates(&self, keytool: &Path, cacerts: &Path) -> Result<String> {
        let args = ["-list", "-v"];
        let cmd = self.keytool_command(keytool, cacerts, &args)?;
        debug!("{:?}", cmd);
        keytool_stdout(&args, cmd.run()?)
    }

    fn delete_certificate(&self, keytool: &Path, cacerts: &Path, alias: &str) -> Result<()> {
        let args = ["-delete", "-alias", alias];
        let cmd = self.keytool_command(keytool, cacerts, &args)?;

        let script = format!("{:?}", cmd);
        debug!(script);

        let mut installation = initialize_installation(self)?;
        let result = cmd.run();
        finalize_installation_from_cmd_result(self, &result, &mut installation, script).ok();

        keytool_stdout(&args, result?)?;
        Ok(())
    }

    fn keytool_command(&self, keytool: &Path, cacerts: &Path, args: &[&str]) -> Result<Expression> {
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

        Ok(cmd(keytool, all_args)
            .full_env(self.env()?)
            .unchecked()
            .stderr_capture()
            .stdout_capture())
    }
}

fn keytool_stdout(args: &[&str], output: Output) -> Result<String> {
    if !output.status.success() {
        bail!(
            "keytool {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn normalize_fingerprint(fingerprint: &str) -> String {
    fingerprint.trim().replace(':', "").to_uppercase()
}

fn is_distrusted(fingerprint: &str) -> bool {
    DISTRUSTED_ROOTS.iter().any(|(_, fp)| *fp == fingerprint)
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
            if is_distrusted(&normalize_fingerprint(fingerprint)) {
                if let Some(name) = alias.take() {
                    aliases.push(name);
                }
            }
        }
    }

    aliases
}

// Reads only the SHA256 lines, so a change in keytool's alias label cannot
// make a remaining root look removed.
fn verify_distrusted_roots_absent(listing: &str) -> Result<()> {
    let fingerprints: Vec<String> = listing
        .lines()
        .filter_map(|line| line.trim().strip_prefix("SHA256:"))
        .map(normalize_fingerprint)
        .collect();

    if fingerprints.is_empty() {
        bail!("Found no SHA256 fingerprints in the Java trust store listing");
    }

    let remaining: Vec<&str> = DISTRUSTED_ROOTS
        .iter()
        .filter(|(_, fp)| fingerprints.iter().any(|found| found == fp))
        .map(|(name, _)| *name)
        .collect();

    if !remaining.is_empty() {
        bail!(
            "Failed to remove distrusted roots from Java: {:?}",
            remaining
        );
    }

    Ok(())
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
    use crate::tool::command_builder::test::{reroute_tools_root, ENV_LOCK};
    use tempfile::TempDir;

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

    const LISTING_WITHOUT_ROOTS: &str = "Alias name: cn_usertrust_rsa_certification_authority [jdk]
Certificate fingerprints:
\t SHA256: E7:93:C9:B0:2F:D8:AA:13:E2:1C:31:22:8A:CC:B0:81:19:64:3B:74:9C:89:89:64:B1:74:6D:46:C3:D4:CB:D2
";

    #[test]
    fn distrusted_aliases_empty_when_roots_absent() {
        assert!(distrusted_aliases(LISTING_WITHOUT_ROOTS).is_empty());
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

    fn installed_java(temp_dir: &TempDir, home: &[&str]) -> (Java, PathBuf) {
        let java = Java {
            version: "22.0.1+8".to_string(),
        };
        reroute_tools_root(temp_dir, &java);
        let java_home = home
            .iter()
            .fold(PathBuf::from(java.directory()), |path, part| {
                path.join(part)
            });
        std::fs::create_dir_all(java_home.join("lib").join("security")).unwrap();
        std::fs::create_dir_all(java_home.join("bin")).unwrap();
        std::fs::write(java_home.join("lib").join("security").join("cacerts"), "").unwrap();
        (java, java_home)
    }

    // A stand-in for keytool: -list prints listing.txt, and -delete
    // replaces listing.txt with after_delete.txt.
    #[cfg(unix)]
    fn install_fake_keytool(java_home: &Path, listing: &str, after_delete: &str) {
        use std::os::unix::fs::PermissionsExt;

        let bin = java_home.join("bin");
        std::fs::write(bin.join("listing.txt"), listing).unwrap();
        std::fs::write(bin.join("after_delete.txt"), after_delete).unwrap();
        let keytool = bin.join("keytool");
        std::fs::write(
            &keytool,
            "#!/bin/sh\n\
             dir=$(dirname \"$0\")\n\
             echo \"$@\" >> \"$dir/calls.txt\"\n\
             case \"$1\" in\n\
             -list) cat \"$dir/listing.txt\" ;;\n\
             -delete) cp \"$dir/after_delete.txt\" \"$dir/listing.txt\" ;;\n\
             *) exit 1 ;;\n\
             esac\n",
        )
        .unwrap();
        std::fs::set_permissions(&keytool, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    fn java_home_finds_linux_layout() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let temp_dir = tempfile::tempdir().unwrap();
        let (java, java_home) = installed_java(&temp_dir, &[]);

        assert_eq!(java.java_home().unwrap(), java_home);
    }

    #[test]
    fn java_home_finds_macos_layout() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let temp_dir = tempfile::tempdir().unwrap();
        let (java, java_home) = installed_java(&temp_dir, &["Contents", "Home"]);

        assert_eq!(java.java_home().unwrap(), java_home);
    }

    #[test]
    fn java_home_errors_without_cacerts() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let temp_dir = tempfile::tempdir().unwrap();
        let java = Java {
            version: "22.0.1+8".to_string(),
        };
        reroute_tools_root(&temp_dir, &java);

        assert!(java.java_home().is_err());
    }

    #[cfg(unix)]
    #[test]
    fn remove_distrusted_roots_deletes_each_distrusted_alias() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let temp_dir = tempfile::tempdir().unwrap();
        let (java, java_home) = installed_java(&temp_dir, &[]);
        install_fake_keytool(&java_home, LISTING, LISTING_WITHOUT_ROOTS);

        java.remove_distrusted_roots().unwrap();

        let calls = std::fs::read_to_string(java_home.join("bin").join("calls.txt")).unwrap();
        let deleted: Vec<&str> = calls
            .lines()
            .filter_map(|line| line.strip_prefix("-delete -alias "))
            .map(|rest| rest.split(" -keystore").next().unwrap())
            .collect();
        assert_eq!(
            deleted,
            vec![
                "cn_aaa_certificate_services,o_comodo_ca_limited [jdk]",
                "cn_emsign_ecc_root_ca__g3,o_emudhra_technologies_limited [jdk]",
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn remove_distrusted_roots_fails_when_a_root_remains() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let temp_dir = tempfile::tempdir().unwrap();
        let (java, java_home) = installed_java(&temp_dir, &[]);
        install_fake_keytool(&java_home, LISTING, LISTING);

        let error = java.remove_distrusted_roots().unwrap_err();

        assert!(error
            .to_string()
            .starts_with("Failed to remove distrusted roots from Java"));
    }

    #[test]
    fn verify_distrusted_roots_absent_passes_without_roots() {
        assert!(verify_distrusted_roots_absent(LISTING_WITHOUT_ROOTS).is_ok());
    }

    #[test]
    fn verify_distrusted_roots_absent_fails_when_a_root_remains() {
        let error = verify_distrusted_roots_absent(LISTING).unwrap_err();

        assert_eq!(
            error.to_string(),
            "Failed to remove distrusted roots from Java: [\"emSign ECC Root CA - G3\", \"AAA Certificate Services\"]"
        );
    }

    #[test]
    fn verify_distrusted_roots_absent_fails_without_fingerprints() {
        let error =
            verify_distrusted_roots_absent("Your keystore contains 0 entries\n").unwrap_err();

        assert_eq!(
            error.to_string(),
            "Found no SHA256 fingerprints in the Java trust store listing"
        );
    }

    #[cfg(unix)]
    #[test]
    fn remove_distrusted_roots_fails_when_alias_label_is_unrecognized() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let temp_dir = tempfile::tempdir().unwrap();
        let (java, java_home) = installed_java(&temp_dir, &[]);
        let localized = LISTING.replace("Alias name:", "別名:");
        install_fake_keytool(&java_home, &localized, &localized);

        let error = java.remove_distrusted_roots().unwrap_err();

        assert!(error
            .to_string()
            .starts_with("Failed to remove distrusted roots from Java"));
    }

    #[cfg(unix)]
    #[test]
    fn remove_distrusted_roots_keeps_listing_out_of_install_log() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let temp_dir = tempfile::tempdir().unwrap();
        let (java, java_home) = installed_java(&temp_dir, &[]);
        install_fake_keytool(&java_home, LISTING, LISTING_WITHOUT_ROOTS);

        java.remove_distrusted_roots().unwrap();

        let log = std::fs::read_to_string(java.install_log_path()).unwrap();
        assert!(!log.contains("SHA256:"));
    }

    #[cfg(unix)]
    #[test]
    fn remove_distrusted_roots_fails_when_keytool_fails() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let temp_dir = tempfile::tempdir().unwrap();
        let (java, java_home) = installed_java(&temp_dir, &[]);
        install_fake_keytool(&java_home, LISTING, LISTING_WITHOUT_ROOTS);
        std::fs::remove_file(java_home.join("bin").join("listing.txt")).unwrap();

        let error = java.remove_distrusted_roots().unwrap_err();

        assert!(error.to_string().starts_with("keytool -list -v failed"));
    }
}
