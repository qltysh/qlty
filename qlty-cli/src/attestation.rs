use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

const QLTY_OWNER: &str = "qltysh";
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_secs(2);

pub fn verify_attestation(archive_path: &Path) -> Result<()> {
    if !is_gh_available() {
        print_gh_not_available_warning();
        return Ok(());
    }

    eprintln!("Verifying provenance...");

    for attempt in 1..=MAX_RETRIES {
        let output = Command::new("gh")
            .args([
                "attestation",
                "verify",
                &archive_path.to_string_lossy(),
                "--owner",
                QLTY_OWNER,
            ])
            .output()
            .context("Failed to run gh attestation verify")?;

        if output.status.success() {
            eprintln!(
                "  {} Verified SLSA provenance from github.com/{}",
                console::style("OK").green().bold(),
                QLTY_OWNER
            );
            return Ok(());
        }

        let exit_code = output.status.code();

        if exit_code == Some(4) {
            print_gh_not_authenticated_warning();
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        let combined_output = format!("{}{}", stderr, stdout);
        if is_transient_error(&combined_output) && attempt < MAX_RETRIES {
            eprintln!(
                "  Transient error during verification (attempt {}/{}), retrying...",
                attempt, MAX_RETRIES
            );
            thread::sleep(RETRY_DELAY * attempt);
            continue;
        }

        let mut message = "Provenance verification failed".to_string();
        if !stderr.is_empty() {
            message.push_str(&format!(": {}", stderr.trim()));
        } else if !stdout.is_empty() {
            message.push_str(&format!(": {}", stdout.trim()));
        }
        bail!(message);
    }

    unreachable!()
}

fn is_transient_error(output: &str) -> bool {
    output.contains("HTTP 5") || output.contains("connection reset")
}

fn is_gh_available() -> bool {
    Command::new("gh")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn print_gh_not_available_warning() {
    eprintln!();
    eprintln!(
        "  {}",
        console::style("WARNING: GitHub CLI (gh) is not installed")
            .yellow()
            .bold()
    );
    eprintln!();
    eprintln!("  Provenance verification requires the GitHub CLI.");
    eprintln!("  Install it from: https://cli.github.com/");
    eprintln!();
    eprintln!("  The upgrade will continue without provenance verification.");
    eprintln!();
}

fn print_gh_not_authenticated_warning() {
    eprintln!();
    eprintln!(
        "  {}",
        console::style("WARNING: GitHub CLI (gh) is not authenticated")
            .yellow()
            .bold()
    );
    eprintln!();
    eprintln!("  Provenance verification requires gh to be authenticated.");
    eprintln!("  Run: gh auth login");
    eprintln!();
    eprintln!("  The upgrade will continue without provenance verification.");
    eprintln!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_constant_is_set() {
        assert_eq!(QLTY_OWNER, "qltysh");
    }
}
