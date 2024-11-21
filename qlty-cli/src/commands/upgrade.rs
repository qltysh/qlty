use crate::QltyRelease;
use crate::{upgrade::ReleaseSpec, Arguments, CommandError, CommandSuccess};
use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand};
use console::style;
use qlty_analysis::utils::fs::path_to_string;
use qlty_analysis::version::QLTY_VERSION;
use qlty_config::sources::SourceUpgrade;
use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};
use tracing::{info, trace, warn};

#[derive(Args, Debug)]
pub struct Upgrade {
    /// The version to upgrade to. Defaults to the latest version.
    #[arg(long)]
    version: Option<String>,

    /// Run the upgrade even if the latest version is already installed.
    #[arg(long)]
    force: bool,

    /// Whether to perform a dry run.
    #[arg(long)]
    dry_run: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Upgrades the source.
    Source(Source),
}

#[derive(Args, Debug, Default)]
pub struct Source {}

impl Upgrade {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let timer = Instant::now();

        if let Some(Commands::Source(_)) = &self.command {
            SourceUpgrade::new().run()?;
            return CommandSuccess::ok();
        }

        let release_spec = match &self.version {
            Some(version) => {
                if version.starts_with('v') {
                    ReleaseSpec::Tag(version[1..].to_owned())
                } else {
                    ReleaseSpec::Tag(version.to_owned())
                }
            }
            None => ReleaseSpec::Latest,
        };

        let release = QltyRelease::load(release_spec)?;

        if !self.force {
            self.print_version_status(&release);
        }

        let zip_bytes = release.download()?;
        let tempdir = tempfile::Builder::new()
            .prefix("qlty")
            .tempdir()
            .context("Unable to create temporary directory")?;
        self.save_tar_xz_to_tempfile(tempdir.path(), &release.filename, &zip_bytes)?;
        let executable_path =
            self.unzip(tempdir.path(), &PathBuf::from(release.filename.clone()))?;
        self.verify_exe(&executable_path, &release.version)?;

        if self.dry_run {
            println!(
                "{}",
                style("Dry run complete. Would have installed to:").yellow()
            );
            println!();
            println!("    {}", executable_path.display());
            println!();
        } else {
            self.install(&executable_path)?;
        }

        SourceUpgrade::new().run()?;

        self.install_completions().ok();
        self.print_result(&timer, &release);
        CommandSuccess::ok()
    }

    fn install_completions(&self) -> Result<()> {
        let mut command = std::process::Command::new(self.target_executable()?);
        command.arg("completions").arg("--install");
        // Swallow outputs and ignore failures.
        command.output().ok();
        Ok(())
    }

    fn print_version_status(&self, release: &QltyRelease) {
        if release.version == QLTY_VERSION {
            println!(
                "{} You're already on the latest version of qlty (which is v{})",
                style("Congrats!").green().bold(),
                release.version
            );

            std::process::exit(0);
        }

        println!(
            "{} {} is out! You're on v{}.",
            style("qlty").bold(),
            style(format!("v{}", release.version)).bold().cyan(),
            QLTY_VERSION
        );
    }

    fn save_tar_xz_to_tempfile(
        &self,
        tempdir: &Path,
        filename: &str,
        zip_bytes: &[u8],
    ) -> Result<()> {
        let filename = PathBuf::from(filename);
        let tempfile = tempdir.join(filename);
        std::fs::write(tempfile, zip_bytes).context("Unable to write to temporary file")?;
        Ok(())
    }

    fn unzip(&self, tempdir: &Path, filename: &Path) -> Result<PathBuf> {
        let output = std::process::Command::new("tar")
            .arg("--extract")
            .arg("--preserve-permissions")
            .arg("--uncompress")
            .arg("--file")
            .arg(filename)
            .current_dir(tempdir)
            .output()
            .context("Unable to extract")?;

        if !output.status.success() {
            bail!(
                "tar failed with exit code {}",
                output
                    .status
                    .code()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "unknown".to_owned())
            );
        }

        let filename_string = path_to_string(filename);
        let base_name = filename_string.strip_suffix(".tar.xz").expect(".tar.xz");
        Ok(tempdir.join(base_name).join("qlty"))
    }

    fn verify_exe(&self, executable: &Path, expected_version: &str) -> Result<()> {
        let output = std::process::Command::new(executable)
            .arg("--version")
            .output()
            .context("Unable to run qlty --version")?;

        if !output.status.success() {
            bail!(
                "qlty --version failed with exit code {}",
                output
                    .status
                    .code()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "unknown".to_owned())
            );
        }

        let output = String::from_utf8_lossy(&output.stdout);
        let output = output.trim();

        if !output.contains(expected_version) {
            bail!(
                "Expected output of qlty --version to include {}, but it was {:?}",
                expected_version,
                output
            );
        }

        Ok(())
    }

    fn install(&self, temporary_executable: &Path) -> Result<()> {
        let target_executable = self.target_executable()?;

        if let Err(error) = std::fs::rename(temporary_executable, &target_executable) {
            warn!(
                "Error occurred renaming {} to {}: {}",
                &temporary_executable.display(),
                &target_executable.display(),
                error
            );

            if error.kind() == std::io::ErrorKind::PermissionDenied {
                trace!("Trying with elevated permissions...");

                let status = Command::new("sudo")
                    .arg("mv")
                    .arg(temporary_executable)
                    .arg(&target_executable)
                    .status()
                    .expect("Failed to run sudo command.");

                if status.success() {
                    info!(
                        "Successfully renamed {} to {} with sudo",
                        temporary_executable.display(),
                        target_executable.display()
                    );
                    Ok(())
                } else {
                    bail!(
                        "sudo mv {} {} failed with: {}",
                        temporary_executable.display(),
                        target_executable.display(),
                        status
                    );
                }
            } else {
                Err(error).with_context(|| {
                    format!(
                        "Unable to rename {} to {}",
                        temporary_executable.display(),
                        target_executable.display()
                    )
                })
            }
        } else {
            info!(
                "Successfully renamed {} to {}",
                temporary_executable.display(),
                target_executable.display()
            );
            Ok(())
        }
    }

    fn target_executable(&self) -> Result<PathBuf> {
        std::env::current_exe().context("Unable to get current executable path")
    }

    fn print_result(&self, start_time: &Instant, release: &QltyRelease) {
        println!("Upgraded in {}s.", start_time.elapsed().as_secs());
        println!();
        println!(
            "{}",
            style(format!("Welcome to qlty v{}!", release))
                .green()
                .bold()
        );
        println!();
        println!("Join the Qlty community:");
        println!();
        println!(
            "    {}",
            style("https://qlty.sh/discord".to_string()).cyan().bold()
        );
        println!();
        println!(
            "{}",
            style("Please update the versions of your sources in qlty.toml to the latest.").bold()
        );
        println!();
    }

    pub fn download_latest(&self, release_filename: &String) -> Result<Vec<u8>> {
        let download_url = format!(
            "https://qlty-releases.s3.amazonaws.com/qlty/latest/{}",
            release_filename
        );
        let response = ureq::get(&download_url)
            .set("User-Agent", &format!("{}/{}", "qlty", QLTY_VERSION))
            .call()
            .with_context(|| format!("Unable to get URL: {}", &download_url))?;

        if response.status() != 200 {
            bail!(
                "GET {} returned {} status",
                &download_url,
                response.status()
            );
        }

        let bytes = QltyRelease::download_with_progress(response)?;

        Ok(bytes)
    }
}
