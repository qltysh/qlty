use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap_complete::Shell;

mod bash;
mod fish;
mod zsh;

pub use bash::Bash;
pub use fish::Fish;
use tracing::warn;
pub use zsh::Zsh;

#[derive(Clone, Debug)]
pub struct TabCompletions {
    pub shell: Shell,
    pub bin_name: String,
    pub completions: String,
}

pub trait TabCompletingShell {
    fn generate(&self, bin_name: &str, command: &mut clap::Command) -> Result<TabCompletions> {
        let mut buf = Vec::new();
        clap_complete::generate(self.clap_shell(), command, bin_name, &mut buf);

        Ok(TabCompletions {
            shell: self.clap_shell(),
            bin_name: bin_name.to_owned(),
            completions: String::from_utf8(buf)?,
        })
    }

    fn install(&mut self, completions: &TabCompletions) -> Result<PathBuf> {
        let directory = self
            .completions_directory()
            .context("Unable to determine completions directory")?;
        let filename = self.completions_filename(completions);
        let path = directory.join(filename);

        let mut file = File::create(&path)?;
        file.write_all(completions.completions.as_bytes())?;

        Ok(path)
    }

    fn update_rc(&self, _completions_path: &Path) -> Result<()> {
        Ok(())
    }

    fn completions_filename(&self, completions: &TabCompletions) -> PathBuf;
    fn completions_directory(&self) -> Option<PathBuf>;
    fn clap_shell(&self) -> Shell;
}

pub fn build(specified: Option<Shell>) -> Option<Box<dyn TabCompletingShell>> {
    let shell = specified.unwrap_or_else(|| Shell::from_env().unwrap_or(Shell::Bash));

    match shell {
        Shell::Bash => Some(Box::new(Bash)),
        Shell::Fish => Some(Box::new(Fish)),
        Shell::Zsh => Some(Box::new(Zsh)),
        _ => {
            warn!("Unsupported shell: {:?}", shell);
            None
        }
    }
}
