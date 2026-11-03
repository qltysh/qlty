use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{bail, Context, Result};
use qlty_config::warn_once;
use toml::{Table, Value};
use tracing::warn;

const CARGO_MANIFEST: &str = "Cargo.toml";

// Cargo builds a package with no edition set as Rust 2015
const DEFAULT_EDITION: &str = "2015";

/// Finds the edition `cargo fmt` passes to rustfmt for a file: the edition of
/// the package with the nearest Cargo.toml.
#[derive(Debug)]
pub struct CargoEditionFinder {
    workspace_root: PathBuf,
    editions_by_directory: Mutex<HashMap<PathBuf, Option<String>>>,
}

impl CargoEditionFinder {
    pub fn new(workspace_root: &Path) -> Self {
        Self {
            workspace_root: workspace_root.to_path_buf(),
            editions_by_directory: Mutex::new(HashMap::new()),
        }
    }

    pub fn find(&self, path: &Path) -> Option<String> {
        let directory = path.parent()?;

        if let Some(edition) = self.editions_by_directory.lock().unwrap().get(directory) {
            return edition.clone();
        }

        let edition = self.compute(directory);

        self.editions_by_directory
            .lock()
            .unwrap()
            .insert(directory.to_path_buf(), edition.clone());

        edition
    }

    fn compute(&self, directory: &Path) -> Option<String> {
        let manifest_directory = self.nearest_manifest_directory(directory)?;

        match self.package_edition(manifest_directory) {
            Ok(edition) => edition,
            Err(err) => {
                let message = format!(
                    "Unable to determine the Rust edition from {}: {:#}",
                    manifest_directory.join(CARGO_MANIFEST).display(),
                    err
                );
                warn!("{}", message);
                warn_once(&message);
                None
            }
        }
    }

    fn nearest_manifest_directory<'a>(&'a self, directory: &'a Path) -> Option<&'a Path> {
        self.ancestors_in_workspace(directory)
            .find(|ancestor| ancestor.join(CARGO_MANIFEST).is_file())
    }

    fn ancestors_in_workspace<'a>(&'a self, directory: &'a Path) -> impl Iterator<Item = &'a Path> {
        directory
            .ancestors()
            .take_while(move |ancestor| ancestor.starts_with(&self.workspace_root))
    }

    fn package_edition(&self, manifest_directory: &Path) -> Result<Option<String>> {
        let manifest = read_manifest(manifest_directory)?;

        let Some(package) = manifest.get("package") else {
            return Ok(None);
        };

        match package.get("edition") {
            None => Ok(Some(DEFAULT_EDITION.to_string())),
            Some(Value::String(edition)) => validate_edition(edition).map(Some),
            Some(Value::Table(edition))
                if edition.get("workspace") == Some(&Value::Boolean(true)) =>
            {
                self.workspace_edition(manifest_directory, package)
                    .map(Some)
            }
            Some(edition) => bail!("Unsupported package.edition value: {}", edition),
        }
    }

    fn workspace_edition(&self, package_directory: &Path, package: &Value) -> Result<String> {
        let workspace_directory = match package.get("workspace").and_then(Value::as_str) {
            Some(workspace_path) => package_directory.join(workspace_path),
            None => self
                .nearest_workspace_directory(package_directory)?
                .context("package.edition.workspace is set but no workspace root was found")?,
        };

        let workspace_manifest = read_manifest(&workspace_directory)?;

        let edition = workspace_manifest
            .get("workspace")
            .and_then(|workspace| workspace.get("package"))
            .and_then(|package| package.get("edition"))
            .and_then(Value::as_str)
            .with_context(|| {
                format!(
                    "{} does not set workspace.package.edition",
                    workspace_directory.join(CARGO_MANIFEST).display()
                )
            })?;

        validate_edition(edition)
    }

    fn nearest_workspace_directory(&self, directory: &Path) -> Result<Option<PathBuf>> {
        for ancestor in self.ancestors_in_workspace(directory) {
            if ancestor.join(CARGO_MANIFEST).is_file()
                && read_manifest(ancestor)?.contains_key("workspace")
            {
                return Ok(Some(ancestor.to_path_buf()));
            }
        }

        Ok(None)
    }
}

fn read_manifest(directory: &Path) -> Result<Table> {
    let manifest_path = directory.join(CARGO_MANIFEST);

    let contents = fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read {}", manifest_path.display()))?;

    contents
        .parse::<Table>()
        .with_context(|| format!("Failed to parse {}", manifest_path.display()))
}

