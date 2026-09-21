use std::path::{Path, PathBuf};

use qlty_slop_one::{Document, Evaluator, JevProvider, Options};
use serde_json::Value;

fn slopdetect_dir() -> Option<PathBuf> {
    std::env::var_os("SLOPDETECT_DIR").map(PathBuf::from)
}

#[test]
#[ignore = "requires SLOPDETECT_DIR with the benchmark sources, Jev cache, and the Python CLI document"]
fn every_benchmark_file_matches_the_python_cli_document() {
    let root = slopdetect_dir().unwrap();
    let expected: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join(".ai/local/slop-one-parity/cli-304.json")).unwrap(),
    )
    .unwrap();
    let evaluator = Evaluator::new(Options {
        cache_dir: root.join(".ai/local/cli-cache"),
        budget_usd: 1.0,
        provider: JevProvider::TypeSafe,
        offline: true,
        include_tests: true,
        include_excluded: false,
    })
    .unwrap();
    let results: Vec<_> = expected["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|result| evaluator.evaluate(Path::new(result["path"].as_str().unwrap())))
        .collect();
    let document = Document::new(evaluator.model_info(), results, evaluator.usage());
    let actual = serde_json::to_value(&document).unwrap();

    assert_eq!(actual["results"], expected["results"]);
    assert_eq!(actual["summary"], expected["summary"]);
    assert_eq!(actual["model"], expected["model"]);
    assert_eq!(actual["usage"]["requests"], 0);
}
