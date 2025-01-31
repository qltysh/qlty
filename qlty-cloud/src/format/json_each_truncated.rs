use super::Formatter;
use qlty_types::analysis::v1::Invocation;
use std::io::Write;

const MAX_STRING_LENGTH: usize = 4_000_000; // Limit long strings to 4MB

#[derive(Debug)]
pub struct InvocationJsonFormatter {
    data: Vec<Invocation>,
}

impl InvocationJsonFormatter {
    pub fn new(data: Vec<Invocation>) -> Box<dyn Formatter> {
        Box::new(Self { data })
    }
}

impl Formatter for InvocationJsonFormatter {
    fn write_to(&self, writer: &mut dyn Write) -> anyhow::Result<()> {
        for invocation in &self.data {
            let truncated_invocation = omit_long_strings(invocation);
            let json_row = serde_json::to_string(&truncated_invocation)?;
            writer.write_all(json_row.as_bytes())?;
            writer.write_all(b"\n")?;
        }

        Ok(())
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use qlty_types::analysis::v1::Invocation;
    use std::io::Cursor;

    #[test]
    fn test_truncate_large_fields() {
        let large_string = "A".repeat(5_000_000); // 5MB string (exceeds 4MB limit)
        let invocation = Invocation {
            stdout: large_string.clone(),
            stderr: large_string.clone(),
            tmpfile_contents: Some(large_string.clone()),
            ..Default::default() // Other fields use default values
        };

        let truncated = omit_long_strings(&invocation);

        // Verify truncation messages are present instead of original data
        assert!(truncated.stdout.contains("Data omitted"));
        assert!(truncated.stderr.contains("Data omitted"));
        assert!(truncated
            .tmpfile_contents
            .as_ref()
            .unwrap()
            .contains("Data omitted"));

        // Verify original large content is no longer there
        assert!(!truncated.stdout.contains(&large_string));
        assert!(!truncated.stderr.contains(&large_string));
        assert!(!truncated.tmpfile_contents.unwrap().contains(&large_string));
    }

    #[test]
    fn test_does_not_truncate_small_fields() {
        let small_string = "Hello, world!".to_string();
        let invocation = Invocation {
            stdout: small_string.clone(),
            stderr: small_string.clone(),
            tmpfile_contents: Some(small_string.clone()),
            ..Default::default()
        };

        let truncated = omit_long_strings(&invocation);

        // Small strings should remain unchanged
        assert_eq!(truncated.stdout, small_string);
        assert_eq!(truncated.stderr, small_string);
        assert_eq!(truncated.tmpfile_contents.unwrap(), small_string);
    }

    #[test]
    fn test_json_formatting_after_truncation() {
        let large_string = "A".repeat(5_000_000);
        let invocation = Invocation {
            stdout: large_string.clone(),
            stderr: large_string.clone(),
            tmpfile_contents: Some(large_string.clone()),
            ..Default::default()
        };

        let formatter = InvocationJsonFormatter::new(vec![invocation]);

        let mut output = Cursor::new(Vec::new()); // Simulate a writable buffer
        formatter.write_to(&mut output).unwrap();

        let output_str = String::from_utf8(output.into_inner()).unwrap();

        // Ensure valid JSON output
        let parsed: serde_json::Value = serde_json::from_str(&output_str).unwrap();
        assert!(parsed.is_object() || parsed.is_array());

        // Ensure truncation messages are in JSON
        assert!(output_str.contains("Data omitted because"));
    }
}
