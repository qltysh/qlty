//! Group files across two snapshots and attribute the period's quality change.
//!
//! Files are matched by same path, Git renames at 50% similarity, inferred
//! directory renames with at least three Git-confirmed peers, and a
//! conservative exact-line split/merge inference. Each matched group
//! contributes `prior_group_mass / prior_project_mass × (mass-weighted score
//! after − before)`. This is a port of slopdetect's `weekly_report.py`.

use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use qlty_slop_one::fsum::fsum;
use serde_json::{Map, Value};

use crate::error::{Error, Result};
use crate::report::{
    Change, Contribution, FileRow, FileSet, MatchEvidence, Measurement, Period, Summary,
    UncomparedGroup, UnresolvedMovement,
};
use crate::run::RunDir;

const MINIMUM_DISTINCTIVE_LENGTH: usize = 24;
const SKIPPED_LINE_PREFIXES: [&str; 9] = [
    "//", "/*", "*", "#", "use ", "pub use ", "import ", "from ", "mod ",
];
const MINIMUM_UNIQUE_LINES: usize = 12;
const MINIMUM_COVERAGE: f64 = 0.65;
const MINIMUM_DIRECTORY_RENAME_SUPPORT: usize = 3;

/// Aggregate statistics over a set of file rows.
pub fn summary<'a>(files: impl IntoIterator<Item = &'a FileRow>) -> Summary {
    let mut candidate_files = 0;
    let mut errors = 0;
    let mut inline_only_skips = 0;
    let mut scored = Vec::new();
    for row in files {
        candidate_files += 1;
        errors += usize::from(row.error.is_some());
        inline_only_skips += usize::from(row.skipped.is_some());
        if let Some(measurement) = row.measurement() {
            scored.push(measurement);
        }
    }
    let total = fsum(scored.iter().map(|row| row.mass));
    let passing = fsum(scored.iter().filter(|row| row.passed).map(|row| row.mass));
    let loc: u64 = scored.iter().map(|row| row.code_lines).sum();
    let weighted = fsum(scored.iter().map(|row| row.code_lines as f64 * row.score));
    Summary {
        candidate_files,
        files: scored.len(),
        fails: scored.iter().filter(|row| !row.passed).count(),
        errors,
        inline_only_skips,
        loc,
        mass: total,
        maintainable_fraction: (total != 0.0).then(|| passing / total),
        median: median(scored.iter().map(|row| row.score)),
        loc_weighted_mean: (loc != 0).then(|| weighted / loc as f64),
    }
}

/// The median of `values`, as `statistics.median` computes it.
fn median(values: impl IntoIterator<Item = f64>) -> Option<f64> {
    let mut sorted: Vec<f64> = values.into_iter().collect();
    sorted.sort_by(f64::total_cmp);
    let count = sorted.len();
    if count == 0 {
        return None;
    }
    let middle = count / 2;
    if count % 2 == 1 {
        Some(sorted[middle])
    } else {
        Some((sorted[middle - 1] + sorted[middle]) / 2.0)
    }
}

/// A Git-detected rename between two commits, with its `R0nn` status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rename {
    pub old: String,
    pub new: String,
    pub status: String,
}

/// The renames `git diff --name-status --find-renames=50% -l10000` reports
/// between two commits, in Git's output order.
///
/// Git's own rename detection is used so the recorded similarity matches the
/// slopdetect exports byte for byte.
pub fn renamed_paths(repository: &Path, old_commit: &str, new_commit: &str) -> Result<Vec<Rename>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args([
            "diff",
            "--name-status",
            "-z",
            "--find-renames=50%",
            "-l10000",
            old_commit,
            new_commit,
        ])
        .output()?;
    if !output.status.success() {
        return Err(Error::GitCommand(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    parse_name_status(&output.stdout)
}

fn parse_name_status(stdout: &[u8]) -> Result<Vec<Rename>> {
    let parts: Vec<&str> = stdout
        .split(|byte| *byte == 0)
        .map(|part| {
            std::str::from_utf8(part).map_err(|_| {
                Error::Mismatch("git diff reported a path that is not UTF-8".to_owned())
            })
        })
        .collect::<Result<_>>()?;
    let mut renames = Vec::new();
    let mut index = 0;
    while index < parts.len() && !parts[index].is_empty() {
        let status = parts[index];
        if status.starts_with('R') || status.starts_with('C') {
            if status.starts_with('R') {
                let (Some(old), Some(new)) = (parts.get(index + 1), parts.get(index + 2)) else {
                    return Err(Error::Mismatch(
                        "truncated git diff rename record".to_owned(),
                    ));
                };
                renames.push(Rename {
                    old: (*old).to_owned(),
                    new: (*new).to_owned(),
                    status: status.to_owned(),
                });
            }
            index += 3;
        } else {
            index += 2;
        }
    }
    Ok(renames)
}

/// Which snapshot a group node belongs to.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Side {
    Old,
    New,
}

/// One recorded match between an old path and a new path.
#[derive(Clone, Debug)]
struct Link<'a> {
    old: &'a str,
    new: &'a str,
    evidence: MatchEvidence,
}

impl Link<'_> {
    fn kind(&self) -> &str {
        self.evidence
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or("")
    }

    /// The evidence object as exported: `old`, `new`, then the match details.
    fn to_value(&self) -> MatchEvidence {
        let mut evidence = Map::new();
        evidence.insert("old".to_owned(), Value::from(self.old));
        evidence.insert("new".to_owned(), Value::from(self.new));
        evidence.extend(
            self.evidence
                .iter()
                .map(|(key, value)| (key.clone(), value.clone())),
        );
        evidence
    }
}

/// The old and new paths of one matched group, sorted.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GroupMembers<'a> {
    pub old: Vec<&'a str>,
    pub new: Vec<&'a str>,
}

