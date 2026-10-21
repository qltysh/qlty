use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

const QLTY_OWNER: &str = "qltysh";

// gh 2.49.0 through 2.66.x can exit 0 when no attestation matches (CVE-2025-25204).
const MINIMUM_GH_VERSION: (u32, u32, u32) = (2, 67, 0);

pub fn verify_attestation(archive_path: &Path) -> Result<()> {
    check_gh_version(gh_version_output().as_deref())?;

    eprintln!("Verifying provenance...");

    let output = Command::new("gh")
        .args([
            "attestation",
            "verify",
            &archive_path.to_string_lossy(),
            "--owner",
            QLTY_OWNER,
            "--hostname",
            "github.com",
        ])
        .output()
        .context("Failed to run gh attestation verify")?;

    check_verification(
        output.status.code(),
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    )?;

    eprintln!(
        "  {} Verified SLSA provenance from github.com/{}",
        console::style("OK").green().bold(),
        QLTY_OWNER
    );
    Ok(())
}

fn gh_version_output() -> Option<String> {
    Command::new("gh")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
}

fn check_gh_version(version_output: Option<&str>) -> Result<()> {
    let (major, minor, patch) = MINIMUM_GH_VERSION;
    let minimum = format!("{major}.{minor}.{patch}");

    let Some(version_output) = version_output else {
        bail!(
            "Provenance verification requires the GitHub CLI (gh) {minimum} or later. Install the GitHub CLI from https://cli.github.com/"
        );
    };

    let version = parse_gh_version(version_output)
        .with_context(|| format!("Unable to read the GitHub CLI version from: {version_output}"))?;

    if version < MINIMUM_GH_VERSION {
        let (major, minor, patch) = version;
        bail!(
            "GitHub CLI (gh) {major}.{minor}.{patch} is older than {minimum}, which provenance verification requires. Upgrade it from https://cli.github.com/"
        );
    }

    Ok(())
}

fn parse_gh_version(version_output: &str) -> Option<(u32, u32, u32)> {
    let version = version_output.split_whitespace().nth(2)?;
    let mut parts = version.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

fn check_verification(exit_code: Option<i32>, stdout: &str, stderr: &str) -> Result<()> {
    match exit_code {
        Some(0) => Ok(()),
        Some(4) => bail!(
            "Provenance verification failed: the GitHub CLI (gh) is not signed in. Run: gh auth login"
        ),
        _ => {
            let mut message = "Provenance verification failed".to_string();
            if !stderr.is_empty() {
                message.push_str(&format!(": {}", stderr.trim()));
            } else if !stdout.is_empty() {
                message.push_str(&format!(": {}", stdout.trim()));
            }
            bail!(message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_constant_is_set() {
        assert_eq!(QLTY_OWNER, "qltysh");
    }

    #[test]
    fn missing_gh_fails() {
        let error = check_gh_version(None).unwrap_err().to_string();
        assert!(error.contains("Install the GitHub CLI"));
    }

    #[test]
    fn old_gh_fails() {
        let error = check_gh_version(Some("gh version 2.66.0 (2025-02-01)\n"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("2.66.0 is older than 2.67.0"));
    }

    #[test]
    fn current_gh_passes() {
        assert!(check_gh_version(Some(
            "gh version 2.97.0 (2026-07-31)\nhttps://github.com/cli/cli/releases/tag/v2.97.0\n"
        ))
        .is_ok());
    }

    #[test]
    fn exit_zero_verifies() {
        assert!(check_verification(Some(0), "", "").is_ok());
    }

    #[test]
    fn exit_four_fails_with_sign_in_hint() {
        let error = check_verification(Some(4), "", "To get started with GitHub CLI")
            .unwrap_err()
            .to_string();
        assert!(error.contains("gh auth login"));
    }

    #[test]
    fn other_exit_fails_with_gh_output() {
        let error = check_verification(Some(1), "", "Error: HTTP 404: Not Found\n")
            .unwrap_err()
            .to_string();
        assert_eq!(
            error,
            "Provenance verification failed: Error: HTTP 404: Not Found"
        );
    }
}
