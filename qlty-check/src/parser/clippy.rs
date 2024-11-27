use std::path::PathBuf;

use crate::source_reader::offset_to_location;

use super::Parser;
use anyhow::Result;
use qlty_analysis::{join_path_string, utils::fs::path_to_string};
use qlty_types::analysis::v1::{
    Category, Issue, Level, Location, Range, Replacement, Suggestion, SuggestionSource,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{trace, warn};
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ClippyIssue {
    message: Option<Message>,
    success: Option<bool>,
    manifest_path: Option<String>,
    package_id: Option<String>,

    #[serde(skip)]
    package_name: String,

    #[serde(skip)]
    path_prefix: PathBuf,
}

impl ClippyIssue {
    fn from_json(line: &str) -> Self {
        let mut issue: ClippyIssue = serde_json::from_str(line).unwrap();
        issue.initialize();
        issue
    }

    fn initialize(&mut self) {
        if let Some(package_id) = &self.package_id {
            let package_id = package_id
                .split(' ')
                .last()
                .unwrap()
                .replace(['(', ')'], "");
            if let Ok(url) = Url::parse(&package_id) {
                self.package_name = url.path_segments().unwrap().last().unwrap().to_string();
            }
        }

        if let Some(manifest_path) = &self.manifest_path {
            let manifest_path_buf = PathBuf::from(manifest_path);
            self.path_prefix = if manifest_path_buf.ends_with("Cargo.toml") {
                manifest_path_buf.parent().unwrap().to_path_buf()
            } else {
                manifest_path_buf
            };
        }
    }

    fn resolve_path(&self, file_name: &String) -> String {
        let mut file_path = PathBuf::from(file_name);
        if !self.package_name.is_empty() && file_path.starts_with(&self.package_name) {
            file_path = file_path.iter().skip(1).collect::<PathBuf>();
        }

        path_to_string(join_path_string!(&self.path_prefix, file_path))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WorkspaceEntry {
    src_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    rendered: Option<String>,
    code: Option<Code>,
    spans: Vec<Span>,
    level: String,
    message: String,
    children: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Code {
    code: String,
    explanation: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Span {
    byte_start: u32,
    byte_end: u32,
    column_end: u32,
    column_start: u32,
    line_end: u32,
    line_start: u32,
    file_name: String,
    suggested_replacement: Option<String>,
    suggestion_applicability: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Clippy {}

impl Parser for Clippy {
    fn parse(&self, _plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let lines = output.trim().lines();

        for line in lines {
            let clippy_issue = ClippyIssue::from_json(line);

            if Some(false) == clippy_issue.success {
                warn!(
                    "Clippy output includes success: false, {:?}, raw line: {:?}",
                    clippy_issue, line
                );

                issues.push(Issue {
                    tool: "clippy".into(),
                    rule_key: "build_failure".into(),
                    message: "Clippy failed to run because Rust could not compile.".into(),
                    level: Level::High.into(),
                    ..Default::default()
                });
            }

            if clippy_issue.message.is_none() {
                trace!("Clippy issue message is None: {:?}", clippy_issue);
                continue;
            }

            if clippy_issue.message.as_ref().unwrap().code.is_none() {
                trace!("Clippy issue code is None: {:?}", clippy_issue);
                continue;
            }

            if clippy_issue.manifest_path.is_none() {
                trace!("Clippy issue manifest_path is None: {:?}", clippy_issue);
                continue;
            }

            let message = clippy_issue.message.as_ref().unwrap();
            let code = message.code.clone();
            let suggestion = self.build_suggestion(message, &clippy_issue);

            let issue = Issue {
                tool: "clippy".into(),
                message: message.message.clone(),
                category: Category::Lint.into(),
                level: Level::Medium.into(),
                rule_key: code.unwrap().code.replace("clippy::", ""),
                documentation_url: self.extract_url(&message.rendered.clone().unwrap_or_default()),
                suggestions: suggestion.into_iter().collect(),
                location: Some(Location {
                    path: clippy_issue.resolve_path(&message.spans[0].file_name),
                    range: Some(Range {
                        start_line: message.spans[0].line_start,
                        start_column: message.spans[0].column_start,
                        end_line: message.spans[0].line_end,
                        end_column: message.spans[0].column_end,
                        start_byte: Some(message.spans[0].byte_start),
                        end_byte: Some(message.spans[0].byte_end),
                        ..Default::default()
                    }),
                }),
                ..Default::default()
            };

            issues.push(issue);
        }

        Ok(issues)
    }
}

impl Clippy {
    fn extract_url(&self, s: &str) -> String {
        let url_regex = Regex::new(r"https?://[^\s]+").unwrap(); // Simple regex to match HTTP and HTTPS URLs
        match url_regex.find(s).map(|mat| mat.as_str().to_string()) {
            Some(url) => url,
            None => "".to_string(),
        }
    }

    fn build_suggestion(&self, message: &Message, issue: &ClippyIssue) -> Option<Suggestion> {
        let replacements: Vec<_> = message
            .children
            .iter()
            .filter(|child| child.level == "help")
            .flat_map(|child| self.collect_replacements(child, issue))
            .collect();

        if replacements.is_empty() {
            return None;
        }

        Some(Suggestion {
            replacements,
            source: SuggestionSource::Tool.into(),
            ..Default::default()
        })
    }

    fn collect_replacements(&self, message: &Message, issue: &ClippyIssue) -> Vec<Replacement> {
        message
            .spans
            .iter()
            .filter(|span| {
                span.suggested_replacement.is_some()
                    && span.suggestion_applicability == Some("MachineApplicable".into())
            })
            .map(|span| Replacement {
                data: span.suggested_replacement.clone().unwrap(),
                location: Some(Location {
                    path: issue.resolve_path(&span.file_name),
                    range: self.calculate_replacement_range(span),
                }),
            })
            .collect()
    }

    fn calculate_replacement_range(&self, span: &Span) -> Option<Range> {
        let replacement = span.suggested_replacement.clone().unwrap();
        let (rep_end_line, rep_end_column) =
            offset_to_location(replacement.as_str(), replacement.len());
        let end_line = span.line_start + rep_end_line as u32 - 1;
        let end_column = if rep_end_line == 1 {
            span.column_start + rep_end_column as u32 - 1
        } else {
            rep_end_column as u32
        };

        Some(Range {
            start_line: span.line_start,
            end_line,
            start_column: span.column_start,
            end_column,
            start_byte: Some(span.byte_start),
            end_byte: Some(span.byte_end),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let clippy = Clippy::default();

        let input = r###"
        {"reason":"compiler-message","package_id":"path+file:///tmp","manifest_path":"/tmp/Cargo.toml","target":{"kind":["bin"],"crate_types":["bin"],"name":"basic","src_path":"/tmp/src/main.rs","edition":"2021","doc":true,"doctest":false,"test":true},"message":{"rendered":"warning: this `if` branch is empty\n  --> src/main.rs:13:5\n   |\n13 |     if x == y || x < y {}\n   |     ^^^^^^^^^^^^^^^^^^^^^ help: you can remove it: `x == y || x < y;`\n   |\n   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#needless_if\n   = note: `#[warn(clippy::needless_if)]` on by default\n\n","$message_type":"diagnostic","children":[{"children":[],"code":null,"level":"help","message":"for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#needless_if","rendered":null,"spans":[]},{"children":[],"code":null,"level":"note","message":"`#[warn(clippy::needless_if)]` on by default","rendered":null,"spans":[]},{"children":[],"code":null,"level":"help","message":"you can remove it","rendered":null,"spans":[{"byte_end":378,"byte_start":357,"column_end":26,"column_start":5,"expansion":null,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":13,"line_start":13,"suggested_replacement":"x == y || x < y;","suggestion_applicability":"MachineApplicable","text":[{"highlight_end":26,"highlight_start":5,"text":"    if x == y || x < y {}"}]}]}],"code":{"code":"clippy::needless_if","explanation":null},"level":"warning","message":"this `if` branch is empty","spans":[{"byte_end":378,"byte_start":357,"column_end":26,"column_start":5,"expansion":null,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":13,"line_start":13,"suggested_replacement":null,"suggestion_applicability":null,"text":[{"highlight_end":26,"highlight_start":5,"text":"    if x == y || x < y {}"}]}]}}
        {"reason":"compiler-message","package_id":"basic 0.1.0 (path+file:///tmp#1.0)","manifest_path":"/tmp/Cargo.toml","target":{"kind":["bin"],"crate_types":["bin"],"name":"basic","src_path":"/tmp/src/main.rs","edition":"2021","doc":true,"doctest":false,"test":true},"message":{"rendered":"warning: this binary expression can be simplified\n  --> src/main.rs:13:8\n   |\n13 |     if x == y || x < y {}\n   |        ^^^^^^^^^^^^^^^ help: try: `x <= y`\n   |\n   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#double_comparisons\n   = note: `#[warn(clippy::double_comparisons)]` on by default\n\n","$message_type":"diagnostic","children":[{"children":[],"code":null,"level":"help","message":"for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#double_comparisons","rendered":null,"spans":[]},{"children":[],"code":null,"level":"note","message":"`#[warn(clippy::double_comparisons)]` on by default","rendered":null,"spans":[]},{"children":[],"code":null,"level":"help","message":"try","rendered":null,"spans":[{"byte_end":375,"byte_start":360,"column_end":23,"column_start":8,"expansion":null,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":13,"line_start":13,"suggested_replacement":"x <= y","suggestion_applicability":"MachineApplicable","text":[{"highlight_end":23,"highlight_start":8,"text":"    if x == y || x < y {}"}]}]}],"code":{"code":"clippy::double_comparisons","explanation":null},"level":"warning","message":"this binary expression can be simplified","spans":[{"byte_end":375,"byte_start":360,"column_end":23,"column_start":8,"expansion":null,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":13,"line_start":13,"suggested_replacement":null,"suggestion_applicability":null,"text":[{"highlight_end":23,"highlight_start":8,"text":"    if x == y || x < y {}"}]}]}}
        {"reason":"compiler-message","package_id":"basic 0.1.0 (path+file:///tmp)","manifest_path":"/tmp/Cargo.toml","target":{"kind":["bin"],"crate_types":["bin"],"name":"basic","src_path":"/tmp/src/main.rs","edition":"2021","doc":true,"doctest":false,"test":true},"message":{"rendered":"warning: literal with an empty format string\n  --> src/main.rs:17:20\n   |\n17 |     println!(\"{}\", \"empty format literal\");\n   |                    ^^^^^^^^^^^^^^^^^^^^^^\n   |\n   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#print_literal\n   = note: `#[warn(clippy::print_literal)]` on by default\nhelp: try\n   |\n17 -     println!(\"{}\", \"empty format literal\");\n17 +     println!(\"empty format literal\");\n   |\n\n","$message_type":"diagnostic","children":[{"children":[],"code":null,"level":"help","message":"for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#print_literal","rendered":null,"spans":[]},{"children":[],"code":null,"level":"note","message":"`#[warn(clippy::print_literal)]` on by default","rendered":null,"spans":[]},{"children":[],"code":null,"level":"help","message":"try","rendered":null,"spans":[{"byte_end":453,"byte_start":451,"column_end":17,"column_start":15,"expansion":null,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":17,"line_start":17,"suggested_replacement":"empty format literal","suggestion_applicability":"MachineApplicable","text":[{"highlight_end":17,"highlight_start":15,"text":"    println!(\"{}\", \"empty format literal\");"}]},{"byte_end":478,"byte_start":454,"column_end":42,"column_start":18,"expansion":null,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":17,"line_start":17,"suggested_replacement":"","suggestion_applicability":"MachineApplicable","text":[{"highlight_end":42,"highlight_start":18,"text":"    println!(\"{}\", \"empty format literal\");"}]}]}],"code":{"code":"clippy::print_literal","explanation":null},"level":"warning","message":"literal with an empty format string","spans":[{"byte_end":478,"byte_start":456,"column_end":42,"column_start":20,"expansion":null,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":17,"line_start":17,"suggested_replacement":null,"suggestion_applicability":null,"text":[{"highlight_end":42,"highlight_start":20,"text":"    println!(\"{}\", \"empty format literal\");"}]}]}}
        {"reason":"compiler-message","package_id":"basic 0.1.0 (path+file:///tmp)","manifest_path":"/tmp/Cargo.toml","target":{"kind":["bin"],"crate_types":["bin"],"name":"basic","src_path":"/tmp/src/main.rs","edition":"2021","doc":true,"doctest":false,"test":true},"message":{"rendered":"warning: the loop variable `i` is only used to index `vec`\n  --> src/main.rs:21:14\n   |\n21 |     for i in 0..vec.len() {\n   |              ^^^^^^^^^^^^\n   |\n   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#needless_range_loop\n   = note: `#[warn(clippy::needless_range_loop)]` on by default\nhelp: consider using an iterator\n   |\n21 |     for <item> in &vec {\n   |         ~~~~~~    ~~~~\n\n","$message_type":"diagnostic","children":[{"children":[],"code":null,"level":"help","message":"for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#needless_range_loop","rendered":null,"spans":[]},{"children":[],"code":null,"level":"note","message":"`#[warn(clippy::needless_range_loop)]` on by default","rendered":null,"spans":[]},{"children":[],"code":null,"level":"help","message":"consider using an iterator","rendered":null,"spans":[{"byte_end":553,"byte_start":552,"column_end":10,"column_start":9,"expansion":null,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":21,"line_start":21,"suggested_replacement":"<item>","suggestion_applicability":"Unspecified","text":[{"highlight_end":10,"highlight_start":9,"text":"    for i in 0..vec.len() {"}]},{"byte_end":569,"byte_start":557,"column_end":26,"column_start":14,"expansion":null,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":21,"line_start":21,"suggested_replacement":"&vec","suggestion_applicability":"Unspecified","text":[{"highlight_end":26,"highlight_start":14,"text":"    for i in 0..vec.len() {"}]}]}],"code":{"code":"clippy::needless_range_loop","explanation":null},"level":"warning","message":"the loop variable `i` is only used to index `vec`","spans":[{"byte_end":569,"byte_start":557,"column_end":26,"column_start":14,"expansion":null,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":21,"line_start":21,"suggested_replacement":null,"suggestion_applicability":null,"text":[{"highlight_end":26,"highlight_start":14,"text":"    for i in 0..vec.len() {"}]}]}}
        {"reason":"compiler-message","package_id":"basic 0.1.0 (path+file:///tmp)","manifest_path":"/tmp/Cargo.toml","target":{"kind":["bin"],"crate_types":["bin"],"name":"basic","src_path":"/tmp/src/main.rs","edition":"2021","doc":true,"doctest":false,"test":true},"message":{"rendered":"warning: this seems like a manual implementation of the non-exhaustive pattern\n --> src/main.rs:4:1\n  |\n4 |   pub enum Gibberish {\n  |   ^-----------------\n  |   |\n  |  _help: add the attribute: `#[non_exhaustive] pub enum Gibberish`\n  | |\n5 | |     Foo(String),\n6 | |     #[doc(hidden)]\n7 | |     __Nonexhaustive,\n8 | | }\n  | |_^\n  |\nhelp: remove this variant\n --> src/main.rs:7:5\n  |\n7 |     __Nonexhaustive,\n  |     ^^^^^^^^^^^^^^^\n  = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#manual_non_exhaustive\n  = note: `#[warn(clippy::manual_non_exhaustive)]` on by default\n\n","$message_type":"diagnostic","children":[{"children":[],"code":null,"level":"help","message":"remove this variant","rendered":null,"spans":[{"byte_end":306,"byte_start":291,"column_end":20,"column_start":5,"expansion":null,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":7,"line_start":7,"suggested_replacement":null,"suggestion_applicability":null,"text":[{"highlight_end":20,"highlight_start":5,"text":"    __Nonexhaustive,"}]}]},{"children":[],"code":null,"level":"help","message":"for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#manual_non_exhaustive","rendered":null,"spans":[]},{"children":[],"code":null,"level":"note","message":"`#[warn(clippy::manual_non_exhaustive)]` on by default","rendered":null,"spans":[]},{"children":[],"code":null,"level":"help","message":"add the attribute","rendered":null,"spans":[{"byte_end":248,"byte_start":230,"column_end":19,"column_start":1,"expansion":null,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":4,"line_start":4,"suggested_replacement":"#[non_exhaustive] pub enum Gibberish","suggestion_applicability":"Unspecified","text":[{"highlight_end":19,"highlight_start":1,"text":"pub enum Gibberish {"}]}]}],"code":{"code":"clippy::manual_non_exhaustive","explanation":null},"level":"warning","message":"this seems like a manual implementation of the non-exhaustive pattern","spans":[{"byte_end":309,"byte_start":230,"column_end":2,"column_start":1,"expansion":null,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":8,"line_start":4,"suggested_replacement":null,"suggestion_applicability":null,"text":[{"highlight_end":21,"highlight_start":1,"text":"pub enum Gibberish {"},{"highlight_end":17,"highlight_start":1,"text":"    Foo(String),"},{"highlight_end":19,"highlight_start":1,"text":"    #[doc(hidden)]"},{"highlight_end":21,"highlight_start":1,"text":"    __Nonexhaustive,"},{"highlight_end":2,"highlight_start":1,"text":"}"}]}]}}
        {"reason":"compiler-message","package_id":"basic 0.1.0 (path+file:///tmp)","manifest_path":"/tmp/Cargo.toml","target":{"kind":["bin"],"crate_types":["bin"],"name":"basic","src_path":"/tmp/src/main.rs","edition":"2021","doc":true,"doctest":false,"test":true},"message":{"rendered":"warning: useless use of `vec!`\n  --> src/main.rs:20:15\n   |\n20 |     let vec = vec!['a', 'b', 'c'];\n   |               ^^^^^^^^^^^^^^^^^^^ help: you can use an array directly: `['a', 'b', 'c']`\n   |\n   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#useless_vec\n   = note: `#[warn(clippy::useless_vec)]` on by default\n\n","$message_type":"diagnostic","children":[{"children":[],"code":null,"level":"help","message":"for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#useless_vec","rendered":null,"spans":[]},{"children":[],"code":null,"level":"note","message":"`#[warn(clippy::useless_vec)]` on by default","rendered":null,"spans":[]},{"children":[],"code":null,"level":"help","message":"you can use an array directly","rendered":null,"spans":[{"byte_end":542,"byte_start":523,"column_end":34,"column_start":15,"expansion":null,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":20,"line_start":20,"suggested_replacement":"['a', 'b', 'c']","suggestion_applicability":"MachineApplicable","text":[{"highlight_end":34,"highlight_start":15,"text":"    let vec = vec!['a', 'b', 'c'];"}]}]}],"code":{"code":"clippy::useless_vec","explanation":null},"level":"warning","message":"useless use of `vec!`","spans":[{"byte_end":542,"byte_start":523,"column_end":34,"column_start":15,"expansion":null,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":20,"line_start":20,"suggested_replacement":null,"suggestion_applicability":null,"text":[{"highlight_end":34,"highlight_start":15,"text":"    let vec = vec!['a', 'b', 'c'];"}]}]}}
        {"reason":"compiler-message","package_id":"basic 0.1.0 (path+file:///tmp)","manifest_path":"/tmp/Cargo.toml","target":{"kind":["bin"],"crate_types":["bin"],"name":"basic","src_path":"/tmp/src/main.rs","edition":"2021","doc":true,"doctest":false,"test":true},"message":{"rendered":"warning: 6 warnings emitted\n\n","$message_type":"diagnostic","children":[],"code":null,"level":"warning","message":"6 warnings emitted","spans":[]}}
        {"reason":"compiler-artifact","package_id":"basic 0.1.0 (path+file:///tmp)","manifest_path":"/tmp/Cargo.toml","target":{"kind":["bin"],"crate_types":["bin"],"name":"basic","src_path":"/tmp/src/main.rs","edition":"2021","doc":true,"doctest":false,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/tmp/target/debug/deps/libbasic-b0326ffc109bdba8.rmeta"],"executable":null,"fresh":false}
        {"reason":"build-finished","success":true}
        "###;

        let issues = clippy.parse("clippy", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
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
            - source: SUGGESTION_SOURCE_TOOL
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
            - source: SUGGESTION_SOURCE_TOOL
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
            - source: SUGGESTION_SOURCE_TOOL
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
            - source: SUGGESTION_SOURCE_TOOL
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
    fn parse_build_error() {
        let clippy = Clippy::default();

        let input = r###"
            {"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#libc@0.2.164","manifest_path":"/Users/arslan/work/code_climate/qlty/.qlty/results/cargo_home/registry/src/index.crates.io-6f17d22bba15001f/libc-0.2.164/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/arslan/work/code_climate/qlty/.qlty/results/cargo_home/registry/src/index.crates.io-6f17d22bba15001f/libc-0.2.164/build.rs","edition":"2015","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","extra_traits","std"],"filenames":["/Users/arslan/work/code_climate/qlty/.qlty/results/cargo_target_dir/debug/build/libc-ab2b57d6096c028e/build-script-build"],"executable":null,"fresh":true}
            {"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#libc@0.2.164","linked_libs":[],"linked_paths":[],"cfgs":["freebsd11","emscripten_new_stat_abi","libc_priv_mod_use","libc_union","libc_const_size_of","libc_align","libc_int128","libc_core_cvoid","libc_packedN","libc_cfg_target_vendor","libc_non_exhaustive","libc_long_array","libc_ptr_addr_of","libc_underscore_const_names","libc_const_extern_fn"],"env":[],"out_dir":"/Users/arslan/work/code_climate/qlty/.qlty/results/cargo_target_dir/debug/build/libc-cd0db8e5c9e0e840/out"}
            {"reason":"compiler-artifact","package_id":"path+file:///Users/arslan/work/code_climate/qlty/qlty-check#0.452.0","manifest_path":"/Users/arslan/work/code_climate/qlty/qlty-check/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"qlty-check","src_path":"/Users/arslan/work/code_climate/qlty/qlty-check/src/lib.rs","edition":"2021","doc":true,"doctest":false,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/arslan/work/code_climate/qlty/.qlty/results/cargo_target_dir/debug/deps/libqlty_check-01d40922354d01fb.rmeta"],"executable":null,"fresh":false}
            {"reason":"compiler-message","package_id":"path+file:///Users/arslan/work/code_climate/qlty/qlty-cli#qlty@0.452.0","manifest_path":"/Users/arslan/work/code_climate/qlty/qlty-cli/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"qlty","src_path":"/Users/arslan/work/code_climate/qlty/qlty-cli/src/lib.rs","edition":"2021","doc":true,"doctest":false,"test":true},"message":{"rendered":"error[E0412]: cannot find type `PanicHookInfo` in module `std::panic`\n  --> qlty-cli/src/telemetry.rs:92:50\n   |\n92 |     pub fn panic(&self, panic_info: &std::panic::PanicHookInfo<'_>) -> Result<()> {\n   |                                                  ^^^^^^^^^^^^^ help: a struct with a similar name exists: `PanicInfo`\n  --> /rustc/25ef9e3d85d934b27d9dada2f9dd52b1dc63bb04/library/core/src/panic/panic_info.rs:26:1\n   |\n   = note: similarly named struct `PanicInfo` defined here\n\n","$message_type":"diagnostic","children":[{"children":[],"code":null,"level":"help","message":"a struct with a similar name exists","rendered":null,"spans":[{"byte_end":2748,"byte_start":2735,"column_end":63,"column_start":50,"expansion":null,"file_name":"qlty-cli/src/telemetry.rs","is_primary":true,"label":null,"line_end":92,"line_start":92,"suggested_replacement":"PanicInfo","suggestion_applicability":"MaybeIncorrect","text":[{"highlight_end":63,"highlight_start":50,"text":"    pub fn panic(&self, panic_info: &std::panic::PanicHookInfo<'_>) -> Result<()> {"}]}]}],"code":{"code":"E0412","explanation":"A used type name is not in scope.\n\nErroneous code examples:\n\n```compile_fail,E0412\nimpl Something {} // error: type name `Something` is not in scope\n\n// or:\n\ntrait Foo {\n    fn bar(N); // error: type name `N` is not in scope\n}\n\n// or:\n\nfn foo(x: T) {} // type name `T` is not in scope\n```\n\nTo fix this error, please verify you didn't misspell the type name, you did\ndeclare it or imported it into the scope. Examples:\n\n```\nstruct Something;\n\nimpl Something {} // ok!\n\n// or:\n\ntrait Foo {\n    type N;\n\n    fn bar(_: Self::N); // ok!\n}\n\n// or:\n\nfn foo<T>(x: T) {} // ok!\n```\n\nAnother case that causes this error is when a type is imported into a parent\nmodule. To fix this, you can follow the suggestion and use File directly or\n`use super::File;` which will import the types from the parent namespace. An\nexample that causes this error is below:\n\n```compile_fail,E0412\nuse std::fs::File;\n\nmod foo {\n    fn some_function(f: File) {}\n}\n```\n\n```\nuse std::fs::File;\n\nmod foo {\n    // either\n    use super::File;\n    // or\n    // use std::fs::File;\n    fn foo(f: File) {}\n}\n# fn main() {} // don't insert it for us; that'll break imports\n```\n"},"level":"error","message":"cannot find type `PanicHookInfo` in module `std::panic`","spans":[{"byte_end":603,"byte_start":579,"column_end":25,"column_start":1,"expansion":null,"file_name":"/rustc/25ef9e3d85d934b27d9dada2f9dd52b1dc63bb04/library/core/src/panic/panic_info.rs","is_primary":false,"label":"similarly named struct `PanicInfo` defined here","line_end":26,"line_start":26,"suggested_replacement":null,"suggestion_applicability":null,"text":[]},{"byte_end":2748,"byte_start":2735,"column_end":63,"column_start":50,"expansion":null,"file_name":"qlty-cli/src/telemetry.rs","is_primary":true,"label":null,"line_end":92,"line_start":92,"suggested_replacement":null,"suggestion_applicability":null,"text":[{"highlight_end":63,"highlight_start":50,"text":"    pub fn panic(&self, panic_info: &std::panic::PanicHookInfo<'_>) -> Result<()> {"}]}]}}
            {"reason":"compiler-message","package_id":"path+file:///Users/arslan/work/code_climate/qlty/qlty-cli#qlty@0.452.0","manifest_path":"/Users/arslan/work/code_climate/qlty/qlty-cli/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"qlty","src_path":"/Users/arslan/work/code_climate/qlty/qlty-cli/src/lib.rs","edition":"2021","doc":true,"doctest":false,"test":true},"message":{"rendered":"error: aborting due to 1 previous error\n\n","$message_type":"diagnostic","children":[],"code":null,"level":"error","message":"aborting due to 1 previous error","spans":[]}}
            {"reason":"compiler-message","package_id":"path+file:///Users/arslan/work/code_climate/qlty/qlty-cli#qlty@0.452.0","manifest_path":"/Users/arslan/work/code_climate/qlty/qlty-cli/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"qlty","src_path":"/Users/arslan/work/code_climate/qlty/qlty-cli/src/lib.rs","edition":"2021","doc":true,"doctest":false,"test":true},"message":{"rendered":"For more information about this error, try `rustc --explain E0412`.\n","$message_type":"diagnostic","children":[],"code":null,"level":"failure-note","message":"For more information about this error, try `rustc --explain E0412`.","spans":[]}}
            {"reason":"build-finished","success":false}
        "###;

        let issues = clippy.parse("clippy", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r###"
        - tool: clippy
          ruleKey: E0412
          message: "cannot find type `PanicHookInfo` in module `std::panic`"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: /rustc/25ef9e3d85d934b27d9dada2f9dd52b1dc63bb04/library/core/src/panic/panic_info.rs
            range:
              startLine: 26
              startColumn: 1
              endLine: 26
              endColumn: 25
              startByte: 579
              endByte: 603
        - tool: clippy
          ruleKey: build_failure
          message: Clippy failed to run because Rust could not compile.
          level: LEVEL_HIGH
        "###);
    }

    #[test]
    fn clippy_issue_resolve_path() {
        let issue = ClippyIssue::from_json(
            r#"{"package_id":"path+file:///a/package_name#1.0.0","manifest_path":"/a/package_name/Cargo.toml"}"#,
        );
        assert_eq!(issue.package_name, "package_name");
        assert_eq!(
            issue.resolve_path(&String::from("package_name/src/main.rs")),
            "/a/package_name/src/main.rs"
        );
        assert_eq!(
            issue.resolve_path(&String::from("not_package_name/src/main.rs")),
            "/a/package_name/not_package_name/src/main.rs"
        );
    }

    #[test]
    fn clippy_issue_package_id() {
        let issue = ClippyIssue::from_json(r#"{"package_id":"path+file:///a/pkg#1.0.0"}"#);
        assert_eq!(issue.package_name, "pkg");

        let issue = ClippyIssue::from_json(r#"{"package_id":"path+file:///pkg#1.0.0"}"#);
        assert_eq!(issue.package_name, "pkg");

        let issue = ClippyIssue::from_json(r#"{"package_id":"file:///pkg"}"#);
        assert_eq!(issue.package_name, "pkg");

        let issue = ClippyIssue::from_json(r#"{"package_id":"prefix 1.0 (file:///pkg)"}"#);
        assert_eq!(issue.package_name, "pkg");
    }

    #[test]
    fn clippy_issue_invalid_package_id() {
        let issue = ClippyIssue::from_json(
            r#"{"package_id":"INVALID_URL","manifest_path":"/a/Cargo.toml"}"#,
        );
        assert_eq!(issue.package_name, "");
        assert_eq!(
            issue.resolve_path(&String::from("INVALID_URL/a/b")),
            "/a/INVALID_URL/a/b"
        );
    }

    #[test]
    fn clippy_issue_invalid_manifest() {
        let issue = ClippyIssue::from_json(r#"{"package_id":"file:///pkg","manifest_path":"/a"}"#);
        assert_eq!(issue.resolve_path(&String::from("b/c")), "/a/b/c");
    }
}
