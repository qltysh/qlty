use crate::QltyRelease;
use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::{Args, Subcommand};
use console::style;
use qlty_config::sources::SourceUpgrade;
use qlty_config::version::QLTY_VERSION;
use std::time::Instant;

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

        let release = QltyRelease::load(&self.version)?;

        if !self.force {
            self.print_version_status(&release);
        }

        if self.dry_run {
            println!(
                "{}",
                style("Dry run complete. Would have installed to:").yellow()
            );
            println!();
            println!("    {}", std::env::current_exe()?.display());
            println!();
            return CommandSuccess::ok();
        }

        release.run_upgrade_command()?;

        SourceUpgrade::new().run().ok();

        self.install_completions().ok();
        self.print_result(&timer, &release);
        CommandSuccess::ok()
    }

    fn install_completions(&self) -> Result<()> {
        let mut command = std::process::Command::new(std::env::current_exe()?);
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

    fn print_result(&self, start_time: &Instant, release: &QltyRelease) {
        println!("Upgraded in {}s.", start_time.elapsed().as_secs());
        println!();
        println!(
            "{}",
            style(format!("Welcome to qlty v{}!", release.version))
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
}
