pub const MAX_SNIPPET_LINES: usize = 1000;
pub const MAX_SNIPPET_BYTES: usize = 50 * 1024; // 50 kilobytes

pub fn truncate_snippet(snippet: &str) -> String {
    let mut truncated = String::new();
    let mut byte_count = 0;

    for line in snippet.split_inclusive('\n').take(MAX_SNIPPET_LINES) {
        if byte_count + line.len() > MAX_SNIPPET_BYTES {
            break;
        }

        truncated.push_str(line);
        byte_count += line.len();
    }

    // Remove the trailing newline if present and it wasn't in the original
    if truncated.ends_with('\n') && !snippet.ends_with('\n') {
        truncated.pop();
    }

    truncated
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_truncate_snippet_no_truncation_needed() {
        let input = "Line 1\nLine 2\nLine 3";
        let result = truncate_snippet(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_truncate_snippet_by_line_count() {
        let lines: Vec<String> = (1..=1500).map(|i| format!("Line {}", i)).collect();
        let input = lines.join("\n");
        let result = truncate_snippet(&input);

        let result_lines: Vec<&str> = result.lines().collect();
        assert_eq!(result_lines.len(), 1000);
        assert_eq!(result_lines[0], "Line 1");
        assert_eq!(result_lines[999], "Line 1000");
    }

    #[test]
    fn test_truncate_snippet_by_byte_size() {
        let long_line = "a".repeat(60 * 1024);
        let input = format!("Short line\n{}", long_line);
        let result = truncate_snippet(&input);

        assert!(result.len() <= 50 * 1024);
        assert!(result.starts_with("Short line"));
        assert!(!result.contains(&long_line));
    }

    #[test]
    fn test_truncate_snippet_byte_size_respects_line_boundaries() {
        let line_size = 100;
        let line = "a".repeat(line_size);
        let num_lines = (50 * 1024) / line_size + 10;
        let lines: Vec<String> = (0..num_lines).map(|_| line.clone()).collect();
        let input = lines.join("\n");

        let result = truncate_snippet(&input);

        assert!(result.len() <= 50 * 1024);
        let result_lines: Vec<&str> = result.lines().collect();
        for line in result_lines {
            assert_eq!(line.len(), line_size);
        }
    }

    #[test]
    fn test_truncate_snippet_empty_string() {
        let input = "";
        let result = truncate_snippet(input);
        assert_eq!(result, "");
    }

    #[test]
    fn test_truncate_snippet_single_line() {
        let input = "Single line";
        let result = truncate_snippet(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_truncate_snippet_exactly_at_limits() {
        let lines: Vec<String> = (1..=1000).map(|i| format!("Line {}", i)).collect();
        let input = lines.join("\n");
        let result = truncate_snippet(&input);
        assert_eq!(result, input);
    }
}