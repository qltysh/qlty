use crate::Transformer;
use qlty_types::tests::v1::{CoverageMetadata, ReportFile};

#[derive(Debug, Clone, Default)]
pub struct Plan {
    pub metadata: CoverageMetadata,
    pub report_files: Vec<ReportFile>,
    pub transformers: Vec<Box<dyn Transformer>>,
    pub skip_missing_files: bool,
    pub auto_path_fixing_enabled: bool,
    pub merge: bool,
}
