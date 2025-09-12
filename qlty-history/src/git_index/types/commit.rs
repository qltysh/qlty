use crate::git_index::tsv_writer;
use chrono::{DateTime, Local};
use std::io::Write;

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub author: String,
    pub time: DateTime<Local>,
    pub message: String,
    pub files_added: u32,
    pub files_deleted: u32,
    pub files_renamed: u32,
    pub files_modified: u32,
    pub lines_added: u32,
    pub lines_deleted: u32,
    pub hunks_added: u32,
    pub hunks_removed: u32,
    pub hunks_changed: u32,
}

impl Commit {
    pub fn write_text_without_newline(&self, out: &mut impl Write) -> std::io::Result<()> {
        tsv_writer::write_escaped_string(out, &self.hash)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_escaped_string(out, &self.author)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.time.timestamp())?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_escaped_string(out, &self.message)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.files_added)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.files_deleted)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.files_renamed)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.files_modified)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.lines_added)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.lines_deleted)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.hunks_added)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.hunks_removed)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_text(out, self.hunks_changed)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_commit_write_text_without_newline() {
        let time = Local.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        let commit = Commit {
            hash: "abc123def456".to_string(),
            author: "John Doe".to_string(),
            time,
            message: "feat: add new feature".to_string(),
            files_added: 3,
            files_deleted: 1,
            files_renamed: 2,
            files_modified: 5,
            lines_added: 150,
            lines_deleted: 75,
            hunks_added: 10,
            hunks_removed: 5,
            hunks_changed: 8,
        };

        let mut output = Vec::new();
        commit.write_text_without_newline(&mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        assert!(result.contains("abc123def456"));
        assert!(result.contains("John Doe"));
        assert!(result.contains(&time.timestamp().to_string()));
        assert!(result.contains("feat: add new feature"));
        assert!(result.contains("3\t"));
        assert!(result.contains("1\t"));
        assert!(result.contains("2\t"));
        assert!(result.contains("5\t"));
        assert!(result.contains("150"));
        assert!(result.contains("75"));
        assert!(result.contains("10"));
        assert!(result.contains("8"));
    }

    #[test]
    fn test_commit_write_with_special_chars_in_message() {
        let time = Local.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let commit = Commit {
            hash: "xyz789".to_string(),
            author: "Jane\tSmith".to_string(),
            time,
            message: "fix: handle\nnewlines\tand\\backslash".to_string(),
            files_added: 0,
            files_deleted: 0,
            files_renamed: 0,
            files_modified: 1,
            lines_added: 1,
            lines_deleted: 1,
            hunks_added: 0,
            hunks_removed: 0,
            hunks_changed: 1,
        };

        let mut output = Vec::new();
        commit.write_text_without_newline(&mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        assert!(result.contains("Jane\\tSmith"));
        assert!(result.contains("fix: handle\\nnewlines\\tand\\\\backslash"));
    }

    #[test]
    fn test_commit_write_empty_values() {
        let time = Local.with_ymd_and_hms(2024, 2, 1, 12, 0, 0).unwrap();
        let commit = Commit {
            hash: "empty123".to_string(),
            author: "".to_string(),
            time,
            message: "".to_string(),
            files_added: 0,
            files_deleted: 0,
            files_renamed: 0,
            files_modified: 0,
            lines_added: 0,
            lines_deleted: 0,
            hunks_added: 0,
            hunks_removed: 0,
            hunks_changed: 0,
        };

        let mut output = Vec::new();
        commit.write_text_without_newline(&mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        assert!(result.contains("empty123"));
        assert!(result.contains(&time.timestamp().to_string()));

        let parts: Vec<&str> = result.split('\t').collect();
        assert_eq!(parts.len(), 13);
        assert_eq!(parts[0], "empty123");
        assert_eq!(parts[1], "");
        assert_eq!(parts[3], "");
    }
}
