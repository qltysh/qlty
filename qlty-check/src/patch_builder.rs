use crate::source_reader::SourceReader;
use diffy::DiffOptions;
use itertools::Itertools;
use qlty_config::issue_transformer::IssueTransformer;
use qlty_types::analysis::v1::{Issue, Location, Replacement};
use std::sync::Arc;
use tracing::{debug, warn};

pub const PATCH_CONTEXT_LENGTH: usize = 3;

#[derive(Debug, Clone)]
pub struct PatchBuilder {
    source_reader: Arc<dyn SourceReader>,
}

impl IssueTransformer for PatchBuilder {
    fn transform(&self, mut issue: Issue) -> Option<Issue> {
        for suggestion in issue.suggestions.iter_mut() {
            let replacements = suggestion.replacements.clone();
            suggestion.replacements = self.sorted_replacements(replacements).collect();
            suggestion.patch = self.build_patch(&suggestion.replacements, &issue.location);
        }

        Some(issue)
    }

    fn clone_box(&self) -> Box<dyn IssueTransformer> {
        Box::new(self.clone())
    }
}

impl PatchBuilder {
    pub fn new(source_reader: impl SourceReader + Clone + 'static) -> Self {
        Self {
            source_reader: Arc::new(source_reader.clone()),
        }
    }

    fn sorted_replacements(
        &self,
        replacements: impl IntoIterator<Item = Replacement>,
    ) -> std::vec::IntoIter<Replacement> {
        replacements
            .into_iter()
            .sorted_by(|a, b| a.range().end_byte.cmp(&b.range().end_byte))
    }

    fn build_patch(&self, replacements: &Vec<Replacement>, location: &Option<Location>) -> String {
        let mut patch = String::from("");
        let file_path = if let Some(location) = location {
            &location.path
        } else {
            &replacements[0].location.as_ref().unwrap().path
        };

        // TODO(loren): we don't yet support replacements across multiple files.
        let contents = self.source_reader.read(file_path.into());

        if let Ok(data) = &contents {
            let mut mdata = data.clone();
            let apply = replacements.iter().rev().all(|replacement| {
                let range = replacement.range();

                let (start_byte, end_byte) = if let (Some(start_byte), Some(end_byte)) =
                    (range.start_byte, range.end_byte)
                {
                    (start_byte as usize, end_byte as usize)
                } else {
                    let start = calculate_byte_offset(
                        &mdata,
                        range.start_line as usize,
                        range.start_column as usize,
                    );
                    let end = calculate_byte_offset(
                        &mdata,
                        range.end_line as usize,
                        range.end_column as usize,
                    );
                    if let (Some(start), Some(end)) = (start, end) {
                        (start, end)
                    } else {
                        warn!(
                            "Failed to calculate byte offsets for {:?}, range ({}:{})-({}:{})",
                            file_path,
                            range.start_line,
                            range.start_column,
                            range.end_line,
                            range.end_column,
                        );
                        return false;
                    }
                };

                replace_in_range(
                    &mut mdata,
                    start_byte,
                    end_byte,
                    &replacement.data,
                    file_path,
                )
            });

            if apply {
                patch = DiffOptions::new()
                    .set_context_len(PATCH_CONTEXT_LENGTH)
                    .create_patch(data, &mdata)
                    .to_string();
                debug!("\n{}", patch);
            } else {
                warn!("Failed to generate patch for {:?}", file_path);
            }
        } else {
            warn!("Failed to read file {:?}", file_path);
        }

        patch
    }
}

fn calculate_byte_offset(content: &str, line: usize, column: usize) -> Option<usize> {
    let lines: Vec<&str> = content.lines().collect();

    if line > 0 && line <= lines.len() {
        let line_str = lines[line - 1]; // Convert 1-based line to 0-based index
        let index = if column > 0 {
            column - 1 // Convert 1-based column to 0-based index
        } else {
            0 // If column is 0/missing, use the start of the line
        };

        let byte_offset = if let Some((byte_index, _)) = line_str.char_indices().nth(index) {
            // Calculate the absolute byte offset from the start of the content
            content
                .lines()
                .take(line - 1)
                .map(|l| l.len() + 1) // Add 1 for newline character
                .sum::<usize>()
                + byte_index
        } else {
            let byte_offset = content
                .lines()
                .take(line)
                .map(|l| l.len() + 1) // Add 1 for newline character
                .sum::<usize>();

            byte_offset - 1 // Remove last newline character from the sum
        };

        Some(byte_offset)
    } else {
        None
    }
}

