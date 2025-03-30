use crate::Formatter;
use anyhow::Result;
use serde::Serialize;
use std::io::Write;

/// Formatter for JSON data
#[derive(Debug)]
pub struct JsonFormatter<T: Serialize> {
    data: T,
}

impl<T: Serialize + 'static> JsonFormatter<T> {
    /// Create a new JSON formatter with the given data
    pub fn new(data: T) -> Self {
        Self { data }
    }

    /// Create a new JSON formatter with a reference to the given data
    pub fn new_ref<'a>(data: &'a T) -> JsonFormatter<&'a T>
    where
        &'a T: Serialize + 'static,
    {
        JsonFormatter { data }
    }

    /// Create a boxed JSON formatter with the given data
    pub fn boxed(data: T) -> Box<dyn Formatter> {
        Box::new(Self { data })
    }

    /// Create a boxed JSON formatter with a reference to the given data
    pub fn boxed_ref<'a>(data: &'a T) -> Box<dyn Formatter + 'a>
    where
        &'a T: Serialize,
    {
        Box::new(JsonFormatter { data })
    }
}

impl<T: Serialize> Formatter for JsonFormatter<T> {
    fn write_to(&self, writer: &mut dyn Write) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.data)?;
        writer.write_all(json.as_bytes())?;
        writer.write_all(b"\n")?;
        Ok(())
    }
}
