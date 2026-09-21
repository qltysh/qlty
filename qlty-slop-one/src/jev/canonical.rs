//! Python `json.dumps` compatibility, so cache keys equal slopdetect's
//! `digest(body)` = SHA-256 of
//! `json.dumps(body, sort_keys=True, separators=(',', ':'))`.
//!
//! Python's default `ensure_ascii=True` escapes every character outside
//! `0x20..=0x7e` as `\uXXXX` (astral characters as surrogate pairs), keeps
//! the short forms for `\n \r \t \b \f`, escapes `"` and `\`, and leaves `/`
//! alone. Keys sort by code point, which is UTF-8 byte order.

use serde_json::Value;
use sha2::{Digest as _, Sha256};

use crate::error::{Error, Result};

/// The SHA-256 hex digest of the canonical form of `value`.
pub fn digest(value: &Value) -> Result<String> {
    let canonical = canonical(value)?;
    Ok(format!("{:x}", Sha256::digest(canonical.as_bytes())))
}

/// `json.dumps(value, sort_keys=True, separators=(',', ':'))`.
pub fn canonical(value: &Value) -> Result<String> {
    let mut writer = Writer {
        ensure_ascii: true,
        item_separator: ",",
        key_separator: ":",
        out: String::new(),
    };
    writer.write(value)?;
    Ok(writer.out)
}

/// `len(json.dumps(value, ensure_ascii=False).encode())`, the size
/// slopdetect guards requests with. Key order does not affect the length.
pub fn utf8_length(value: &Value) -> Result<usize> {
    let mut writer = Writer {
        ensure_ascii: false,
        item_separator: ", ",
        key_separator: ": ",
        out: String::new(),
    };
    writer.write(value)?;
    Ok(writer.out.len())
}

struct Writer {
    ensure_ascii: bool,
    item_separator: &'static str,
    key_separator: &'static str,
    out: String,
}

impl Writer {
    fn write(&mut self, value: &Value) -> Result<()> {
        match value {
            Value::Null => self.out.push_str("null"),
            Value::Bool(true) => self.out.push_str("true"),
            Value::Bool(false) => self.out.push_str("false"),
            Value::Number(number) => self.write_number(number)?,
            Value::String(text) => self.write_string(text),
            Value::Array(items) => {
                self.out.push('[');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        self.out.push_str(self.item_separator);
                    }
                    self.write(item)?;
                }
                self.out.push(']');
            }
            Value::Object(object) => {
                let mut keys: Vec<&String> = object.keys().collect();
                keys.sort();
                self.out.push('{');
                for (index, key) in keys.into_iter().enumerate() {
                    if index > 0 {
                        self.out.push_str(self.item_separator);
                    }
                    self.write_string(key);
                    self.out.push_str(self.key_separator);
                    self.write(&object[key])?;
                }
                self.out.push('}');
            }
        }
        Ok(())
    }

    fn write_number(&mut self, number: &serde_json::Number) -> Result<()> {
        if let Some(integer) = number.as_i64() {
            self.out.push_str(&integer.to_string());
            return Ok(());
        }
        if let Some(integer) = number.as_u64() {
            self.out.push_str(&integer.to_string());
            return Ok(());
        }
        Err(Error::Jev(
            "Jev request bodies carry only integers and text.".to_owned(),
        ))
    }

    fn write_string(&mut self, text: &str) {
        self.out.push('"');
        for c in text.chars() {
            match c {
                '"' => self.out.push_str("\\\""),
                '\\' => self.out.push_str("\\\\"),
                '\n' => self.out.push_str("\\n"),
                '\r' => self.out.push_str("\\r"),
                '\t' => self.out.push_str("\\t"),
                '\u{8}' => self.out.push_str("\\b"),
                '\u{c}' => self.out.push_str("\\f"),
                c if (c as u32) < 0x20 => self.write_escape(c as u32),
                c if self.ensure_ascii && (c as u32) >= 0x7f => self.write_escape(c as u32),
                c => self.out.push(c),
            }
        }
        self.out.push('"');
    }

    fn write_escape(&mut self, code: u32) {
        if code < 0x10000 {
            self.out.push_str(&format!("\\u{code:04x}"));
            return;
        }
        let offset = code - 0x10000;
        let high = 0xd800 | ((offset >> 10) & 0x3ff);
        let low = 0xdc00 | (offset & 0x3ff);
        self.out.push_str(&format!("\\u{high:04x}\\u{low:04x}"));
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn sorts_nested_keys_and_uses_compact_separators() {
        let value = json!({"b": 1, "a": {"z": [1, 2], "y": "x"}});
        assert_eq!(
            canonical(&value).unwrap(),
            r#"{"a":{"y":"x","z":[1,2]},"b":1}"#
        );
    }

    #[test]
    fn escapes_non_ascii_as_lowercase_unicode_escapes() {
        assert_eq!(canonical(&json!("λ日")).unwrap(), "\"\\u03bb\\u65e5\"");
    }

    #[test]
    fn escapes_astral_characters_as_surrogate_pairs() {
        assert_eq!(canonical(&json!("🎉")).unwrap(), "\"\\ud83c\\udf89\"");
    }

    #[test]
    fn uses_short_escapes_for_common_controls_and_unicode_escapes_for_others() {
        assert_eq!(
            canonical(&json!("\t\n\r\u{8}\u{c}\u{0}\u{1f}\u{7f}")).unwrap(),
            r#""\t\n\r\b\f\u0000\u001f\u007f""#
        );
    }

    #[test]
    fn escapes_quotes_and_backslashes_but_not_slashes() {
        assert_eq!(canonical(&json!("a\"b\\c/d")).unwrap(), r#""a\"b\\c/d""#);
    }

    #[test]
    fn utf8_length_counts_raw_non_ascii_and_default_separators() {
        let value = json!({"a": "λ", "b": [1, 2]});
        assert_eq!(
            utf8_length(&value).unwrap(),
            r#"{"a": "λ", "b": [1, 2]}"#.len()
        );
    }

    #[test]
    fn utf8_length_keeps_del_unescaped() {
        assert_eq!(utf8_length(&json!("\u{7f}")).unwrap(), 3);
    }

    #[test]
    fn rejects_non_integer_numbers() {
        assert!(canonical(&json!(1.5)).is_err());
    }

    #[test]
    fn digest_is_sha256_of_the_canonical_bytes() {
        assert_eq!(
            digest(&json!({})).unwrap(),
            "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"
        );
    }
}
