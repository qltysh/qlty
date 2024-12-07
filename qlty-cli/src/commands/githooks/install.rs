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

const PRE_COMMIT_HOOK: &str = r#"#!/bin/sh
qlty fmt --trigger pre-commit --index-file="$GIT_INDEX_FILE"
"#;

const PRE_PUSH_HOOK: &str = r#"#!/bin/sh
qlty check --trigger pre-push --upstream-from-pre-push --no-formatters --skip-errored-plugins
"#;

impl Install {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        Workspace::require_initialized()?;

        fs::create_dir_all(QLTY_HOOKS_DIR)?;

        install_hook("pre-commit", PRE_COMMIT_HOOK)?;
        install_hook("pre-push", PRE_PUSH_HOOK)?;

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

    Ok(())
}
