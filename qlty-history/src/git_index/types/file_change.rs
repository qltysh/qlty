use crate::git_index::tsv_writer;
use crate::git_index::types::LineChange;
use std::collections::HashMap;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileChangeType {
    Add,
    Delete,
    Modify,
    Rename,
    Copy,
    Type,
}

impl FileChangeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileChangeType::Add => "Add",
            FileChangeType::Delete => "Delete",
            FileChangeType::Modify => "Modify",
            FileChangeType::Rename => "Rename",
            FileChangeType::Copy => "Copy",
            FileChangeType::Type => "Type",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileChange {
    pub change_type: FileChangeType,
    pub path: String,
    pub old_path: String,
    pub file_extension: String,
    pub lines_added: u32,
    pub lines_deleted: u32,
    pub hunks_added: u32,
    pub hunks_removed: u32,
    pub hunks_changed: u32,
}

impl FileChange {
    pub fn write_text_without_newline(&self, out: &mut impl Write) -> std::io::Result<()> {
        tsv_writer::write_text(out, self.change_type.as_str())?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_escaped_string(out, &self.path)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_escaped_string(out, &self.old_path)?;
        tsv_writer::write_char(out, '\t')?;
        tsv_writer::write_escaped_string(out, &self.file_extension)?;
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

#[derive(Debug)]
pub struct FileDiff {
    pub file_change: FileChange,
    pub line_changes: Vec<LineChange>,
}

pub type CommitDiff = HashMap<String, FileDiff>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_change_type_as_str() {
        assert_eq!(FileChangeType::Add.as_str(), "Add");
        assert_eq!(FileChangeType::Delete.as_str(), "Delete");
        assert_eq!(FileChangeType::Modify.as_str(), "Modify");
        assert_eq!(FileChangeType::Rename.as_str(), "Rename");
        assert_eq!(FileChangeType::Copy.as_str(), "Copy");
        assert_eq!(FileChangeType::Type.as_str(), "Type");
    }

    #[test]
    fn test_file_change_write_text_without_newline() {
        let file_change = FileChange {
            change_type: FileChangeType::Add,
            path: "src/main.rs".to_string(),
            old_path: "".to_string(),
            file_extension: "rs".to_string(),
            lines_added: 100,
            lines_deleted: 0,
            hunks_added: 5,
            hunks_removed: 0,
            hunks_changed: 0,
        };

        let mut output = Vec::new();
        file_change.write_text_without_newline(&mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        assert!(result.starts_with("Add\t"));
        assert!(result.contains("src/main.rs"));
        assert!(result.contains("rs"));
        assert!(result.contains("100"));
        assert!(result.contains("\t0\t"));
        assert!(result.contains("\t5\t"));
    }

    #[test]
    fn test_file_change_write_rename() {
        let file_change = FileChange {
            change_type: FileChangeType::Rename,
            path: "src/new_name.rs".to_string(),
            old_path: "src/old_name.rs".to_string(),
            file_extension: "rs".to_string(),
            lines_added: 10,
            lines_deleted: 5,
            hunks_added: 1,
            hunks_removed: 1,
            hunks_changed: 2,
        };

        let mut output = Vec::new();
        file_change.write_text_without_newline(&mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        assert!(result.starts_with("Rename\t"));
        assert!(result.contains("src/new_name.rs"));
        assert!(result.contains("src/old_name.rs"));
    }

    #[test]
    fn test_file_change_write_with_special_chars() {
        let file_change = FileChange {
            change_type: FileChangeType::Modify,
            path: "path/with\ttab.txt".to_string(),
            old_path: "path/with\nnewline.txt".to_string(),
            file_extension: "txt".to_string(),
            lines_added: 1,
            lines_deleted: 1,
            hunks_added: 0,
            hunks_removed: 0,
            hunks_changed: 1,
        };

        let mut output = Vec::new();
        file_change.write_text_without_newline(&mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        assert!(result.contains("path/with\\ttab.txt"));
        assert!(result.contains("path/with\\nnewline.txt"));
    }
}
