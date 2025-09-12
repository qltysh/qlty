use crate::git_index::tsv_writer;
use chrono::{DateTime, Local};
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineType {
    Empty,
    Comment,
    Punct,
    Code,
}

impl LineType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LineType::Empty => "Empty",
            LineType::Comment => "Comment",
            LineType::Punct => "Punct",
            LineType::Code => "Code",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LineChange {
    pub sign: i8,
    pub line_number_old: u32,
    pub line_number_new: u32,
    pub hunk_num: u32,
    pub hunk_start_line_number_old: u32,
    pub hunk_start_line_number_new: u32,
    pub hunk_lines_added: u32,
    pub hunk_lines_deleted: u32,
    pub hunk_context: String,
    pub line: String,
    pub indent: u8,
    pub line_type: LineType,
    pub prev_commit_hash: String,
    pub prev_author: String,
    pub prev_time: Option<DateTime<Local>>,
}

impl LineChange {
    pub fn classify_line(full_line: &str) -> (String, u8, LineType) {
        let mut num_spaces = 0u32;
        let mut chars = full_line.chars().peekable();

        while let Some(&ch) = chars.peek() {
            if ch == ' ' {
                num_spaces += 1;
            } else if ch == '\t' {
                num_spaces += 4;
            } else {
                break;
            }
            chars.next();
        }

        let indent = num_spaces.min(255) as u8;
        let line: String = chars.collect();

        let line_type = if line.is_empty() {
            LineType::Empty
        } else if line.starts_with("//")
            || line.starts_with("/*")
            || line.starts_with("* ")
            || line.starts_with("# ")
        {
            LineType::Comment
        } else {
            let has_alphanum = line.chars().any(|c| c.is_alphanumeric());
            if has_alphanum {
                LineType::Code
            } else {
                LineType::Punct
            }
        };

        (line, indent, line_type)
    }

