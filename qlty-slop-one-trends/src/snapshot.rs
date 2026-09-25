//! Freezing a run: first-parent history, snapshot selection, tree listing,
//! scope rules, blob preparation, and the manifest. Port of
//! `WeeklyRun.freeze` in slopdetect's `weekly_scan.py`.
//!
//! The current checkout's scope rules govern every historical snapshot, so a
//! rule change never makes an old period look different from a new one.
//! Historical `linguist-*` attributes are resolved at each frozen commit.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

use chrono::{DateTime, FixedOffset, NaiveDate, Offset as _, Timelike as _, Utc};
use chrono_tz::Tz;
use git2::{ObjectType, Oid, Repository, TreeWalkMode, TreeWalkResult};
use qlty_config::version::{GIT_COMMIT_QLTY, LONG_VERSION, QLTY_VERSION};
use qlty_slop_one::content::MeasurementCache;
use qlty_slop_one::jev;
use qlty_slop_one::model::EXTRACTOR_PROFILE;
use qlty_slop_one::scope::{match_git_attribute, Rule, ScopeRules};
use qlty_slop_one::source::{source_text, PreparedSource};
use qlty_slop_one::Model;
use serde_json::{json, Value};
use sha2::{Digest as _, Sha256};

use crate::error::{Error, Result};
use crate::periods::{isoformat, select_snapshots, History, Interval, SelectedPeriod};
use crate::run::{
    ExcludedPath, FileEntry, Manifest, QltyIdentity, RunDir, SkippedPath, Snapshot, Version,
};

/// Source suffixes SlopOne scores, lowercase without the dot.
pub const SUFFIXES: [&str; 32] = [
    "rs", "rb", "py", "php", "js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx", "mtsx", "ctsx",
    "go", "java", "c", "h", "cc", "cpp", "cxx", "hpp", "hh", "hxx", "cs", "ex", "exs", "kt", "kts",
    "scala", "swift", "vb",
];
const REGULAR_MODES: [i32; 2] = [0o100644, 0o100755];
const HEADER_BYTES: usize = 16384;
const GIT_ATTRIBUTES: [&str; 2] = ["linguist-generated", "linguist-vendored"];
const GIT_ATTRIBUTES_FILE: &str = ".gitattributes";
const ATTRIBUTE_READER_DIR: &str = ".attributes";
const ORIGIN_HEAD: &str = "refs/remotes/origin/HEAD";
const NO_CODE: &str = "No code remains after inline-test exclusion";
const TIMESTAMP_POLICY: &str =
    "First-parent committer time, cumulative maximum to preserve ancestry.";
const CHANGE_FORMULA: &str = "prior_group_mass / prior_project_scored_mass * (new_group_mass_weighted_score - prior_group_mass_weighted_score)";
const DEFAULT_SCOPE: &str = "Tracked supported authored source files. Default test-path and inline-test exclusions, generated/dependency/build/documentation exclusions, historical Git attributes, and explicit project rules. Exclusions apply before scoring in every snapshot.";
const DEFAULT_TEST_RULES: &str = "default test-path exclusions";
const DEFAULT_SOURCE_RULES: &str =
    "default generated/dependency/build/documentation exclusions and historical Git attributes";
const DEFAULT_GIT_ATTRIBUTES_POLICY: &str =
    "Each frozen commit, without checkout/global/system overrides.";
const WORKSPACE_GIT_ATTRIBUTES_POLICY: &str =
    "Not applied; qlty workspace exclude_patterns replace the default source rules.";
const REPOSITORY_URL_HINT: &str = "Use --repository-url with a web URL for repository diff links";