/// Union-find over old and new paths, remembering how each pair was joined.
/// Groups appear in the order their first node was seen.
#[derive(Debug, Default)]
pub struct Groups<'a> {
    nodes: Vec<(Side, &'a str)>,
    index: HashMap<(Side, &'a str), usize>,
    parents: Vec<usize>,
    links: Vec<Link<'a>>,
}

impl<'a> Groups<'a> {
    fn node(&mut self, node: (Side, &'a str)) -> usize {
        if let Some(&index) = self.index.get(&node) {
            return index;
        }
        let index = self.nodes.len();
        self.nodes.push(node);
        self.index.insert(node, index);
        self.parents.push(index);
        index
    }

    fn root(&mut self, node: usize) -> usize {
        let mut root = node;
        while self.parents[root] != root {
            root = self.parents[root];
        }
        let mut current = node;
        while self.parents[current] != root {
            let next = self.parents[current];
            self.parents[current] = root;
            current = next;
        }
        root
    }

    /// Joins an old path with a new path and records the evidence.
    pub fn join(&mut self, old: &'a str, new: &'a str, evidence: MatchEvidence) {
        let old_node = self.node((Side::Old, old));
        let new_node = self.node((Side::New, new));
        let old_root = self.root(old_node);
        let new_root = self.root(new_node);
        self.parents[old_root] = new_root;
        self.links.push(Link { old, new, evidence });
    }

    /// Whether the two paths are in the same group.
    pub fn joined(&mut self, old: &'a str, new: &'a str) -> bool {
        let (Some(&old_node), Some(&new_node)) = (
            self.index.get(&(Side::Old, old)),
            self.index.get(&(Side::New, new)),
        ) else {
            return false;
        };
        self.root(old_node) == self.root(new_node)
    }

    /// Every group's members, in order of first appearance.
    pub fn members(&mut self) -> Vec<GroupMembers<'a>> {
        let mut groups: Vec<GroupMembers<'a>> = Vec::new();
        let mut positions: HashMap<usize, usize> = HashMap::new();
        for index in 0..self.nodes.len() {
            let root = self.root(index);
            let position = *positions.entry(root).or_insert_with(|| {
                groups.push(GroupMembers::default());
                groups.len() - 1
            });
            let (side, path) = self.nodes[index];
            match side {
                Side::Old => groups[position].old.push(path),
                Side::New => groups[position].new.push(path),
            }
        }
        for group in &mut groups {
            group.old.sort_unstable();
            group.new.sort_unstable();
        }
        groups
    }
}

/// The nontrivial lines of a prepared source, whitespace-normalized: at
/// least 24 characters and not a comment, import, or module declaration.
pub fn distinctive_lines(text: &str) -> HashSet<String> {
    let mut lines = HashSet::new();
    for line in python_lines(text) {
        let normalized = line
            .split(is_python_space)
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if normalized.chars().count() < MINIMUM_DISTINCTIVE_LENGTH {
            continue;
        }
        if SKIPPED_LINE_PREFIXES
            .iter()
            .any(|prefix| normalized.starts_with(prefix))
        {
            continue;
        }
        lines.insert(normalized);
    }
    lines
}

/// Splits like `str.splitlines()`: every Unicode line boundary, with
/// `\r\n` as one break and no trailing empty line.
fn python_lines(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((offset, character)) = chars.next() {
        if !is_python_line_break(character) {
            continue;
        }
        lines.push(&text[start..offset]);
        let mut end = offset + character.len_utf8();
        if character == '\r' {
            if let Some((next_offset, '\n')) = chars.peek().copied() {
                chars.next();
                end = next_offset + 1;
            }
        }
        start = end;
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

fn is_python_line_break(character: char) -> bool {
    matches!(
        character,
        '\n' | '\r'
            | '\u{0b}'
            | '\u{0c}'
            | '\u{1c}'
            | '\u{1d}'
            | '\u{1e}'
            | '\u{85}'
            | '\u{2028}'
            | '\u{2029}'
    )
}

/// Whitespace as `str.split()` sees it: Unicode white space plus the ASCII
/// separator controls.
fn is_python_space(character: char) -> bool {
    character.is_whitespace() || matches!(character, '\u{1c}'..='\u{1f}')
}

/// A mass-weighted before/after comparison of one group.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroupContribution {
    pub before_score: f64,
    pub after_score: f64,
    pub before_mass: f64,
    pub after_mass: f64,
    pub previous_mass_share: f64,
    pub score_change: f64,
    pub contribution: f64,
}

/// The group's contribution, or `None` when either side or the denominator
/// has no mass.
pub fn group_contribution(
    before: &[Measurement],
    after: &[Measurement],
    denominator: f64,
) -> Option<GroupContribution> {
    let old_mass = fsum(before.iter().map(|row| row.mass));
    let new_mass = fsum(after.iter().map(|row| row.mass));
    if old_mass == 0.0 || new_mass == 0.0 || denominator == 0.0 {
        return None;
    }
    let old_score = fsum(before.iter().map(|row| row.mass * row.score)) / old_mass;
    let new_score = fsum(after.iter().map(|row| row.mass * row.score)) / new_mass;
    Some(GroupContribution {
        before_score: old_score,
        after_score: new_score,
        before_mass: old_mass,
        after_mass: new_mass,
        previous_mass_share: old_mass / denominator,
        score_change: new_score - old_score,
        contribution: old_mass / denominator * (new_score - old_score),
    })
}

/// Exact-line evidence that `target` was extracted from `source`.
#[derive(Clone, Debug, PartialEq)]
pub struct ExtractionLink<'a> {
    pub source: &'a str,
    pub target: &'a str,
    pub unique_lines: usize,
    pub shared_lines: usize,
    pub target_lines: usize,
    pub overlap: f64,
}

impl ExtractionLink<'_> {
    fn evidence(&self, kind: &str) -> MatchEvidence {
        let mut evidence = Map::new();
        evidence.insert("kind".to_owned(), Value::from(kind));
        evidence.insert("unique_lines".to_owned(), Value::from(self.unique_lines));
        evidence.insert("shared_lines".to_owned(), Value::from(self.shared_lines));
        evidence.insert("target_lines".to_owned(), Value::from(self.target_lines));
        evidence.insert("overlap".to_owned(), Value::from(self.overlap));
        evidence
    }
}

type Signature = Rc<HashSet<String>>;

/// The frozen run and repository a comparison reads from, with a cache of
/// each content's distinctive lines.
pub struct History<'a> {
    run: &'a RunDir,
    repository: PathBuf,
    signatures: RefCell<HashMap<String, Signature>>,
}

