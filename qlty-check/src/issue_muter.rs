use crate::source_reader::SourceReader;
use qlty_analysis::{
    code::{all_captured_nodes, File},
    lang,
    utils::filename_to_language,
    Language,
};
use qlty_config::issue_transformer::IssueTransformer;
use qlty_types::analysis::v1::Issue;
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tracing::{debug, trace};

#[derive(Debug)]
pub struct IssueMuter {
    source_reader: Arc<dyn SourceReader>,
    files: RwLock<HashMap<PathBuf, Box<IgnoreParser>>>,
}

impl Clone for IssueMuter {
    fn clone(&self) -> Self {
        Self {
            source_reader: self.source_reader.clone(),
            files: RwLock::new(self.files.read().unwrap().clone()),
        }
    }
}

impl IssueTransformer for IssueMuter {
    fn transform(&self, issue: Issue) -> Option<Issue> {
        self.parse_issue_file(&issue);

        if let Some(ref location) = issue.location {
            let map = self.files.read().unwrap();
            if let Some(file) = map.get::<PathBuf>(&issue.path().unwrap_or_default().into()) {
                if let Some(range) = location.range() {
                    if Self::rule_key_is_ignored(
                        file,
                        &issue.tool,
                        &issue.rule_key,
                        range.start_line as usize,
                    ) {
                        return None;
                    }
                }
            }
        }

        Some(issue)
    }

    fn clone_box(&self) -> Box<dyn IssueTransformer> {
        Box::new(self.clone())
    }
}

impl IssueMuter {
    pub fn new(source_reader: impl SourceReader + Clone + 'static) -> Self {
        Self {
            source_reader: Arc::new(source_reader),
            files: RwLock::new(HashMap::new()),
        }
    }

    fn rule_key_is_ignored(parser: &IgnoreParser, tool: &str, rule_key: &str, line: usize) -> bool {
        parser.ignore_rule_at_line(line, tool.to_string())
            || parser.ignore_rule_at_line(line, format!("{}/{}", tool, rule_key))
            || parser.ignore_rule_at_line(line, format!("{}:{}", tool, rule_key))
    }

