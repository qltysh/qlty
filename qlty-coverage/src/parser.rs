use anyhow::{Context, Result};
use qlty_types::tests::v1::FileCoverage;
use std::path::Path;

mod clover;
mod cobertura;
mod coverprofile;
mod dotcover;
mod jacoco;
mod lcov;
mod qlty;
mod simplecov;

pub use clover::Clover;
pub use cobertura::Cobertura;
pub use coverprofile::Coverprofile;
pub use dotcover::Dotcover;
pub use jacoco::Jacoco;
pub use lcov::Lcov;
pub use qlty::Qlty;
pub use simplecov::Simplecov;

pub trait Parser {
    fn parse_file(&self, path: &Path) -> Result<Vec<FileCoverage>> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read coverage file: {}", path.display()))?;
        self.parse_text(&text)
            .with_context(|| format!("Failed to parse coverage file: {}", path.display()))
    }

    fn parse_text(&self, text: &str) -> Result<Vec<FileCoverage>>;
}
