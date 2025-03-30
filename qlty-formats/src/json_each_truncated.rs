use crate::Formatter;
use anyhow::Result;
use qlty_types::analysis::v1::Invocation;
use std::io::Write;

/// Maximum string length for invocation fields (4MB)
const MAX_STRING_LENGTH: usize = 4_000_000;

/// Formatter for multiple invocation records with truncation of large fields
#[derive(Debug)]
pub struct InvocationJsonFormatter {
    data: Vec<Invocation>,
}

impl InvocationJsonFormatter {
    /// Create a new invocation JSON formatter
    pub fn new(data: Vec<Invocation>) -> Self {
        Self { data }
    }

    /// Create a new invocation JSON formatter with a reference to the data
    pub fn new_ref(data: &[Invocation]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    /// Create a boxed invocation JSON formatter
    pub fn boxed(data: Vec<Invocation>) -> Box<dyn Formatter> {
        Box::new(Self::new(data))
    }

    /// Create a boxed invocation JSON formatter with a reference to the data
    pub fn boxed_ref(data: &[Invocation]) -> Box<dyn Formatter> {
        Box::new(Self::new_ref(data))
    }
}

impl Formatter for InvocationJsonFormatter {
    fn write_to(&self, writer: &mut dyn Write) -> Result<()> {
        for invocation in &self.data {
            let truncated_invocation = omit_long_strings(invocation);
            let json_row = serde_json::to_string(&truncated_invocation)?;
            writer.write_all(json_row.as_bytes())?;
            writer.write_all(b"\n")?;
        }

        Ok(())
    }
}

/// Truncate excessively long strings in the invocation
fn omit_long_strings(invocation: &Invocation) -> Invocation {
    let mut invocation: Invocation = invocation.clone();

    if invocation.stdout.len() > MAX_STRING_LENGTH {
        invocation.stdout = format!(
            "Data omitted because {} bytes is larger than the maximum of {} bytes",
            invocation.stdout.len(),
            MAX_STRING_LENGTH
        );
    }
    if invocation.stderr.len() > MAX_STRING_LENGTH {
        invocation.stderr = format!(
            "Data omitted because {} bytes is larger than the maximum of {} bytes",
            invocation.stderr.len(),
            MAX_STRING_LENGTH
        );
    }
    if let Some(tmpfile_contents) = &invocation.tmpfile_contents {
        if tmpfile_contents.len() > MAX_STRING_LENGTH {
            invocation.tmpfile_contents = Some(format!(
                "Data omitted because {} bytes is larger than the maximum of {} bytes",
                tmpfile_contents.len(),
                MAX_STRING_LENGTH
            ));
        }
    }

    invocation
}
