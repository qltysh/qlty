use super::Formatter;
use std::{
    fs::File,
    io::{copy, Write},
    path::PathBuf,
};

#[derive(Debug)]
pub struct CopyFormatter {
    path: PathBuf,
}

impl CopyFormatter {
    pub fn new(path: PathBuf) -> Box<dyn Formatter> {
        Box::new(Self { path })
    }
}

impl Formatter for CopyFormatter {
    fn write_to(&self, writer: &mut dyn Write) -> anyhow::Result<()> {
        copy(&mut File::open(&self.path)?, writer)?;
        Ok(())
    }
}
