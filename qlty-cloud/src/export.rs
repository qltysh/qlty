pub mod analysis;
// coverage module moved to qlty-coverage crate

pub use analysis::AnalysisExport;
// CoverageExport now available directly from qlty-coverage::export

#[derive(Debug, Clone, Copy, Default)]
pub enum ExportFormat {
    #[default]
    Json,
    Protobuf,
}
