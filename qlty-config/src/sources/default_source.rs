use std::path::Path;

use super::{source::SourceFetch, Source, SourceFile};
use anyhow::{bail, Result};
use qlty_plugins::Plugins;

#[derive(Debug, Clone)]
pub struct DefaultSource {}

impl Source for DefaultSource {
    fn files(&self) -> Result<Vec<SourceFile>> {
        let mut source_files = vec![];

        for file_path in Plugins::iter() {
            let source_file = self.get_file(Path::new(file_path.as_ref()))?;

            if let Some(file) = source_file {
                source_files.push(file);
            } else {
                bail!(
                    "Could not find expected file in default source: {}",
                    file_path
                );
            }
        }

        Ok(source_files)
    }

    fn get_file(&self, file_name: &Path) -> Result<Option<SourceFile>> {
        let file_path = file_name.to_str().expect("file path is not valid");

        if let Some(embedded_file) = Plugins::get(file_path) {
            Ok(Some(SourceFile {
                path: file_name.to_path_buf(),
                contents: String::from_utf8_lossy(&embedded_file.data).to_string(),
            }))
        } else {
            Ok(None)
        }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_plugin_tomls() {
        let default_source = DefaultSource {};
        let plugin_tomls = default_source.files().unwrap();
        assert_eq!(plugin_tomls.len(), 46);
    }
}