fn validate_edition(edition: &str) -> Result<String> {
    if edition.is_empty() || !edition.chars().all(|c| c.is_ascii_digit()) {
        bail!("Invalid edition: {:?}", edition);
    }

    Ok(edition.to_string())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::{create_dir_all, write};
    use tempfile::{tempdir, TempDir};

    fn write_file(root: &Path, path: &str, contents: &str) {
        let path = root.join(path);
        create_dir_all(path.parent().unwrap()).unwrap();
        write(path, contents).unwrap();
    }

    fn find_edition(root: &TempDir, path: &str) -> Option<String> {
        CargoEditionFinder::new(root.path()).find(&root.path().join(path))
    }

    #[test]
    fn test_package_edition() {
        let root = tempdir().unwrap();
        write_file(root.path(), "Cargo.toml", "[package]\nedition = \"2021\"\n");

        assert_eq!(find_edition(&root, "src/lib.rs"), Some("2021".to_string()));
    }

    #[test]
    fn test_nearest_package_edition() {
        let root = tempdir().unwrap();
        write_file(root.path(), "Cargo.toml", "[package]\nedition = \"2018\"\n");
        write_file(
            root.path(),
            "crates/foo/Cargo.toml",
            "[package]\nedition = \"2024\"\n",
        );

        assert_eq!(
            find_edition(&root, "crates/foo/src/lib.rs"),
            Some("2024".to_string())
        );
    }

    #[test]
    fn test_package_without_edition() {
        let root = tempdir().unwrap();
        write_file(root.path(), "Cargo.toml", "[package]\nname = \"foo\"\n");

        assert_eq!(find_edition(&root, "src/lib.rs"), Some("2015".to_string()));
    }

    #[test]
    fn test_inherited_workspace_edition() {
        let root = tempdir().unwrap();
        write_file(
            root.path(),
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/*\"]\n\n[workspace.package]\nedition = \"2021\"\n",
        );
        write_file(
            root.path(),
            "crates/foo/Cargo.toml",
            "[package]\nedition.workspace = true\n",
        );

        assert_eq!(
            find_edition(&root, "crates/foo/src/lib.rs"),
            Some("2021".to_string())
        );
    }

    #[test]
    fn test_inherited_edition_from_workspace_path() {
        let root = tempdir().unwrap();
        write_file(
            root.path(),
            "Cargo.toml",
            "[workspace]\n\n[workspace.package]\nedition = \"2018\"\n",
        );
        write_file(
            root.path(),
            "other/Cargo.toml",
            "[workspace]\n\n[workspace.package]\nedition = \"2021\"\n",
        );
        write_file(
            root.path(),
            "other/foo/Cargo.toml",
            "[package]\nworkspace = \"../..\"\nedition.workspace = true\n",
        );

        assert_eq!(
            find_edition(&root, "other/foo/src/lib.rs"),
            Some("2018".to_string())
        );
    }

    #[test]
    fn test_inherited_edition_from_package_workspace_root() {
        let root = tempdir().unwrap();
        write_file(
            root.path(),
            "Cargo.toml",
            "[package]\nedition.workspace = true\n\n[workspace.package]\nedition = \"2021\"\n",
        );

        assert_eq!(find_edition(&root, "src/main.rs"), Some("2021".to_string()));
    }

    #[test]
    fn test_inherited_edition_without_workspace() {
        let root = tempdir().unwrap();
        write_file(
            root.path(),
            "Cargo.toml",
            "[package]\nedition.workspace = true\n",
        );

        assert_eq!(find_edition(&root, "src/lib.rs"), None);
    }

    #[test]
    fn test_virtual_manifest() {
        let root = tempdir().unwrap();
        write_file(
            root.path(),
            "Cargo.toml",
            "[workspace]\n\n[workspace.package]\nedition = \"2021\"\n",
        );

        assert_eq!(find_edition(&root, "scripts/build.rs"), None);
    }

    #[test]
    fn test_no_manifest() {
        let root = tempdir().unwrap();

        assert_eq!(find_edition(&root, "src/lib.rs"), None);
    }

    #[test]
    fn test_manifest_outside_workspace_root() {
        let root = tempdir().unwrap();
        write_file(root.path(), "Cargo.toml", "[package]\nedition = \"2021\"\n");
        let finder = CargoEditionFinder::new(&root.path().join("project"));

        assert_eq!(finder.find(&root.path().join("project/src/lib.rs")), None);
    }

    #[test]
    fn test_invalid_manifest() {
        let root = tempdir().unwrap();
        write_file(root.path(), "Cargo.toml", "[package\n");

        assert_eq!(find_edition(&root, "src/lib.rs"), None);
    }

    #[test]
    fn test_invalid_edition() {
        let root = tempdir().unwrap();
        write_file(
            root.path(),
            "Cargo.toml",
            "[package]\nedition = \"2021 --help\"\n",
        );

        assert_eq!(find_edition(&root, "src/lib.rs"), None);
    }
}
