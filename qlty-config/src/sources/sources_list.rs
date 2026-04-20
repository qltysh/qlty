use super::{Source, SourceFetch};
use crate::TomlMerge;
use anyhow::{bail, Result};

#[derive(Default, Clone)]
pub struct SourcesList {
    pub sources: Vec<Box<dyn Source>>,
}

impl std::fmt::Debug for SourcesList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourcesList")
            .field("sources", &self.sources)
            .finish()
    }
}

impl SourcesList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toml(&self) -> Result<toml::Value> {
        let mut toml: toml::Value = toml::Value::Table(toml::value::Table::new());

        for source in &self.sources {
            toml = TomlMerge::merge(toml, source.toml()?).unwrap();
        }

        Ok(toml)
    }

    pub fn validate_sources_cached(&self) -> Result<()> {
        for source in &self.sources {
            if !source.is_cached() {
                bail!(
                    "Source is not available locally. Run without --skip-source-fetch, or run `qlty sources fetch` first."
                );
            }
        }

        Ok(())
    }
}

impl SourceFetch for SourcesList {
    fn fetch(&self) -> Result<()> {
        for source in &self.sources {
            source.fetch()?;
        }

        Ok(())
    }

    fn sources(&self) -> Vec<Box<dyn Source>> {
        self.sources.clone()
    }

    fn clone_box(&self) -> Box<dyn SourceFetch> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::sources::{DefaultSource, GitSource, GitSourceReference, LocalSource};
    use crate::Library;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_validate_sources_cached_with_empty_list() {
        let sources_list = SourcesList::new();
        assert!(sources_list.validate_sources_cached().is_ok());
    }

    #[test]
    fn test_validate_sources_cached_with_default_source() {
        let mut sources_list = SourcesList::new();
        sources_list.sources.push(Box::new(DefaultSource {}));
        assert!(sources_list.validate_sources_cached().is_ok());
    }

    #[test]
    fn test_validate_sources_cached_with_local_source() {
        let mut sources_list = SourcesList::new();
        sources_list.sources.push(Box::new(LocalSource {
            root: Path::new("/nonexistent/path").to_path_buf(),
        }));
        assert!(sources_list.validate_sources_cached().is_ok());
    }

    #[test]
    fn test_validate_sources_cached_errors_when_git_source_not_cached() {
        let temp_dir = tempdir().unwrap();
        let library = Library::new(temp_dir.path()).unwrap();
        let git_source = GitSource {
            library,
            origin: "https://github.com/qltysh/plugins".to_string(),
            reference: GitSourceReference::Tag("v1.0.0".to_string()),
        };

        let mut sources_list = SourcesList::new();
        sources_list.sources.push(Box::new(git_source));

        assert!(sources_list.validate_sources_cached().is_err());
    }
}