fn replace_in_range(
    mdata: &mut String,
    start: usize,
    end: usize,
    replacement: &str,
    file_path: &str,
) -> bool {
    let mut end = end;
    if end == mdata.len() && end != 0 {
        // Adjust end byte to avoid potential off-by-one error
        end -= 1;
    }

    if start < mdata.len() && end < mdata.len() {
        mdata.replace_range(start..end, replacement);
        true
    } else {
        warn!(
            "Failed to generate patch for {:?}, range {}-{} out of bounds (filelen={}, data={:?})",
            file_path,
            start,
            end,
            mdata.len(),
            replacement,
        );
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        parser::{clippy::Clippy, golangci_lint::GolangciLint, shellcheck::Shellcheck, Parser},
        source_reader::SourceReaderFs,
    };
    use std::path::PathBuf;
    use tracing_test::traced_test;

    fn new_from_cache<const N: usize>(list: [(PathBuf, String); N]) -> PatchBuilder {
        PatchBuilder::new(SourceReaderFs::with_cache(list.into()))
    }

    fn transformed_issues(issues: Vec<Issue>, patch_builder: &PatchBuilder) -> Vec<Issue> {
        issues
            .iter()
            .map(|issue| patch_builder.transform(issue.clone()))
            .filter(Option::is_some)
            .map(Option::unwrap)
            .collect()
    }

    #[test]
    fn parse() {
        let input = include_str!("../tests/fixtures/planner/patch_builder/parse.01.output.txt");
        let patch_builder = new_from_cache([(
            "/tmp/src/main.rs".into(),
            include_str!("../tests/fixtures/planner/patch_builder/parse.01.input.rs").into(),
        )]);

        let issues = Clippy::default().parse("clippy", input).ok().unwrap();
        let issues = transformed_issues(issues, &patch_builder);
        insta::assert_yaml_snapshot!(issues, @r#"
        - tool: clippy
          ruleKey: needless_if
          message: "this `if` branch is empty"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://rust-lang.github.io/rust-clippy/master/index.html#needless_if"
          location:
            path: /tmp/src/main.rs
            range:
              startLine: 13
              startColumn: 5
              endLine: 13
              endColumn: 26
              startByte: 357
              endByte: 378
          suggestions:
            - patch: "--- original\n+++ modified\n@@ -10,7 +10,7 @@\n fn main() {\n     let x = 1;\n     let y = 2;\n-    if x == y || x < y {}\n+    x == y || x < y;\n     println!(\"Hello World\");\n\n     // empty format literal\n"
              source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: x == y || x < y;
                  location:
                    path: /tmp/src/main.rs
                    range:
                      startLine: 13
                      startColumn: 5
                      endLine: 13
                      endColumn: 21
                      startByte: 357
                      endByte: 378
        - tool: clippy
          ruleKey: double_comparisons
          message: this binary expression can be simplified
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://rust-lang.github.io/rust-clippy/master/index.html#double_comparisons"
          location:
            path: /tmp/src/main.rs
            range:
              startLine: 13
              startColumn: 8
              endLine: 13
              endColumn: 23
              startByte: 360
              endByte: 375
          suggestions:
            - patch: "--- original\n+++ modified\n@@ -10,7 +10,7 @@\n fn main() {\n     let x = 1;\n     let y = 2;\n-    if x == y || x < y {}\n+    if x <= y {}\n     println!(\"Hello World\");\n\n     // empty format literal\n"
              source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: x <= y
                  location:
                    path: /tmp/src/main.rs
                    range:
                      startLine: 13
                      startColumn: 8
                      endLine: 13
                      endColumn: 14
                      startByte: 360
                      endByte: 375
        - tool: clippy
          ruleKey: print_literal
          message: literal with an empty format string
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://rust-lang.github.io/rust-clippy/master/index.html#print_literal"
          location:
            path: /tmp/src/main.rs
            range:
              startLine: 17
              startColumn: 20
              endLine: 17
              endColumn: 42
              startByte: 456
              endByte: 478
          suggestions:
            - patch: "--- original\n+++ modified\n@@ -14,7 +14,7 @@\n     println!(\"Hello World\");\n\n     // empty format literal\n-    println!(\"{}\", \"empty format literal\");\n+    println!(\"empty format literal\");\n\n     // needless range loop\n     let vec = vec!['a', 'b', 'c'];\n"
              source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: empty format literal
                  location:
                    path: /tmp/src/main.rs
                    range:
                      startLine: 17
                      startColumn: 15
                      endLine: 17
                      endColumn: 35
                      startByte: 451
                      endByte: 453
                - location:
                    path: /tmp/src/main.rs
                    range:
                      startLine: 17
                      startColumn: 18
                      endLine: 17
                      endColumn: 18
                      startByte: 454
                      endByte: 478
        - tool: clippy
          ruleKey: needless_range_loop
          message: "the loop variable `i` is only used to index `vec`"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://rust-lang.github.io/rust-clippy/master/index.html#needless_range_loop"
          location:
            path: /tmp/src/main.rs
            range:
              startLine: 21
              startColumn: 14
              endLine: 21
              endColumn: 26
              startByte: 557
              endByte: 569
        - tool: clippy
          ruleKey: manual_non_exhaustive
          message: this seems like a manual implementation of the non-exhaustive pattern
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://rust-lang.github.io/rust-clippy/master/index.html#manual_non_exhaustive"
          location:
            path: /tmp/src/main.rs
            range:
              startLine: 4
              startColumn: 1
              endLine: 8
              endColumn: 2
              startByte: 230
              endByte: 309
        - tool: clippy
          ruleKey: useless_vec
          message: "useless use of `vec!`"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://rust-lang.github.io/rust-clippy/master/index.html#useless_vec"
          location:
            path: /tmp/src/main.rs
            range:
              startLine: 20
              startColumn: 15
              endLine: 20
              endColumn: 34
              startByte: 523
              endByte: 542
          suggestions:
            - patch: "--- original\n+++ modified\n@@ -17,7 +17,7 @@\n     println!(\"{}\", \"empty format literal\");\n\n     // needless range loop\n-    let vec = vec!['a', 'b', 'c'];\n+    let vec = ['a', 'b', 'c'];\n     for i in 0..vec.len() {\n         println!(\"{}\", vec[i]);\n     }\n"
              source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: "['a', 'b', 'c']"
                  location:
                    path: /tmp/src/main.rs
                    range:
                      startLine: 20
                      startColumn: 15
                      endLine: 20
                      endColumn: 30
                      startByte: 523
                      endByte: 542
        "#);
    }

    #[test]
    fn parse_with_non_contiguous_replacements() {
        let input = include_str!("../tests/fixtures/planner/patch_builder/parse.02.output.txt");
        let patch_builder = new_from_cache([(
            "/tmp/src/main.rs".into(),
            include_str!("../tests/fixtures/planner/patch_builder/parse.02.input.rs").into(),
        )]);

        let issues = Clippy::default().parse("clippy", input).ok().unwrap();
        let issues = transformed_issues(issues, &patch_builder);
        insta::assert_yaml_snapshot!(issues, @r##"
        - tool: clippy
          ruleKey: derivable_impls
          message: "this `impl` can be derived"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://rust-lang.github.io/rust-clippy/master/index.html#derivable_impls"
          location:
            path: /tmp/src/main.rs
            range:
              startLine: 6
              startColumn: 1
              endLine: 10
              endColumn: 2
              startByte: 35
              endByte: 113
          suggestions:
            - patch: "--- original\n+++ modified\n@@ -1,10 +1,8 @@\n+#[derive(Default)]\n pub enum MyEnum {\n+    #[default]\n     A,\n     B,\n }\n\n-impl Default for MyEnum {\n-    fn default() -> Self {\n-        MyEnum::A\n-    }\n-}\n+\n"
              source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: "#[derive(Default)]\n"
                  location:
                    path: /tmp/src/main.rs
                    range:
                      startLine: 1
                      startColumn: 1
                      endLine: 2
                      endColumn: 1
                      startByte: 0
                      endByte: 0
                - data: "#[default]\n    "
                  location:
                    path: /tmp/src/main.rs
                    range:
                      startLine: 2
                      startColumn: 5
                      endLine: 3
                      endColumn: 5
                      startByte: 22
                      endByte: 22
                - location:
                    path: /tmp/src/main.rs
                    range:
                      startLine: 6
                      startColumn: 1
                      endLine: 6
                      endColumn: 1
                      startByte: 35
                      endByte: 114
        "##);
    }

    #[traced_test]
    #[test]
    fn parse_invalid_patch_warns() {
        let input = include_str!("../tests/fixtures/planner/patch_builder/parse.02.output.txt");
        let patch_builder = new_from_cache([("/tmp/src/main.rs".into(), "".into())]);

        let issues = Clippy::default().parse("clippy", input).ok().unwrap();
        println!("{:?}", issues);
        let issues = transformed_issues(issues, &patch_builder);

        assert!(logs_contain(
            r#"Failed to generate patch for "/tmp/src/main.rs""#
        ));
        assert!(issues
            .iter()
            .all(|issue| issue.suggestions.iter().all(|suggestion| {
                println!("suggestion.patch {:?}", suggestion.patch);
                return suggestion.patch.is_empty();
            })));
    }

    #[test]
    fn parse_line_replacements() {
        let input = include_str!("../tests/fixtures/planner/patch_builder/parse.03.output.txt");
        let patch_builder = new_from_cache([(
            "/tmp/src/main.sh".into(),
            include_str!("../tests/fixtures/planner/patch_builder/parse.03.input.sh").into(),
        )]);

        let issues = Shellcheck {}.parse("shellcheck", input).ok().unwrap();
        let issues = transformed_issues(issues, &patch_builder);
        insta::assert_yaml_snapshot!(issues, @r#"
        - tool: shellcheck
          ruleKey: "2154"
          message: x is referenced but not assigned.
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: /tmp/src/main.sh
            range:
              startLine: 3
              startColumn: 6
              endLine: 3
              endColumn: 8
        - tool: shellcheck
          ruleKey: "2086"
          message: Double quote to prevent globbing and word splitting.
          level: LEVEL_LOW
          category: CATEGORY_LINT
          location:
            path: /tmp/src/main.sh
            range:
              startLine: 3
              startColumn: 6
              endLine: 3
              endColumn: 8
          suggestions:
            - patch: "--- original\n+++ modified\n@@ -1,4 +1,4 @@\n #!/bin/sh\n\n-echo $x\n+echo \"$x\"\n echo \"You are running on `uname`\"\n"
              source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: "\""
                  location:
                    path: /tmp/src/main.sh
                    range:
                      startLine: 3
                      startColumn: 6
                      endLine: 3
                      endColumn: 6
                - data: "\""
                  location:
                    path: /tmp/src/main.sh
                    range:
                      startLine: 3
                      startColumn: 8
                      endLine: 3
                      endColumn: 8
        - tool: shellcheck
          ruleKey: "2006"
          message: "Use $(...) notation instead of legacy backticks `...`."
          level: LEVEL_LOW
          category: CATEGORY_LINT
          location:
            path: /tmp/src/main.sh
            range:
              startLine: 4
              startColumn: 26
              endLine: 4
              endColumn: 33
          suggestions:
            - patch: "--- original\n+++ modified\n@@ -1,4 +1,4 @@\n #!/bin/sh\n\n echo $x\n-echo \"You are running on `uname`\"\n+echo \"You are running on $(uname)\"\n"
              source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: $(
                  location:
                    path: /tmp/src/main.sh
                    range:
                      startLine: 4
                      startColumn: 26
                      endLine: 4
                      endColumn: 27
                - data: )
                  location:
                    path: /tmp/src/main.sh
                    range:
                      startLine: 4
                      startColumn: 32
                      endLine: 4
                      endColumn: 33
        "#)
    }

    #[test]
    fn parse_line_replacements_golangci() {
        let input = include_str!("../tests/fixtures/planner/patch_builder/parse.04.output.txt");
        let patch_builder = new_from_cache([(
            "/tmp/src/main.go".into(),
            include_str!("../tests/fixtures/planner/patch_builder/parse.04.input.go").into(),
        )]);

        let issues = GolangciLint {}.parse("golangci-lint", input).ok().unwrap();
        let issues = transformed_issues(issues, &patch_builder);
        insta::assert_yaml_snapshot!(issues, @r###"
        - tool: golangci-lint
          ruleKey: errcheck
          message: "Error return value of `time.Parse` is not checked"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: /tmp/src/main.go
            range:
              startLine: 12
              startColumn: 12
        - tool: golangci-lint
          ruleKey: godot
          message: Comment should end in a period
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: /tmp/src/main.go
            range:
              startLine: 7
              startColumn: 34
          suggestions:
            - patch: "--- original\n+++ modified\n@@ -4,7 +4,7 @@\n import \"fmt\"\n\n // ✋✋✋✋\n-// this is the main function 🏃\n+// this is the main function 🏃.\n func main() {\n \t// This is a comment\n \tfmt.Println(\"Heloo World!\") // Intentional typo: \"Heloo\" instead of \"Hello\"\n"
              source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: // this is the main function 🏃.
                  location:
                    path: /tmp/src/main.go
                    range:
                      startLine: 7
                      endLine: 7
                      endColumn: 34
        - tool: golangci-lint
          ruleKey: goimports
          message: "File is not `goimports`-ed"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: /tmp/src/main.go
            range:
              startLine: 3
          suggestions:
            - patch: "--- original\n+++ modified\n@@ -1,7 +1,9 @@\n package main\n\n-import \"time\"\n-import \"fmt\"\n+import (\n+\t\"fmt\"\n+\t\"time\"\n+)\n\n // ✋✋✋✋\n // this is the main function 🏃\n"
              source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: "import (\n\t\"fmt\"\n\t\"time\"\n)"
                  location:
                    path: /tmp/src/main.go
                    range:
                      startLine: 3
                      endLine: 4
                      endColumn: 13
        "###);
    }
}
