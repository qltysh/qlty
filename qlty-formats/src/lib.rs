use anyhow::{anyhow, Context, Result};
use std::path::Path;

pub mod copy;
pub mod gz;
pub mod json;
pub mod json_each;
pub mod json_each_truncated;
pub mod protos;

pub use copy::CopyFormatter;
pub use gz::GzFormatter;
pub use json::JsonFormatter;
pub use json_each::JsonEachRowFormatter;
pub use json_each_truncated::InvocationJsonFormatter;
pub use protos::{ProtoFormatter, ProtosFormatter};

pub trait Formatter {
    fn write_to(&self, writer: &mut dyn std::io::Write) -> Result<()>;

    fn write_to_file(&self, path: &Path) -> Result<()> {
        let directory = path
            .parent()
            .ok_or_else(|| anyhow!("Failed to get parent directory of file: {:?}", path))?;

        std::fs::create_dir_all(directory)
            .with_context(|| format!("Failed to create directory: {:?}", directory))?;

        let mut file = std::fs::File::create(path)?;
        self.write_to(&mut file)
    }

    fn read(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.write_to(&mut buffer)?;
        Ok(buffer)
    }
}
