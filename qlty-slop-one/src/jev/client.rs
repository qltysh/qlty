//! The Jev client: exact request identity, disk cache, credential check,
//! size guard, the port's own retry loop, budget accounting, validation, and
//! byte-weighted aggregation over a file's excerpts.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::Duration;

use lithos_llm::catalog::Catalog;
use lithos_llm::credentials::ConventionalCredentials;
use lithos_llm::types::{ErrorKind, State};
use lithos_llm::{Client, Evaluation};
use rayon::prelude::*;
use serde::Serialize;
use serde_json::{json, Map, Value};
use tokio::runtime::Runtime;

use super::budget::Budget;
use super::cache::Cache;
use super::chunk::chunks;
use super::response::{validate, Answers};
use super::{canonical, Excerpt, JevFeatures, JevProvider, Usage, JEV_MODEL};
use crate::error::{Error, Result};
use crate::features::{questions, QuestionKind};
use crate::fsum::fsum;
use crate::language::Language;

/// slopdetect's conservative ceiling on the request body, in UTF-8 bytes.
const MAX_REQUEST_BYTES: usize = 50000;

const ATTEMPTS: u32 = 4;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(90);

/// The longest provider-requested retry delay the port waits for, in seconds.
const MAX_RETRY_DELAY_SECONDS: f64 = 60.0;

/// HTTP statuses slopdetect retries with backoff.
const UNAVAILABLE_STATUSES: [u16; 6] = [429, 500, 502, 503, 504, 529];

impl JevProvider {
    fn model(self) -> &'static str {
        match self {
            Self::TypeSafe => "typesafe/jev-1.13.0",
            Self::Vercel => "vercel/jev",
        }
    }

    /// The environment variable holding the bearer key.
    pub fn credential(self) -> &'static str {
        match self {
            Self::TypeSafe => "TYPESAFE_API_KEY",
            Self::Vercel => "AI_GATEWAY_API_KEY",
        }
    }

    fn catalog_id(self) -> &'static str {
        match self {
            Self::TypeSafe => "typesafe",
            Self::Vercel => "vercel",
        }
    }

    /// Where the raw body reports input tokens.
    fn usage_pointer(self) -> &'static str {
        match self {
            Self::TypeSafe => "/usage/input_tokens",
            Self::Vercel => "/usage/inputTokens",
        }
    }
}

/// Whether the excerpt is the whole file or one of several.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
enum Scope {
    #[serde(rename = "complete file")]
    CompleteFile,
    #[serde(rename = "consecutive file excerpt")]
    ConsecutiveFileExcerpt,
}

/// The `state` of one Jev request, exactly as slopdetect builds it.
#[derive(Clone, Debug, Serialize)]
pub struct RequestState {
    language: Language,
    source: String,
    scope: Scope,
    excerpt_index: usize,
    excerpt_count: usize,
}

impl RequestState {
    pub fn new(
        language: Language,
        source: &str,
        excerpt_index: usize,
        excerpt_count: usize,
    ) -> Self {
        let scope = if excerpt_count == 1 {
            Scope::CompleteFile
        } else {
            Scope::ConsecutiveFileExcerpt
        };
        Self {
            language,
            source: source.to_owned(),
            scope,
            excerpt_index,
            excerpt_count,
        }
    }

    /// The wire body `{model, state, questions}`.
    fn body(&self) -> Result<Value> {
        let state = serde_json::to_value(self)
            .map_err(|_| Error::Jev("Jev request state is not serializable.".to_owned()))?;
        let mut question_map = Map::new();
        for question in questions() {
            let mut entry = Map::new();
            entry.insert("type".to_owned(), json!(question.kind));
            entry.insert("instructions".to_owned(), json!(question.instructions));
            if question.kind == QuestionKind::Score {
                entry.insert("criteria".to_owned(), json!(question.criteria));
            }
            question_map.insert(question.id.clone(), Value::Object(entry));
        }
        Ok(json!({ "model": JEV_MODEL, "state": state, "questions": question_map }))
    }

    fn evaluation(&self, provider: JevProvider) -> Result<Evaluation> {
        let state = serde_json::to_value(self)
            .map_err(|_| Error::Jev("Jev request state is not serializable.".to_owned()))?;
        let mut builder = Evaluation::builder()
            .model(provider.model())
            .state(State::Json(state))
            .timeout(REQUEST_TIMEOUT);
        for question in questions() {
            builder = match question.kind {
                QuestionKind::Noul => {
                    builder.boolean(question.id.as_str(), question.instructions.as_str())
                }
                QuestionKind::Score => builder.score(
                    question.id.as_str(),
                    question.instructions.as_str(),
                    question.criteria.iter().map(String::as_str),
                ),
            };
        }
        builder
            .build()
            .map_err(|error| Error::Jev(format!("Cannot build the Jev request: {error}")))
    }
}

