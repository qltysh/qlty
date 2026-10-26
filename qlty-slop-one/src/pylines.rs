//! Python `str.splitlines` semantics, which slopdetect used for line counts
//! and chunking. Rust's `str::lines` recognizes only `\n` and `\r\n`.

/// Whether `c` ends a line for Python's `str.splitlines`.
fn is_line_boundary(c: char) -> bool {
    matches!(
        c,
        '\n' | '\r'
            | '\x0b'
            | '\x0c'
            | '\x1c'
            | '\x1d'
            | '\x1e'
            | '\u{85}'
            | '\u{2028}'
            | '\u{2029}'
    )
}

/// The lines of `text` with their line endings kept, as
/// `text.splitlines(keepends=True)` returns them.
pub fn split_lines_keepends(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((index, c)) = chars.next() {
        if !is_line_boundary(c) {
            continue;
        }
        let mut end = index + c.len_utf8();
        if c == '\r' {
            if let Some((next_index, '\n')) = chars.peek().copied() {
                end = next_index + 1;
                chars.next();
            }
        }
        lines.push(&text[start..end]);
        start = end;
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

/// `len(text.splitlines())`.
pub fn line_count(text: &str) -> usize {
    split_lines_keepends(text).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_python_boundaries_and_keeps_ends() {
        assert_eq!(
            split_lines_keepends("a\nb\r\nc\rd\x0ce\u{2028}f"),
            ["a\n", "b\r\n", "c\r", "d\x0c", "e\u{2028}", "f"]
        );
    }

    #[test]
    fn trailing_newline_does_not_add_an_empty_line() {
        assert_eq!(split_lines_keepends("a\nb\n"), ["a\n", "b\n"]);
    }

    #[test]
    fn empty_text_has_no_lines() {
        assert_eq!(line_count(""), 0);
    }

    #[test]
    fn form_feed_counts_as_a_line_boundary() {
        assert_eq!(line_count("a\x0cb"), 2);
    }
}
