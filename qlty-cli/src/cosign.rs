use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;
use std::process::Command;

const COSIGN_VERSION: &str = "v2.4.3";
const MIN_COSIGN_VERSION: (u32, u32, u32) = (2, 4, 0);

struct CosignBinary {
    os: &'static str,
    arch: &'static str,
    sha256: &'static str,
}

const COSIGN_BINARIES: &[CosignBinary] = &[
    CosignBinary {
        os: "darwin",
        arch: "amd64",
        sha256: "98a3bfd691f42c6a5b721880116f89210d8fdff61cc0224cd3ef2f8e55a466fb",
    },
    CosignBinary {
        os: "darwin",
        arch: "arm64",
        sha256: "edfc761b27ced77f0f9ca288ff4fac7caa898e1e9db38f4dfdf72160cdf8e638",
    },
    CosignBinary {
        os: "linux",
        arch: "amd64",
        sha256: "caaad125acef1cb81d58dcdc454a1e429d09a750d1e9e2b3ed1aed8964454708",
    },
    CosignBinary {
        os: "linux",
        arch: "arm64",
        sha256: "bd0f9763bca54de88699c3656ade2f39c9a1c7a2916ff35601caf23a79be0629",
    },
];

pub struct CosignProvider;

impl CosignProvider {
    pub fn get_cosign() -> Result<PathBuf> {
        if let Some(path) = Self::find_system_cosign()? {
            return Ok(path);
        }

        let cached_path = Self::cached_cosign_path()?;
        if cached_path.exists() {
            return Ok(cached_path);
        }

        Self::download_cosign()
    }

    fn find_system_cosign() -> Result<Option<PathBuf>> {
        let output = match Command::new("cosign").arg("version").output() {
            Ok(output) => output,
            Err(_) => return Ok(None),
        };

        if !output.status.success() {
            return Ok(None);
        }

        let version_output = String::from_utf8_lossy(&output.stdout);
        if let Some(version) = Self::parse_cosign_version(&version_output) {
            if version >= MIN_COSIGN_VERSION {
                return Ok(Some(PathBuf::from("cosign")));
            }
        }

        Ok(None)
    }

    fn parse_cosign_version(output: &str) -> Option<(u32, u32, u32)> {
        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("cosign version") || line.starts_with("GitVersion:") {
                let version_str = line
                    .split_whitespace()
                    .last()?
                    .trim_start_matches('v')
                    .trim_start_matches("GitVersion:")
                    .trim()
                    .trim_start_matches('v');

                let parts: Vec<&str> = version_str.split('.').collect();
                if parts.len() >= 3 {
                    let major = parts[0].parse().ok()?;
                    let minor = parts[1].parse().ok()?;
                    let patch = parts[2]
                        .split(|c: char| !c.is_ascii_digit())
                        .next()?
                        .parse()
                        .ok()?;
                    return Some((major, minor, patch));
                }
            }
        }
        None
    }

    fn cached_cosign_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Could not determine home directory")?;
        Ok(PathBuf::from(home)
            .join(".qlty")
            .join(format!("cosign-{}", COSIGN_VERSION)))
    }

    fn download_cosign() -> Result<PathBuf> {
        let binary = Self::get_binary_for_platform()?;
        let url = format!(
            "https://github.com/sigstore/cosign/releases/download/{}/cosign-{}-{}",
            COSIGN_VERSION, binary.os, binary.arch
        );

        eprintln!("  Downloading cosign {}...", COSIGN_VERSION);

        let response = qlty_config::http::get(&url)
            .call()
            .with_context(|| format!("Failed to download cosign from {}", url))?;

        let mut data = Vec::new();
        response
            .into_reader()
            .read_to_end(&mut data)
            .context("Failed to read cosign binary")?;

        let computed_hash = Self::compute_sha256(&data);
        if computed_hash != binary.sha256 {
            bail!(
                "SHA256 mismatch for cosign binary: expected {}, got {}",
                binary.sha256,
                computed_hash
            );
        }

        let cached_path = Self::cached_cosign_path()?;
        if let Some(parent) = cached_path.parent() {
            fs::create_dir_all(parent).context("Failed to create ~/.qlty directory")?;
        }

        let mut file = File::create(&cached_path).context("Failed to create cosign binary file")?;
        file.write_all(&data)
            .context("Failed to write cosign binary")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&cached_path, perms)?;
        }

        Ok(cached_path)
    }

    fn get_binary_for_platform() -> Result<&'static CosignBinary> {
        let os = match std::env::consts::OS {
            "macos" => "darwin",
            "linux" => "linux",
            other => bail!("Unsupported operating system: {}", other),
        };

        let arch = match std::env::consts::ARCH {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            other => bail!("Unsupported architecture: {}", other),
        };

        COSIGN_BINARIES
            .iter()
            .find(|b| b.os == os && b.arch == arch)
            .ok_or_else(|| anyhow::anyhow!("No cosign binary available for {}-{}", os, arch))
    }

    fn compute_sha256(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
}

pub fn compute_file_sha256(path: &std::path::Path) -> Result<String> {
    let file = File::open(path).context("Failed to open file for hashing")?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cosign_version_from_output() {
        let output = "cosign version: v2.4.3\nGitCommit: abc123";
        assert_eq!(
            CosignProvider::parse_cosign_version(output),
            Some((2, 4, 3))
        );
    }

    #[test]
    fn parses_cosign_version_git_format() {
        let output =
            "  ______   ______        _______. __    _______ .__   __.\n GitVersion:    v2.4.3";
        assert_eq!(
            CosignProvider::parse_cosign_version(output),
            Some((2, 4, 3))
        );
    }

    #[test]
    fn version_comparison() {
        assert!((2, 4, 3) >= MIN_COSIGN_VERSION);
        assert!((2, 5, 0) >= MIN_COSIGN_VERSION);
        assert!((3, 0, 0) >= MIN_COSIGN_VERSION);
        assert!(!((2, 3, 9) >= MIN_COSIGN_VERSION));
    }
}
