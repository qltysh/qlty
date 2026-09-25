//! Shared JSON comparison for the export and chart data tests.

use serde_json::Value;

/// Every leaf where `actual` differs from `expected`, as `path: expected != actual`.
/// Numbers compare as `f64`; objects compare by key set and value.
pub fn json_differences(expected: &Value, actual: &Value) -> Vec<String> {
    let mut differences = Vec::new();
    collect_differences("$", expected, actual, &mut differences);
    differences
}

fn collect_differences(path: &str, expected: &Value, actual: &Value, out: &mut Vec<String>) {
    match (expected, actual) {
        (Value::Object(expected), Value::Object(actual)) => {
            for (key, expected_value) in expected {
                match actual.get(key) {
                    Some(actual_value) => collect_differences(
                        &format!("{path}.{key}"),
                        expected_value,
                        actual_value,
                        out,
                    ),
                    None => out.push(format!("{path}.{key}: missing from actual")),
                }
            }
            for key in actual.keys() {
                if !expected.contains_key(key) {
                    out.push(format!("{path}.{key}: unexpected in actual"));
                }
            }
        }
        (Value::Array(expected), Value::Array(actual)) => {
            if expected.len() != actual.len() {
                out.push(format!(
                    "{path}: array length {} != {}",
                    expected.len(),
                    actual.len()
                ));
            }
            for (index, (expected_value, actual_value)) in expected.iter().zip(actual).enumerate() {
                collect_differences(
                    &format!("{path}[{index}]"),
                    expected_value,
                    actual_value,
                    out,
                );
            }
        }
        (Value::Number(expected), Value::Number(actual)) => {
            if expected.as_f64() != actual.as_f64() {
                out.push(format!("{path}: {expected} != {actual}"));
            }
        }
        _ => {
            if expected != actual {
                out.push(format!("{path}: {expected} != {actual}"));
            }
        }
    }
}

/// Fails the test with the first differences when there are any.
pub fn assert_same_json(context: &str, expected: &Value, actual: &Value) {
    let differences = json_differences(expected, actual);
    assert!(
        differences.is_empty(),
        "{context}: {} differences, first: {}",
        differences.len(),
        differences
            .iter()
            .take(12)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
