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
    pub fn boxed(path: PathBuf) -> Box<dyn Formatter> {
        Box::new(Self { path })
    }
}

impl Formatter for CopyFormatter {
    fn write_to(&self, writer: &mut dyn Write) -> anyhow::Result<()> {
        copy(&mut File::open(&self.path)?, writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_copy_formatter() -> anyhow::Result<()> {
        let path = PathBuf::from("Cargo.toml");
        let formatter = CopyFormatter::boxed(path.clone());
        let mut buffer = Vec::new();
        formatter.write_to(&mut buffer)?;

        let mut file = File::open(path)?;
        let mut file_buffer = Vec::new();
        file.read_to_end(&mut file_buffer)?;

        assert_eq!(buffer, file_buffer);
        Ok(())
    }
}
