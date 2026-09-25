//! SVG chart and standalone HTML report.

pub mod html;
pub mod svg;

/// Escapes text for an HTML or SVG text node or quoted attribute.
pub(crate) fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            other => escaped.push(other),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_markup_and_quotes() {
        assert_eq!(
            escape_html(r#"a<b & "c" 'd'"#),
            "a&lt;b &amp; &quot;c&quot; &#x27;d&#x27;"
        );
    }
}