/// What a run is frozen with.
#[derive(Clone, Debug)]
pub struct FreezeOptions {
    /// The current checkout, whose scope rules govern every snapshot.
    pub repo: PathBuf,
    pub since: NaiveDate,
    pub interval: Interval,
    pub timezone: Tz,
    /// `origin/HEAD` when configured, otherwise `HEAD`, unless overridden.
    pub git_ref: Option<String>,
    /// Now, unless overridden.
    pub as_of: Option<DateTime<FixedOffset>>,
    /// The repository name from the URL, unless overridden.
    pub project: Option<String>,
    /// The normalized `origin` URL, unless overridden.
    pub repository_url: Option<String>,
    pub cache_directory: PathBuf,
}

/// Freezes the run, or returns the existing manifest when the run is already
/// frozen with the same settings.
pub fn freeze(run: &RunDir, options: &FreezeOptions) -> Result<Manifest> {
    if run.manifest_path().is_file() {
        let manifest = run.load_manifest()?;
        verify_frozen(&manifest, options)?;
        return Ok(manifest);
    }
    let repository = Repository::discover(&options.repo)
        .map_err(|_| Error::NotARepository(options.repo.clone()))?;
    let root = repository
        .workdir()
        .unwrap_or_else(|| repository.path())
        .to_path_buf();
    let root = fs::canonicalize(&root).unwrap_or(root);
    let as_of = options.as_of.unwrap_or_else(now_with_microseconds);
    let git_ref = options
        .git_ref
        .clone()
        .unwrap_or_else(|| default_ref(&repository));
    let head = repository
        .revparse_single(&format!("{git_ref}^{{commit}}"))?
        .id();
    let history = first_parent_history(&repository, head)?;
    let snapshots = select_snapshots(
        &history,
        Some(options.since),
        as_of,
        options.timezone,
        options.interval,
    );
    if snapshots.len() < 2 {
        return Err(Error::TooFewSnapshots);
    }
    let remote_url = match &options.repository_url {
        Some(url) => url.clone(),
        None => origin_url(&repository)?,
    };
    let repository_url = repository_url(&remote_url)?;
    let project = options
        .project
        .clone()
        .unwrap_or_else(|| default_project(&repository_url));
    let rules = ScopeRules::for_repository(&root)?;
    let model = Model::shared()?;
    let measurement_identity = MeasurementCache::try_new(&options.cache_directory)?
        .identity()
        .to_owned();

    fs::create_dir_all(run.root())?;
    let mut listing = Listing::new(&repository, &rules, run);
    let mut snapshots: Vec<Snapshot> = snapshots
        .into_iter()
        .map(|period| listing.list(period))
        .collect::<Result<_>>()?;
    let entries_by_version = listing.finish()?;
    let wanted: HashSet<&str> = snapshots
        .iter()
        .flat_map(|snapshot| snapshot.files.iter().map(|entry| entry.version.as_str()))
        .collect();
    let mut prepared = Prepared::default();
    for (version, entry) in &entries_by_version {
        if wanted.contains(version.as_str()) {
            prepared.prepare(&repository, run, &rules, entry)?;
        }
    }
    for snapshot in &mut snapshots {
        prepared.drop_header_excluded(snapshot);
    }

    let first = history.first().ok_or(Error::TooFewSnapshots)?;
    let boundary = match options.interval {
        Interval::Month => "the first day of each month",
        Interval::Week => "each Monday",
    };
    let manifest = Manifest {
        run_schema_version: 2,
        interval: options.interval,
        repository_path: root.to_string_lossy().into_owned(),
        cache_directory: options.cache_directory.to_string_lossy().into_owned(),
        since: options.since.to_string(),
        git_ref,
        extended_from: None,
        measurement_identity,
        created_at: isoformat(&as_of),
        project,
        repository: repository_url,
        main_commit: head.to_string(),
        first_commit: first.sha().to_owned(),
        first_commit_at: isoformat(&first.committed_at()),
        timezone: options.timezone.name().to_owned(),
        first_parent_commits: history.len(),
        nonmonotonic_commit_dates: history.inversions(),
        timestamp_policy: TIMESTAMP_POLICY.to_owned(),
        snapshot_policy: format!(
            "Last first-parent commit on the selected ref before {boundary} 00:00 {}; current {} ends at frozen as-of time. First snapshot is the baseline.",
            options.timezone.name(),
            options.interval.as_str()
        ),
        change_formula: CHANGE_FORMULA.to_owned(),
        model: serde_json::to_value(model.info())?,
        implementation: json!({"qlty-slop-one": QLTY_VERSION, "git": GIT_COMMIT_QLTY}),
        qlty: QltyIdentity {
            version: LONG_VERSION.to_string(),
            sha256: String::new(),
            executable: None,
            extractor_profile: EXTRACTOR_PROFILE.to_owned(),
        },
        budget_usd: None,
        snapshots,
        versions: prepared.versions,
        scope: scope_description(&rules),
        source_exclusion_policy: json!({
            "id": rules.source_policy(),
            "project_rules": [],
            "git_attributes": if rules.uses_git_attributes() {
                DEFAULT_GIT_ATTRIBUTES_POLICY
            } else {
                WORKSPACE_GIT_ATTRIBUTES_POLICY
            },
        }),
        extra: serde_json::Map::new(),
    };
    manifest.save(&run.manifest_path())?;
    Ok(manifest)
}

