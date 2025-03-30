use crate::Formatter;
use anyhow::Result;
use std::{
    fs::File,
    io::{copy, Write},
    path::{Path, PathBuf},
};

/// Formatter for copying file contents
#[derive(Debug)]
pub struct CopyFormatter {
    path: PathBuf,
}

impl CopyFormatter {
    /// Create a new copy formatter with the given path
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Create a new copy formatter with a reference to the given path
    pub fn new_ref(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    /// Create a boxed copy formatter with the given path
    pub fn boxed(path: PathBuf) -> Box<dyn Formatter> {
        Box::new(Self { path })
    }

    /// Create a boxed copy formatter with a reference to the given path
    pub fn boxed_ref(path: &Path) -> Box<dyn Formatter> {
        Box::new(Self::new_ref(path))
    }
}

impl Formatter for CopyFormatter {
    fn write_to(&self, writer: &mut dyn Write) -> Result<()> {
        copy(&mut File::open(&self.path)?, writer)?;
        Ok(())
    }
}
