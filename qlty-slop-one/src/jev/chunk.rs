//! Splitting a source into consecutive excerpts of at most
//! [`MAX_CHUNK_BYTES`], exactly as slopdetect's `chunks` does.

use crate::pylines::split_lines_keepends;

/// The byte budget of one Jev excerpt.
pub const MAX_CHUNK_BYTES: usize = 18000;

/// The consecutive excerpts of `text`. Complete lines (Python `splitlines`
/// lines, with their endings) stay together while they fit; a line longer
/// than the budget is split between characters. Empty text is one empty
/// excerpt. The excerpts concatenate back to `text`.
pub fn chunks(text: &str) -> Vec<&str> {
    chunks_with_limit(text, MAX_CHUNK_BYTES)
}

pub(crate) fn chunks_with_limit(text: &str, maximum: usize) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut end = 0;
    let mut used = 0;
    let mut cursor = 0;
    for line in split_lines_keepends(text) {
        if line.len() > maximum {
            if end > start {
                parts.push(&text[start..end]);
                start = end;
                used = 0;
            }
            for (offset, c) in line.char_indices() {
                let length = c.len_utf8();
                if used + length > maximum {
                    parts.push(&text[start..end]);
                    start = end;
                    used = 0;
                }
                end = cursor + offset + length;
                used += length;
            }
        } else {
            if end > start && used + line.len() > maximum {
                parts.push(&text[start..end]);
                start = end;
                used = 0;
            }
            end = cursor + line.len();
            used += line.len();
        }
        cursor += line.len();
    }
    if end > start {
        parts.push(&text[start..end]);
    }
    if parts.is_empty() {
        parts.push("");
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_is_one_empty_chunk() {
        assert_eq!(chunks(""), [""]);
    }

    #[test]
    fn short_text_is_one_chunk() {
        assert_eq!(chunks("a\nb\n"), ["a\nb\n"]);
    }

    #[test]
    fn keeps_whole_lines_together_while_they_fit() {
        assert_eq!(
            chunks_with_limit("aaa\nbbb\nccc\n", 8),
            ["aaa\nbbb\n", "ccc\n"]
        );
    }

    #[test]
    fn splits_an_oversized_line_between_characters_by_byte_budget() {
        let text = "λ".repeat(30) + "\n" + &"short\n".repeat(5);
        let pieces = chunks_with_limit(&text, 17);
        assert_eq!(pieces.concat(), text);
        assert!(pieces.iter().all(|piece| piece.len() <= 17));
        assert_eq!(pieces[0], "λ".repeat(8));
    }

    #[test]
    fn flushes_the_pending_lines_before_an_oversized_line() {
        assert_eq!(
            chunks_with_limit("ab\ncdefghijk\n", 5),
            ["ab\n", "cdefg", "hijk\n"]
        );
    }

    #[test]
    fn two_lines_just_over_the_budget_become_two_chunks() {
        let line = "x".repeat(10000) + "\n";
        let text = line.repeat(2);
        let pieces = chunks(&text);
        assert_eq!(pieces.len(), 2);
        assert_eq!(pieces[0], line);
        assert_eq!(pieces[1], line);
    }

    #[test]
    fn a_single_forty_thousand_byte_line_splits_at_the_budget() {
        let text = "y".repeat(40000);
        let pieces = chunks(&text);
        assert_eq!(
            pieces.iter().map(|piece| piece.len()).collect::<Vec<_>>(),
            [18000, 18000, 4000]
        );
        assert_eq!(pieces.concat(), text);
    }

    #[test]
    fn multi_byte_characters_never_straddle_a_boundary() {
        let text = "é".repeat(20001);
        let pieces = chunks(&text);
        assert_eq!(pieces[0].len(), 18000);
        assert_eq!(pieces[0].chars().count(), 9000);
        assert_eq!(pieces.concat(), text);
    }

    #[test]
    fn python_line_boundaries_count_as_line_ends() {
        assert_eq!(
            chunks_with_limit("ab\x0ccd\u{2028}ef", 5),
            ["ab\x0c", "cd\u{2028}", "ef"]
        );
    }

    #[test]
    fn exact_coverage_of_mixed_content() {
        let text = "fn main() {\n    println!(\"héllo ✨\");\n}\n".repeat(1200);
        let pieces = chunks(&text);
        assert_eq!(pieces.concat(), text);
        assert!(pieces.iter().all(|piece| piece.len() <= MAX_CHUNK_BYTES));
        assert!(pieces.iter().all(|piece| piece.ends_with('\n')));
    }
}