/// Freezes the run at `options` even when it was frozen before. The earlier
/// manifest is discarded; scored results stay, since they are keyed by
/// content and remain valid for any freeze.
pub fn refresh(run: &RunDir, options: &FreezeOptions) -> Result<Manifest> {
    if run.manifest_path().is_file() {
        fs::remove_file(run.manifest_path())?;
    }
    freeze(run, options)
}

/// A frozen run cannot silently change its ref, start, timezone, interval,
/// or as-of time; a different setting needs a new run name.
fn verify_frozen(manifest: &Manifest, options: &FreezeOptions) -> Result<()> {
    let mut differences = Vec::new();
    if manifest.interval != options.interval {
        differences.push(format!("interval {}", manifest.interval.as_str()));
    }
    if manifest.since != options.since.to_string() {
        differences.push(format!("since {}", manifest.since));
    }
    if manifest.timezone != options.timezone.name() {
        differences.push(format!("timezone {}", manifest.timezone));
    }
    if options
        .git_ref
        .as_ref()
        .is_some_and(|git_ref| *git_ref != manifest.git_ref)
    {
        differences.push(format!("ref {}", manifest.git_ref));
    }
    if options
        .as_of
        .is_some_and(|as_of| isoformat(&as_of) != manifest.created_at)
    {
        differences.push(format!("as-of {}", manifest.created_at));
    }
    if options
        .project
        .as_ref()
        .is_some_and(|project| *project != manifest.project)
    {
        differences.push(format!("project {}", manifest.project));
    }
    if options
        .repository_url
        .as_ref()
        .is_some_and(|url| repository_url(url).ok().as_ref() != Some(&manifest.repository))
    {
        differences.push(format!("repository {}", manifest.repository));
    }
    if differences.is_empty() {
        Ok(())
    } else {
        Err(Error::Frozen(differences.join(", ")))
    }
}

fn now_with_microseconds() -> DateTime<FixedOffset> {
    let now = Utc::now();
    let micros = now.timestamp_subsec_micros();
    now.with_nanosecond(micros * 1000)
        .unwrap_or(now)
        .fixed_offset()
}

/// `origin/HEAD`'s symbolic target when the remote is configured, else `HEAD`.
fn default_ref(repository: &Repository) -> String {
    repository
        .find_reference(ORIGIN_HEAD)
        .ok()
        .and_then(|reference| reference.symbolic_target().map(str::to_owned))
        .unwrap_or_else(|| "HEAD".to_owned())
}

fn origin_url(repository: &Repository) -> Result<String> {
    let remote = repository
        .find_remote("origin")
        .map_err(|_| Error::Configuration(REPOSITORY_URL_HINT.to_owned()))?;
    remote
        .url()
        .map(str::to_owned)
        .ok_or_else(|| Error::Configuration(REPOSITORY_URL_HINT.to_owned()))
}

