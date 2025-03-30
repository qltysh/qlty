use crate::Formatter;
use anyhow::Result;
use serde::Serialize;
use std::io::Write;

/// Formatter for multiple JSON records, one per line
#[derive(Debug)]
pub struct JsonEachRowFormatter<T: Serialize> {
    data: Vec<T>,
}

impl<T: Serialize + 'static> JsonEachRowFormatter<T> {
    /// Create a new JSON each row formatter with the given data
    pub fn new(data: Vec<T>) -> Self {
        Self { data }
    }

    /// Create a new JSON each row formatter with a reference to the given data
    pub fn new_ref<'a>(data: &'a [T]) -> JsonEachRowFormatter<&'a T>
    where
        &'a T: Serialize + 'static,
    {
        JsonEachRowFormatter {
            data: data.iter().collect(),
        }
    }

    /// Create a boxed JSON each row formatter with the given data
    pub fn boxed(data: Vec<T>) -> Box<dyn Formatter> {
        Box::new(Self { data })
    }

    /// Create a boxed JSON each row formatter with a reference to the given data
    pub fn boxed_ref<'a>(data: &'a [T]) -> Box<dyn Formatter>
    where
        &'a T: Serialize + 'static,
    {
        Box::new(JsonEachRowFormatter {
            data: data.iter().collect(),
        })
    }
}

impl<T: Serialize> Formatter for JsonEachRowFormatter<T> {
    fn write_to(&self, writer: &mut dyn Write) -> Result<()> {
        for row in &self.data {
            let json_row = serde_json::to_string(row)?;
            writer.write_all(json_row.as_bytes())?;
            writer.write_all(b"\n")?;
        }
        Ok(())
    }
}
