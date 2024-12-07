use super::{source::SourceFetch, Source, SourceFile};
use crate::Library;
use anyhow::Result;
use std::path::{Path, PathBuf};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct LocalSource {
    pub library: Library,
    pub origin: PathBuf,
}

impl Source for LocalSource {
    // fn local_root(&self) -> PathBuf {
    //     self.origin.clone()
    // }

    fn source_files(&self) -> Result<Vec<SourceFile>> {
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
        debug!("Skipping source fetch: {:?}", self.origin);
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn SourceFetch> {
        Box::new(self.clone())
    }
}