/// Walks first parents from `head` and returns the history oldest first.
fn first_parent_history(repository: &Repository, head: Oid) -> Result<History> {
    let mut commits = Vec::new();
    let mut next = Some(head);
    while let Some(oid) = next {
        let commit = repository.find_commit(oid)?;
        commits.push((oid.to_string(), committed_at(commit.time())));
        next = commit.parent_id(0).ok();
    }
    commits.reverse();
    Ok(History::from_commits(commits))
}

fn committed_at(time: git2::Time) -> DateTime<FixedOffset> {
    let offset = FixedOffset::east_opt(time.offset_minutes() * 60).unwrap_or(Utc.fix());
    DateTime::from_timestamp(time.seconds(), 0)
        .unwrap_or_default()
        .with_timezone(&offset)
}

/// Python's `Path(name).suffix`: the last dot-suffix, unless the name starts
/// with it or ends with the dot.
fn path_suffix(path: &str) -> &str {
    let name = path.rsplit('/').next().unwrap_or(path);
    match name.rfind('.') {
        Some(index) if index > 0 && index < name.len() - 1 => &name[index..],
        _ => "",
    }
}

fn is_supported_suffix(suffix: &str) -> bool {
    let lowered = suffix.to_lowercase();
    SUFFIXES.contains(&lowered.trim_start_matches('.'))
}

/// A skipped test path's `reason`, in slopdetect's shape.
fn test_reason(rule: &Rule) -> Value {
    json!({"kind": rule.kind(), "matched": rule.matched()})
}

/// The manifest's `scope` text: slopdetect's when the defaults apply alone,
/// otherwise how the checkout's qlty workspace patterns combine with them.
fn scope_description(rules: &ScopeRules) -> String {
    if !rules.is_configured() {
        return DEFAULT_SCOPE.to_owned();
    }
    format!(
        "Tracked supported source files. The checkout's qlty workspace test_patterns ({} the {DEFAULT_TEST_RULES}) and exclude_patterns ({} the {DEFAULT_SOURCE_RULES}), and inline-test exclusions. Exclusions apply before scoring in every snapshot.",
        layering(rules.keeps_default_tests()),
        layering(rules.keeps_default_sources()),
    )
}

fn layering(keeps_defaults: bool) -> &'static str {
    if keeps_defaults {
        "checked before"
    } else {
        "replacing"
    }
}

/// A source exclusion, in slopdetect's `match_source_exclusion` shape.
fn source_exclusion(rules: &ScopeRules, rule: &Rule) -> Value {
    json!({
        "policy": rules.source_policy_for(rule),
        "category": rule.category(),
        "kind": rule.kind(),
        "matched": rule.matched(),
    })
}

/// Lists each snapshot's tree and applies the path, test, and git-attribute
/// rules. Header rules need the blob and run later.
struct Listing<'a> {
    repository: &'a Repository,
    rules: &'a ScopeRules,
    attribute_reader: AttributeReader,
    entries_by_version: BTreeMap<String, FileEntry>,
}

impl<'a> Listing<'a> {
    fn new(repository: &'a Repository, rules: &'a ScopeRules, run: &RunDir) -> Self {
        Self {
            repository,
            rules,
            attribute_reader: AttributeReader::new(run.root().join(ATTRIBUTE_READER_DIR)),
            entries_by_version: BTreeMap::new(),
        }
    }

