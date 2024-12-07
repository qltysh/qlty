use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::{Context as _, Result};
use clap::Args;
use qlty_config::Workspace;
use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[derive(Args, Debug)]
pub struct Uninstall {}

impl Uninstall {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        Workspace::require_initialized()?;

        #[cfg(unix)]
        {
            let git_hooks_dir = Path::new(".git").join("hooks");
            let hooks = [
                git_hooks_dir.join("pre-commit"),
                git_hooks_dir.join("pre-push"),
            ];

            for hook in &hooks {
                if std::fs::exists(hook).unwrap_or_default() {
                    let metadata = fs::metadata(hook).with_context(|| {
                        format!("Failed to read metadata for hook at {}", hook.display())
                    })?;
                    let mut permissions = metadata.permissions();

                    // Remove execute permissions
                    permissions.set_mode(permissions.mode() & !0o111);

                    fs::set_permissions(hook, permissions).with_context(|| {
                        format!(
                            "Failed to remove execute permissions from hook at {}",
                            hook.display()
                        )
                    })?;
                }
            }
        }

        let qlty_hooks_dir = Path::new(".qlty").join("hooks");
        fs::remove_dir_all(&qlty_hooks_dir)
            .with_context(|| format!("Failed to remove {} directory", qlty_hooks_dir.display()))?;

        CommandSuccess::ok()
    }
}
