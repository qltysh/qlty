use std::path::Path;

use super::{source::SourceFetch, Source, SourceFile};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct DefaultSource {}

impl Source for DefaultSource {
    // fn local_root(&self) -> PathBuf {
    //     self.origin.clone()
    // }

    fn source_files(&self) -> Result<Vec<SourceFile>> {
        Ok(vec![]) // TODO
    }

    fn get_config_file(&self, _file_name: &Path) -> Result<Option<SourceFile>> {
        Ok(None)
    }

    fn clone_box(&self) -> Box<dyn Source> {
        Box::new(self.clone())
    }
}

impl SourceFetch for DefaultSource {
    fn fetch(&self) -> Result<()> {
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn SourceFetch> {
        Box::new(self.clone())
    }
}
