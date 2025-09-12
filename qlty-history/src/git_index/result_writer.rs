use crate::git_index::tsv_writer;
use crate::git_index::types::{Commit, CommitDiff};
use anyhow::Result;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Result writer for TSV files
pub struct ResultWriter {
    commits: BufWriter<File>,
    file_changes: BufWriter<File>,
    line_changes: BufWriter<File>,
}

impl ResultWriter {
    pub fn new(output_dir: &Path) -> Result<Self> {
        Ok(Self {
            commits: BufWriter::new(File::create(output_dir.join("commits.tsv"))?),
            file_changes: BufWriter::new(File::create(output_dir.join("file_changes.tsv"))?),
            line_changes: BufWriter::new(File::create(output_dir.join("line_changes.tsv"))?),
        })
    }

    pub fn append_commit(&mut self, commit: &Commit, file_changes: &CommitDiff) -> Result<()> {
        // Write to commits table
        commit.write_text_without_newline(&mut self.commits)?;
        tsv_writer::write_char(&mut self.commits, '\n')?;

        // Write to file_changes table
        for file_diff in file_changes.values() {
            file_diff
                .file_change
                .write_text_without_newline(&mut self.file_changes)?;
            tsv_writer::write_char(&mut self.file_changes, '\t')?;
            commit.write_text_without_newline(&mut self.file_changes)?;
            tsv_writer::write_char(&mut self.file_changes, '\n')?;

            // Write to line_changes table
            for line_change in &file_diff.line_changes {
                line_change.write_text_without_newline(&mut self.line_changes)?;
                tsv_writer::write_char(&mut self.line_changes, '\t')?;
                file_diff
                    .file_change
                    .write_text_without_newline(&mut self.line_changes)?;
                tsv_writer::write_char(&mut self.line_changes, '\t')?;
                commit.write_text_without_newline(&mut self.line_changes)?;
                tsv_writer::write_char(&mut self.line_changes, '\n')?;
            }
        }
        Ok(())
    }

    pub fn finalize(mut self) -> Result<()> {
        self.commits.flush()?;
        self.file_changes.flush()?;
        self.line_changes.flush()?;
        Ok(())
    }
}
