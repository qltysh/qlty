//! Parity of the Jev port with slopdetect: request identity, response
//! validation, chunked aggregation, the offline cache path, client error
//! paths, and (when `SLOPDETECT_DIR` is set) the 304-file benchmark replay.

use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;

use qlty_slop_one::jev::{
    aggregate, chunks, digest, request_key, utf8_length, validate, Answers, Jev, JevProvider,
    RequestState,
};
use qlty_slop_one::language::Language;
use qlty_slop_one::Error;
use serde::Deserialize;
use serde_json::Value;

const QUESTION_IDS: [&str; 7] = [
    "direct_unmaintainable",
    "boilerplate",
    "deep_reasoning",
    "control_flow",
    "duplicated_logic",
    "mirrored_state",
    "hand_enumerated_data",
];

/// Serializes the tests that set or clear credential variables.
static ENVIRONMENT: Mutex<()> = Mutex::new(());

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/jev")
        .join(name);
    fs::read_to_string(path).unwrap()
}

#[derive(Deserialize)]
struct DigestCase {
    body: Value,
    digest: String,
    size: usize,
}

#[derive(Deserialize)]
struct ResponseCase {
    response: Value,
    values: BTreeMap<String, f64>,
}

#[derive(Deserialize)]
struct ExpectedExcerpt {
    start_line: u32,
    end_line: u32,
    value: f64,
    weight_bytes: u64,
}

#[derive(Deserialize)]
struct Expected {
    values: Vec<f64>,
    means: BTreeMap<String, f64>,
    assessments: BTreeMap<String, Vec<ExpectedExcerpt>>,
    excerpts: usize,
    model: String,
}

#[derive(Deserialize)]
struct ExtractCase {
    source: String,
    language: String,
    #[serde(default)]
    chunk_byte_lengths: Vec<usize>,
    chunk_keys: Vec<String>,
    responses: Vec<Value>,
    expected: Expected,
}

fn language(name: &str) -> Language {
    match name {
        "Python" => Language::Python,
        "Rust" => Language::Rust,
        "Java" => Language::Java,
        other => panic!("fixture language {other} is not mapped"),
    }
}

fn bits(values: &[f64]) -> Vec<u64> {
    values.iter().map(|value| value.to_bits()).collect()
}

fn expected_values(case: &ResponseCase) -> Vec<f64> {
    QUESTION_IDS.iter().map(|id| case.values[*id]).collect()
}

fn seed_cache(cache_dir: &Path, keys: &[String], responses: &[Value]) {
    let dir = cache_dir.join("jev");
    fs::create_dir_all(&dir).unwrap();
    for (index, key) in keys.iter().enumerate() {
        let response = &responses[index % responses.len()];
        let document = serde_json::json!({ "key": key, "response": response });
        fs::write(dir.join(format!("{key}.json")), document.to_string()).unwrap();
    }
}

fn replay(case: &ExtractCase) -> qlty_slop_one::jev::JevFeatures {
    let pieces = chunks(&case.source);
    let answers: Vec<Answers> = pieces
        .iter()
        .enumerate()
        .map(|(index, _)| {
            validate(
                &case.responses[index % case.responses.len()],
                JevProvider::TypeSafe,
            )
            .unwrap()
        })
        .collect();
    aggregate(&pieces, &answers).unwrap()
}

#[test]
fn digests_match_python_json_dumps() {
    let cases: Vec<DigestCase> = serde_json::from_str(&fixture("digests.json")).unwrap();
    let actual: Vec<String> = cases
        .iter()
        .map(|case| digest(&case.body).unwrap())
        .collect();
    let expected: Vec<String> = cases.iter().map(|case| case.digest.clone()).collect();
    assert_eq!(actual, expected);
}

#[test]
fn request_sizes_match_python_json_dumps_without_ascii_escaping() {
    let cases: Vec<DigestCase> = serde_json::from_str(&fixture("digests.json")).unwrap();
    let actual: Vec<usize> = cases
        .iter()
        .map(|case| utf8_length(&case.body).unwrap())
        .collect();
    let expected: Vec<usize> = cases.iter().map(|case| case.size).collect();
    assert_eq!(actual, expected);
}