/// The cache key of the request for `state`: slopdetect's `digest(body)`.
pub fn request_key(state: &RequestState) -> Result<String> {
    canonical::digest(&state.body()?)
}

/// A blocking Jev client for one invocation. Owns its Tokio runtime.
pub struct Jev {
    cache: Cache,
    budget: Mutex<Budget>,
    provider: JevProvider,
    offline: bool,
    runtime: Runtime,
    client: Client,
}

impl Jev {
    /// A client over `<cache_dir>/jev` with a per-invocation budget in
    /// dollars. With `offline`, every request must be cached.
    pub fn try_new(
        cache_dir: PathBuf,
        budget_usd: f64,
        provider: JevProvider,
        offline: bool,
    ) -> Result<Self> {
        Self::build(&cache_dir, budget_usd, provider, offline, None)
    }

    /// [`try_new`](Self::try_new) against another endpoint, such as a local
    /// stand-in for the provider.
    pub fn try_new_with_endpoint(
        cache_dir: PathBuf,
        budget_usd: f64,
        provider: JevProvider,
        offline: bool,
        base_url: &str,
    ) -> Result<Self> {
        Self::build(&cache_dir, budget_usd, provider, offline, Some(base_url))
    }

    fn build(
        cache_dir: &Path,
        budget_usd: f64,
        provider: JevProvider,
        offline: bool,
        base_url: Option<&str>,
    ) -> Result<Self> {
        let budget = Budget::try_new(budget_usd)?;
        // Both `ring` and `aws-lc-rs` are linked, so rustls needs an explicit
        // process-level provider before the first TLS handshake.
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let client = Self::client(provider, base_url)?;
        Ok(Self {
            cache: Cache::new(cache_dir, provider),
            budget: Mutex::new(budget),
            provider,
            offline,
            runtime,
            client,
        })
    }

    fn client(provider: JevProvider, base_url: Option<&str>) -> Result<Client> {
        let mut catalog = Catalog::builder().with_builtin();
        if let Some(base_url) = base_url {
            let overlay = toml::to_string(&json!({
                "providers": { provider.catalog_id(): { "base_url": base_url } }
            }))
            .map_err(|error| Error::Jev(format!("Cannot build the Jev catalog: {error}")))?;
            catalog = catalog
                .overlay_toml(&overlay)
                .map_err(|error| Error::Jev(format!("Cannot build the Jev catalog: {error}")))?;
        }
        let catalog = catalog
            .build()
            .map_err(|error| Error::Jev(format!("Cannot build the Jev catalog: {error}")))?;
        let build = Client::builder()
            .catalog(catalog)
            .credentials(ConventionalCredentials::new())
            .enabled_providers([
                JevProvider::TypeSafe.catalog_id(),
                JevProvider::Vercel.catalog_id(),
            ])
            .build()
            .map_err(|error| Error::Jev(format!("Cannot build the Jev client: {error}")))?;
        Ok(build.client)
    }

    pub fn usage(&self) -> Usage {
        self.budget().summary()
    }

    fn budget(&self) -> MutexGuard<'_, Budget> {
        self.budget.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The validated answers for one excerpt, from the cache when the exact
    /// request was answered before.
    pub fn request(&self, state: &RequestState) -> Result<Answers> {
        let body = state.body()?;
        let key = canonical::digest(&body)?;
        if let Some(cached) = self.cache.read(&key)? {
            return validate(&cached, self.provider);
        }
        if self.offline {
            return Err(Error::Offline);
        }
        let credential = self.provider.credential();
        if std::env::var_os(credential).is_none_or(|value| value.is_empty()) {
            return Err(Error::MissingCredential(credential));
        }
        if canonical::utf8_length(&body)? > MAX_REQUEST_BYTES {
            return Err(Error::Jev(
                "Jev request exceeds the conservative size limit.".to_owned(),
            ));
        }
        let evaluation = state.evaluation(self.provider)?;
        let raw = self.send(&evaluation)?;
        let answers = validate(&raw, self.provider)?;
        self.cache.write(&key, &raw)?;
        Ok(answers)
    }

