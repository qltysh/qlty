pub mod blame;
pub mod options;
pub mod result_writer;
pub mod tsv_writer;
pub mod types;

use anyhow::{Context, Result};
use blame::Snapshot;
use chrono::{Local, TimeZone};
use options::GitIndexOptions;
use regex::Regex;
use result_writer::ResultWriter;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use tracing::{debug, info, warn};
use types::{Commit, CommitDiff, FileChange, FileChangeType, FileDiff, LineChange, LineType};

pub use options::GitIndexOptions as GitIndexOptionsExport;

pub struct GitIndex {
    options: GitIndexOptions,
}

struct ProcessContext<'a> {
    snapshot: &'a mut Snapshot,
    diff_hashes: &'a mut HashSet<u64>,
    skip_paths_regex: &'a Option<Regex>,
    skip_messages_regex: &'a Option<Regex>,
    result_writer: &'a mut ResultWriter,
}

impl GitIndex {
    pub fn new(options: GitIndexOptions) -> Self {
        Self { options }
    }

    pub fn run(&self) -> Result<()> {
        info!("Starting git-index processing");

        let (skip_paths_regex, skip_messages_regex) = self.options.compile_regexes()?;
        let skip_commits = self.options.get_skip_commits_set();

        let commits = self.get_commit_list()?;
        info!("Found {} commits to process", commits.len());

        let num_threads = self.options.threads.unwrap_or_else(num_cpus::get);
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .context("Failed to build thread pool")?
            .install(|| {
                self.process_commits(commits, skip_paths_regex, skip_messages_regex, skip_commits)
            })?;

        Ok(())
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
        let mut result_writer = ResultWriter::new(&self.options.output_dir)?;

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

            let mut context = ProcessContext {
                snapshot: &mut snapshot,
                diff_hashes: &mut diff_hashes,
                skip_paths_regex: &skip_paths_regex,
                skip_messages_regex: &skip_messages_regex,
                result_writer: &mut result_writer,
            };

            match self.process_single_commit(hash, commit_num, &mut context) {
                Ok(_) => {}
                Err(e) => {
                    warn!("Failed to process commit {}: {}", hash, e);
                }
            }

            if let Some(ref stop_hash) = self.options.stop_after_commit {
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
        context: &mut ProcessContext,
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

        if let Some(ref regex) = context.skip_messages_regex {
            if regex.is_match(&message) {
                debug!("Skipping commit {} due to message filter", hash);
                return Ok(());
            }
        }

        if self.options.skip_commits_without_parents && commit_num != 0 && parent_hash.is_empty() {
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
            context.skip_paths_regex,
            commit_num,
        )?;

        if self.options.skip_commits_with_duplicate_diffs {
            let hash = self.calculate_diff_hash(&commit_diff);
            if !context.diff_hashes.insert(hash) {
                debug!("Skipping commit {} with duplicate diff", commit.hash);
                return Ok(());
            }
        }

        self.update_snapshot(context.snapshot, &commit, &mut commit_diff)?;
        context.result_writer.append_commit(&commit, &commit_diff)?;

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
                if let Some(path) = line.strip_prefix("--- a/") {
                    current_file = Some(path.to_string());
                } else if line == "--- /dev/null" {
                    current_file = None;
                }
            } else if line.starts_with("+++ ") {
                if current_file.is_none() {
                    if let Some(path) = line.strip_prefix("+++ b/") {
                        current_file = Some(path.to_string());
                    }
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

            if self.options.diff_size_limit > 0
                && commit_num != 0
                && diff_size > self.options.diff_size_limit
            {
                debug!(
                    "Skipping commit {} with diff size {} > {}",
                    commit.hash, diff_size, self.options.diff_size_limit
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

            let file_blame = snapshot.entry(path.clone()).or_default();
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
