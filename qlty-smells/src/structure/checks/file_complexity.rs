use crate::metrics::metrics::complexity;
use qlty_analysis::code::File;
use qlty_analysis::code::NodeFilter;
use qlty_analysis::utils::fs::path_to_string;
use qlty_types::analysis::v1::{Issue, Level};
use qlty_types::calculate_effort_minutes;
use std::collections::HashMap;
use std::sync::Arc;
use tree_sitter::Tree;

use super::issue_for;

pub const CHECK_NAME: &str = "file-complexity";

const BASE_EFFORT_MINUTES: u32 = 50;
const EFFORT_MINUTES_PER_VALUE_DELTA: u32 = 20;

pub fn check(threshold: usize, source_file: Arc<File>, tree: &Tree) -> Vec<Issue> {
    let mut issues: Vec<Issue> = Vec::new();
    let count = complexity(&source_file, &tree.root_node(), &NodeFilter::empty());
    let message = format!("High total complexity (count = {})", count);
    let mut partial_fingerprints = HashMap::new();
    let path = source_file.path.to_string_lossy();

    partial_fingerprints.insert("file.path".to_string(), path_to_string(path.as_ref()));

    if count >= threshold {
        let value_delta = count as u32 - threshold as u32;

        issues.push(Issue {
            rule_key: CHECK_NAME.to_string(),
            message,
            level: Level::Medium.into(),
            value: count as u32,
            value_delta,
            partial_fingerprints,
            effort_minutes: calculate_effort_minutes(
                value_delta,
                BASE_EFFORT_MINUTES,
                EFFORT_MINUTES_PER_VALUE_DELTA,
            ),
            ..issue_for(&source_file, &tree.root_node())
        });
    }

    issues
}

#[cfg(test)]
mod test {
    use super::*;

    mod python {
        use super::*;

        #[test]
        fn file_complexity_not_found() {
            let source_file = Arc::new(File::from_string(
                "python",
                r#"
                foo()
            "#,
            ));
            assert_eq!(0, check(1, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn file_complexity_found() {
            let source_file = Arc::new(File::from_string(
                "python",
                r#"
                if foo:
                    if bar:
                        if baz:
                            if qux:
                                if quux:
                                    if quuz:
                                        if buz:
                                            if zzz:
                                                if yyy:
                                                    if aaa:
                                                        pass
            "#
                .trim(),
            ));
            insta::assert_yaml_snapshot!(check(10, source_file.clone(), &source_file.parse()), @r#"
            - tool: qlty
              driver: structure
              ruleKey: file-complexity
              message: High total complexity (count = 55)
              level: LEVEL_MEDIUM
              language: LANGUAGE_PYTHON
              category: CATEGORY_STRUCTURE
              snippet: "if foo:\n                    if bar:\n                        if baz:\n                            if qux:\n                                if quux:\n                                    if quuz:\n                                        if buz:\n                                            if zzz:\n                                                if yyy:\n                                                    if aaa:\n                                                        pass"
              snippetWithContext: "if foo:\n                    if bar:\n                        if baz:\n                            if qux:\n                                if quux:\n                                    if quuz:\n                                        if buz:\n                                            if zzz:\n                                                if yyy:\n                                                    if aaa:\n                                                        pass"
              effortMinutes: 950
              value: 55
              valueDelta: 45
              location:
                path: STRING
                range:
                  startLine: 1
                  startColumn: 1
                  endLine: 11
                  endColumn: 61
                  startByte: 0
                  endByte: 466
              partialFingerprints:
                file.path: STRING
            "#);
        }
    }
}
