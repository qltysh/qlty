use crate::Formatter;
use anyhow::Result;
use prost::{bytes::BytesMut, Message};
use std::io::Write;

/// Formatter for multiple Protocol Buffer messages
#[derive(Debug)]
pub struct ProtosFormatter<T>
where
    T: IntoIterator,
    T::Item: Message,
{
    records: T,
}

impl<T> ProtosFormatter<T>
where
    T: IntoIterator + Clone + 'static,
    T::Item: Message,
{
    /// Create a new Protocol Buffers formatter for a collection of messages
    pub fn new(records: T) -> Box<dyn Formatter> {
        Box::new(Self {
            records: records.clone(),
        })
    }
}

impl<T> Formatter for ProtosFormatter<T>
where
    T: IntoIterator + Clone,
    T::Item: Message,
{
    fn write_to(&self, writer: &mut dyn Write) -> Result<()> {
        let mut buffer = BytesMut::new();

        for record in self.records.clone().into_iter() {
            record.encode_length_delimited(&mut buffer).unwrap();
        }

        writer.write_all(&buffer)?;
        Ok(())
    }
}

/// Formatter for a single Protocol Buffer message
#[derive(Debug)]
pub struct ProtoFormatter<T: Message> {
    record: T,
}

impl<T: Message + 'static> ProtoFormatter<T> {
    /// Create a new Protocol Buffer formatter for a single message
    pub fn new(record: T) -> Box<dyn Formatter> {
        Box::new(Self { record })
    }
}

impl<T: Message> Formatter for ProtoFormatter<T> {
    fn write_to(&self, writer: &mut dyn Write) -> Result<()> {
        let mut buffer = BytesMut::new();
        self.record.encode(&mut buffer)?;
        writer.write_all(&buffer)?;
        Ok(())
    }
}