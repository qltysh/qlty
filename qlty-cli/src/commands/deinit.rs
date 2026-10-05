use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::{anyhow, Result};
use clap::Args;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm};
use log::warn;
use qlty_config::Workspace;

#[derive(Args, Debug)]
pub struct Deinit {
    /// Proceed without confirmation
    #[arg(short, long)]
    pub yes: bool,
}

impl Deinit {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let root = Workspace::assert_git_directory_root()?;
        let qlty_dir = root.join(".qlty");

        if qlty_dir.exists() {
            if self.yes
                || Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(
                        "This will remove qlty from your repository. Do you want to continue?",
                    )
                    .default(true)
                    .show_default(true)
                    .interact()
                    .unwrap()
            {
                std::fs::remove_dir_all(&qlty_dir).map_err(|err| {
                    anyhow!(
                        "Unable to remove .qlty directory: {} ({:?})",
                        qlty_dir.display(),
                        err
                    )
                })?;

                warn!("Removed .qlty directory: {:?}", &qlty_dir);
                println!("{} Removed .qlty/ directory", style("✔").green());
                println!("{} Removed qlty from this repository", style("✔").green());
                CommandSuccess::ok()
            } else {
                println!("{} Cancelling deinit", style("✖").red());
                Err(anyhow!("User cancelled deinit.").into())
            }
        } else {
            eprintln!(
                "{} qlty is not initialized in this repository",
                style("✖").red()
            );
            Err(
                anyhow!("Cancelled deinit because qlty is not initialized in this repository.")
                    .into(),
            )
        }
    }
}