#[test]
fn validated_values_match_python_validate() {
    let cases: Vec<ResponseCase> = serde_json::from_str(&fixture("responses.json")).unwrap();
    let actual: Vec<Vec<u64>> = cases
        .iter()
        .map(|case| {
            bits(
                validate(&case.response, JevProvider::TypeSafe)
                    .unwrap()
                    .values(),
            )
        })
        .collect();
    let expected: Vec<Vec<u64>> = cases
        .iter()
        .map(|case| bits(&expected_values(case)))
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn chunked_source_splits_at_the_python_boundaries() {
    let case: ExtractCase = serde_json::from_str(&fixture("chunked.json")).unwrap();
    let lengths: Vec<usize> = chunks(&case.source)
        .iter()
        .map(|piece| piece.len())
        .collect();
    assert_eq!(lengths, case.chunk_byte_lengths);
}

#[test]
fn chunked_request_keys_match_python_digests() {
    let case: ExtractCase = serde_json::from_str(&fixture("chunked.json")).unwrap();
    let pieces = chunks(&case.source);
    let keys: Vec<String> = pieces
        .iter()
        .enumerate()
        .map(|(index, piece)| {
            request_key(&RequestState::new(
                language(&case.language),
                piece,
                index,
                pieces.len(),
            ))
            .unwrap()
        })
        .collect();
    assert_eq!(keys, case.chunk_keys);
}

#[test]
fn chunked_means_match_python_extract() {
    let case: ExtractCase = serde_json::from_str(&fixture("chunked.json")).unwrap();
    let features = replay(&case);
    assert_eq!(bits(&features.values), bits(&case.expected.values));
    assert_eq!(features.excerpts, case.expected.excerpts);
    assert_eq!(features.model, case.expected.model);
    let means: Vec<u64> = QUESTION_IDS
        .iter()
        .map(|id| features.means[*id].to_bits())
        .collect();
    let expected: Vec<u64> = QUESTION_IDS
        .iter()
        .map(|id| case.expected.means[*id].to_bits())
        .collect();
    assert_eq!(means, expected);
}

#[test]
fn chunked_excerpts_match_python_line_spans_and_weights() {
    let case: ExtractCase = serde_json::from_str(&fixture("chunked.json")).unwrap();
    let features = replay(&case);
    let actual: Vec<(u32, u32, u64, u64)> = features.assessments["mirrored_state"]
        .iter()
        .map(|excerpt| {
            (
                excerpt.start_line,
                excerpt.end_line,
                excerpt.weight_bytes,
                excerpt.value.to_bits(),
            )
        })
        .collect();
    let expected: Vec<(u32, u32, u64, u64)> = case.expected.assessments["mirrored_state"]
        .iter()
        .map(|excerpt| {
            (
                excerpt.start_line,
                excerpt.end_line,
                excerpt.weight_bytes,
                excerpt.value.to_bits(),
            )
        })
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn single_excerpt_matches_python_extract() {
    let case: ExtractCase = serde_json::from_str(&fixture("single.json")).unwrap();
    let pieces = chunks(&case.source);
    let key = request_key(&RequestState::new(
        language(&case.language),
        pieces[0],
        0,
        1,
    ))
    .unwrap();
    assert_eq!(vec![key], case.chunk_keys);
    let features = replay(&case);
    assert_eq!(bits(&features.values), bits(&case.expected.values));
    let excerpt = &features.assessments["boilerplate"][0];
    let expected = &case.expected.assessments["boilerplate"][0];
    assert_eq!(
        (excerpt.start_line, excerpt.end_line),
        (expected.start_line, expected.end_line)
    );
}

#[test]
fn offline_extract_replays_a_seeded_slopdetect_cache() {
    let case: ExtractCase = serde_json::from_str(&fixture("chunked.json")).unwrap();
    let dir = tempfile::tempdir().unwrap();
    seed_cache(dir.path(), &case.chunk_keys, &case.responses);
    let jev = Jev::try_new(dir.path().to_path_buf(), 1.0, JevProvider::TypeSafe, true).unwrap();
    let features = jev.extract(&case.source, language(&case.language)).unwrap();
    assert_eq!(bits(&features.values), bits(&case.expected.values));
    assert_eq!(jev.usage().requests, 0);
}

#[test]
fn offline_cache_miss_is_an_offline_error() {
    let dir = tempfile::tempdir().unwrap();
    let jev = Jev::try_new(dir.path().to_path_buf(), 1.0, JevProvider::TypeSafe, true).unwrap();
    let outcome = jev.extract("uncached\n", Language::Python);
    assert!(matches!(outcome, Err(Error::Offline)));
}

#[test]
fn missing_credential_is_reported_by_variable_name() {
    let _guard = ENVIRONMENT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::remove_var("TYPESAFE_API_KEY");
    let dir = tempfile::tempdir().unwrap();
    let jev = Jev::try_new(dir.path().to_path_buf(), 1.0, JevProvider::TypeSafe, false).unwrap();
    let outcome = jev.request(&RequestState::new(Language::Python, "x = 1\n", 0, 1));
    assert!(matches!(
        outcome,
        Err(Error::MissingCredential("TYPESAFE_API_KEY"))
    ));
}

#[test]
fn oversized_request_is_refused_before_sending() {
    let _guard = ENVIRONMENT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::set_var("TYPESAFE_API_KEY", "test-only-key");
    let dir = tempfile::tempdir().unwrap();
    let jev = Jev::try_new(dir.path().to_path_buf(), 1.0, JevProvider::TypeSafe, false).unwrap();
    let source = "x".repeat(60000);
    let outcome = jev.request(&RequestState::new(Language::Python, &source, 0, 1));
    assert_eq!(
        outcome.unwrap_err().to_string(),
        "Jev request exceeds the conservative size limit."
    );
    assert_eq!(jev.usage().requests, 0);
}

/// Answers one request with a 401 whose body echoes the credential.
fn rejecting_server(echo: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = Vec::new();
        let mut buffer = [0u8; 4096];
        loop {
            let read = stream.read(&mut buffer).unwrap();
            request.extend_from_slice(&buffer[..read]);
            if read == 0 || body_complete(&request) {
                break;
            }
        }
        let body = format!(
            r#"{{"detail": {{"error_type": "authentication_error", "message": "{echo}"}}}}"#
        );
        let response = format!(
            "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
    });
    format!("http://{address}")
}

fn body_complete(request: &[u8]) -> bool {
    let text = String::from_utf8_lossy(request);
    let Some(split) = text.find("\r\n\r\n") else {
        return false;
    };
    let headers = &text[..split];
    let length = headers
        .lines()
        .find_map(|line| {
            line.to_ascii_lowercase()
                .strip_prefix("content-length:")
                .map(|value| value.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    request.len() >= split + 4 + length
}

#[test]
fn rejected_request_reports_the_status_without_the_credential() {
    let _guard = ENVIRONMENT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::set_var("TYPESAFE_API_KEY", "private-test-value");
    let base_url = rejecting_server("private-test-value");
    let dir = tempfile::tempdir().unwrap();
    let jev = Jev::try_new_with_endpoint(
        dir.path().to_path_buf(),
        1.0,
        JevProvider::TypeSafe,
        false,
        &base_url,
    )
    .unwrap();
    let message = jev
        .request(&RequestState::new(Language::Python, "x = 1\n", 0, 1))
        .unwrap_err()
        .to_string();
    assert!(message.contains("401"));
    assert!(!message.contains("private-test-value"));
    assert_eq!(jev.usage().unconfirmed_requests, 1);
}

#[derive(Deserialize)]
struct TrainingRow {
    id: String,
}

#[derive(Deserialize)]
struct TrainingRecord {
    language: String,
}

#[derive(Deserialize)]
struct TrainingFeatures {
    rows: Vec<TrainingRow>,
    records: Vec<TrainingRecord>,
    values: Vec<Vec<f64>>,
}

/// slopdetect's `common.py::source_text` normalization.
fn source_text(path: &Path) -> String {
    let text = fs::read_to_string(path).unwrap();
    text.strip_prefix('\u{feff}')
        .unwrap_or(&text)
        .replace("\r\n", "\n")
        .replace('\r', "\n")
}

fn benchmark_source(root: &Path, row: &TrainingRow) -> PathBuf {
    let dir = root.join(".research/sources").join(&row.id);
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(files.len(), 1);
    files.remove(0)
}

#[test]
#[ignore = "replays the 304-file benchmark from a slopdetect checkout; set SLOPDETECT_DIR"]
fn benchmark_means_replay_bit_exactly_from_the_slopdetect_cache() {
    let Some(root) = std::env::var_os("SLOPDETECT_DIR").map(PathBuf::from) else {
        eprintln!("skipping: set SLOPDETECT_DIR");
        return;
    };
    let training: TrainingFeatures = serde_json::from_str(
        &fs::read_to_string(root.join(".ai/local/actual-magnitudes-001/training-features.json"))
            .unwrap(),
    )
    .unwrap();
    let jev = Jev::try_new(
        root.join(".ai/local/cli-cache"),
        1.0,
        JevProvider::TypeSafe,
        true,
    )
    .unwrap();
    let mut mismatches = Vec::new();
    let mut excerpts = 0;
    for (index, row) in training.rows.iter().enumerate() {
        let text = source_text(&benchmark_source(&root, row));
        let features = jev
            .extract(&text, language(&training.records[index].language))
            .unwrap();
        excerpts += features.excerpts;
        let expected = &training.values[index][32..39];
        if bits(&features.values) != bits(expected) {
            mismatches.push(format!(
                "{}: {:?} != {:?}",
                row.id, features.values, expected
            ));
        }
    }
    println!(
        "rows {} excerpts {} mismatches {}",
        training.rows.len(),
        excerpts,
        mismatches.len()
    );
    assert_eq!(mismatches, Vec::<String>::new());
    assert_eq!(jev.usage().requests, 0);
}
