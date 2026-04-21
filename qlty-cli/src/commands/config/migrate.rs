use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::Args;
use qlty_config::{MigrateConfig, MigrationSettings, Workspace};

const CLASSIC_CONFIG_NAME: &str = ".codeclimate.yml";

#[derive(Args, Debug, Clone)]
pub struct Migrate {
    #[arg(long)]
    /// Prints the migrated version of qlty.toml to the console without saving it to disk.
    pub dry_run: bool,
}

impl Migrate {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let workspace = Workspace::require_initialized()?;

        if !workspace.config_exists()? {
            return Err(CommandError::Unknown {
                source: anyhow::anyhow!("Could not find a .qlty/qlty.toml file."),
            });
        }

        let classic_config_path = workspace.root.join(CLASSIC_CONFIG_NAME);

        if !classic_config_path.exists() {
            return Err(CommandError::Unknown {
                source: anyhow::anyhow!("Could not find a .codeclimate.yml file."),
            });
        }

        let migration_settings = MigrationSettings::new(
            &workspace.root,
            workspace.config()?,
            &workspace.config_path()?,
            &classic_config_path,
            self.dry_run,
        )?;

        MigrateConfig::new(migration_settings)?.migrate()?;

        CommandSuccess::ok()
    }
}
