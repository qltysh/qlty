use crate::initializer::{Settings, SourceSpec};
use crate::{Arguments, CommandError, CommandSuccess};
use crate::{Initializer, QltyRelease};
use anyhow::Result;
use clap::Args;
use console::style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use duct::cmd;
use itertools::Itertools;
use num_format::{Locale, ToFormattedString};
use qlty_config::Workspace;
use std::io::Write;
use tabwriter::TabWriter;

#[derive(Args, Debug)]
pub struct Init {
    /// Answer yes to all prompts
    #[arg(short, long)]
    pub yes: bool,

    /// Answer no to all prompts
    #[arg(short, long, conflicts_with = "yes")]
    pub no: bool,

    /// Skip enabling plugins
    #[arg(long)]
    pub skip_plugins: bool,

    /// Print the generated configuration to stdout instead of saving to disk
    #[arg(long)]
    pub dry_run: bool,

    /// Initialize without default source.
    #[arg(long)]
    pub skip_default_source: bool,

    /// A custom source to use for plugins.
    /// This can be a URL(name=url) or
    /// a path to a local directory(name=directory).
    #[arg(long, value_parser = SourceSpec::new)]
    pub source: Option<SourceSpec>,

    /// Warning: this option has been deprecated!
    /// Enable plugin prefix detection.
    #[arg(hide = true, long)]
    pub with_prefixes: bool,
}

impl Init {
    pub fn execute(&self, args: &Arguments) -> Result<CommandSuccess, CommandError> {
        if self.with_prefixes {
            eprintln!(
                "{} The --with-prefixes option has been deprecated and is no longer needed.",
                style("⚠").yellow()
            );
        }

        if !args.no_upgrade_check {
            QltyRelease::upgrade_check().ok();
        }

        Workspace::assert_git_directory_root()?;

        let workspace = Workspace::new()?;
        if workspace.config_exists()? {
            eprintln!(
                "{} qlty is already initialized in the current directory.",
                style("✖").red()
            );
            workspace.fetch_sources()?;
        } else {
            if !self.dry_run {
                eprintln!(
                    "{} Initializing qlty in the current directory...",
                    style("›").bold()
                );
                let library = workspace.library()?;
                library.create()?;
                self.print_check("Created .qlty/ directory");
            }

            let mut initializer = Initializer::new(Settings {
                workspace,
                skip_plugins: self.skip_plugins,
                skip_default_source: self.skip_default_source,
                source: self.source.clone(),
            })?;

            initializer.prepare()?;
            initializer.compute()?;

            if self.dry_run {
                println!("{}", initializer.qlty_toml()?);
                return CommandSuccess::ok();
            } else {
                initializer.write()?;
            }

            self.print_check(&format!(
                "Created qlty.toml config file with {} plugins",
                initializer.plugins.len()
            ));

            if !self.skip_plugins {
                self.print_enabled_plugins(&initializer)?;
            }

            if !self.skip_plugins {
                self.plugins_post_init(&initializer)?;
            }
        }

        self.print_next_steps();
        CommandSuccess::ok()
    }

    fn print_next_steps(&self) {
        println!();
        println!("{}", style("What's next?").bold());
        println!();
        println!(
            "{}",
            style("  1. Read the documentation: https://qlty.sh/docs").bold()
        );
        println!();
        println!("  {}", style("2. Get help and give feedback").bold());
        println!(
            "     {}",
            style("Our developers are on Discord: https://qlty.sh/discord").dim()
        );
        println!();
    }

    fn print_check(&self, message: &str) {
        eprintln!("{} {}", style("✔").green(), message,);
    }

    fn print_enabled_plugins(&self, initializer: &Initializer) -> Result<()> {
        if initializer.plugins.is_empty() {
            return Ok(());
        }

        println!();
        let mut tw = TabWriter::new(vec![]);

        tw.write_all(
            format!(
                "{}\t{}\t{}\t{}\n",
                style("Plugin").bold().underlined(),
                style("Version").bold().underlined(),
                style("Targets").bold().underlined(),
                style("Config").bold().underlined(),
            )
            .as_bytes(),
        )
        .unwrap();

        for installed_plugin in &initializer.plugins {
            let config_files = installed_plugin
                .config_files
                .iter()
                .map(|f| f.path.file_name().unwrap().to_string_lossy())
                .join(", ");

            let formatted = installed_plugin
                .files_count
                .to_formatted_string(&Locale::en);

            tw.write_all(
                format!(
                    "{}\t{}\t{}\t{}\n",
                    style(&installed_plugin.name),
                    style(&installed_plugin.version).dim(),
                    style(format!("{} files", formatted)).dim(),
                    style(config_files).dim(),
                )
                .as_bytes(),
            )
            .unwrap();
        }

        tw.flush().unwrap();
        let written = String::from_utf8(tw.into_inner().unwrap()).unwrap();
        println!("{}", written);

        Ok(())
    }

    fn plugins_post_init(&self, initializer: &Initializer) -> Result<()> {
        if initializer.plugins.is_empty() {
            return Ok(());
        }

        let mut sampled = false;

        if !self.no && (self.yes || self.confirm_sample()?) {
            self.sample()?;
            sampled = true;
        }

        if !sampled && !self.no && (self.yes || self.confirm_install()?) {
            self.install_plugins(initializer)?;
        }

        Ok(())
    }

    fn confirm_sample(&self) -> Result<bool> {
        Ok(Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Would you like to run plugins to see a sample of issues?")
            .default(true)
            .show_default(true)
            .interact()?)
    }

    fn confirm_install(&self) -> Result<bool> {
        Ok(Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Would you like to install plugins now? (Recommended)")
            .default(true)
            .show_default(true)
            .interact()?)
    }

    fn sample(&self) -> Result<()> {
        println!();
        println!(
            "{} Running {} check --sample 5",
            style("›").bold(),
            self.binary_name()?
        );
        println!();

        cmd!(self.current_exe()?, "check", "--sample", "5").run()?;
        Ok(())
    }

    fn install_plugins(&self, initializer: &Initializer) -> Result<()> {
        println!();
        println!(
            "{} Installing {} plugins",
            style("›").bold(),
            initializer.plugins.len()
        );
        println!();

        cmd!(self.current_exe()?, "install").run()?;
        Ok(())
    }

    fn binary_name(&self) -> Result<String> {
        let current_exe = self.current_exe()?;
        let binary_name = current_exe
            .file_name()
            .expect("Unable to identify current executable")
            .to_str()
            .unwrap();
        Ok(binary_name.to_owned())
    }

    fn current_exe(&self) -> Result<std::path::PathBuf> {
        let current_exe = std::env::current_exe()?;
        Ok(current_exe)
    }
}