impl<'a> History<'a> {
    pub fn new(run: &'a RunDir, repository: impl Into<PathBuf>) -> Self {
        Self {
            run,
            repository: repository.into(),
            signatures: RefCell::new(HashMap::new()),
        }
    }

    /// The distinctive lines of a row's prepared content; empty when the
    /// row was not measured.
    pub fn signatures(&self, row: &FileRow) -> Result<Signature> {
        let Some(key) = row.measurement_key.as_deref().filter(|key| !key.is_empty()) else {
            return Ok(Rc::new(HashSet::new()));
        };
        if let Some(signature) = self.signatures.borrow().get(key) {
            return Ok(Rc::clone(signature));
        }
        let text = fs::read_to_string(self.run.prepared_path(key, &row.suffix))?;
        let signature = Rc::new(distinctive_lines(&text));
        self.signatures
            .borrow_mut()
            .insert(key.to_owned(), Rc::clone(&signature));
        Ok(signature)
    }

    /// Conservative exact-line evidence, recorded for review, not a
    /// semantic proof.
    ///
    /// A target must retain at least 65% of its nontrivial lines from one
    /// source, with at least 12 lines unique to that source among the
    /// candidate sources. Its evidence must be at least twice that for the
    /// next candidate.
    pub fn extraction_links<'p>(
        &self,
        sources: &[(&'p str, &FileRow)],
        targets: &[(&'p str, &FileRow)],
    ) -> Result<Vec<ExtractionLink<'p>>> {
        let source_signatures: HashMap<&'p str, Signature> = sources
            .iter()
            .map(|(name, row)| Ok((*name, self.signatures(row)?)))
            .collect::<Result<_>>()?;
        let mut inverted: HashMap<&str, Vec<&'p str>> = HashMap::new();
        for (name, _) in sources {
            for line in source_signatures[name].iter() {
                inverted.entry(line.as_str()).or_default().push(*name);
            }
        }
        let unique: HashMap<&str, &'p str> = inverted
            .iter()
            .filter(|(_, names)| names.len() == 1)
            .map(|(line, names)| (*line, names[0]))
            .collect();
        let mut links = Vec::new();
        for (target, row) in targets {
            let lines = self.signatures(row)?;
            if lines.len() < MINIMUM_UNIQUE_LINES {
                continue;
            }
            let mut counts: HashMap<&'p str, usize> = HashMap::new();
            for line in lines.iter() {
                if let Some(source) = unique.get(line.as_str()) {
                    *counts.entry(*source).or_default() += 1;
                }
            }
            let Some((source, count)) = counts.iter().max_by_key(|(_, count)| **count) else {
                continue;
            };
            let runner_up = counts
                .iter()
                .filter(|(name, _)| *name != source)
                .map(|(_, count)| *count)
                .max();
            let shared = lines
                .iter()
                .filter(|line| source_signatures[source].contains(line.as_str()))
                .count();
            let overlap = shared as f64 / lines.len() as f64;
            let decisive = runner_up.is_none_or(|second| *count >= 2 * second);
            if *count >= MINIMUM_UNIQUE_LINES && overlap >= MINIMUM_COVERAGE && decisive {
                links.push(ExtractionLink {
                    source,
                    target,
                    unique_lines: *count,
                    shared_lines: shared,
                    target_lines: lines.len(),
                    overlap,
                });
            }
        }
        Ok(links)
    }

    /// The quality change from `before` to `after`.
    ///
    /// With `detect_structure`, directory renames and split/merge movements
    /// are inferred in addition to same paths and Git renames.
    pub fn change(
        &self,
        before: &Period,
        after: &Period,
        detect_structure: bool,
    ) -> Result<Change> {
        let old = rows_by_path(before)?;
        let new = rows_by_path(after)?;
        let renames = renamed_paths(&self.repository, &before.commit, &after.commit)?;
        let mut matching = Matching::new(&old, &new);
        matching.match_same_paths();
        matching.match_git_renames(&renames);
        let mut rejected = Vec::new();
        if detect_structure {
            matching.match_directory_renames(&renames);
            rejected = matching.infer_movements(self)?;
        }
        matching.finish(before.summary.mass, rejected)
    }
}

fn rows_by_path(period: &Period) -> Result<HashMap<&str, &FileRow>> {
    let Some(files) = &period.files else {
        return Err(Error::Mismatch(format!(
            "snapshot {} has no file rows to compare",
            period.commit
        )));
    };
    Ok(files.iter().map(|row| (row.path.as_str(), row)).collect())
}

/// The in-progress matching of one snapshot's files onto the next.
struct Matching<'r> {
    old: &'r HashMap<&'r str, &'r FileRow>,
    new: &'r HashMap<&'r str, &'r FileRow>,
    groups: Groups<'r>,
    old_to_new: HashMap<&'r str, &'r str>,
    new_to_old: HashMap<&'r str, &'r str>,
    used_old: HashSet<&'r str>,
    used_new: HashSet<&'r str>,
}

impl<'r> Matching<'r> {
    fn new(old: &'r HashMap<&'r str, &'r FileRow>, new: &'r HashMap<&'r str, &'r FileRow>) -> Self {
        Self {
            old,
            new,
            groups: Groups::default(),
            old_to_new: HashMap::new(),
            new_to_old: HashMap::new(),
            used_old: HashSet::new(),
            used_new: HashSet::new(),
        }
    }

    fn pair(&mut self, old: &'r str, new: &'r str, evidence: MatchEvidence) {
        self.groups.join(old, new, evidence);
        self.used_old.insert(old);
        self.used_new.insert(new);
        self.old_to_new.insert(old, new);
        self.new_to_old.insert(new, old);
    }

    fn match_same_paths(&mut self) {
        let shared: BTreeSet<&str> = self
            .old
            .keys()
            .filter(|path| self.new.contains_key(*path))
            .copied()
            .collect();
        for path in shared {
            let mut evidence = Map::new();
            evidence.insert("kind".to_owned(), Value::from("same_path"));
            self.pair(path, path, evidence);
        }
    }