    fn list(&mut self, period: SelectedPeriod) -> Result<Snapshot> {
        let commit = self
            .repository
            .find_commit(Oid::from_str(&period.commit)?)?;
        let tree = commit.tree()?;
        let mut files = Vec::new();
        let mut skipped = Vec::new();
        let mut source_excluded = Vec::new();
        let mut other_extensions = BTreeMap::new();
        let mut excluded_modes = BTreeMap::new();
        let mut has_attributes_file = false;
        tree.walk(TreeWalkMode::PreOrder, |directory, entry| {
            if entry.kind() == Some(ObjectType::Tree) {
                return TreeWalkResult::Ok;
            }
            let name = String::from_utf8_lossy(entry.name_bytes());
            let path = format!("{directory}{name}");
            let mode = entry.filemode();
            if entry.kind() != Some(ObjectType::Blob) || !REGULAR_MODES.contains(&mode) {
                *excluded_modes.entry(format!("{mode:06o}")).or_insert(0) += 1;
                return TreeWalkResult::Ok;
            }
            if name == GIT_ATTRIBUTES_FILE {
                has_attributes_file = true;
            }
            let suffix = path_suffix(&path).to_owned();
            if !is_supported_suffix(&suffix) {
                let key = if suffix.is_empty() { "[none]" } else { &suffix };
                *other_extensions.entry(key.to_owned()).or_insert(0) += 1;
                return TreeWalkResult::Ok;
            }
            if let Some(rule) = self.rules.test_exclusion(&path) {
                skipped.push(SkippedPath {
                    path,
                    reason: test_reason(&rule),
                });
                return TreeWalkResult::Ok;
            }
            let blob = entry.id().to_string();
            if let Some(rule) = self.rules.source_exclusion(&path, None) {
                source_excluded.push(ExcludedPath {
                    path,
                    blob: Some(blob),
                    suffix: None,
                    version: None,
                    source_sha256: None,
                    exclusion: source_exclusion(self.rules, &rule),
                });
                return TreeWalkResult::Ok;
            }
            let version = format!("{blob}{suffix}");
            let entry = FileEntry {
                path,
                blob,
                suffix,
                version: version.clone(),
            };
            self.entries_by_version
                .entry(version)
                .or_insert_with(|| entry.clone());
            files.push(entry);
            TreeWalkResult::Ok
        })?;
        if self.rules.uses_git_attributes() && has_attributes_file && !files.is_empty() {
            let paths: Vec<&str> = files.iter().map(|entry| entry.path.as_str()).collect();
            let attributes = self
                .attribute_reader
                .read(self.repository, &period.commit, &paths)?;
            let mut retained = Vec::with_capacity(files.len());
            for entry in files {
                match attribute_rule(attributes.get(entry.path.as_str())) {
                    Some(rule) => source_excluded.push(ExcludedPath {
                        path: entry.path,
                        blob: Some(entry.blob),
                        suffix: Some(entry.suffix),
                        version: Some(entry.version),
                        source_sha256: None,
                        exclusion: source_exclusion(self.rules, &rule),
                    }),
                    None => retained.push(entry),
                }
            }
            files = retained;
        }
        tracing::info!(
            "SNAPSHOT {} {} {} candidate files; {} paths excluded",
            period.bounds.start(),
            &period.commit[..8.min(period.commit.len())],
            files.len(),
            skipped.len() + source_excluded.len()
        );
        Ok(Snapshot {
            bounds: period.bounds,
            as_of: period.as_of,
            partial: period.partial,
            commit: period.commit,
            committed_at: period.committed_at,
            first_parent_index: period.first_parent_index,
            files,
            skipped,
            source_excluded,
            other_extensions,
            excluded_modes,
        })
    }

    /// Releases the attribute reader and returns the first entry seen for
    /// each version.
    fn finish(self) -> Result<BTreeMap<String, FileEntry>> {
        self.attribute_reader.close()?;
        Ok(self.entries_by_version)
    }
}

fn attribute_rule(attributes: Option<&HashMap<String, String>>) -> Option<Rule> {
    let attributes = attributes?;
    GIT_ATTRIBUTES
        .iter()
        .find_map(|key| match_git_attribute(key, attributes.get(*key)?))
}

