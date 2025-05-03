use crate::commands::*;
use crate::commands::{auth, cache, config, plugins};
use crate::{CommandError, CommandSuccess};
use anyhow::Result;
use clap::{Parser, Subcommand};
use console::style;
use qlty_config::version::LONG_VERSION;

#[derive(Parser, Debug)]
#[command(author, version, about = "This is qlty, the Qlty command line interface.", long_version = LONG_VERSION.as_str(), long_about = None)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Log with more information
    #[clap(long, global = true)]
    pub debug: bool,

    /// Do not check for updates
    #[clap(long, global = true)]
    pub no_upgrade_check: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage authentication
    #[command(hide = true)]
    Auth(auth::Arguments),

    /// Run an analysis build
    #[command(hide = true)]
    Build(Build),

    /// Manage cache
    Cache(cache::Arguments),

    /// Run linters
    Check(Check),

    /// Generate or install Qlty CLI shell completions
    Completions(Completions),

    /// Print current version
    Config(config::Arguments),

    /// View, transform, and publish code coverage
    Coverage(coverage::Arguments),

    /// Open the Qlty Cloud dashboard in the browser
    Dashboard(Dashboard),

    /// Remove Qlty from the current repository
    Deinit(Deinit),

    /// Join our Discord server (opens in the browser)
    Discord(Discord),

    /// Open the docs website in the browser
    Docs(Docs),

    /// Auto-format files by rewriting them
    Fmt(Fmt),

    /// Manage Git hooks
    #[command(hide = true)]
    Githooks(githooks::Arguments),

    /// Set up Qlty in the current repository
    Init(Init),

    /// Install linters and their dependencies
    Install(Install),

    /// Compute code quality metrics
    Metrics(Metrics),

    /// Intentionally panic to test crash handling
    #[command(hide = true)]
    Panic(Panic),

    #[command(hide = true)]
    /// Parse source code
    Parse(Parse),

    #[command(hide = true)]
    /// Apply patches from check
    Patch(Patch),

    /// Manage plugins
    Plugins(plugins::Arguments),

    /// Find code smells like duplication and complexity
    Smells(Smells),

    /// Send telemetry
    #[command(hide = true)]
    Telemetry(Telemetry),

    /// Upgrade the Qlty CLI
    Upgrade(Upgrade),

    #[command(hide = true)]
    /// Validate the project
    Validate(Validate),

    /// Print the current Qlty CLI version
    Version(Version),
}

impl Arguments {
    pub fn execute(&self) -> Result<CommandSuccess, CommandError> {
        if self.command.is_none() {
            self.print_default_output();
            return CommandSuccess::ok();
        }

        match &self.command.as_ref().unwrap() {
            Commands::Auth(command) => command.execute(self),
            Commands::Build(command) => command.execute(self),
            Commands::Cache(command) => command.execute(self),
            Commands::Check(command) => command.execute(self),
            Commands::Completions(command) => command.execute(self),
            Commands::Config(command) => command.execute(self),
            Commands::Coverage(command) => command.execute(self),
            Commands::Dashboard(command) => command.execute(self),
            Commands::Deinit(command) => command.execute(self),
            Commands::Discord(command) => command.execute(self),
            Commands::Docs(command) => command.execute(self),
            Commands::Fmt(command) => command.execute(self),
            Commands::Githooks(command) => command.execute(self),
            Commands::Install(command) => command.execute(self),
            Commands::Init(command) => command.execute(self),
            Commands::Metrics(command) => command.execute(self),
            Commands::Panic(command) => command.execute(self),
            Commands::Parse(command) => command.execute(self),
            Commands::Patch(command) => command.execute(self),
            Commands::Plugins(command) => command.execute(self),
            Commands::Smells(command) => command.execute(self),
            Commands::Telemetry(command) => command.execute(self),
            Commands::Upgrade(command) => command.execute(self),
            Commands::Validate(command) => command.execute(self),
            Commands::Version(command) => command.execute(self),
        }
    }

    fn print_default_output(&self) {
        eprintln!("This is qlty, the Qlty CLI.");
        eprintln!();
        eprintln!(
            "{} {} [COMMAND]",
            style("Usage:").bold().underlined(),
            style("qlty").bold()
        );
        eprintln!();

        // This is nice from a UX standpoint. However, we store access tokens in the keychain
        // and when the CLI tries to access them it can cause an auth challenge. Therefore
        // I am disabling this for now until we can figure out a better way to handle this.
        //
        // if !self.has_access_token() {
        //     eprintln!(
        //         "It doesn't look like you're logged in. Try \"qlty auth signup\" to create an"
        //     );
        //     eprintln!("account, or \"qlty auth login\" to log in to an existing account.");
        //     eprintln!();
        // }

        eprintln!(
            "{}",
            style("Here's a few commands to get you started:").bold()
        );
        eprintln!();
        eprintln!(
            "  {}       {}",
            style("qlty init").cyan().bold(),
            style("Setup up your repository for Qlty").dim()
        );
        eprintln!(
            "  {}    {}",
            style("qlty metrics").cyan().bold(),
            style("Calculate code quality metrics").dim()
        );
        eprintln!(
            "  {}      {}",
            style("qlty check").cyan().bold(),
            style("Run linters and analyzers").dim()
        );
        eprintln!(
            "  {}       {}",
            style("qlty docs").cyan().bold(),
            style("Open Qlty docs in a browser").dim()
        );
        eprintln!(
            "  {}     {}",
            style("qlty smells").cyan().bold(),
            style("Find code smells like duplication").dim()
        );
        eprintln!(
            "  {}        {}",
            style("qlty fmt").cyan().bold(),
            style("Auto-format your code").dim()
        );
        eprintln!();
        eprintln!("{}", style("Qlty CLI works best with Qlty Cloud.").bold());
        eprintln!();
        eprintln!("  {}", style("https://qlty.sh/dashboard").cyan().bold());
        eprintln!("  Qlty Cloud integrates with GitHub to fully automate code quality.");
        eprintln!();
        eprintln!("{}", style("If you need help along the way:").bold());
        eprintln!();
        eprintln!("  Use `qlty [command] --help` for more information about a command.");
        eprintln!(
            "  Join our Discord at https://qlty.sh/discord to get help from the Qlty community."
        );
        eprintln!();
        eprintln!(
            "{}{}{}",
            style("For a full list of commands, run `").dim(),
            style("qlty help").cyan().bold(),
            style("`.").dim()
        );
    }
}

pub fn is_subcommand(subcommand: &str) -> bool {
    // Don't track telemetry commands, otherwise it will create an infinite loop
    let args = std::env::args().collect::<Vec<String>>();
    args.iter().any(|arg| arg == subcommand)
}