    fn match_git_renames(&mut self, renames: &[Rename]) {
        for rename in renames {
            let (Some(old), Some(new)) = (
                self.old.get_key_value(rename.old.as_str()),
                self.new.get_key_value(rename.new.as_str()),
            ) else {
                continue;
            };
            let (old, new) = (*old.0, *new.0);
            if self.used_old.contains(old) || self.used_new.contains(new) {
                continue;
            }
            let mut evidence = Map::new();
            evidence.insert("kind".to_owned(), Value::from("git_rename"));
            evidence.insert("similarity".to_owned(), Value::from(rename.status.as_str()));
            self.pair(old, new, evidence);
        }
    }

    /// Directory/crate renames can also include heavily edited files that
    /// fall below Git's individual-file similarity threshold. Require at
    /// least three Git-confirmed peers and an unambiguous relative path.
    fn match_directory_renames(&mut self, renames: &[Rename]) {
        let votes = directory_rename_votes(renames);
        let mut candidates: Vec<(&'r str, Vec<(&'r str, MatchEvidence)>)> = Vec::new();
        for old in self.unused_old() {
            let mut options: Vec<(&'r str, MatchEvidence)> = Vec::new();
            for ((old_prefix, new_prefix), support) in &votes {
                if *support < MINIMUM_DIRECTORY_RENAME_SUPPORT {
                    continue;
                }
                if !old_prefix.is_empty() && !old.starts_with(&format!("{old_prefix}/")) {
                    continue;
                }
                let relative = if old_prefix.is_empty() {
                    old
                } else {
                    &old[old_prefix.len() + 1..]
                };
                let candidate = if new_prefix.is_empty() {
                    relative.to_owned()
                } else {
                    format!("{new_prefix}/{relative}")
                };
                let Some((&new, _)) = self.new.get_key_value(candidate.as_str()) else {
                    continue;
                };
                if self.used_new.contains(new) {
                    continue;
                }
                let mut evidence = Map::new();
                evidence.insert("kind".to_owned(), Value::from("inferred_directory_rename"));
                evidence.insert("old_prefix".to_owned(), Value::from(old_prefix.as_str()));
                evidence.insert("new_prefix".to_owned(), Value::from(new_prefix.as_str()));
                evidence.insert("supporting_git_renames".to_owned(), Value::from(*support));
                match options.iter_mut().find(|(path, _)| *path == new) {
                    Some(option) => option.1 = evidence,
                    None => options.push((new, evidence)),
                }
            }
            if !options.is_empty() {
                candidates.push((old, options));
            }
        }
        let mut destinations: HashMap<&str, usize> = HashMap::new();
        for (_, options) in &candidates {
            for (new, _) in options {
                *destinations.entry(new).or_default() += 1;
            }
        }
        for (old, options) in candidates {
            let [(new, evidence)] = options.as_slice() else {
                continue;
            };
            if destinations[new] == 1 {
                self.pair(old, new, evidence.clone());
            }
        }
    }

    fn unused_old(&self) -> BTreeSet<&'r str> {
        self.old
            .keys()
            .filter(|path| !self.used_old.contains(*path))
            .copied()
            .collect()
    }

