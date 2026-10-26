//! Input preparation and Rust inline test removal, ported from slopdetect's
//! `common.py::source_text` and `source.py`.
//!
//! Removed test modules leave a line map from analyzed lines back to the
//! original file so findings can be reported at their original locations.

use std::fs;
use std::path::Path;

use serde::Serialize;
use tree_sitter::{Node, Parser, Tree};

use crate::error::{Error, Result};
use crate::grammar;
use crate::jev::Excerpt;
use crate::measure::LineSpan;
use crate::pylines;

pub const MAX_SOURCE_BYTES: u64 = 16 * 1024 * 1024;

const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";
const TEST_MODULE_NAMES: [&str; 2] = ["test", "tests"];
const EXCLUDED_MODULE_REASON: &str = "inline module named test or tests";
const COMMENTS: [&str; 2] = ["line_comment", "block_comment"];
/// Macro token trees are not expanded Rust syntax and are never rewritten.
const OPAQUE_NODES: [&str; 3] = ["macro_definition", "macro_invocation", "attribute_item"];
const NON_CODE_ITEMS: [&str; 5] = [
    "line_comment",
    "block_comment",
    "inner_attribute_item",
    "attribute_item",
    "shebang",
];

/// Reads and normalizes a source file exactly as slopdetect did before any
/// analysis: UTF-8 with an optional BOM, line endings converted to `\n`.
pub fn source_text(path: &Path) -> Result<String> {
    let metadata = fs::metadata(path).map_err(|_| Error::NotASourceFile(path.to_path_buf()))?;
    if !metadata.is_file() {
        return Err(Error::NotASourceFile(path.to_path_buf()));
    }
    if metadata.len() > MAX_SOURCE_BYTES {
        return Err(Error::SourceTooLarge(path.to_path_buf()));
    }
    source_text_from_bytes(path, &fs::read(path)?)
}

/// [`source_text`] for contents already in memory, such as a blob from Git.
/// `path` names the file in errors.
pub fn source_text_from_bytes(path: &Path, raw: &[u8]) -> Result<String> {
    if raw.len() as u64 > MAX_SOURCE_BYTES {
        return Err(Error::SourceTooLarge(path.to_path_buf()));
    }
    let bytes = raw.strip_prefix(UTF8_BOM).unwrap_or(raw);
    let decoded =
        std::str::from_utf8(bytes).map_err(|_| Error::SourceNotUtf8(path.to_path_buf()))?;
    let text = decoded.replace("\r\n", "\n").replace('\r', "\n");
    if text.contains('\0') || text.chars().all(is_python_whitespace) {
        return Err(Error::SourceEmptyOrBinary(path.to_path_buf()));
    }
    Ok(text)
}

/// Whether Python's `str.strip()` removes `c`. Python also strips the
/// separator controls `\x1c`..`\x1f`, which Unicode does not call whitespace.
pub(crate) fn is_python_whitespace(c: char) -> bool {
    c.is_whitespace() || matches!(c, '\x1c'..='\x1f')
}

/// How the analyzed text was selected from the original file.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum SourcePolicy {
    #[serde(rename = "whole-file")]
    WholeFile,
    #[serde(rename = "rust-inline-tests-001")]
    RustInlineTests,
}

/// An inline Rust test module removed before analysis, with original lines.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExcludedModule {
    pub name: String,
    pub start_line: u32,
    pub end_line: u32,
    pub reason: &'static str,
}

/// The `source_scope` document for an analyzed file.
#[derive(Clone, Debug, Serialize)]
pub struct SourceInfo {
    pub policy: SourcePolicy,
    pub original_lines: usize,
    pub analyzed_lines: usize,
    pub excluded_lines: usize,
    pub excluded_modules: Vec<ExcludedModule>,
}

/// The text to analyze and its mapping back to the original file.
#[derive(Clone, Debug)]
pub struct PreparedSource {
    text: String,
    line_map: Vec<u32>,
    original_lines: usize,
    excluded_modules: Vec<ExcludedModule>,
    policy: SourcePolicy,
    has_code: bool,
}

