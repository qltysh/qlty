use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::Args;
use console::style;
use qlty_config::Workspace;

#[derive(Args, Debug)]
pub struct List {}

impl List {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let workspace = Workspace::require_initialized()?;
        workspace.fetch_sources()?;
        let config = workspace.config()?;

        let mut available_plugin_names = config
            .plugins
            .definitions
            .iter()
            .filter(|(_, plugin)| !plugin.hidden)
            .map(|(name, _)| name)
            .collect::<Vec<_>>();

        available_plugin_names.sort();

        let enabled_plugin_names = config
            .plugin
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<_>>();

        println!(
            "{}",
            style(format!(
                "{} available plugins:",
                available_plugin_names.len()
            ))
            .bold()
        );
        println!();

        for name in &available_plugin_names {
            let enabled = enabled_plugin_names.contains(name);

            if enabled {
                println!("  {} {}", style("✔").green().bold(), style(name).bold());
            } else {
                println!("    {}", style(name).dim());
            }
        }

        println!();
        println!("Learn more: {}", style("https://qlty.sh/d/plugins").bold());
        println!();
        println!(
            "Run {} to enable plugins",
            style("qlty plugins enable [plugin]").cyan().bold()
        );
        CommandSuccess::ok()
    }
}
