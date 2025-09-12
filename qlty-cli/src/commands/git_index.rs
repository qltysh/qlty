use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use clap::Args;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};

#[derive(Args, Debug)]
pub struct GitIndex {
    /// Skip commits without parents (except the initial commit)
    #[arg(long, default_value = "true")]
    pub skip_commits_without_parents: bool,

    /// Skip commits with duplicate diffs
    #[arg(long, default_value = "true")]
    pub skip_commits_with_duplicate_diffs: bool,

    /// Skip paths that match this regular expression
    #[arg(long)]
    pub skip_paths: Option<String>,

    /// Skip commits whose messages match this regular expression
    #[arg(long)]
    pub skip_commits_with_messages: Option<String>,

    /// Skip commits with diffs larger than this size (number of added + removed lines)
    #[arg(long, default_value = "100000")]
    pub diff_size_limit: usize,

    /// Number of threads to use for processing
    #[arg(long)]
    pub threads: Option<usize>,

    /// Output directory for TSV files
    #[arg(long, default_value = ".")]
    pub output_dir: PathBuf,

    /// Skip specific commits by hash
    #[arg(long)]
    pub skip_commit: Vec<String>,

    /// Stop processing after this commit hash
    #[arg(long)]
    pub stop_after_commit: Option<String>,
}

/// Helper functions for TSV output (inspired by ClickHouse WriteHelpers.h)
mod tsv_writer {
    use std::io::Write;

    /// Write a character to the output
    pub fn write_char(out: &mut impl Write, c: char) -> std::io::Result<()> {
        write!(out, "{}", c)
    }

    /// Write escaped string for TSV format
    /// Escapes: \b, \f, \n, \r, \t, \0, \\
    pub fn write_escaped_string(out: &mut impl Write, s: &str) -> std::io::Result<()> {
        for ch in s.chars() {
            match ch {
                '\t' => write!(out, "\\t")?,
                '\n' => write!(out, "\\n")?,
                '\r' => write!(out, "\\r")?,
                '\\' => write!(out, "\\\\")?,
                '\0' => write!(out, "\\0")?,
                '\u{0008}' => write!(out, "\\b")?, // backspace
                '\u{000C}' => write!(out, "\\f")?, // form feed
                _ => write!(out, "{}", ch)?,
            }
        }
        Ok(())
    }

    /// Write text (for numbers and other non-string values)
    pub fn write_text<T: std::fmt::Display>(out: &mut impl Write, value: T) -> std::io::Result<()> {
        write!(out, "{}", value)
    }
}

#[derive(Debug, Clone)]
struct Commit {
    hash: String,
    author: String,
    time: DateTime<Local>,
    message: String,
    files_added: u32,
    files_deleted: u32,
    files_renamed: u32,
    files_modified: u32,
    lines_added: u32,
    lines_deleted: u32,
    hunks_added: u32,
    hunks_removed: u32,
    hunks_changed: u32,
}