/// Resolves `linguist-*` attributes at a commit with `git check-attr
/// --source`, through an empty bare repository that borrows the object
/// store, so only attribute files in that tree apply: never the checkout's
/// `.git/info/attributes`, the global file, or the system file.
struct AttributeReader {
    directory: PathBuf,
    objects: Option<PathBuf>,
}

impl AttributeReader {
    fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            objects: None,
        }
    }

    fn read(
        &mut self,
        repository: &Repository,
        commit: &str,
        paths: &[&str],
    ) -> Result<HashMap<String, HashMap<String, String>>> {
        let objects = match &self.objects {
            Some(objects) => objects.clone(),
            None => {
                let objects = self.open(repository)?;
                self.objects = Some(objects.clone());
                objects
            }
        };
        let mut input = Vec::new();
        for path in paths {
            input.extend_from_slice(path.as_bytes());
            input.push(0);
        }
        let mut child = Command::new("git")
            .arg("-C")
            .arg(&self.directory)
            .args([
                "-c",
                "core.attributesFile=/dev/null",
                "check-attr",
                "-z",
                "--stdin",
            ])
            .arg(format!("--source={commit}"))
            .args(GIT_ATTRIBUTES)
            .env("GIT_ATTR_NOSYSTEM", "1")
            .env("GIT_ALTERNATE_OBJECT_DIRECTORIES", &objects)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| Error::GitAttributes(format!("could not run git: {error}")))?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::GitAttributes("git check-attr has no stdin".to_owned()))?;
        let writer = thread::spawn(move || stdin.write_all(&input));
        let output = child.wait_with_output()?;
        writer
            .join()
            .map_err(|_| Error::GitAttributes("could not send paths to git".to_owned()))??;
        if !output.status.success() {
            return Err(Error::GitAttributes(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ));
        }
        let mut values: HashMap<String, HashMap<String, String>> = paths
            .iter()
            .map(|path| ((*path).to_owned(), HashMap::new()))
            .collect();
        let fields: Vec<&[u8]> = output.stdout.split(|&byte| byte == 0).collect();
        for triple in fields.as_chunks::<3>().0 {
            let path = String::from_utf8_lossy(triple[0]).into_owned();
            let key = String::from_utf8_lossy(triple[1]).into_owned();
            let value = String::from_utf8_lossy(triple[2]).into_owned();
            values.entry(path).or_default().insert(key, value);
        }
        Ok(values)
    }

    fn open(&self, repository: &Repository) -> Result<PathBuf> {
        let output = Command::new("git")
            .arg("-C")
            .arg(repository.path())
            .args([
                "rev-parse",
                "--path-format=absolute",
                "--git-path",
                "objects",
            ])
            .output()
            .map_err(|error| Error::GitAttributes(format!("could not run git: {error}")))?;
        if !output.status.success() {
            return Err(Error::GitAttributes(
                "could not locate the Git object store".to_owned(),
            ));
        }
        let objects = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
        if self.directory.exists() {
            fs::remove_dir_all(&self.directory)?;
        }
        Repository::init_bare(&self.directory)?;
        Ok(objects)
    }

    fn close(self) -> Result<()> {
        if self.directory.exists() {
            fs::remove_dir_all(&self.directory)?;
        }
        Ok(())
    }
}

/// Reads each unique blob once, applies the header rule, and prepares the
/// text for measurement.
#[derive(Default)]
struct Prepared {
    versions: BTreeMap<String, Version>,
    header_excluded: HashMap<String, HeaderExclusion>,
}

struct HeaderExclusion {
    source_sha256: String,
    exclusion: Value,
}

