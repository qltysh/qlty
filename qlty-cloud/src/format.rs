// Re-export the Formatter trait and formatters from qlty-formats
pub use qlty_formats::{Formatter, CopyFormatter, GzFormatter, JsonFormatter, JsonEachRowFormatter, InvocationJsonFormatter, ProtoFormatter, ProtosFormatter};

// Define a stub SarifFormatter for backward compatibility
pub struct SarifFormatter;

impl SarifFormatter {
    pub fn boxed_from_issues(_issues: Vec<qlty_types::analysis::v1::Issue>) -> Box<dyn Formatter> {
        unimplemented!("SarifFormatter has been moved to qlty-cli")
    }
}