use std::{path::PathBuf, sync::Arc};

use qlty_analysis::source_reader::{SourceReader, SourceReaderFs};

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: PathBuf,
    pub source_reader: Arc<dyn SourceReader + Send + Sync>,
}

impl SourceReader for Workspace {
    fn read(&self, relative_path: PathBuf) -> std::io::Result<String> {
        let staged_file_path = self.root.join(relative_path);
        self.source_reader.read(staged_file_path)
    }

    fn write(&self, _relative_path: PathBuf, _content: String) -> std::io::Result<()> {
        Ok(())
    }
}

impl Workspace {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            source_reader: Arc::<SourceReaderFs>::default(),
        }
    }
}