impl Commit {
    fn write_text_without_newline(&self, out: &mut impl Write) -> std::io::Result<()> {
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum FileChangeType {
    Add,
    Delete,
    Modify,
    Rename,
    Copy,
    Type,
}

impl FileChangeType {
    fn as_str(&self) -> &'static str {
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
struct FileChange {
    change_type: FileChangeType,
    path: String,
    old_path: String,
    file_extension: String,
    lines_added: u32,
    lines_deleted: u32,
    hunks_added: u32,
    hunks_removed: u32,
    hunks_changed: u32,
}

impl FileChange {
    fn write_text_without_newline(&self, out: &mut impl Write) -> std::io::Result<()> {
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum LineType {
    Empty,
    Comment,
    Punct,
    Code,
}

impl LineType {
    fn as_str(&self) -> &'static str {
        match self {
            LineType::Empty => "Empty",
            LineType::Comment => "Comment",
            LineType::Punct => "Punct",
            LineType::Code => "Code",
        }
    }
}

#[derive(Debug, Clone)]
struct LineChange {
    sign: i8,
    line_number_old: u32,
    line_number_new: u32,
    hunk_num: u32,
    hunk_start_line_number_old: u32,
    hunk_start_line_number_new: u32,
    hunk_lines_added: u32,
    hunk_lines_deleted: u32,
    hunk_context: String,
    line: String,
    indent: u8,
    line_type: LineType,
    prev_commit_hash: String,
    prev_author: String,
    prev_time: Option<DateTime<Local>>,
}

impl LineChange {
    fn classify_line(full_line: &str) -> (String, u8, LineType) {
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

    fn write_text_without_newline(&self, out: &mut impl Write) -> std::io::Result<()> {
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

#[derive(Debug)]
struct FileDiff {
    file_change: FileChange,
    line_changes: Vec<LineChange>,
}

type CommitDiff = HashMap<String, FileDiff>;

#[derive(Debug, Clone)]
struct FileBlame {
    lines: Vec<Commit>,
}

impl FileBlame {
    fn new() -> Self {
        Self { lines: Vec::new() }
    }

    fn find(&self, line_num: usize) -> Option<&Commit> {
        if line_num > 0 && line_num <= self.lines.len() {
            Some(&self.lines[line_num - 1])
        } else {
            None
        }
    }

    fn add_line(&mut self, line_num: usize, commit: Commit) {
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

    fn remove_line(&mut self, line_num: usize) {
        if line_num > 0 && line_num <= self.lines.len() {
            self.lines.remove(line_num - 1);
        }
    }
}

type Snapshot = HashMap<String, FileBlame>;

/// Result writer for TSV files
struct ResultWriter {
    commits: BufWriter<File>,
    file_changes: BufWriter<File>,
    line_changes: BufWriter<File>,
}

impl ResultWriter {
    fn new(output_dir: &Path) -> Result<Self> {
        Ok(Self {
            commits: BufWriter::new(File::create(output_dir.join("commits.tsv"))?),
            file_changes: BufWriter::new(File::create(output_dir.join("file_changes.tsv"))?),
            line_changes: BufWriter::new(File::create(output_dir.join("line_changes.tsv"))?),
        })
    }

    fn append_commit(&mut self, commit: &Commit, file_changes: &CommitDiff) -> Result<()> {
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

    fn finalize(mut self) -> Result<()> {
        self.commits.flush()?;
        self.file_changes.flush()?;
        self.line_changes.flush()?;
        Ok(())
    }
}

impl GitIndex {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        info!("Starting git-index command");

        let skip_paths_regex = self
            .skip_paths
            .as_ref()
            .map(|pattern| Regex::new(pattern))
            .transpose()
            .context("Invalid skip-paths regex")?;

        let skip_messages_regex = self
            .skip_commits_with_messages
            .as_ref()
            .map(|pattern| Regex::new(pattern))
            .transpose()
            .context("Invalid skip-commits-with-messages regex")?;

        let skip_commits: HashSet<String> = self.skip_commit.iter().cloned().collect();

        let commits = self.get_commit_list()?;
        info!("Found {} commits to process", commits.len());

        let num_threads = self.threads.unwrap_or_else(num_cpus::get);
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .context("Failed to build thread pool")?
            .install(|| {
                self.process_commits(commits, skip_paths_regex, skip_messages_regex, skip_commits)
            })?;

        CommandSuccess::ok()
    }

    fn get_commit_list(&self) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(["log", "--reverse", "--no-merges", "--pretty=%H"])
            .output()
            .context("Failed to execute git log")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "git log failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let commits: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();

        Ok(commits)
    }

    fn process_commits(
        &self,
        commits: Vec<String>,
        skip_paths_regex: Option<Regex>,
        skip_messages_regex: Option<Regex>,
        skip_commits: HashSet<String>,
    ) -> Result<()> {
        let total_commits = commits.len();
        let mut snapshot = Snapshot::new();
        let mut diff_hashes = HashSet::new();
        let mut result_writer = ResultWriter::new(&self.output_dir)?;

        for (commit_num, hash) in commits.iter().enumerate() {
            if skip_commits.contains(hash) {
                continue;
            }

            eprintln!(
                "{}% {} {}",
                commit_num * 100 / total_commits,
                commit_num,
                hash
            );

            match self.process_single_commit(
                hash,
                commit_num,
                &mut snapshot,
                &mut diff_hashes,
                &skip_paths_regex,
                &skip_messages_regex,
                &mut result_writer,
            ) {
                Ok(_) => {}
                Err(e) => {
                    warn!("Failed to process commit {}: {}", hash, e);
                }
            }

            if let Some(ref stop_hash) = self.stop_after_commit {
                if hash == stop_hash {
                    info!("Stopping after commit {}", hash);
                    break;
                }
            }
        }

        result_writer.finalize()?;
        Ok(())
    }

    fn process_single_commit(
        &self,
        hash: &str,
        commit_num: usize,
        snapshot: &mut Snapshot,
        diff_hashes: &mut HashSet<u64>,
        skip_paths_regex: &Option<Regex>,
        skip_messages_regex: &Option<Regex>,
        result_writer: &mut ResultWriter,
    ) -> Result<()> {
        let output = Command::new("git")
            .args([
                "show",
                "--raw",
                "--pretty=format:%ct%x00%aN%x00%P%x00%s%x00",
                "--patch",
                "--unified=0",
                hash,
            ])
            .output()
            .context("Failed to execute git show")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "git show failed for {}: {}",
                hash,
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let content = String::from_utf8_lossy(&output.stdout);
        let mut lines = content.lines();

        let header = lines.next().unwrap_or("");
        let parts: Vec<&str> = header.split('\0').collect();
        if parts.len() < 4 {
            return Err(anyhow::anyhow!(
                "Invalid git show output for commit {}",
                hash
            ));
        }

        let timestamp = parts[0]
            .parse::<i64>()
            .context("Failed to parse timestamp")?;
        let author = parts[1].to_string();
        let parent_hash = parts[2].to_string();
        let message = parts[3].to_string();

        if let Some(ref regex) = skip_messages_regex {
            if regex.is_match(&message) {
                debug!("Skipping commit {} due to message filter", hash);
                return Ok(());
            }
        }

        if self.skip_commits_without_parents && commit_num != 0 && parent_hash.is_empty() {
            debug!("Skipping commit {} without parents", hash);
            return Ok(());
        }

        let mut commit = Commit {
            hash: hash.to_string(),
            author,
            time: Local.timestamp_opt(timestamp, 0).unwrap(),
            message,
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

        let mut commit_diff = CommitDiff::new();

        self.parse_git_show_output(
            lines,
            &mut commit,
            &mut commit_diff,
            skip_paths_regex,
            commit_num,
        )?;

        if self.skip_commits_with_duplicate_diffs {
            let hash = self.calculate_diff_hash(&commit_diff);
            if !diff_hashes.insert(hash) {
                debug!("Skipping commit {} with duplicate diff", commit.hash);
                return Ok(());
            }
        }

        self.update_snapshot(snapshot, &commit, &mut commit_diff)?;
        result_writer.append_commit(&commit, &commit_diff)?;

        Ok(())
    }

    fn parse_git_show_output(
        &self,
        lines: std::str::Lines,
        commit: &mut Commit,
        commit_diff: &mut CommitDiff,
        skip_paths_regex: &Option<Regex>,
        commit_num: usize,
    ) -> Result<()> {
        let mut current_file: Option<String> = None;
        let mut line_change = LineChange {
            sign: 0,
            line_number_old: 0,
            line_number_new: 0,
            hunk_num: 0,
            hunk_start_line_number_old: 0,
            hunk_start_line_number_new: 0,
            hunk_lines_added: 0,
            hunk_lines_deleted: 0,
            hunk_context: String::new(),
            line: String::new(),
            indent: 0,
            line_type: LineType::Empty,
            prev_commit_hash: String::new(),
            prev_author: String::new(),
            prev_time: None,
        };

        let mut diff_size = 0;

        for line in lines {
            if line.starts_with(':') {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    let change_type_char = parts[4].chars().next().unwrap_or('?');
                    let (change_type, path, old_path) = match change_type_char {
                        'A' => {
                            commit.files_added += 1;
                            (
                                FileChangeType::Add,
                                parts.get(5).unwrap_or(&"").to_string(),
                                String::new(),
                            )
                        }
                        'D' => {
                            commit.files_deleted += 1;
                            (
                                FileChangeType::Delete,
                                parts.get(5).unwrap_or(&"").to_string(),
                                String::new(),
                            )
                        }
                        'M' => {
                            commit.files_modified += 1;
                            (
                                FileChangeType::Modify,
                                parts.get(5).unwrap_or(&"").to_string(),
                                String::new(),
                            )
                        }
                        'R' => {
                            commit.files_renamed += 1;
                            let old = parts.get(5).unwrap_or(&"").to_string();
                            let new = parts.get(6).unwrap_or(&"").to_string();
                            (FileChangeType::Rename, new, old)
                        }
                        'C' => {
                            let old = parts.get(5).unwrap_or(&"").to_string();
                            let new = parts.get(6).unwrap_or(&"").to_string();
                            (FileChangeType::Copy, new, old)
                        }
                        'T' => (
                            FileChangeType::Type,
                            parts.get(5).unwrap_or(&"").to_string(),
                            String::new(),
                        ),
                        _ => continue,
                    };

                    if let Some(ref regex) = skip_paths_regex {
                        if regex.is_match(&path) {
                            continue;
                        }
                    }

                    let file_extension = Path::new(&path)
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("")
                        .to_string();

                    let file_change = FileChange {
                        change_type,
                        path: path.clone(),
                        old_path,
                        file_extension,
                        lines_added: 0,
                        lines_deleted: 0,
                        hunks_added: 0,
                        hunks_removed: 0,
                        hunks_changed: 0,
                    };

                    commit_diff.insert(
                        path,
                        FileDiff {
                            file_change,
                            line_changes: Vec::new(),
                        },
                    );
                }
            } else if line.starts_with("--- ") {
                if line.starts_with("--- a/") {
                    current_file = Some(line[6..].to_string());
                } else if line == "--- /dev/null" {
                    current_file = None;
                }
            } else if line.starts_with("+++ ") {
                if line.starts_with("+++ b/") && current_file.is_none() {
                    current_file = Some(line[6..].to_string());
                }
            } else if line.starts_with("@@ ") {
                if let Some(ref file_path) = current_file {
                    if let Some(file_diff) = commit_diff.get_mut(file_path) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 4 {
                            let old_range = parts[1].strip_prefix('-').unwrap_or(parts[1]);
                            let new_range = parts[2].strip_prefix('+').unwrap_or(parts[2]);

                            let (old_start, old_lines) = parse_range(old_range);
                            let (new_start, new_lines) = parse_range(new_range);

                            line_change.hunk_num += 1;
                            line_change.hunk_start_line_number_old = old_start;
                            line_change.hunk_start_line_number_new = new_start.max(1);
                            line_change.hunk_lines_added = new_lines;
                            line_change.hunk_lines_deleted = old_lines;
                            line_change.line_number_old = old_start;
                            line_change.line_number_new = new_start.max(1);

                            if parts.len() > 4 {
                                line_change.hunk_context = parts[4..].join(" ");
                            }

                            if old_lines > 0 && new_lines > 0 {
                                commit.hunks_changed += 1;
                                file_diff.file_change.hunks_changed += 1;
                            } else if old_lines > 0 {
                                commit.hunks_removed += 1;
                                file_diff.file_change.hunks_removed += 1;
                            } else if new_lines > 0 {
                                commit.hunks_added += 1;
                                file_diff.file_change.hunks_added += 1;
                            }
                        }
                    }
                }
            } else if line.starts_with('-') && !line.starts_with("---") {
                diff_size += 1;
                if let Some(ref file_path) = current_file {
                    if let Some(file_diff) = commit_diff.get_mut(file_path) {
                        commit.lines_deleted += 1;
                        file_diff.file_change.lines_deleted += 1;

                        let line_content = &line[1..];
                        let (clean_line, indent, line_type) =
                            LineChange::classify_line(line_content);

                        let mut lc = line_change.clone();
                        lc.sign = -1;
                        lc.line = clean_line;
                        lc.indent = indent;
                        lc.line_type = line_type;

                        file_diff.line_changes.push(lc);
                        line_change.line_number_old += 1;
                    }
                }
            } else if line.starts_with('+') && !line.starts_with("+++") {
                diff_size += 1;
                if let Some(ref file_path) = current_file {
                    if let Some(file_diff) = commit_diff.get_mut(file_path) {
                        commit.lines_added += 1;
                        file_diff.file_change.lines_added += 1;

                        let line_content = &line[1..];
                        let (clean_line, indent, line_type) =
                            LineChange::classify_line(line_content);

                        let mut lc = line_change.clone();
                        lc.sign = 1;
                        lc.line = clean_line;
                        lc.indent = indent;
                        lc.line_type = line_type;

                        file_diff.line_changes.push(lc);
                        line_change.line_number_new += 1;
                    }
                }
            }

            if self.diff_size_limit > 0 && commit_num != 0 && diff_size > self.diff_size_limit {
                debug!(
                    "Skipping commit {} with diff size {} > {}",
                    commit.hash, diff_size, self.diff_size_limit
                );
                return Ok(());
            }
        }

        Ok(())
    }

    fn update_snapshot(
        &self,
        snapshot: &mut Snapshot,
        commit: &Commit,
        file_changes: &mut CommitDiff,
    ) -> Result<()> {
        for (path, file_diff) in file_changes.iter_mut() {
            if !file_diff.file_change.old_path.is_empty() && file_diff.file_change.old_path != *path
            {
                if let Some(old_blame) = snapshot.remove(&file_diff.file_change.old_path) {
                    snapshot.insert(path.clone(), old_blame);
                }
            }

            let file_blame = snapshot.entry(path.clone()).or_insert_with(FileBlame::new);
            let mut deleted_lines: HashMap<u32, Commit> = HashMap::new();

            for line_change in &mut file_diff.line_changes {
                if line_change.sign == -1 {
                    if let Some(prev_commit) = file_blame.find(line_change.line_number_old as usize)
                    {
                        if prev_commit.time <= commit.time {
                            line_change.prev_commit_hash = prev_commit.hash.clone();
                            line_change.prev_author = prev_commit.author.clone();
                            line_change.prev_time = Some(prev_commit.time);
                            deleted_lines.insert(line_change.line_number_old, prev_commit.clone());
                        }
                    }
                } else if line_change.sign == 1 {
                    let this_line_in_prev = line_change.hunk_start_line_number_old
                        + (line_change.line_number_new - line_change.hunk_start_line_number_new);

                    if let Some(prev_commit) = deleted_lines.get(&this_line_in_prev) {
                        if prev_commit.time <= commit.time {
                            line_change.prev_commit_hash = prev_commit.hash.clone();
                            line_change.prev_author = prev_commit.author.clone();
                            line_change.prev_time = Some(prev_commit.time);
                        }
                    }
                }
            }

            for line_change in &file_diff.line_changes {
                if line_change.sign == -1 {
                    file_blame.remove_line(line_change.line_number_new as usize);
                } else if line_change.sign == 1 {
                    file_blame.add_line(line_change.line_number_new as usize, commit.clone());
                }
            }
        }

        Ok(())
    }

    fn calculate_diff_hash(&self, commit_diff: &CommitDiff) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        let mut sorted_files: Vec<_> = commit_diff.iter().collect();
        sorted_files.sort_by_key(|(path, _)| *path);

        for (path, file_diff) in sorted_files {
            path.hash(&mut hasher);
            (file_diff.file_change.change_type as u8).hash(&mut hasher);
            file_diff.file_change.old_path.hash(&mut hasher);

            for line_change in &file_diff.line_changes {
                line_change.sign.hash(&mut hasher);
                line_change.line_number_old.hash(&mut hasher);
                line_change.line_number_new.hash(&mut hasher);
                line_change.line.hash(&mut hasher);
                line_change.indent.hash(&mut hasher);
            }
        }

        hasher.finish()
    }
}

fn parse_range(range: &str) -> (u32, u32) {
    let parts: Vec<&str> = range.split(',').collect();
    let start = parts[0].parse::<u32>().unwrap_or(0);
    let count = if parts.len() > 1 {
        parts[1].parse::<u32>().unwrap_or(1)
    } else {
        1
    };
    (start, count)
}