    fn parse_issue_file(&self, issue: &Issue) {
        if issue.path().is_none() {
            return;
        }

        let path = PathBuf::from(issue.path().unwrap());
        let mut files = self.files.write().unwrap();
        if files.contains_key(&path) {
            return; // file is already parsed, skip reparsing
        }

        let language = filename_to_language(path.to_str().unwrap()).unwrap_or_default();
        if let Ok(source) = self.source_reader.read(path.clone()) {
            files.insert(path.clone(), Box::new(IgnoreParser::new(source, language)));
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Comment {
    is_full_line: bool,
    lines: usize,
    ignore_directive_rules: HashSet<RuleSpecifier>,
}

#[derive(Debug, Default, Clone)]
struct IgnoreParser {
    lines: HashMap<usize, HashSet<String>>,
    enabled_rules: HashSet<String>,
    once_add_rules: HashSet<String>,
    once_remove_rules: HashSet<String>,
    matching_indent_rules: HashSet<(String, usize, bool)>,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
enum RuleSpecifier {
    Enable(String),                    // `+` prefix
    Disable(String),                   // `-` prefix
    IgnoreUntilMatchingIndent(String), // non-prefixed
    IgnoreNext(String),                // `>` prefix
}

impl IgnoreParser {
    fn new(source: String, language: String) -> Self {
        let mut parser = IgnoreParser::default();
        parser.parse(source, language);
        parser
    }

    fn parse(&mut self, source: String, language: String) {
        let comments = Self::extract_comment_nodes(&source, &language);
        let lines: Vec<_> = source.lines().collect();

        self.enabled_rules.clear();
        self.once_add_rules.clear();

        let mut index = 0;
        while index < lines.len() {
            let comment = comments.get(&index);
            self.parse_update_state(comment, lines[index]);
            self.parse_apply_rules(comment, lines[index], index);

            if let Some(comment) = comment {
                index += comment.lines;
            } else {
                index += 1;
            }
        }
    }

    fn parse_update_state(&mut self, comment: Option<&Comment>, line: &str) {
        if let Some(comment) = comment {
            for rule_specifier in &comment.ignore_directive_rules {
                if comment.is_full_line {
                    match rule_specifier {
                        RuleSpecifier::Enable(rule) => {
                            self.enabled_rules.insert(rule.clone());
                        }
                        RuleSpecifier::Disable(rule) => {
                            self.enabled_rules.remove(rule);
                        }
                        RuleSpecifier::IgnoreNext(rule) => {
                            self.once_add_rules.insert(rule.clone());
                        }
                        RuleSpecifier::IgnoreUntilMatchingIndent(rule) => {
                            self.matching_indent_rules.insert((
                                rule.clone(),
                                Self::count_indent(line),
                                false,
                            ));
                        }
                    }
                } else {
                    match rule_specifier {
                        RuleSpecifier::Enable(rule) => {
                            self.once_add_rules.insert(rule.clone());
                        }
                        RuleSpecifier::Disable(rule) => {
                            self.once_remove_rules.insert(rule.clone());
                        }
                        RuleSpecifier::IgnoreNext(rule) => {
                            self.once_add_rules.insert(rule.clone());
                        }
                        RuleSpecifier::IgnoreUntilMatchingIndent(rule) => {
                            self.once_add_rules.insert(rule.clone());
                        }
                    }
                }
            }
        }
    }

    fn parse_apply_rules(&mut self, comment: Option<&Comment>, line: &str, index: usize) {
        let clear_rules_after_use = match comment {
            Some(comment) => !comment.is_full_line,
            None => true,
        };
        if line.trim().is_empty() {
            return; // never apply rules to empty lines
        }

        // line contains code, apply any enabled rules to this line
        // also apply all queued rules (once_rules) and discard
        let line_indent = Self::count_indent(line);
        let new_matching_indent_rules = self
            .matching_indent_rules
            .iter()
            .flat_map(|(rule, specifier_indent, remove_next)| {
                let should_remove_next = line_indent <= *specifier_indent;
                if should_remove_next && *remove_next {
                    None
                } else {
                    Some((rule.clone(), *specifier_indent, should_remove_next))
                }
            })
            .collect::<Vec<_>>();
        if clear_rules_after_use {
            self.matching_indent_rules.clear();
        }
        self.matching_indent_rules.extend(new_matching_indent_rules);

        let mut rules: HashSet<_> = self
            .enabled_rules
            .iter()
            .chain(self.once_add_rules.iter())
            .chain(self.matching_indent_rules.iter().map(|(rule, _, _)| rule))
            .cloned()
            .collect();
        for rule in &self.once_remove_rules {
            if clear_rules_after_use {
                rules.remove(rule);
            }
        }
        if !rules.is_empty() {
            let adjusted_line = index + 1;
            trace!("Applying rules to line {}: {:?}", adjusted_line, rules);
            self.lines.insert(adjusted_line, rules);
        }

        if clear_rules_after_use {
            self.once_add_rules.clear();
            self.once_remove_rules.clear();
        }
    }

    fn extract_comment_nodes(source: &str, language: &str) -> HashMap<usize, Comment> {
        if let Some(language) = lang::from_str(language) {
            Self::extract_comment_nodes_tree_sitter(source, language.deref())
        } else {
            Self::extract_comment_nodes_from_unknown_language(source)
        }
    }

    fn extract_comment_nodes_tree_sitter(
        source: &str,
        language: &dyn Language,
    ) -> HashMap<usize, Comment> {
        let lines: Vec<_> = source.lines().collect();
        let file = File::from_string(language.name(), source);
        let tree = file.parse();
        let root_node = tree.root_node();
        let query = format!(
            "[{}]",
            language
                .comment_nodes()
                .iter()
                .map(|c| format!("({})", c))
                .collect::<Vec<_>>()
                .join(" ")
        );

        all_captured_nodes(query.as_str(), &file, &root_node)
            .iter()
            .map(|comment| {
                let range = comment.range();
                let text = comment.utf8_text(source.as_bytes()).unwrap().to_string();
                let is_full_line = text
                    .lines()
                    .any(|text_line| lines[range.start_point.row].trim() == text_line.trim());

                let mut ignore_directive_rules = HashSet::<_>::new();
                for line in text.lines() {
                    if let Some(rules) = Self::extract_ignored_rules(line) {
                        debug!("Found qlty-ignore directive: {:?}", rules);
                        ignore_directive_rules.extend(rules)
                    }
                }

                (
                    range.start_point.row,
                    Comment {
                        is_full_line,
                        lines: text.lines().count(),
                        ignore_directive_rules,
                    },
                )
            })
            .collect()
    }

    // Fallback for unknown languages. This simple parser only supports full-line single-line comments.
    fn extract_comment_nodes_from_unknown_language(source: &str) -> HashMap<usize, Comment> {
        source
            .lines()
            .enumerate()
            .flat_map(|(index, line)| {
                Regex::new(r#"^(?://|#).*|/\*.*\*/$"#)
                    .unwrap()
                    .captures(line.trim())
                    .map(|_| {
                        (
                            index,
                            Comment {
                                is_full_line: true,
                                lines: 1,
                                ignore_directive_rules: Self::extract_ignored_rules(line)
                                    .unwrap_or_default(),
                            },
                        )
                    })
            })
            .collect()
    }

    fn ignore_rule_at_line(&self, line: usize, rule: String) -> bool {
        if let Some(rules) = self.lines.get(&line) {
            if rules.contains(&rule) {
                return true;
            }
        }

        false
    }

    fn extract_rule(raw_rule: String) -> RuleSpecifier {
        let rule = raw_rule.trim();
        if let Some(rule) = rule.strip_prefix('+') {
            RuleSpecifier::Enable(rule.into())
        } else if let Some(rule) = rule.strip_prefix('-') {
            RuleSpecifier::Disable(rule.into())
        } else if let Some(rule) = rule.strip_prefix('>') {
            RuleSpecifier::IgnoreNext(rule.into())
        } else {
            RuleSpecifier::IgnoreUntilMatchingIndent(rule.into())
        }
    }

    fn extract_ignored_rules(line: &str) -> Option<HashSet<RuleSpecifier>> {
        Regex::new(r#"^(?:/?\*+|/+|#+)\s*qlty-ignore(?:\((.*)\)|(?::\s*|\s+)?(.*?)(?:\*/)?$)"#)
            .unwrap()
            .captures(line.trim())
            .map(|c| {
                c.get(1)
                    .unwrap_or_else(|| c.get(2).expect("no rule found"))
                    .as_str()
                    .split([' ', ','])
                    .filter(|s| !s.is_empty())
                    .map(|s| Self::extract_rule(s.to_string()))
                    .collect()
            })
    }

    fn count_indent(line: &str) -> usize {
        line.chars().take_while(|c| c.is_whitespace()).count()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::source_reader::SourceReaderFs;
    use itertools::Itertools;
    use qlty_config::issue_transformer::IssueTransformer;
    use std::collections::{HashMap, HashSet};

    fn parser_rules<'a>(parser: &'a IgnoreParser) -> Vec<(usize, Vec<&'a str>)> {
        parser
            .lines
            .iter()
            .map(|(k, v)| (*k, v.iter().map(|s| s.as_str()).sorted().collect()))
            .sorted_by(|(a, _), (b, _)| a.cmp(b))
            .collect()
    }

    fn make_issue(rule_key: &str, line: u32) -> qlty_types::analysis::v1::Issue {
        qlty_types::analysis::v1::Issue {
            tool: "clippy".into(),
            rule_key: rule_key.into(),
            language: qlty_types::analysis::v1::Language::Rust.into(),
            location: Some(qlty_types::analysis::v1::Location {
                path: "example.rs".into(),
                range: Some(qlty_types::analysis::v1::Range {
                    start_line: line,
                    ..Default::default()
                }),
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_ignore_parser_extract_ignored_rules() {
        fn run(rules: &str) -> Option<HashSet<RuleSpecifier>> {
            IgnoreParser::extract_ignored_rules(rules)
        }

        let rules = Some(HashSet::from([
            RuleSpecifier::IgnoreUntilMatchingIndent("R1".into()),
            RuleSpecifier::Enable("R2".into()),
            RuleSpecifier::Disable("R3".into()),
            RuleSpecifier::IgnoreNext("R4".into()),
        ]));

        assert_eq!(run(r#"// not-qlty-ignore: rule"#), None);
        assert_eq!(run(r#"// qlty-ignore: R1,+R2, -R3   >R4"#), rules);
        assert_eq!(run(r#"/*qlty-ignore:R1,+R2,-R3,>R4*/"#), rules);
        assert_eq!(run(r#"*** qlty-ignore: R1,+R2,-R3,>R4"#), rules);
        assert_eq!(run(r#"# qlty-ignore: R1,+R2, -R3   >R4*/"#), rules);
        assert_eq!(run(r#"### qlty-ignore: R1,+R2, -R3   >R4*/"#), rules);
        assert_eq!(
            run(r#"///qlty-ignore(R1,+R2, -R3   >R4): explanation"#),
            rules
        );
    }

    #[test]
    fn test_ignore_parser_rust() {
        let source = indoc::indoc! {r#"
            /// qlty-ignore: +rule1, rule2, -rule8
            /**
             ** qlty-ignore(>rule3,+rule5): explanation
             **/
            fn example() {
                let x = 1; // qlty-ignore: rule6, -rule1, +rule8
                // qlty-ignore: rule4, -rule1
                let str = "
                // qlty-ignore: +rule7
                ";
                return;
            }
            struct X { }
        "#};

        let parser = IgnoreParser::new(source.to_string(), "rust".into());
        assert_eq!(
            parser_rules(&parser),
            vec![
                (1, vec!["rule1", "rule2"]),
                (2, vec!["rule1", "rule2", "rule3", "rule5"]),
                (5, vec!["rule1", "rule2", "rule3", "rule5"]),
                (6, vec!["rule2", "rule5", "rule6", "rule8"]),
                (7, vec!["rule2", "rule4", "rule5"]),
                (8, vec!["rule2", "rule4", "rule5"]),
                (9, vec!["rule2", "rule5"]),
                (10, vec!["rule2", "rule5"]),
                (11, vec!["rule2", "rule5"]),
                (12, vec!["rule2", "rule5"]),
                (13, vec!["rule5"]),
            ]
        );

        assert_eq!(parser.ignore_rule_at_line(5, "rule1".into()), true);
        assert_eq!(parser.ignore_rule_at_line(5, "rule8".into()), false);
    }

    #[test]
    fn test_ignore_parser_ruby() {
        let source = indoc::indoc! {r#"
            ## qlty-ignore: +rule1, rule2, -rule8
            #
            # qlty-ignore(>rule3,+rule5): explanation
            #
            def example
                puts "hi" # qlty-ignore: rule6, -rule1, +rule8
                # qlty-ignore: rule4, -rule1
                let str = "
                # qlty-ignore: +rule7
                ";
                return
            end
            def other; end
        "#};

        let parser = IgnoreParser::new(source.to_string(), "ruby".into());
        assert_eq!(
            parser_rules(&parser),
            vec![
                (1, vec!["rule1", "rule2"]),
                (2, vec!["rule1", "rule2"]),
                (3, vec!["rule1", "rule2", "rule3", "rule5"]),
                (4, vec!["rule1", "rule2", "rule3", "rule5"]),
                (5, vec!["rule1", "rule2", "rule3", "rule5"]),
                (6, vec!["rule2", "rule5", "rule6", "rule8"]),
                (7, vec!["rule2", "rule4", "rule5"]),
                (8, vec!["rule2", "rule4", "rule5"]),
                (9, vec!["rule2", "rule5"]),
                (10, vec!["rule2", "rule5"]),
                (11, vec!["rule2", "rule5"]),
                (12, vec!["rule2", "rule5"]),
                (13, vec!["rule5"]),
            ]
        );

        assert_eq!(parser.ignore_rule_at_line(5, "rule1".into()), true);
        assert_eq!(parser.ignore_rule_at_line(5, "rule8".into()), false);
    }

    #[test]
    fn test_ignore_parser_unknown_language_shell() {
        let source = indoc::indoc! {r#"
            # qlty-ignore: rule1
            // qlty-ignore(rule2): explanation
            /* qlty-ignore: rule3 */
            echo "hi" # qlty-ignore: rule4
            # qlty-ignore: rule5
            if [ -f "file" ]; then
              echo "file exists"
            fi
            echo "done"
        "#};

        let parser = IgnoreParser::new(source.to_string(), "shell".into());
        assert_eq!(
            parser_rules(&parser),
            vec![
                (1, vec!["rule1"]),
                (2, vec!["rule1", "rule2"]),
                (3, vec!["rule1", "rule2", "rule3"]),
                (4, vec!["rule1", "rule2", "rule3"]),
                (5, vec!["rule1", "rule2", "rule3", "rule5"]),
                (6, vec!["rule5"]),
                (7, vec!["rule5"]),
                (8, vec!["rule5"])
            ]
        );
    }

    #[test]
    fn test_issue_muter() {
        let source_reader = SourceReaderFs::with_cache(HashMap::from([(
            "example.rs".into(),
            indoc::indoc! {r#"
            // qlty-ignore: +clippy
            fn example() {
                let issueL3 = true;
                let issueL4 = true;

                // qlty-ignore(-clippy)

                let issueL8 = true;
                let issueL9 = true; // qlty-ignore: clippy/special
                let issueL10 = true; // qlty-ignore: clippy:special
                let issueL11 = true;
            }
        "#}
            .into(),
        )]));

        let ignorer = IssueMuter::new(source_reader);
        assert_eq!(ignorer.transform(make_issue("any_rule", 3)), None);
        assert_eq!(ignorer.transform(make_issue("any_other_rule", 4)), None);
        assert_ne!(ignorer.transform(make_issue("any_rule", 8)), None);
        assert_eq!(ignorer.transform(make_issue("special", 9)), None);
        assert_eq!(ignorer.transform(make_issue("special", 10)), None);
        assert_ne!(ignorer.transform(make_issue("special", 11)), None);
    }

    #[test]
    fn test_issue_muter_unknown_source_language() {
        let source_reader = SourceReaderFs::with_cache(HashMap::from([(
            "example.unknown".into(),
            "A\n# qlty-ignore: clippy\nB".into(),
        )]));

        let ignorer = IssueMuter::new(source_reader);

        let mut issue = make_issue("any_rule", 1);
        issue.location.as_mut().unwrap().path = "example.unknown".into();
        assert_eq!(ignorer.transform(issue.clone()), Some(issue));

        let mut issue = make_issue("any_rule", 2);
        issue.location.as_mut().unwrap().path = "example.unknown".into();
        assert_eq!(ignorer.transform(issue), None);

        let mut issue = make_issue("any_rule", 3);
        issue.location.as_mut().unwrap().path = "example.unknown".into();
        assert_eq!(ignorer.transform(issue), None);
    }
}
