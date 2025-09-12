use std::io::Write;

/// Write a character to the output
pub fn write_char(out: &mut impl Write, c: char) -> std::io::Result<()> {
    write!(out, "{}", c)
}

/// Write escaped string for TSV format
/// Escapes: \b, \f, \n, \r, \t, \0, \\
pub fn write_escaped_string(out: &mut impl Write, s: &str) -> std::io::Result<()> {
    for ch in s.chars() {
        match ch {
            '\t' => write!(out, "\\t")?,
            '\n' => write!(out, "\\n")?,
            '\r' => write!(out, "\\r")?,
            '\\' => write!(out, "\\\\")?,
            '\0' => write!(out, "\\0")?,
            '\u{0008}' => write!(out, "\\b")?, // backspace
            '\u{000C}' => write!(out, "\\f")?, // form feed
            _ => write!(out, "{}", ch)?,
        }
    }
    Ok(())
}

/// Write text (for numbers and other non-string values)
pub fn write_text<T: std::fmt::Display>(out: &mut impl Write, value: T) -> std::io::Result<()> {
    write!(out, "{}", value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_char() {
        let mut output = Vec::new();
        write_char(&mut output, 'a').unwrap();
        assert_eq!(output, b"a");

        let mut output = Vec::new();
        write_char(&mut output, '\t').unwrap();
        assert_eq!(output, b"\t");
    }

    #[test]
    fn test_write_escaped_string_no_escaping() {
        let mut output = Vec::new();
        write_escaped_string(&mut output, "hello world").unwrap();
        assert_eq!(output, b"hello world");
    }

    #[test]
    fn test_write_escaped_string_tab() {
        let mut output = Vec::new();
        write_escaped_string(&mut output, "hello\tworld").unwrap();
        assert_eq!(output, b"hello\\tworld");
    }

    #[test]
    fn test_write_escaped_string_newline() {
        let mut output = Vec::new();
        write_escaped_string(&mut output, "hello\nworld").unwrap();
        assert_eq!(output, b"hello\\nworld");
    }

    #[test]
    fn test_write_escaped_string_carriage_return() {
        let mut output = Vec::new();
        write_escaped_string(&mut output, "hello\rworld").unwrap();
        assert_eq!(output, b"hello\\rworld");
    }

    #[test]
    fn test_write_escaped_string_backslash() {
        let mut output = Vec::new();
        write_escaped_string(&mut output, "hello\\world").unwrap();
        assert_eq!(output, b"hello\\\\world");
    }

    #[test]
    fn test_write_escaped_string_null() {
        let mut output = Vec::new();
        write_escaped_string(&mut output, "hello\0world").unwrap();
        assert_eq!(output, b"hello\\0world");
    }

    #[test]
    fn test_write_escaped_string_backspace() {
        let mut output = Vec::new();
        write_escaped_string(&mut output, "hello\u{0008}world").unwrap();
        assert_eq!(output, b"hello\\bworld");
    }

    #[test]
    fn test_write_escaped_string_form_feed() {
        let mut output = Vec::new();
        write_escaped_string(&mut output, "hello\u{000C}world").unwrap();
        assert_eq!(output, b"hello\\fworld");
    }

    #[test]
    fn test_write_escaped_string_multiple_escapes() {
        let mut output = Vec::new();
        write_escaped_string(&mut output, "tab:\t newline:\n backslash:\\").unwrap();
        assert_eq!(output, b"tab:\\t newline:\\n backslash:\\\\");
    }

    #[test]
    fn test_write_text_numbers() {
        let mut output = Vec::new();
        write_text(&mut output, 42).unwrap();
        assert_eq!(output, b"42");

        let mut output = Vec::new();
        write_text(&mut output, 3.14).unwrap();
        assert_eq!(output, b"3.14");
    }

    #[test]
    fn test_write_text_strings() {
        let mut output = Vec::new();
        write_text(&mut output, "hello").unwrap();
        assert_eq!(output, b"hello");
    }
}
