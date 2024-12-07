use super::{source::SourceFetch, Source, SourceFile};
use anyhow::{Context as _, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct LocalSource {
    pub root: PathBuf,
}

impl Source for LocalSource {
    fn paths(&self) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        let walkdir = WalkDir::new(&self.root).into_iter();

        for entry in walkdir {
            let entry = entry.with_context(|| {
                format!(
                    "Could not read the local source directory {}",
                    self.root.display()
                )
            })?;
            let path = entry.path();

            if path.is_file() {
                paths.push(path.to_path_buf());
            }
        }

        Ok(paths)
    }

    fn get_file(&self, file_name: &Path) -> Result<Option<SourceFile>> {
        let path = self.root.join(file_name);

        if path.is_file() {
            Ok(Some(SourceFile {
                path: path.clone(),
                contents: fs::read_to_string(&path).with_context(|| {
                    format!(
                        "Could not read the file {} from the local source {}",
                        path.display(),
                        self.root.display()
                    )
                })?,
            }))
        } else {
            Ok(None)
        }
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