impl PreparedSource {
    /// Selects the code to analyze. Only Rust files with `include_tests`
    /// off lose their inline `test`/`tests` modules; everything else is
    /// analyzed whole.
    pub fn prepare(text: &str, suffix: &str, include_tests: bool) -> Result<Self> {
        let original_lines = pylines::line_count(text);
        let mut prepared = Self {
            text: text.to_owned(),
            line_map: (1..=line_number(original_lines)).collect(),
            original_lines,
            excluded_modules: Vec::new(),
            policy: SourcePolicy::WholeFile,
            has_code: true,
        };
        if !suffix.eq_ignore_ascii_case(".rs") || include_tests {
            return Ok(prepared);
        }

        let mut parser = rust_parser();
        let raw = text.as_bytes();
        let tree = parse(&mut parser, raw).ok_or(Error::RustParse)?;
        if tree.root_node().has_error() {
            return Err(Error::RustParse);
        }
        prepared.policy = SourcePolicy::RustInlineTests;
        let modules = test_modules(tree.root_node(), raw)?;
        if modules.is_empty() {
            return Ok(prepared);
        }
        let spans: Vec<(usize, usize)> = modules.iter().map(|module| module.span).collect();
        let (filtered, line_map) = remove_spans(raw, &spans);
        let filtered_tree = parse(&mut parser, &filtered).ok_or(Error::RustFilter)?;
        if filtered_tree.root_node().has_error() {
            return Err(Error::RustFilter);
        }
        let mut cursor = filtered_tree.walk();
        prepared.has_code = filtered_tree
            .root_node()
            .named_children(&mut cursor)
            .any(|node| !NON_CODE_ITEMS.contains(&node.kind()));
        prepared.text = String::from_utf8(filtered).map_err(|_| Error::RustFilter)?;
        prepared.line_map = line_map;
        prepared.excluded_modules = modules.into_iter().map(|module| module.excluded).collect();
        Ok(prepared)
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// The original 1-based line number of each analyzed line.
    pub fn line_map(&self) -> &[u32] {
        &self.line_map
    }

    pub fn original_lines(&self) -> usize {
        self.original_lines
    }

    pub fn excluded_modules(&self) -> &[ExcludedModule] {
        &self.excluded_modules
    }

    pub fn policy(&self) -> SourcePolicy {
        self.policy
    }

    /// Whether any item other than comments and attributes remains.
    pub fn has_code(&self) -> bool {
        self.has_code
    }

    pub fn info(&self) -> SourceInfo {
        SourceInfo {
            policy: self.policy,
            original_lines: self.original_lines,
            analyzed_lines: self.line_map.len(),
            excluded_lines: self.original_lines - self.line_map.len(),
            excluded_modules: self.excluded_modules.clone(),
        }
    }

    /// The original line ranges covered by analyzed lines `start..=end`,
    /// merged where the original lines are consecutive.
    pub fn ranges(&self, start: u32, end: u32) -> Result<Vec<LineSpan>> {
        let analyzed = line_number(self.line_map.len());
        if !(1 <= start && start <= end && end <= analyzed) {
            return Err(Error::LocationOutOfRange);
        }
        let mut ranges: Vec<LineSpan> = Vec::new();
        for &line in &self.line_map[start as usize - 1..end as usize] {
            match ranges.last_mut() {
                Some(last) if line == last.end_line + 1 => last.end_line = line,
                _ => ranges.push(LineSpan {
                    start_line: line,
                    end_line: line,
                }),
            }
        }
        Ok(ranges)
    }

    /// Rewrites analyzed-line spans into original-line spans, splitting a
    /// span that straddles a removed module.
    pub fn restore_spans(&self, spans: &mut Vec<LineSpan>) -> Result<()> {
        if self.excluded_modules.is_empty() {
            return Ok(());
        }
        let mut restored = Vec::new();
        for span in spans.iter() {
            restored.extend(self.ranges(span.start_line, span.end_line)?);
        }
        *spans = restored;
        Ok(())
    }

    /// Rewrites excerpt bounds into original lines and records the original
    /// ranges each excerpt covers.
    pub fn restore_excerpts(&self, excerpts: &mut [Excerpt]) -> Result<()> {
        if self.excluded_modules.is_empty() {
            return Ok(());
        }
        for excerpt in excerpts.iter_mut() {
            let ranges = self.ranges(excerpt.start_line, excerpt.end_line)?;
            let (Some(first), Some(last)) = (ranges.first(), ranges.last()) else {
                return Err(Error::LocationOutOfRange);
            };
            excerpt.start_line = first.start_line;
            excerpt.end_line = last.end_line;
            excerpt.source_ranges = Some(ranges);
        }
        Ok(())
    }
}

struct TestModule {
    span: (usize, usize),
    excluded: ExcludedModule,
}

fn rust_parser() -> Parser {
    let mut parser = Parser::new();
    parser
        .set_language(&grammar::rust())
        .expect("the pinned Rust grammar matches the tree-sitter runtime");
    parser
}

fn parse(parser: &mut Parser, raw: &[u8]) -> Option<Tree> {
    parser.parse(raw, None)
}

/// Finds inline `test`/`tests` modules in document order without descending
/// into removed modules or macro token trees.
fn test_modules(root: Node, raw: &[u8]) -> Result<Vec<TestModule>> {
    let mut modules = Vec::new();
    let mut pending = vec![root];
    let mut cursor = root.walk();
    while let Some(node) = pending.pop() {
        if let Some(name) = test_module_name(node, raw)? {
            let start = attached_start(node);
            modules.push(TestModule {
                span: (start, node.end_byte()),
                excluded: ExcludedModule {
                    name,
                    start_line: line_number(count_newlines(&raw[..start]) + 1),
                    end_line: line_number(count_newlines(&raw[..node.end_byte()]) + 1),
                    reason: EXCLUDED_MODULE_REASON,
                },
            });
            continue;
        }
        if OPAQUE_NODES.contains(&node.kind()) {
            continue;
        }
        let children: Vec<Node> = node.named_children(&mut cursor).collect();
        pending.extend(children.into_iter().rev());
    }
    Ok(modules)
}

/// The module name when `node` is an inline module named `test` or `tests`.
fn test_module_name(node: Node, raw: &[u8]) -> Result<Option<String>> {
    if node.kind() != "mod_item" || node.child_by_field_name("body").is_none() {
        return Ok(None);
    }
    let Some(name_node) = node.child_by_field_name("name") else {
        return Ok(None);
    };
    let name = name_node.utf8_text(raw).map_err(|_| Error::RustParse)?;
    let name = name.strip_prefix("r#").unwrap_or(name);
    if TEST_MODULE_NAMES.contains(&name) {
        Ok(Some(name.to_owned()))
    } else {
        Ok(None)
    }
}

/// The start of the module's outer attributes and docs, without consuming
/// other items. Ordinary comments between attributes and the item are
/// included by the enclosing byte span; comments before the first attribute
/// stay intact.
fn attached_start(node: Node) -> usize {
    let mut start = node.start_byte();
    let mut previous = node.prev_named_sibling();
    while let Some(sibling) = previous {
        let is_comment = COMMENTS.contains(&sibling.kind());
        let is_outer_doc = is_comment && sibling.child_by_field_name("outer").is_some();
        if sibling.kind() == "attribute_item" || is_outer_doc {
            start = sibling.start_byte();
        } else if !is_comment {
            break;
        }
        previous = sibling.prev_named_sibling();
    }
    start
}

/// Removes the byte spans and the lines they empty, keeping every other
/// line's original number. A space replaces each removed span so adjacent
/// tokens do not join.
fn remove_spans(raw: &[u8], spans: &[(usize, usize)]) -> (Vec<u8>, Vec<u32>) {
    let mut output = Vec::with_capacity(raw.len());
    let mut line_map = Vec::new();
    let mut offset = 0;
    let mut index = 0;
    for (position, line) in split_bytes_keepends(raw).into_iter().enumerate() {
        let end = offset + line.len();
        while index < spans.len() && spans[index].1 <= offset {
            index += 1;
        }
        let mut cursor = offset;
        let mut kept = Vec::new();
        let mut touched = false;
        let mut current = index;
        while current < spans.len() && spans[current].0 < end {
            let (start, stop) = spans[current];
            kept.extend_from_slice(&raw[cursor..cursor.max(start)]);
            kept.push(b' ');
            cursor = end.min(stop);
            touched = true;
            current += 1;
        }
        kept.extend_from_slice(&raw[cursor..end]);
        if !touched || !kept.iter().all(|&byte| is_ascii_python_whitespace(byte)) {
            if line.ends_with(b"\n") && !kept.ends_with(b"\n") {
                kept.push(b'\n');
            }
            output.extend_from_slice(&kept);
            line_map.push(line_number(position + 1));
        }
        offset = end;
    }
    (output, line_map)
}

/// `bytes.splitlines(keepends=True)`: only `\n`, `\r`, and `\r\n` end lines.
fn split_bytes_keepends(raw: &[u8]) -> Vec<&[u8]> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut index = 0;
    while index < raw.len() {
        let byte = raw[index];
        index += 1;
        if byte == b'\n' || byte == b'\r' {
            if byte == b'\r' && raw.get(index) == Some(&b'\n') {
                index += 1;
            }
            lines.push(&raw[start..index]);
            start = index;
        }
    }
    if start < raw.len() {
        lines.push(&raw[start..]);
    }
    lines
}