impl Prepared {
    fn prepare(
        &mut self,
        repository: &Repository,
        run: &RunDir,
        rules: &ScopeRules,
        entry: &FileEntry,
    ) -> Result<()> {
        let blob = repository.find_blob(Oid::from_str(&entry.blob)?)?;
        let raw = blob.content();
        let source_sha256 = format!("{:x}", Sha256::digest(raw));
        let head = String::from_utf8_lossy(&raw[..raw.len().min(HEADER_BYTES)]);
        if let Some(rule) = rules.source_exclusion(&entry.path, Some(&head)) {
            self.header_excluded.insert(
                entry.version.clone(),
                HeaderExclusion {
                    source_sha256,
                    exclusion: source_exclusion(rules, &rule),
                },
            );
            return Ok(());
        }
        let source_path = run.source_path(&entry.version);
        if let Some(parent) = source_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&source_path, raw)?;
        let mut version = Version {
            suffix: entry.suffix.clone(),
            bytes: raw.len(),
            source_sha256,
            source: Some(format!("sources/{}", entry.version)),
            source_scope: None,
            skipped: None,
            measurement_key: None,
            prepared_bytes: None,
            excerpts: None,
            error: None,
        };
        match prepare_version(run, &source_path, &entry.suffix, &mut version) {
            Ok(()) => {}
            Err(error) => version.error = Some(error.to_string()),
        }
        self.versions.insert(entry.version.clone(), version);
        Ok(())
    }

    /// Moves versions the header rule excluded from `files` to
    /// `source_excluded`, as the Python did after reading the blobs.
    fn drop_header_excluded(&self, snapshot: &mut Snapshot) {
        let mut retained = Vec::with_capacity(snapshot.files.len());
        for entry in snapshot.files.drain(..) {
            match self.header_excluded.get(&entry.version) {
                Some(excluded) => snapshot.source_excluded.push(ExcludedPath {
                    path: entry.path,
                    blob: Some(entry.blob),
                    suffix: Some(entry.suffix),
                    version: Some(entry.version),
                    source_sha256: Some(excluded.source_sha256.clone()),
                    exclusion: excluded.exclusion.clone(),
                }),
                None => retained.push(entry),
            }
        }
        snapshot.files = retained;
    }
}

/// Normalizes and prepares one written source; a failure becomes the
/// version's `error` rather than a frozen run failure.
fn prepare_version(
    run: &RunDir,
    source_path: &Path,
    suffix: &str,
    version: &mut Version,
) -> std::result::Result<(), qlty_slop_one::Error> {
    let text = source_text(source_path)?;
    let prepared = PreparedSource::prepare(&text, suffix, false)?;
    version.source_scope = Some(serde_json::to_value(prepared.info())?);
    if !prepared.has_code() {
        version.skipped = Some(NO_CODE.to_owned());
        return Ok(());
    }
    let key = jev::digest(&json!([suffix, prepared.text()]))?;
    let prepared_path = run.prepared_path(&key, suffix);
    if let Some(parent) = prepared_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if !prepared_path.exists() {
        fs::write(&prepared_path, prepared.text())?;
    }
    version.measurement_key = Some(key);
    version.prepared_bytes = Some(prepared.text().len());
    version.excerpts = Some(jev::chunks(prepared.text()).len());
    Ok(())
}

/// Normalizes GitHub-compatible SSH and HTTPS remotes to a credential-free
/// `https://` URL. Port of `weekly_history.repository_url`.
pub fn repository_url(remote: &str) -> Result<String> {
    let remote = remote.trim();
    let remote = remote.strip_suffix(".git").unwrap_or(remote);
    let remote = remote.trim_end_matches('/');
    if let Some(url) = scp_like_url(remote) {
        return Ok(url);
    }
    let Some((scheme, rest)) = remote.split_once("://") else {
        return Err(Error::Configuration(REPOSITORY_URL_HINT.to_owned()));
    };
    if !matches!(scheme, "http" | "https" | "ssh") {
        return Err(Error::Configuration(REPOSITORY_URL_HINT.to_owned()));
    }
    let (authority, path) = match rest.find('/') {
        Some(index) => rest.split_at(index),
        None => (rest, ""),
    };
    let host_port = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    let (hostname, port) = match host_port.rsplit_once(':') {
        Some((host, port)) if !port.is_empty() => (host, Some(port)),
        _ => (host_port, None),
    };
    if hostname.is_empty() || path.is_empty() {
        return Err(Error::Configuration(REPOSITORY_URL_HINT.to_owned()));
    }
    let port = match port {
        Some(port) if scheme != "ssh" => {
            let number: u16 = port
                .parse()
                .map_err(|_| Error::Configuration(REPOSITORY_URL_HINT.to_owned()))?;
            format!(":{number}")
        }
        _ => String::new(),
    };
    Ok(format!(
        "https://{}{port}{}",
        hostname.to_lowercase(),
        path.trim_end_matches('/')
    ))
}

