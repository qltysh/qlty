use anyhow::{Context, Result};
use std::{fs::create_dir_all, path::Path};
use tracing::error;

pub fn ensure_parent_exists(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
    } else {
        error!(
            "No parent directory for destination file: {:?}, this could cause issues",
            path
        );
    }

    Ok(())
}

#[cfg(unix)]
pub fn create_symlink(from: &Path, to: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(from, to)
}

#[cfg(windows)]
pub fn create_symlink(from: &Path, to: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(from, to)
}