/// Whether `bytes.strip()` removes `byte`.
fn is_ascii_python_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b'\x0b' | b'\x0c')
}

fn count_newlines(raw: &[u8]) -> usize {
    raw.iter().filter(|&&byte| byte == b'\n').count()
}

fn line_number(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prepare(text: &str) -> PreparedSource {
        PreparedSource::prepare(text, ".rs", false).unwrap()
    }

    #[test]
    fn removes_nested_modules_with_attached_attributes_and_docs() {
        let text = "//! crate docs
pub mod production {
    pub fn before() -> &'static str { r###\"mod tests { }\"### }
    /// Unit tests, with Unicode: λ
    #[cfg(all(test, feature = \"testing\"))]
    #[allow(dead_code)]
    pub(crate) mod tests {
        /* nested /* } */ comment */
        const TEXT: &str = r##\"} mod test {\"##;
        #[test] fn example() { assert_eq!('}', '}'); }
        mod test { fn inner() {} }
    }
    pub fn after() {}
}
mod test { fn another() {} }
pub fn last() {}
";
        let prepared = prepare(text);
        assert_eq!(
            prepared.text(),
            "//! crate docs
pub mod production {
    pub fn before() -> &'static str { r###\"mod tests { }\"### }
    pub fn after() {}
}
pub fn last() {}
"
        );
        assert_eq!(prepared.line_map(), [1, 2, 3, 13, 14, 16]);
    }

    #[test]
    fn reports_excluded_modules_with_original_lines() {
        let prepared = prepare("fn a() {}\n#[cfg(test)]\nmod tests {\n}\nmod r#test {}\n");
        let modules: Vec<(&str, u32, u32)> = prepared
            .excluded_modules()
            .iter()
            .map(|module| (module.name.as_str(), module.start_line, module.end_line))
            .collect();
        assert_eq!(modules, [("tests", 2, 4), ("test", 5, 5)]);
    }

    #[test]
    fn info_counts_excluded_lines() {
        let prepared = prepare("fn a() {}\nmod tests {\nfn x() {}\n}\nfn b() {}\n");
        let info = serde_json::to_value(prepared.info()).unwrap();
        assert_eq!(info["policy"], "rust-inline-tests-001");
        assert_eq!(info["original_lines"], 5);
        assert_eq!(info["analyzed_lines"], 2);
        assert_eq!(info["excluded_lines"], 3);
        assert_eq!(
            info["excluded_modules"][0]["reason"],
            EXCLUDED_MODULE_REASON
        );
    }

    #[test]
    fn preserves_macros_comments_external_modules_and_similar_names() {
        let text = "// mod tests { this is a comment }
macro_rules! fixture { () => { mod tests { fn generated() {} } }; }
fixture! { mod test { fn supplied() {} } }
mod tests;
mod test;
mod test_support { pub fn helper() {} }
mod testsuite { pub fn run() {} }
";
        let prepared = prepare(text);
        assert_eq!(prepared.text(), text);
        assert!(prepared.excluded_modules().is_empty());
    }

    #[test]
    fn same_line_removal_keeps_surrounding_code() {
        let text =
            "#[allow(dead_code)] fn keep() {} #[cfg(test)] mod tests { fn gone() {} } fn later() {}
mod parent { mod r#test { fn gone() {} } pub fn keep() {} }
fn final_item() {}
";
        let prepared = prepare(text);
        assert_eq!(
            prepared.text(),
            "#[allow(dead_code)] fn keep() {}   fn later() {}
mod parent {   pub fn keep() {} }
fn final_item() {}
"
        );
        assert_eq!(prepared.line_map(), [1, 2, 3]);
    }

    #[test]
    fn multiline_removal_inside_a_line_keeps_both_ends() {
        let prepared = prepare("fn first() {} mod tests {\nfn test_fn() {}\n} fn second() {}\n");
        assert_eq!(prepared.text(), "fn first() {}  \n  fn second() {}\n");
        assert_eq!(prepared.line_map(), [1, 3]);
        assert!(prepared.has_code());
    }

    #[test]
    fn test_only_file_has_no_code() {
        let prepared = prepare("//! docs\n#![allow(dead_code)]\nmod test { fn x() {} }\n");
        assert!(!prepared.has_code());
    }

    #[test]
    fn include_tests_keeps_the_exact_source() {
        let text = "#[cfg(test)] mod tests { fn x() {} }\n";
        let prepared = PreparedSource::prepare(text, ".rs", true).unwrap();
        assert_eq!(prepared.text(), text);
        assert_eq!(prepared.policy(), SourcePolicy::WholeFile);
    }

    #[test]
    fn other_languages_are_analyzed_whole() {
        let prepared =
            PreparedSource::prepare("def test_example():\n    assert True\n", ".py", false)
                .unwrap();
        assert_eq!(prepared.policy(), SourcePolicy::WholeFile);
        assert_eq!(prepared.line_map(), [1, 2]);
    }

    #[test]
    fn suffix_match_ignores_case() {
        let prepared = PreparedSource::prepare("mod tests {}\n", ".RS", false).unwrap();
        assert_eq!(prepared.policy(), SourcePolicy::RustInlineTests);
    }

    #[test]
    fn invalid_rust_fails_instead_of_deleting_uncertain_code() {
        let error = PreparedSource::prepare("mod tests { fn broken(", ".rs", false).unwrap_err();
        assert!(matches!(error, Error::RustParse));
    }

    #[test]
    fn large_test_module_line_numbers() {
        let text = format!(
            "fn keep() {{}}\n#[cfg(test)]\nmod tests {{\n{}}}\nfn after() {{}}\n",
            "    // filler\n".repeat(400)
        );
        let prepared = prepare(&text);
        assert_eq!(prepared.text(), "fn keep() {}\nfn after() {}\n");
        assert_eq!(prepared.excluded_modules()[0].end_line, 404);
        assert_eq!(prepared.line_map(), [1, 405]);
    }

    #[test]
    fn ranges_merge_consecutive_original_lines() {
        let prepared = prepare("fn a() {}\nfn b() {}\nmod tests {\n}\nfn c() {}\n");
        assert_eq!(
            prepared.ranges(1, 3).unwrap(),
            [
                LineSpan {
                    start_line: 1,
                    end_line: 2
                },
                LineSpan {
                    start_line: 5,
                    end_line: 5
                }
            ]
        );
    }

    #[test]
    fn ranges_reject_locations_outside_the_analyzed_code() {
        let prepared = prepare("fn a() {}\nmod tests {\nfn x() {}\n}\nfn b() {}\n");
        assert!(matches!(
            prepared.ranges(1, 3).unwrap_err(),
            Error::LocationOutOfRange
        ));
    }

    #[test]
    fn restore_spans_expands_into_original_ranges() {
        let prepared = prepare("fn a() {}\nmod tests {\nfn x() {}\n}\nfn b() {}\n");
        let mut spans = vec![LineSpan {
            start_line: 1,
            end_line: 2,
        }];
        prepared.restore_spans(&mut spans).unwrap();
        assert_eq!(
            spans,
            [
                LineSpan {
                    start_line: 1,
                    end_line: 1
                },
                LineSpan {
                    start_line: 5,
                    end_line: 5
                }
            ]
        );
    }

    #[test]
    fn restore_excerpts_sets_bounds_and_source_ranges() {
        let prepared = prepare("fn a() {}\nmod tests {\nfn x() {}\n}\nfn b() {}\n");
        let mut excerpts = vec![Excerpt {
            start_line: 1,
            end_line: 2,
            value: 0.5,
            weight_bytes: 20,
            source_ranges: None,
        }];
        prepared.restore_excerpts(&mut excerpts).unwrap();
        let excerpt = serde_json::to_value(&excerpts[0]).unwrap();
        assert_eq!(
            excerpt,
            serde_json::json!({
                "start_line": 1,
                "end_line": 5,
                "value": 0.5,
                "weight_bytes": 20,
                "source_ranges": [
                    {"start_line": 1, "end_line": 1},
                    {"start_line": 5, "end_line": 5}
                ]
            })
        );
    }

    #[test]
    fn restoration_is_a_no_op_without_excluded_modules() {
        let prepared = prepare("fn a() {}\nfn b() {}\n");
        let mut spans = vec![LineSpan {
            start_line: 1,
            end_line: 2,
        }];
        prepared.restore_spans(&mut spans).unwrap();
        assert_eq!(
            spans,
            [LineSpan {
                start_line: 1,
                end_line: 2
            }]
        );
    }

    fn write_source(bytes: &[u8]) -> tempfile::NamedTempFile {
        let file = tempfile::NamedTempFile::new().unwrap();
        fs::write(file.path(), bytes).unwrap();
        file
    }

    #[test]
    fn source_text_strips_the_bom_and_normalizes_line_endings() {
        let file = write_source(b"\xEF\xBB\xBFa\r\nb\rc\n");
        assert_eq!(source_text(file.path()).unwrap(), "a\nb\nc\n");
    }

    #[test]
    fn source_text_rejects_directories() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            source_text(dir.path()).unwrap_err(),
            Error::NotASourceFile(_)
        ));
    }

    #[test]
    fn source_text_rejects_missing_files() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            source_text(&dir.path().join("missing.rs")).unwrap_err(),
            Error::NotASourceFile(_)
        ));
    }

    #[test]
    fn source_text_rejects_files_over_16_mib() {
        let file = tempfile::NamedTempFile::new().unwrap();
        file.as_file().set_len(MAX_SOURCE_BYTES + 1).unwrap();
        assert!(matches!(
            source_text(file.path()).unwrap_err(),
            Error::SourceTooLarge(_)
        ));
    }

    #[test]
    fn source_text_rejects_invalid_utf8() {
        let file = write_source(b"fn main() {} \xff\n");
        assert!(matches!(
            source_text(file.path()).unwrap_err(),
            Error::SourceNotUtf8(_)
        ));
    }

    #[test]
    fn source_text_rejects_nul_bytes() {
        let file = write_source(b"fn main() {}\x00\n");
        assert!(matches!(
            source_text(file.path()).unwrap_err(),
            Error::SourceEmptyOrBinary(_)
        ));
    }

    #[test]
    fn source_text_rejects_whitespace_only_input() {
        let file = write_source(" \t\n\x1c\u{a0}\u{2028}".as_bytes());
        assert!(matches!(
            source_text(file.path()).unwrap_err(),
            Error::SourceEmptyOrBinary(_)
        ));
    }

    #[test]
    fn source_text_keeps_a_second_bom_as_content() {
        let file = write_source("\u{feff}\u{feff}".as_bytes());
        assert_eq!(source_text(file.path()).unwrap(), "\u{feff}");
    }

    #[test]
    fn source_text_from_bytes_normalizes_like_a_file() {
        assert_eq!(
            source_text_from_bytes(Path::new("blob.py"), b"\xEF\xBB\xBFa\r\nb\rc\n").unwrap(),
            "a\nb\nc\n"
        );
    }

    #[test]
    fn source_text_from_bytes_rejects_binary_data() {
        assert!(matches!(
            source_text_from_bytes(Path::new("blob.py"), b"x\x00").unwrap_err(),
            Error::SourceEmptyOrBinary(_)
        ));
    }

    #[test]
    fn split_bytes_keepends_recognizes_cr_and_crlf() {
        assert_eq!(
            split_bytes_keepends(b"a\r\nb\rc\nd"),
            [&b"a\r\n"[..], b"b\r", b"c\n", b"d"]
        );
    }

    #[test]
    fn python_whitespace_includes_separator_controls() {
        assert!(is_python_whitespace('\x1c'));
        assert!(!is_python_whitespace('\u{feff}'));
    }
}
