mod analysis;

pub use analysis::AnalysisExport;

#[derive(Debug, Clone, Copy, Default)]
pub enum ExportFormat {
    #[default]
    Json,
    Protobuf,
}
