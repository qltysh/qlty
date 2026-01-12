use crate::cosign::{compute_file_sha256, CosignProvider};
use anyhow::{bail, Context, Result};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

const QLTY_OWNER: &str = "qltysh";
const QLTY_REPO: &str = "qlty";
const OIDC_ISSUER: &str = "https://token.actions.githubusercontent.com";

pub fn verify_attestation(archive_path: &Path) -> Result<()> {
    let digest = compute_file_sha256(archive_path)?;
    eprintln!("Verifying provenance...");

    let bundle_file = fetch_attestation_bundle(&digest)?;
    let cosign = CosignProvider::get_cosign()?;
    verify_with_cosign(&cosign, archive_path, bundle_file.path())?;

    eprintln!(
        "✓ Verified SLSA provenance from github.com/{}/{}",
        QLTY_OWNER, QLTY_REPO
    );
    Ok(())
}

fn fetch_attestation_bundle(digest: &str) -> Result<NamedTempFile> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/attestations/sha256:{}",
        QLTY_OWNER, QLTY_REPO, digest
    );

    let response = qlty_config::http::get(&url)
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .call()
        .with_context(|| format!("Failed to fetch attestation from GitHub API: {}", url))?;

    if response.status() == 404 {
        bail!("No attestation found for this artifact. The binary may not have been built with attestations enabled.");
    }

    let json: serde_json::Value = response
        .into_json()
        .context("Failed to parse attestation response")?;

    let attestations = json["attestations"]
        .as_array()
        .context("No attestations field in response")?;

    if attestations.is_empty() {
        bail!("No attestations found for this artifact");
    }

    let bundle = &attestations[0]["bundle"];
    if bundle.is_null() {
        bail!("Attestation bundle is missing");
    }

    let mut bundle_file = NamedTempFile::new().context("Failed to create temp file for bundle")?;
    bundle_file
        .write_all(bundle.to_string().as_bytes())
        .context("Failed to write attestation bundle")?;
    bundle_file.flush()?;

    Ok(bundle_file)
}

fn verify_with_cosign(cosign: &Path, archive: &Path, bundle: &Path) -> Result<()> {
    let identity_regexp = format!("^https://github.com/{}/{}/", QLTY_OWNER, QLTY_REPO);

    let output = Command::new(cosign)
        .args([
            "verify-blob-attestation",
            "--bundle",
            &bundle.to_string_lossy(),
            "--new-bundle-format",
            "--certificate-oidc-issuer",
            OIDC_ISSUER,
            "--certificate-identity-regexp",
            &identity_regexp,
            "--type",
            "slsaprovenance",
            &archive.to_string_lossy(),
        ])
        .output()
        .context("Failed to run cosign")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Provenance verification failed: {}", stderr.trim());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_set() {
        assert_eq!(QLTY_OWNER, "qltysh");
        assert_eq!(QLTY_REPO, "qlty");
        assert_eq!(OIDC_ISSUER, "https://token.actions.githubusercontent.com");
    }
}
