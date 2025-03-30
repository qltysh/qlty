use crate::Formatter;
use anyhow::Result;
use flate2::{write::GzEncoder, Compression};
use std::io::Write;

/// Formatter for gzip compression
pub struct GzFormatter {
    formatter: Box<dyn Formatter>,
}

impl GzFormatter {
    /// Create a new gzip formatter that wraps another formatter
    pub fn new(formatter: Box<dyn Formatter>) -> Self {
        GzFormatter { formatter }
    }

    /// Create a boxed gzip formatter that wraps another formatter
    pub fn boxed(formatter: Box<dyn Formatter>) -> Box<dyn Formatter> {
        Box::new(Self::new(formatter))
    }
}

impl Formatter for GzFormatter {
    fn write_to(&self, writer: &mut dyn Write) -> Result<()> {
        let raw_data = self.formatter.read()?;
        let mut encoder = GzEncoder::new(writer, Compression::default());
        encoder.write_all(&raw_data)?;
        encoder.finish()?;
        Ok(())
    }
}