    fn unused_new(&self) -> BTreeSet<&'r str> {
        self.new
            .keys()
            .filter(|path| !self.used_new.contains(*path))
            .copied()
            .collect()
    }

    fn content_changed(&self, old: &str, new: &str) -> bool {
        self.old[old].measurement_key != self.new[new].measurement_key
    }

    /// Infers splits and merges from exact-line evidence, then prunes
    /// mappings whose unmatched endpoints lack majority coverage. Returns
    /// the rejected movements.
    fn infer_movements(&mut self, history: &History) -> Result<Vec<UnresolvedMovement>> {
        let added = self.unused_new();
        let deleted = self.unused_old();
        let old_changed: Vec<(&'r str, &FileRow)> = self
            .old
            .iter()
            .filter(|(path, _)| {
                self.old_to_new
                    .get(*path)
                    .is_none_or(|new| self.content_changed(path, new))
            })
            .map(|(path, row)| (*path, *row))
            .collect();
        let new_changed: Vec<(&'r str, &FileRow)> = self
            .new
            .iter()
            .filter(|(path, _)| {
                self.new_to_old
                    .get(*path)
                    .is_none_or(|old| self.content_changed(old, path))
            })
            .map(|(path, row)| (*path, *row))
            .collect();
        let added_rows: Vec<(&'r str, &FileRow)> =
            added.iter().map(|path| (*path, self.new[path])).collect();
        let deleted_rows: Vec<(&'r str, &FileRow)> =
            deleted.iter().map(|path| (*path, self.old[path])).collect();

        let mut inferred: Vec<Link<'r>> = history
            .extraction_links(&old_changed, &added_rows)?
            .into_iter()
            .map(|link| Link {
                old: link.source,
                new: link.target,
                evidence: link.evidence("inferred_split"),
            })
            .collect();
        inferred.extend(
            history
                .extraction_links(&new_changed, &deleted_rows)?
                .into_iter()
                .map(|link| Link {
                    old: link.target,
                    new: link.source,
                    evidence: link.evidence("inferred_merge"),
                }),
        );

        // Matching a few extracted helpers must not give them the full weight
        // of a deleted large file. Unmatched endpoints need majority coverage
        // on both sides. Prune again when removing an edge changes coverage.
        let mut rejected = Vec::new();
        while !inferred.is_empty() {
            let forward = endpoints(inferred.iter().map(|link| (link.old, link.new)));
            let reverse = endpoints(inferred.iter().map(|link| (link.new, link.old)));
            let mut bad_old: HashSet<&str> = HashSet::new();
            let mut bad_new: HashSet<&str> = HashSet::new();
            for (old, targets) in &forward {
                if self.old_to_new.contains_key(old) {
                    continue;
                }
                let others: Vec<&FileRow> = targets.iter().map(|new| self.new[new]).collect();
                let coverage = history.coverage(self.old[old], &others)?;
                if coverage < MINIMUM_COVERAGE {
                    bad_old.insert(*old);
                    rejected.push(UnresolvedMovement {
                        before_paths: vec![(*old).to_owned()],
                        after_paths: targets.iter().map(|path| (*path).to_owned()).collect(),
                        reason: "Insufficient coverage of deleted source".to_owned(),
                        coverage,
                    });
                }
            }
            for (new, sources) in &reverse {
                if self.new_to_old.contains_key(new) {
                    continue;
                }
                let others: Vec<&FileRow> = sources.iter().map(|old| self.old[old]).collect();
                let coverage = history.coverage(self.new[new], &others)?;
                if coverage < MINIMUM_COVERAGE {
                    bad_new.insert(*new);
                    rejected.push(UnresolvedMovement {
                        before_paths: sources.iter().map(|path| (*path).to_owned()).collect(),
                        after_paths: vec![(*new).to_owned()],
                        reason: "Insufficient coverage of added source".to_owned(),
                        coverage,
                    });
                }
            }
            if bad_old.is_empty() && bad_new.is_empty() {
                break;
            }
            inferred.retain(|link| !bad_old.contains(link.old) && !bad_new.contains(link.new));
        }
        for link in inferred {
            self.groups.join(link.old, link.new, link.evidence);
            self.used_old.insert(link.old);
            self.used_new.insert(link.new);
        }
        Ok(rejected)
    }

    fn finish(mut self, denominator: f64, rejected: Vec<UnresolvedMovement>) -> Result<Change> {
        let mut compared: Vec<Contribution> = Vec::new();
        let mut missing = Vec::new();
        let mut covered = 0.0;
        let members = self.groups.members();
        let movements: Vec<&Link> = self
            .groups
            .links
            .iter()
            .filter(|link| link.kind() != "same_path")
            .collect();
        for group in members {
            let before_rows: Vec<&FileRow> = group.old.iter().map(|path| self.old[path]).collect();
            let after_rows: Vec<&FileRow> = group.new.iter().map(|path| self.new[path]).collect();
            let before_paths: Vec<String> =
                group.old.iter().map(|path| (*path).to_owned()).collect();
            let after_paths: Vec<String> =
                group.new.iter().map(|path| (*path).to_owned()).collect();
            let measured = |rows: &[&FileRow]| -> Option<Vec<Measurement>> {
                rows.iter().map(|row| row.measurement()).collect()
            };
            let (Some(before), Some(after)) = (measured(&before_rows), measured(&after_rows))
            else {
                missing.push(UncomparedGroup {
                    before_paths,
                    after_paths,
                    reason: "At least one member was not scored".to_owned(),
                });
                continue;
            };
            let Some(contribution) = group_contribution(&before, &after, denominator) else {
                missing.push(UncomparedGroup {
                    before_paths,
                    after_paths,
                    reason: "Zero mass prevents a weighted comparison".to_owned(),
                });
                continue;
            };
            covered += contribution.before_mass;
            let matching = movements
                .iter()
                .filter(|link| group.old.contains(&link.old) && group.new.contains(&link.new))
                .map(|link| link.to_value())
                .collect();
            let changed = before_rows.len() != 1
                || after_rows.len() != 1
                || before_rows[0].measurement_key != after_rows[0].measurement_key;
            if !changed && contribution.contribution.abs() >= 1e-12 {
                return Err(Error::Mismatch(format!(
                    "unchanged content {} has a non-zero contribution",
                    before_paths.join(", ")
                )));
            }
            compared.push(Contribution {
                before_paths,
                after_paths,
                before_score: contribution.before_score,
                after_score: contribution.after_score,
                before_mass: contribution.before_mass,
                after_mass: contribution.after_mass,
                previous_mass_share: contribution.previous_mass_share,
                score_change: contribution.score_change,
                contribution: contribution.contribution,
                matching,
                changed,
            });
        }
        let contributions = || compared.iter().map(|record| record.contribution);
        let positives = fsum(contributions().filter(|value| *value > 0.0));
        let negatives = fsum(contributions().filter(|value| *value < 0.0));
        if covered > denominator + 1e-7 {
            return Err(Error::Mismatch(
                "Previous mass was counted more than once".to_owned(),
            ));
        }
        if !is_close(positives + negatives, fsum(contributions()), 1e-12) {
            return Err(Error::Mismatch(
                "positive and negative contributions do not sum to the total".to_owned(),
            ));
        }
        let additions: Vec<&FileRow> = self
            .unused_new()
            .iter()
            .map(|path| self.new[path])
            .collect();
        let removals: Vec<&FileRow> = self
            .unused_old()
            .iter()
            .map(|path| self.old[path])
            .collect();
        let eligible = denominator - fsum(removals.iter().filter_map(|row| row.mass));
        let has_denominator = denominator != 0.0;
        let mut changed_contributions: Vec<Contribution> = compared
            .iter()
            .filter(|record| record.changed)
            .cloned()
            .collect();
        changed_contributions.sort_by(|a, b| b.contribution.abs().total_cmp(&a.contribution.abs()));
        Ok(Change {
            positive: has_denominator.then_some(positives),
            negative: has_denominator.then_some(negatives),
            net: has_denominator.then_some(positives + negatives),
            compared_groups: compared.len(),
            changed_groups: compared.iter().filter(|record| record.changed).count(),
            compared_previous_mass: covered,
            previous_project_mass: denominator,
            comparison_coverage_of_retained_scored_mass: (eligible != 0.0)
                .then(|| covered / eligible),
            contributions: changed_contributions,
            uncompared_groups: missing,
            unresolved_movements: rejected,
            additions: FileSet {
                summary: summary(additions.iter().copied()),
                files: additions.iter().map(|row| row.path.clone()).collect(),
            },
            removals: FileSet {
                summary: summary(removals.iter().copied()),
                files: removals.iter().map(|row| row.path.clone()).collect(),
            },
            structural_groups: compared
                .iter()
                .filter(|record| record.before_paths.len() != 1 || record.after_paths.len() != 1)
                .count(),
            rename_only_sensitivity: None,
        })
    }
}

impl History<'_> {
    /// The share of `row`'s distinctive lines present in any of `others`.
    fn coverage(&self, row: &FileRow, others: &[&FileRow]) -> Result<f64> {
        let lines = self.signatures(row)?;
        if lines.is_empty() {
            return Ok(0.0);
        }
        let represented: Vec<Signature> = others
            .iter()
            .map(|other| self.signatures(other))
            .collect::<Result<_>>()?;
        let shared = lines
            .iter()
            .filter(|line| {
                represented
                    .iter()
                    .any(|other| other.contains(line.as_str()))
            })
            .count();
        Ok(shared as f64 / lines.len() as f64)
    }
}

/// Each source's targets, in order of the source's first appearance, with
/// targets sorted.
fn endpoints<'a>(
    pairs: impl Iterator<Item = (&'a str, &'a str)>,
) -> Vec<(&'a str, BTreeSet<&'a str>)> {
    let mut order: Vec<(&str, BTreeSet<&str>)> = Vec::new();
    let mut positions: HashMap<&str, usize> = HashMap::new();
    for (source, target) in pairs {
        let position = *positions.entry(source).or_insert_with(|| {
            order.push((source, BTreeSet::new()));
            order.len() - 1
        });
        order[position].1.insert(target);
    }
    order
}

/// Counts how many Git renames support each (old prefix, new prefix) pair,
/// in order of first appearance.
fn directory_rename_votes(renames: &[Rename]) -> Vec<((String, String), usize)> {
    let mut votes: Vec<((String, String), usize)> = Vec::new();
    for rename in renames {
        let old_parts: Vec<&str> = rename.old.split('/').collect();
        let new_parts: Vec<&str> = rename.new.split('/').collect();
        let common = old_parts
            .iter()
            .rev()
            .zip(new_parts.iter().rev())
            .take_while(|(a, b)| a == b)
            .count();
        if common == 0 {
            continue;
        }
        let key = (
            old_parts[..old_parts.len() - common].join("/"),
            new_parts[..new_parts.len() - common].join("/"),
        );
        match votes.iter_mut().find(|(existing, _)| *existing == key) {
            Some(vote) => vote.1 += 1,
            None => votes.push((key, 1)),
        }
    }
    votes
}

/// `math.isclose` with the default relative tolerance and an absolute one.
fn is_close(a: f64, b: f64, absolute_tolerance: f64) -> bool {
    (a - b).abs() <= (1e-9 * a.abs().max(b.abs())).max(absolute_tolerance)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measured(score: f64, mass: f64) -> Measurement {
        Measurement {
            score,
            passed: score > 5.0,
            code_lines: 10,
            mass,
        }
    }

    fn row(path: &str, score: Option<f64>, mass: f64, code_lines: u64) -> FileRow {
        FileRow {
            path: path.to_owned(),
            blob: String::new(),
            suffix: ".rs".to_owned(),
            version: String::new(),
            source_sha256: String::new(),
            source_scope: None,
            score,
            score_scale: None,
            passed: score.map(|score| score > 5.0),
            language: None,
            code_lines: score.map(|_| code_lines),
            cyclomatic: None,
            mass: score.map(|_| mass),
            error: score.is_none().then(|| "parse failed".to_owned()),
            measurement_key: None,
            skipped: None,
        }
    }

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    #[test]
    fn summary_counts_scored_and_unscored_rows() {
        let rows = [
            row("a.rs", Some(8.0), 10.0, 100),
            row("b.rs", Some(3.0), 30.0, 300),
            row("c.rs", None, 0.0, 0),
        ];
        let result = summary(&rows);
        assert_eq!(result.candidate_files, 3);
        assert_eq!(result.files, 2);
        assert_eq!(result.fails, 1);
        assert_eq!(result.errors, 1);
        assert_eq!(result.loc, 400);
        assert_eq!(result.mass, 40.0);
        assert_eq!(result.maintainable_fraction, Some(0.25));
        assert_eq!(result.median, Some(5.5));
        assert_eq!(result.loc_weighted_mean, Some(4.25));
    }

    #[test]
    fn summary_of_no_scored_rows_has_no_averages() {
        let rows = [row("c.rs", None, 0.0, 0)];
        let result = summary(&rows);
        assert_eq!(result.files, 0);
        assert_eq!(result.maintainable_fraction, None);
        assert_eq!(result.median, None);
        assert_eq!(result.loc_weighted_mean, None);
    }

    #[test]
    fn median_of_odd_count_is_the_middle_value() {
        assert_eq!(median([3.0, 1.0, 2.0]), Some(2.0));
    }

    #[test]
    fn median_of_even_count_averages_the_middle_values() {
        assert_eq!(median([4.0, 1.0, 3.0, 2.0]), Some(2.5));
    }

    #[test]
    fn group_contribution_weights_by_prior_mass() {
        let a = group_contribution(&[measured(4.0, 10.0)], &[measured(6.0, 20.0)], 100.0).unwrap();
        let b = group_contribution(&[measured(8.0, 5.0)], &[measured(6.0, 2.0)], 100.0).unwrap();
        assert!(approx(a.contribution, 0.2));
        assert!(approx(b.contribution, -0.1));
    }

    #[test]
    fn unchanged_split_contributes_nothing() {
        let split = group_contribution(
            &[measured(4.0, 10.0)],
            &[measured(4.0, 3.0), measured(4.0, 4.0)],
            100.0,
        )
        .unwrap();
        assert_eq!(split.contribution, 0.0);
    }

    #[test]
    fn improved_split_contributes_its_prior_share() {
        let split = group_contribution(
            &[measured(4.0, 10.0)],
            &[measured(6.0, 3.0), measured(6.0, 4.0)],
            100.0,
        )
        .unwrap();
        assert!(approx(split.contribution, 0.2));
    }

    #[test]
    fn zero_mass_has_no_contribution() {
        assert_eq!(
            group_contribution(&[measured(4.0, 0.0)], &[measured(6.0, 0.0)], 100.0),
            None
        );
    }

    #[test]
    fn distinctive_lines_normalize_whitespace_and_skip_short_lines() {
        let text = "  let   value = compute_something(argument);  \nshort line\n";
        let lines = distinctive_lines(text);
        assert_eq!(
            lines,
            HashSet::from(["let value = compute_something(argument);".to_owned()])
        );
    }

    #[test]
    fn distinctive_lines_skip_comments_and_imports() {
        let text = "// a comment that is long enough to count\nimport { something } from 'somewhere';\nuse std::collections::HashMap as Map;\n# a shell style comment line here\nconst value = something_meaningful_here();";
        let lines = distinctive_lines(text);
        assert_eq!(
            lines,
            HashSet::from(["const value = something_meaningful_here();".to_owned()])
        );
    }

    #[test]
    fn distinctive_lines_count_characters_not_bytes() {
        let text = "ééééééééééééééééééééééé\n";
        assert!(distinctive_lines(text).is_empty());
        let text = "éééééééééééééééééééééééé\n";
        assert_eq!(distinctive_lines(text).len(), 1);
    }

    #[test]
    fn python_lines_split_on_form_feeds_and_carriage_returns() {
        assert_eq!(python_lines("a\r\nb\rc\x0cd\n"), vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn python_lines_keep_a_trailing_unterminated_line() {
        assert_eq!(python_lines("a\nb"), vec!["a", "b"]);
    }

    #[test]
    fn groups_join_transitively() {
        let mut groups = Groups::default();
        let same = || {
            let mut evidence = Map::new();
            evidence.insert("kind".to_owned(), Value::from("same_path"));
            evidence
        };
        groups.join("a", "x", same());
        groups.join("b", "y", same());
        groups.join("a", "y", same());
        assert!(groups.joined("a", "y"));
        assert!(groups.joined("b", "x"));
        groups.join("c", "z", same());
        assert!(!groups.joined("c", "x"));
    }

    #[test]
    fn group_members_are_sorted_and_ordered_by_first_appearance() {
        let mut groups = Groups::default();
        groups.join("m", "m", Map::new());
        groups.join("b", "y", Map::new());
        groups.join("a", "y", Map::new());
        assert_eq!(
            groups.members(),
            vec![
                GroupMembers {
                    old: vec!["m"],
                    new: vec!["m"]
                },
                GroupMembers {
                    old: vec!["a", "b"],
                    new: vec!["y"]
                },
            ]
        );
    }

    #[test]
    fn directory_votes_count_shared_trailing_components() {
        let renames = [
            Rename {
                old: "old/a/f.rs".to_owned(),
                new: "new/a/f.rs".to_owned(),
                status: "R090".to_owned(),
            },
            Rename {
                old: "old/g.rs".to_owned(),
                new: "new/g.rs".to_owned(),
                status: "R090".to_owned(),
            },
            Rename {
                old: "x/h.rs".to_owned(),
                new: "y/i.rs".to_owned(),
                status: "R060".to_owned(),
            },
        ];
        assert_eq!(
            directory_rename_votes(&renames),
            vec![(("old".to_owned(), "new".to_owned()), 2)]
        );
    }

    #[test]
    fn name_status_parsing_keeps_renames_only() {
        let stdout = b"A\0added.rs\0R087\0old.rs\0new.rs\0M\0changed.rs\0C075\0src.rs\0copy.rs\0D\0gone.rs\0";
        assert_eq!(
            parse_name_status(stdout).unwrap(),
            vec![Rename {
                old: "old.rs".to_owned(),
                new: "new.rs".to_owned(),
                status: "R087".to_owned(),
            }]
        );
    }

    struct TemporaryHistory {
        directory: tempfile::TempDir,
        commits: Vec<String>,
    }

    impl TemporaryHistory {
        /// A run directory beside a repository with `commits` snapshots, each
        /// renaming `first.rs` on from the one before with a small edit.
        fn new(snapshots: usize) -> Self {
            let directory = tempfile::tempdir().unwrap();
            let repository = directory.path().join("repository");
            fs::create_dir_all(&repository).unwrap();
            let git = |arguments: &[&str]| {
                let output = Command::new("git")
                    .arg("-C")
                    .arg(&repository)
                    .args([
                        "-c",
                        "user.name=Tests",
                        "-c",
                        "user.email=tests@example.com",
                        "-c",
                        "commit.gpgsign=false",
                    ])
                    .args(arguments)
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                String::from_utf8(output.stdout).unwrap().trim().to_owned()
            };
            git(&["init", "-q"]);
            let body: String = (0..40)
                .map(|index| format!("let value_{index} = compute_the_thing(argument_{index});\n"))
                .collect();
            let mut commits = Vec::new();
            for index in 0..snapshots {
                let name = format!("file_{index}.rs");
                if index > 0 {
                    fs::remove_file(repository.join(format!("file_{}.rs", index - 1))).unwrap();
                }
                fs::write(
                    repository.join(&name),
                    format!("{body}// revision {index}\n"),
                )
                .unwrap();
                git(&["add", "-A"]);
                git(&["commit", "-q", "-m", &format!("snapshot {index}")]);
                commits.push(git(&["rev-parse", "HEAD"]));
            }
            Self { directory, commits }
        }

        fn repository(&self) -> PathBuf {
            self.directory.path().join("repository")
        }

        fn run(&self) -> RunDir {
            RunDir::new(self.directory.path().join("run"))
        }

        fn prepare(&self, key: &str, lines: &[String]) {
            let path = self.run().prepared_path(key, ".rs");
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, lines.join("\n")).unwrap();
        }
    }

    fn line(index: usize) -> String {
        format!("let value_{index} = compute_the_thing(argument_{index});")
    }

    fn prepared_row(path: &str, key: &str, score: f64, mass: f64) -> FileRow {
        FileRow {
            measurement_key: Some(key.to_owned()),
            ..row(path, Some(score), mass, 10)
        }
    }

    fn period(commit: &str, files: Vec<FileRow>) -> Period {
        Period {
            bounds: crate::run::PeriodBounds::Week {
                week_start: "2026-01-05".to_owned(),
                week_end_exclusive: "2026-01-12".to_owned(),
            },
            as_of: "2026-01-12".to_owned(),
            partial: false,
            commit: commit.to_owned(),
            committed_at: "2026-01-11".to_owned(),
            first_parent_index: 0,
            summary: summary(&files),
            files: Some(files),
            test_paths_excluded: 0,
            test_exclusions: None,
            source_exclusions: None,
            analysis_errors: Vec::new(),
            change: None,
        }
    }

    #[test]
    fn git_renames_are_read_from_the_command_line() {
        let history = TemporaryHistory::new(2);
        let renames = renamed_paths(
            &history.repository(),
            &history.commits[0],
            &history.commits[1],
        )
        .unwrap();
        assert_eq!(renames.len(), 1);
        assert_eq!(renames[0].old, "file_0.rs");
        assert_eq!(renames[0].new, "file_1.rs");
        assert!(renames[0].status.starts_with("R0"));
    }

    #[test]
    fn git_renames_become_matches_with_their_similarity() {
        let history = TemporaryHistory::new(2);
        let run = history.run();
        let comparer = History::new(&run, history.repository());
        let before = period(
            &history.commits[0],
            vec![prepared_row("file_0.rs", "k0", 4.0, 10.0)],
        );
        let after = period(
            &history.commits[1],
            vec![prepared_row("file_1.rs", "k1", 6.0, 10.0)],
        );
        let change = comparer.change(&before, &after, false).unwrap();
        assert_eq!(change.contributions.len(), 1);
        assert_eq!(change.contributions[0].before_paths, vec!["file_0.rs"]);
        assert_eq!(change.contributions[0].after_paths, vec!["file_1.rs"]);
        assert_eq!(change.contributions[0].matching[0]["kind"], "git_rename");
        assert!(change.contributions[0].matching[0]["similarity"]
            .as_str()
            .unwrap()
            .starts_with("R0"));
        assert!(change.additions.files.is_empty());
        assert!(change.removals.files.is_empty());
    }

    #[test]
    fn splits_with_majority_coverage_are_inferred() {
        let history = TemporaryHistory::new(2);
        let big: Vec<String> = (0..40).map(line).collect();
        let part_one: Vec<String> = big[..20]
            .iter()
            .cloned()
            .chain((100..105).map(line))
            .collect();
        let part_two: Vec<String> = big[20..35]
            .iter()
            .cloned()
            .chain((200..205).map(line))
            .collect();
        history.prepare("big", &big);
        history.prepare("one", &part_one);
        history.prepare("two", &part_two);
        let run = history.run();
        let comparer = History::new(&run, history.repository());
        let before = period(
            &history.commits[0],
            vec![prepared_row("big.rs", "big", 4.0, 100.0)],
        );
        let after = period(
            &history.commits[1],
            vec![
                prepared_row("one.rs", "one", 6.0, 50.0),
                prepared_row("two.rs", "two", 6.0, 40.0),
            ],
        );
        let change = comparer.change(&before, &after, true).unwrap();
        assert_eq!(change.contributions.len(), 1);
        assert_eq!(
            change.contributions[0].after_paths,
            vec!["one.rs", "two.rs"]
        );
        assert_eq!(change.contributions[0].matching.len(), 2);
        assert_eq!(
            change.contributions[0].matching[0]["kind"],
            "inferred_split"
        );
        assert_eq!(change.contributions[0].matching[0]["unique_lines"], 20);
        assert_eq!(change.structural_groups, 1);
        assert!(change.unresolved_movements.is_empty());
    }

    #[test]
    fn partial_extractions_are_pruned_and_reported() {
        let history = TemporaryHistory::new(2);
        let source: Vec<String> = (0..40).map(line).collect();
        let helper: Vec<String> = source[..15]
            .iter()
            .cloned()
            .chain((100..105).map(line))
            .collect();
        history.prepare("source", &source);
        history.prepare("helper", &helper);
        let run = history.run();
        let comparer = History::new(&run, history.repository());
        let before = period(
            &history.commits[0],
            vec![prepared_row("source.rs", "source", 4.0, 100.0)],
        );
        let after = period(
            &history.commits[1],
            vec![prepared_row("helper.rs", "helper", 6.0, 20.0)],
        );
        let change = comparer.change(&before, &after, true).unwrap();
        assert!(change.contributions.is_empty());
        assert_eq!(change.unresolved_movements.len(), 1);
        assert_eq!(
            change.unresolved_movements[0].before_paths,
            vec!["source.rs"]
        );
        assert_eq!(
            change.unresolved_movements[0].after_paths,
            vec!["helper.rs"]
        );
        assert_eq!(
            change.unresolved_movements[0].reason,
            "Insufficient coverage of deleted source"
        );
        assert_eq!(change.unresolved_movements[0].coverage, 0.375);
        assert_eq!(change.additions.files, vec!["helper.rs"]);
        assert_eq!(change.removals.files, vec!["source.rs"]);
    }

    #[test]
    fn pruning_repeats_when_a_removed_edge_lowers_coverage() {
        let history = TemporaryHistory::new(2);
        let source: Vec<String> = (0..40).map(line).collect();
        let good: Vec<String> = source[..12]
            .iter()
            .cloned()
            .chain((100..106).map(line))
            .collect();
        let absorber: Vec<String> = source[12..]
            .iter()
            .cloned()
            .chain((200..216).map(line))
            .collect();
        history.prepare("source", &source);
        history.prepare("good", &good);
        history.prepare("absorber", &absorber);
        let run = history.run();
        let comparer = History::new(&run, history.repository());
        let before = period(
            &history.commits[0],
            vec![prepared_row("source.rs", "source", 4.0, 100.0)],
        );
        let after = period(
            &history.commits[1],
            vec![
                prepared_row("absorber.rs", "absorber", 6.0, 40.0),
                prepared_row("good.rs", "good", 6.0, 20.0),
            ],
        );
        let change = comparer.change(&before, &after, true).unwrap();
        assert!(change.contributions.is_empty());
        let reasons: Vec<&str> = change
            .unresolved_movements
            .iter()
            .map(|movement| movement.reason.as_str())
            .collect();
        assert_eq!(
            reasons,
            vec![
                "Insufficient coverage of added source",
                "Insufficient coverage of deleted source"
            ]
        );
        assert_eq!(
            change.unresolved_movements[0].after_paths,
            vec!["absorber.rs"]
        );
        assert_eq!(change.unresolved_movements[1].after_paths, vec!["good.rs"]);
        assert_eq!(change.removals.files, vec!["source.rs"]);
    }

    #[test]
    fn is_close_uses_relative_and_absolute_tolerance() {
        assert!(is_close(1.0, 1.0 + 1e-13, 1e-12));
        assert!(is_close(1e6, 1e6 + 1e-4, 1e-12));
        assert!(!is_close(0.0, 1e-9, 1e-12));
    }
}