    /// The raw provider body, after up to four attempts. Every attempt
    /// reserves the maximum request cost; only a confirmed success replaces
    /// it with the reported usage.
    fn send(&self, evaluation: &Evaluation) -> Result<Value> {
        for attempt in 0..ATTEMPTS {
            let reservation = self.budget().reserve()?;
            let outcome = self
                .runtime
                .block_on(self.client.evaluate(evaluation.clone()));
            match outcome {
                Ok(verdict) => {
                    let raw = verdict.raw.ok_or_else(|| {
                        Error::Jev("Jev returned an invalid response.".to_owned())
                    })?;
                    self.budget()
                        .complete(reservation, raw.pointer(self.provider.usage_pointer()))?;
                    return Ok(raw);
                }
                Err(error) => wait_before_retry(attempt, &error)?,
            }
        }
        Err(Error::Jev("Jev retry limit reached.".to_owned()))
    }

    /// The seven byte-weighted means over the file's excerpts.
    ///
    /// Excerpts are requested concurrently on the current rayon pool, so a
    /// large file is not a serial chain of round trips. Answers keep excerpt
    /// order, and every attempt still reserves budget on its own.
    pub fn extract(&self, text: &str, language: Language) -> Result<JevFeatures> {
        let pieces = chunks(text);
        let answers = pieces
            .par_iter()
            .enumerate()
            .map(|(index, piece)| {
                let state = RequestState::new(language, piece, index, pieces.len());
                self.request(&state)
            })
            .collect::<Result<Vec<_>>>()?;
        aggregate(&pieces, &answers)
    }
}

/// How one failed attempt is handled.
enum Failure {
    /// The request never completed; retry after exponential backoff.
    Transport,
    /// The provider is throttling or failing; retry after the longer of the
    /// backoff and the provider's advised delay.
    Unavailable,
    /// The provider refused the request; do not retry.
    Rejected,
}

impl Failure {
    fn classify(error: &lithos_llm::Error) -> Self {
        let status_unavailable = error
            .status()
            .is_some_and(|status| UNAVAILABLE_STATUSES.contains(&status));
        match error.kind() {
            ErrorKind::Network | ErrorKind::Timeout => Self::Transport,
            ErrorKind::RateLimit | ErrorKind::Server => Self::Unavailable,
            _ if status_unavailable => Self::Unavailable,
            _ => Self::Rejected,
        }
    }
}

/// Sleeps before the next attempt, or fails when slopdetect would.
fn wait_before_retry(attempt: u32, error: &lithos_llm::Error) -> Result<()> {
    let last = attempt + 1 == ATTEMPTS;
    let backoff = f64::from(2u32.pow(attempt));
    match Failure::classify(error) {
        Failure::Transport => {
            if last {
                return Err(Error::Jev(
                    "Jev request failed after four attempts.".to_owned(),
                ));
            }
            thread::sleep(Duration::from_secs_f64(backoff));
            Ok(())
        }
        Failure::Unavailable => {
            if last {
                return Err(Error::Jev(format!(
                    "Jev is unavailable: {}",
                    http_label(error)
                )));
            }
            let advised = error
                .provider_retry_after()
                .map_or(0.0, |delay| delay.as_secs_f64());
            let delay = backoff.max(advised);
            if !delay.is_finite() || delay > MAX_RETRY_DELAY_SECONDS {
                return Err(Error::Jev(
                    "Jev requested a long retry delay; try again later.".to_owned(),
                ));
            }
            thread::sleep(Duration::from_secs_f64(delay));
            Ok(())
        }
        Failure::Rejected => Err(Error::Jev(format!(
            "Jev rejected the request: {}",
            http_label(error)
        ))),
    }
}

/// `HTTP <status>` when the failure came from a response, else the client's
/// own message. Provider bodies are never included, so a credential echoed
/// by a provider cannot reach an error.
fn http_label(error: &lithos_llm::Error) -> String {
    match error.status() {
        Some(status) => format!("HTTP {status}"),
        None => error.message().to_owned(),
    }
}

