use super::{source::SourceFetch, Source, SourceFile};
use anyhow::{Context as _, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct LocalSource {
    pub root: PathBuf,
}

impl Source for LocalSource {
    fn files(&self) -> Result<Vec<SourceFile>> {
        let mut source_files = Vec::new();

        let read_dir = fs::read_dir(&self.root).with_context(|| {
            format!(
                "Could not read the local source directory {}",
                self.root.display()
            )
        })?;

        for entry in read_dir {
            let path = entry?.path();

            if path.is_file() {
                source_files.push(SourceFile {
                    path: path.clone(),
                    contents: fs::read_to_string(&path).with_context(|| {
                        format!(
                            "Could not read the file {} from the local source {}",
                            path.display(),
                            self.root.display()
                        )
                    })?,
                });
            }
        }

        Ok(source_files)
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
