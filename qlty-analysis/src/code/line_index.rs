use std::{
    collections::{HashMap, HashSet},
    ops::RangeInclusive,
    path::{Path, PathBuf},
};

type LineNumber = u32;

#[derive(Debug, Default, Clone)]
pub struct FileInfo {
    pub new_file: bool,
    pub line_numbers: HashSet<LineNumber>,
}

impl FileInfo {
    pub fn insert(&mut self, line_number: LineNumber) {
        self.line_numbers.insert(line_number);
    }
}

#[derive(Debug, Default, Clone)]
pub struct FileIndex {
    inner: HashMap<PathBuf, FileInfo>,
}

impl FileIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_line(&mut self, path: &Path, line_number: LineNumber) {
        if !self.inner.contains_key(path) {
            self.inner.insert(
                path.to_path_buf(),
                FileInfo {
                    new_file: false,
                    line_numbers: HashSet::new(),
                },
            );
        }

        self.inner.get_mut(path).unwrap().insert(line_number);
    }

    pub fn insert_file(&mut self, path: &Path) {
        if !self.inner.contains_key(path) {
            self.inner.insert(
                path.to_path_buf(),
                FileInfo {
                    new_file: true,
                    line_numbers: HashSet::new(),
                },
            );
        }
    }

    pub fn matches_path(&self, path: &Path) -> bool {
        self.inner.contains_key(path)
    }

    /// If any of the inputes overlap the index, return true. Otherwise, return false.
    pub fn matches_line_range(
        &self,
        path: &Path,
        mut line_numbers: RangeInclusive<LineNumber>,
    ) -> bool {
        let (start, end) = (*line_numbers.start(), *line_numbers.end());

        // if end < start it is an empty range, so we treat it as a single line range
        // that starts and ends at start.
        if end < start {
            line_numbers = start..=start;
        }

        if let Some(file_info) = self.inner.get(path) {
            if file_info.new_file {
                return true;
            }

            for line_number in line_numbers {
                if file_info.line_numbers.get(&line_number).is_some() {
                    return true;
                }
            }

            false
        } else {
            false
        }
    }
}