/// The byte-weighted means over consecutive excerpts and their answers,
/// with slopdetect's line spans. `pieces` are the excerpts of one file in
/// order, as [`chunks`] returns them.
pub fn aggregate(pieces: &[&str], answers: &[Answers]) -> Result<JevFeatures> {
    if pieces.is_empty() || pieces.len() != answers.len() {
        return Err(Error::Jev(
            "Jev answers do not match the excerpts.".to_owned(),
        ));
    }
    let mut assessments: BTreeMap<String, Vec<Excerpt>> = questions()
        .iter()
        .map(|question| (question.id.clone(), Vec::with_capacity(pieces.len())))
        .collect();
    let mut line: u32 = 1;
    let mut total_bytes: u64 = 0;
    for (piece, answer) in pieces.iter().zip(answers) {
        let weight_bytes = (piece.len() as u64).max(1);
        let newlines = piece.bytes().filter(|byte| *byte == b'\n').count() as u32;
        let last_line = line + newlines - u32::from(piece.ends_with('\n'));
        let end_line = line.max(last_line);
        for (question, value) in questions().iter().zip(answer.values()) {
            let excerpts = assessments
                .get_mut(&question.id)
                .ok_or_else(|| Error::Jev("Jev answers do not match the questions.".to_owned()))?;
            excerpts.push(Excerpt {
                start_line: line,
                end_line,
                value: *value,
                weight_bytes,
                source_ranges: None,
            });
        }
        line += newlines;
        total_bytes += weight_bytes;
    }
    let means: BTreeMap<String, f64> = assessments
        .iter()
        .map(|(id, excerpts)| {
            let weighted = excerpts
                .iter()
                .map(|excerpt| excerpt.value * excerpt.weight_bytes as f64);
            (id.clone(), fsum(weighted) / total_bytes as f64)
        })
        .collect();
    let values = questions()
        .iter()
        .map(|question| means[&question.id])
        .collect();
    Ok(JevFeatures {
        values,
        means,
        assessments,
        model: JEV_MODEL.to_owned(),
        excerpts: pieces.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn answers(value: f64) -> Answers {
        Answers::try_new(vec![value; 7]).unwrap()
    }

    #[test]
    fn aggregates_by_byte_weight_with_line_spans() {
        let features = aggregate(&["aa\n", "bbbb\n"], &[answers(0.0), answers(1.0)]).unwrap();
        assert_eq!(features.values, vec![5.0 / 8.0; 7]);
        let spans: Vec<(u32, u32)> = features.assessments["boilerplate"]
            .iter()
            .map(|excerpt| (excerpt.start_line, excerpt.end_line))
            .collect();
        assert_eq!(spans, [(1, 1), (2, 2)]);
        assert_eq!(features.excerpts, 2);
        assert_eq!(features.model, "jev-1.13.0");
    }

    #[test]
    fn an_excerpt_without_a_trailing_newline_ends_on_its_last_line() {
        let features = aggregate(&["a\nb\nc"], &[answers(0.5)]).unwrap();
        let excerpt = &features.assessments["control_flow"][0];
        assert_eq!((excerpt.start_line, excerpt.end_line), (1, 3));
        assert_eq!(excerpt.weight_bytes, 5);
    }

    #[test]
    fn an_empty_excerpt_weighs_one_byte() {
        let features = aggregate(&[""], &[answers(0.25)]).unwrap();
        assert_eq!(features.assessments["boilerplate"][0].weight_bytes, 1);
        assert_eq!(features.values, vec![0.25; 7]);
    }

    #[test]
    fn a_second_excerpt_starts_after_the_first_ones_newlines() {
        let features = aggregate(&["x\ny", "z\n"], &[answers(0.0), answers(0.0)]).unwrap();
        let spans: Vec<(u32, u32)> = features.assessments["boilerplate"]
            .iter()
            .map(|excerpt| (excerpt.start_line, excerpt.end_line))
            .collect();
        assert_eq!(spans, [(1, 2), (2, 2)]);
    }

    #[test]
    fn rejects_mismatched_answer_counts() {
        assert!(aggregate(&["a"], &[]).is_err());
    }

    #[test]
    fn request_body_matches_slopdetects_shape() {
        let state = RequestState::new(Language::Python, "x = 1\n", 0, 1);
        let body = state.body().unwrap();
        assert_eq!(body["model"], "jev-1.13.0");
        assert_eq!(body["state"]["language"], "Python");
        assert_eq!(body["state"]["scope"], "complete file");
        assert_eq!(body["state"]["excerpt_index"], 0);
        assert_eq!(body["state"]["excerpt_count"], 1);
        assert_eq!(body["questions"]["boilerplate"]["type"], "noul");
        assert!(body["questions"]["boilerplate"].get("criteria").is_none());
        assert_eq!(
            body["questions"]["control_flow"]["criteria"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
    }

    #[test]
    fn several_excerpts_are_consecutive_file_excerpts() {
        let state = RequestState::new(Language::Java, "class A {}\n", 1, 3);
        assert_eq!(
            state.body().unwrap()["state"]["scope"],
            "consecutive file excerpt"
        );
    }

    #[test]
    fn languages_use_their_display_names() {
        let state = RequestState::new(Language::CSharp, "", 0, 1);
        assert_eq!(state.body().unwrap()["state"]["language"], "C_Sharp");
    }
}
