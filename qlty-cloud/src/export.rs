pub mod analysis;
pub mod coverage;

pub use analysis::AnalysisExport;
pub use coverage::CoverageExport;

#[derive(Debug, Clone, Copy)]
#[derive(Default)]
pub enum ExportFormat {
    #[default]
    Json,
    Protobuf,
}


