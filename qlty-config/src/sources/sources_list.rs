use super::{Source, SourceFetch};
use crate::TomlMerge;
use anyhow::Result;

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
