use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use std::path::PathBuf;

const JAVA_SRC_DIR_PATTERNS: &[&str] = &[
    "**/src/main/java",
    "**/src/main/kotlin",
    "**/src/*/java",
    "**/src/*/kotlin",
];

const DEFAULT_EXCLUSION_COMPONENTS: &[&str] = &["test", "tests", "build", "node_modules"];

#[derive(Debug)]
pub struct JavaSrcDirFinder {
    root: PathBuf,
    exclude_patterns: Vec<String>,
    has_qlty_config: bool,
}

impl JavaSrcDirFinder {
    pub fn new(root: PathBuf, exclude_patterns: Vec<String>, has_qlty_config: bool) -> Self {
        Self {
            root,
            exclude_patterns,
            has_qlty_config,
        }
    }

    pub fn find(&self) -> Result<Vec<PathBuf>> {
        let include_globset = self.build_include_globset()?;
        let exclude_globset = self.build_exclude_globset()?;

        let mut found_dirs = Vec::new();

        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .build();

        for entry in walker {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let relative_path = path
                .strip_prefix(&self.root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if include_globset.is_match(&relative_path) {
                if self.should_exclude(&relative_path, &exclude_globset) {
                    tracing::debug!("Excluding Java src dir: {}", path.display());
                    continue;
                }

                tracing::debug!("Found Java src dir: {}", path.display());
                found_dirs.push(path.to_path_buf());
            }
        }

        found_dirs.sort();
        Ok(found_dirs)
    }

    fn build_include_globset(&self) -> Result<GlobSet> {
        let mut builder = GlobSetBuilder::new();

        for pattern in JAVA_SRC_DIR_PATTERNS {
            builder.add(Glob::new(pattern)?);
        }

        Ok(builder.build()?)
    }

    fn build_exclude_globset(&self) -> Result<GlobSet> {
        let mut builder = GlobSetBuilder::new();

        if self.has_qlty_config {
            for pattern in &self.exclude_patterns {
                builder.add(Glob::new(pattern)?);
            }
        }

        Ok(builder.build()?)
    }

    fn should_exclude(&self, relative_path: &str, exclude_globset: &GlobSet) -> bool {
        if self.has_qlty_config {
            exclude_globset.is_match(relative_path)
        } else {
            self.matches_default_exclusions(relative_path)
        }
    }

    fn matches_default_exclusions(&self, relative_path: &str) -> bool {
        let components: Vec<&str> = relative_path.split('/').collect();

        for component in &components {
            let lower = component.to_lowercase();

            if DEFAULT_EXCLUSION_COMPONENTS
                .iter()
                .any(|exc| lower == *exc || lower.starts_with(exc) || lower.ends_with(exc))
            {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn create_dir(base: &Path, relative: &str) {
        let path = base.join(relative);
        fs::create_dir_all(&path).unwrap();
    }

    #[test]
    fn finds_maven_structure() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_dir(root, "src/main/java/com/example");
        create_dir(root, "src/main/kotlin/com/example");

        let finder = JavaSrcDirFinder::new(root.to_path_buf(), vec![], false);
        let dirs = finder.find().unwrap();

        assert_eq!(dirs.len(), 2);
        assert!(dirs
            .iter()
            .any(|p| p.ends_with("src/main/java") || p.ends_with("src\\main\\java")));
        assert!(dirs
            .iter()
            .any(|p| p.ends_with("src/main/kotlin") || p.ends_with("src\\main\\kotlin")));
    }

    #[test]
    fn finds_gradle_structure() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_dir(root, "app/src/main/java/com/example");
        create_dir(root, "lib/src/debug/java/com/example");
        create_dir(root, "lib/src/release/kotlin/com/example");

        let finder = JavaSrcDirFinder::new(root.to_path_buf(), vec![], false);
        let dirs = finder.find().unwrap();

        assert_eq!(dirs.len(), 3);
    }

    #[test]
    fn excludes_node_modules_without_config() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_dir(root, "src/main/java/com/example");
        create_dir(root, "node_modules/some-package/src/main/java");

        let finder = JavaSrcDirFinder::new(root.to_path_buf(), vec![], false);
        let dirs = finder.find().unwrap();

        assert_eq!(dirs.len(), 1);
        assert!(dirs[0].ends_with("src/main/java") || dirs[0].ends_with("src\\main\\java"));
    }

    #[test]
    fn excludes_test_directories_without_config() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_dir(root, "src/main/java/com/example");
        create_dir(root, "src/test/java/com/example");
        create_dir(root, "test/src/main/java/com/example");
        create_dir(root, "build/src/main/java/com/example");

        let finder = JavaSrcDirFinder::new(root.to_path_buf(), vec![], false);
        let dirs = finder.find().unwrap();

        assert_eq!(dirs.len(), 1);
        assert!(dirs[0].ends_with("src/main/java") || dirs[0].ends_with("src\\main\\java"));
    }

    #[test]
    fn uses_qlty_config_exclusions() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_dir(root, "app/src/main/java/com/example");
        create_dir(root, "legacy/src/main/java/com/example");

        let finder = JavaSrcDirFinder::new(
            root.to_path_buf(),
            vec!["legacy/**".to_string()],
            true, // has_qlty_config = true
        );
        let dirs = finder.find().unwrap();

        assert_eq!(dirs.len(), 1);
        assert!(
            dirs[0].to_string_lossy().contains("app/src/main/java")
                || dirs[0].to_string_lossy().contains("app\\src\\main\\java")
        );
    }

    #[test]
    fn returns_empty_when_no_sources() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_dir(root, "src/lib");
        create_dir(root, "app/code");

        let finder = JavaSrcDirFinder::new(root.to_path_buf(), vec![], false);
        let dirs = finder.find().unwrap();

        assert!(dirs.is_empty());
    }

    #[test]
    fn handles_nested_projects() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_dir(root, "project-a/src/main/java/com/a");
        create_dir(root, "project-b/src/main/kotlin/com/b");
        create_dir(root, "project-c/module/src/main/java/com/c");

        let finder = JavaSrcDirFinder::new(root.to_path_buf(), vec![], false);
        let dirs = finder.find().unwrap();

        assert_eq!(dirs.len(), 3);
    }
}
