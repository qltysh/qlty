//! Live Jev calls through each provider. Ignored by default; they need
//! `TYPESAFE_API_KEY` or `AI_GATEWAY_API_KEY` in the environment and spend a
//! fraction of a cent.

use std::fs;

use qlty_slop_one::jev::{Jev, JevProvider};
use qlty_slop_one::language::Language;

const SOURCE: &str = r#"
fn total(items: &[u32]) -> u32 {
    let mut sum = 0;
    for item in items {
        sum += item;
    }
    sum
}
"#;

fn evaluate_twice(provider: JevProvider, namespace: &str) {
    let dir = tempfile::tempdir().unwrap();
    let jev = Jev::try_new(dir.path().to_path_buf(), 0.01, provider, false).unwrap();

    let first = jev.extract(SOURCE, Language::Rust).unwrap();
    assert_eq!(first.values.len(), 7);
    assert!(first.values.iter().all(|value| (0.0..=1.0).contains(value)));
    assert_eq!(first.excerpts, 1);
    let usage = jev.usage();
    assert_eq!(usage.requests, 1);
    assert_eq!(usage.unconfirmed_requests, 0);
    assert!(usage.input_tokens > 0);
    assert_eq!(fs::read_dir(dir.path().join(namespace)).unwrap().count(), 1);

    let second = jev.extract(SOURCE, Language::Rust).unwrap();
    let bits = |values: &[f64]| {
        values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    };
    assert_eq!(bits(&second.values), bits(&first.values));
    assert_eq!(jev.usage().requests, 1);
}

#[test]
#[ignore = "live TypeSafe AI call; requires TYPESAFE_API_KEY"]
fn typesafe_answers_and_caches_the_seven_questions() {
    evaluate_twice(JevProvider::TypeSafe, "jev");
}

#[test]
#[ignore = "live Vercel AI Gateway call; requires AI_GATEWAY_API_KEY"]
fn vercel_answers_and_caches_the_seven_questions() {
    evaluate_twice(JevProvider::Vercel, "jev-vercel");
}
