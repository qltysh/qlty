use crate::git_index::types::Commit;
use chrono::{Local, TimeZone};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FileBlame {
    lines: Vec<Commit>,
}

impl Default for FileBlame {
    fn default() -> Self {
        Self::new()
    }
}

impl FileBlame {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn find(&self, line_num: usize) -> Option<&Commit> {
        if line_num > 0 && line_num <= self.lines.len() {
            Some(&self.lines[line_num - 1])
        } else {
            None
        }
    }

    pub fn add_line(&mut self, line_num: usize, commit: Commit) {
        while self.lines.len() < line_num {
            self.lines.push(Commit {
                hash: String::new(),
                author: String::new(),
                time: Local.timestamp_opt(0, 0).unwrap(),
                message: String::new(),
                files_added: 0,
                files_deleted: 0,
                files_renamed: 0,
                files_modified: 0,
                lines_added: 0,
                lines_deleted: 0,
                hunks_added: 0,
                hunks_removed: 0,
                hunks_changed: 0,
            });
        }
        if line_num > 0 {
            if line_num <= self.lines.len() {
                self.lines.insert(line_num - 1, commit);
            } else {
                self.lines.push(commit);
            }
        }
    }

    pub fn remove_line(&mut self, line_num: usize) {
        if line_num > 0 && line_num <= self.lines.len() {
            self.lines.remove(line_num - 1);
        }
    }
}

pub type Snapshot = HashMap<String, FileBlame>;