/// `user@host:path` without a scheme.
fn scp_like_url(remote: &str) -> Option<String> {
    let (user_host, path) = remote.split_once(':')?;
    let (user, host) = user_host.split_once('@')?;
    if user.is_empty() || user.contains('/') || host.is_empty() || host.contains('/') {
        return None;
    }
    Some(format!("https://{host}/{path}"))
}

/// The repository name: the last segment of its URL.
pub fn default_project(repository_url: &str) -> String {
    repository_url
        .rsplit('/')
        .next()
        .unwrap_or(repository_url)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffix_follows_pathlib() {
        assert_eq!(path_suffix("src/lib.rs"), ".rs");
        assert_eq!(path_suffix("a/b.tar.gz"), ".gz");
        assert_eq!(path_suffix(".eslintrc"), "");
        assert_eq!(path_suffix("dir.d/Makefile"), "");
        assert_eq!(path_suffix("trailing."), "");
    }

    #[test]
    fn supported_suffixes_ignore_case() {
        assert!(is_supported_suffix(".TS"));
        assert!(!is_supported_suffix(".json"));
        assert!(!is_supported_suffix(""));
    }

    #[test]
    fn repository_url_normalizes_ssh_scheme_remotes() {
        assert_eq!(
            repository_url("ssh://git@github.com/owner/demo.git").unwrap(),
            "https://github.com/owner/demo"
        );
    }

    #[test]
    fn repository_url_normalizes_scp_like_remotes() {
        assert_eq!(
            repository_url("git@github.com:owner/demo.git").unwrap(),
            "https://github.com/owner/demo"
        );
    }

    #[test]
    fn repository_url_drops_credentials() {
        assert_eq!(
            repository_url("https://secret@github.com/owner/demo.git").unwrap(),
            "https://github.com/owner/demo"
        );
    }

    #[test]
    fn repository_url_keeps_https_ports_and_lowercases_hosts() {
        assert_eq!(
            repository_url("https://GitHub.example:8443/owner/demo/").unwrap(),
            "https://github.example:8443/owner/demo"
        );
    }

    #[test]
    fn repository_url_drops_ssh_ports() {
        assert_eq!(
            repository_url("ssh://git@github.com:22/owner/demo").unwrap(),
            "https://github.com/owner/demo"
        );
    }

    #[test]
    fn repository_url_rejects_local_paths() {
        assert!(matches!(
            repository_url("/srv/git/demo.git").unwrap_err(),
            Error::Configuration(_)
        ));
    }

    #[test]
    fn default_project_is_the_last_url_segment() {
        assert_eq!(default_project("https://github.com/owner/demo"), "demo");
    }

    #[test]
    fn test_reason_has_the_python_shape() {
        let rule = ScopeRules::defaults()
            .test_exclusion("lib/module.test.ts")
            .unwrap();
        assert_eq!(
            test_reason(&rule),
            json!({"kind": "filename", "matched": "test"})
        );
    }

    #[test]
    fn source_exclusion_has_the_python_shape() {
        let rules = ScopeRules::defaults();
        let rule = rules.source_exclusion("docs/prototype.tsx", None).unwrap();
        assert_eq!(
            source_exclusion(&rules, &rule),
            json!({
                "policy": "authored-source-002",
                "category": "documentation",
                "kind": "directory",
                "matched": "docs",
            })
        );
    }
}
