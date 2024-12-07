use super::{source::SourceFetch, Source, SourceFile};
use anyhow::Result;
use std::path::{Path, PathBuf};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct LocalSource {
    pub root: PathBuf,
}

impl Source for LocalSource {
    fn files(&self) -> Result<Vec<SourceFile>> {
        Ok(vec![]) // TODO
    }

    fn get_file(&self, file_name: &Path) -> Result<Option<SourceFile>> {
        Ok(None)
    }

    fn clone_box(&self) -> Box<dyn Source> {
        Box::new(self.clone())
    }
}

impl SourceFetch for LocalSource {
    fn fetch(&self) -> Result<()> {
        debug!("Skipping source fetch: {:?}", self.root);
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn SourceFetch> {
        Box::new(self.clone())
    }
}
