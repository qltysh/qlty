use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::{Context as _, Result};
use clap::Args;
use qlty_config::Workspace;
use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt as _;

#[derive(Args, Debug)]
pub struct Install {}

const QLTY_HOOKS_DIR: &str = ".qlty/hooks";

impl Install {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        Workspace::require_initialized()?;

        fs::create_dir_all(QLTY_HOOKS_DIR)?;

        install_hook("pre-commit", include_str!("./pre_commit.sh"))?;
        install_hook("pre-push", include_str!("./pre_push.sh"))?;

        CommandSuccess::ok()
    }
}

fn install_hook(hook_name: &str, contents: &str) -> Result<()> {
    let script_filename = format!("{}.sh", hook_name);
    let hook_script_path = Path::new(QLTY_HOOKS_DIR).join(script_filename.clone());
    fs::write(&hook_script_path, contents).with_context(|| {
        format!(
            "Failed to write {} hook to {}",
            hook_name,
            hook_script_path.display()
        )
    })?;

    let git_hooks_dir = Path::new(".git").join("hooks");

    if !git_hooks_dir.exists() {
        fs::create_dir_all(&git_hooks_dir).with_context(|| {
            format!(
                "Failed to create git hooks directory at {}",
                git_hooks_dir.display()
            )
        })?;
    }
    let symlink_path = git_hooks_dir.join(hook_name);

    if symlink_path.exists() {
        fs::remove_file(&symlink_path).with_context(|| {
            format!(
                "Failed to remove existing {} symlink at {}",
                hook_name,
                symlink_path.display()
            )
        })?;
    }

    let hook_relative_path = Path::new("..")
        .join("..")
        .join(".qlty")
        .join("hooks")
        .join(script_filename);

    let symlink_result;

    #[cfg(windows)]
    {
        symlink_result = std::os::windows::fs::symlink_file(&hook_relative_path, &symlink_path);
    }
    #[cfg(unix)]
    {
        symlink_result = std::os::unix::fs::symlink(&hook_relative_path, &symlink_path);
    }

    symlink_result.with_context(|| {
        format!(
            "Failed to create symlink from {} to {}",
            hook_relative_path.display(),
            symlink_path.display()
        )
    })?;

    #[cfg(unix)]
    {
        let metadata = fs::metadata(&symlink_path).with_context(|| {
            format!(
                "Failed to get metadata for {} symlink at {}",
                hook_name,
                symlink_path.display()
            )
        })?;

        let mut perms = metadata.permissions();
        perms.set_mode(0o755);

        fs::set_permissions(&symlink_path, perms).with_context(|| {
            format!(
                "Failed to set permissions on {} symlink at {}",
                hook_name,
                symlink_path.display()
            )
        })?;
    }

    println!(
        "Installed git hook '{}' at {}",
        hook_name,
        hook_script_path.display()
    );

    Ok(())
}