    pub fn write_text_without_newline(&self, out: &mut impl Write) -> std::io::Result<()> {
        tsv_writer::write_text(out, self.sign)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.line_number_old)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.line_number_new)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.hunk_num)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.hunk_start_line_number_old)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.hunk_start_line_number_new)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.hunk_lines_added)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.hunk_lines_deleted)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_escaped_string(out, &self.hunk_context)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_escaped_string(out, &self.line)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.indent)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.line_type.as_str())?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_escaped_string(out, &self.prev_commit_hash)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_escaped_string(out, &self.prev_author)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.prev_time.map_or(0, |t| t.timestamp()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_type_as_str() {
        assert_eq!(LineType::Empty.as_str(), "Empty");
        assert_eq!(LineType::Comment.as_str(), "Comment");
        assert_eq!(LineType::Punct.as_str(), "Punct");
        assert_eq!(LineType::Code.as_str(), "Code");
    }

    #[test]
    fn test_classify_line_empty() {
        let (line, indent, line_type) = LineChange::classify_line("");
        assert_eq!(line, "");
        assert_eq!(indent, 0);
        assert_eq!(line_type, LineType::Empty);

        let (line, indent, line_type) = LineChange::classify_line("    ");
        assert_eq!(line, "");
        assert_eq!(indent, 4);
        assert_eq!(line_type, LineType::Empty);

        let (line, indent, line_type) = LineChange::classify_line("\t");
        assert_eq!(line, "");
        assert_eq!(indent, 4);
        assert_eq!(line_type, LineType::Empty);
    }

    #[test]
    fn test_classify_line_comment() {
        let (line, indent, line_type) = LineChange::classify_line("// this is a comment");
        assert_eq!(line, "// this is a comment");
        assert_eq!(indent, 0);
        assert_eq!(line_type, LineType::Comment);

        let (line, indent, line_type) = LineChange::classify_line("  // indented comment");
        assert_eq!(line, "// indented comment");
        assert_eq!(indent, 2);
        assert_eq!(line_type, LineType::Comment);

        let (line, indent, line_type) = LineChange::classify_line("/* block comment */");
        assert_eq!(line, "/* block comment */");
        assert_eq!(indent, 0);
        assert_eq!(line_type, LineType::Comment);

        let (line, indent, line_type) = LineChange::classify_line("* documentation");
        assert_eq!(line, "* documentation");
        assert_eq!(indent, 0);
        assert_eq!(line_type, LineType::Comment);

        let (line, indent, line_type) = LineChange::classify_line("# python comment");
        assert_eq!(line, "# python comment");
        assert_eq!(indent, 0);
        assert_eq!(line_type, LineType::Comment);
    }

    #[test]
    fn test_classify_line_code() {
        let (line, indent, line_type) = LineChange::classify_line("let x = 42;");
        assert_eq!(line, "let x = 42;");
        assert_eq!(indent, 0);
        assert_eq!(line_type, LineType::Code);

        let (line, indent, line_type) = LineChange::classify_line("    return true;");
        assert_eq!(line, "return true;");
        assert_eq!(indent, 4);
        assert_eq!(line_type, LineType::Code);

        let (line, indent, line_type) = LineChange::classify_line("a123");
        assert_eq!(line, "a123");
        assert_eq!(indent, 0);
        assert_eq!(line_type, LineType::Code);
    }

    #[test]
    fn test_classify_line_punct() {
        let (line, indent, line_type) = LineChange::classify_line("{");
        assert_eq!(line, "{");
        assert_eq!(indent, 0);
        assert_eq!(line_type, LineType::Punct);

        let (line, indent, line_type) = LineChange::classify_line("    }");
        assert_eq!(line, "}");
        assert_eq!(indent, 4);
        assert_eq!(line_type, LineType::Punct);

        let (line, indent, line_type) = LineChange::classify_line(";;;");
        assert_eq!(line, ";;;");
        assert_eq!(indent, 0);
        assert_eq!(line_type, LineType::Punct);

        let (line, indent, line_type) = LineChange::classify_line("---");
        assert_eq!(line, "---");
        assert_eq!(indent, 0);
        assert_eq!(line_type, LineType::Punct);
    }

    #[test]
    fn test_classify_line_indentation_spaces() {
        let (_, indent, _) = LineChange::classify_line("  code");
        assert_eq!(indent, 2);

        let (_, indent, _) = LineChange::classify_line("    code");
        assert_eq!(indent, 4);

        let (_, indent, _) = LineChange::classify_line("        code");
        assert_eq!(indent, 8);
    }

    #[test]
    fn test_classify_line_indentation_tabs() {
        let (_, indent, _) = LineChange::classify_line("\tcode");
        assert_eq!(indent, 4);

        let (_, indent, _) = LineChange::classify_line("\t\tcode");
        assert_eq!(indent, 8);

        let (_, indent, _) = LineChange::classify_line("\t\t\tcode");
        assert_eq!(indent, 12);
    }

    #[test]
    fn test_classify_line_indentation_mixed() {
        let (_, indent, _) = LineChange::classify_line("  \tcode");
        assert_eq!(indent, 6);

        let (_, indent, _) = LineChange::classify_line("\t  code");
        assert_eq!(indent, 6);

        let (_, indent, _) = LineChange::classify_line(" \t code");
        assert_eq!(indent, 6);
    }

    #[test]
    fn test_classify_line_indentation_max() {
        let long_indent = " ".repeat(300);
        let input = format!("{}code", long_indent);
        let (_, indent, _) = LineChange::classify_line(&input);
        assert_eq!(indent, 255);
    }

    #[test]
    fn test_line_change_write_text_without_newline() {
        let line_change = LineChange {
            sign: 1,
            line_number_old: 10,
            line_number_new: 20,
            hunk_num: 1,
            hunk_start_line_number_old: 5,
            hunk_start_line_number_new: 15,
            hunk_lines_added: 3,
            hunk_lines_deleted: 2,
            hunk_context: "fn test()".to_string(),
            line: "let x = 42;".to_string(),
            indent: 4,
            line_type: LineType::Code,
            prev_commit_hash: "abc123".to_string(),
            prev_author: "John Doe".to_string(),
            prev_time: None,
        };

        let mut output = Vec::new();
        line_change.write_text_without_newline(&mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        assert!(result.contains("1\t"));
        assert!(result.contains("10\t"));
        assert!(result.contains("20\t"));
        assert!(result.contains("Code"));
        assert!(result.contains("abc123"));
        assert!(result.contains("John Doe"));
        assert!(result.ends_with("0"));
    }

    #[test]
    fn test_line_change_write_with_timestamp() {
        use chrono::TimeZone;

        let timestamp = Local.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let line_change = LineChange {
            sign: -1,
            line_number_old: 5,
            line_number_new: 0,
            hunk_num: 1,
            hunk_start_line_number_old: 5,
            hunk_start_line_number_new: 0,
            hunk_lines_added: 0,
            hunk_lines_deleted: 1,
            hunk_context: "fn main()".to_string(),
            line: "deleted line".to_string(),
            indent: 0,
            line_type: LineType::Code,
            prev_commit_hash: "def456".to_string(),
            prev_author: "Jane Smith".to_string(),
            prev_time: Some(timestamp),
        };

        let mut output = Vec::new();
        line_change.write_text_without_newline(&mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        assert!(result.contains(&timestamp.timestamp().to_string()));
    }
}
