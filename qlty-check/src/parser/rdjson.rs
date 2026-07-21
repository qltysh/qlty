use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{
    Category, Issue, Level, Location, Range, Replacement, Suggestion, SuggestionSource,
};
use serde::{Deserialize, Serialize};

// Reviewdog Diagnostic Format (rdformat):
// https://github.com/reviewdog/reviewdog/tree/master/proto/rdf
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RdjsonOutput {
    #[serde(default)]
    pub severity: Option<String>,
    pub diagnostics: Vec<RdjsonDiagnostic>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RdjsonDiagnostic {
    pub message: String,
    pub location: RdjsonLocation,
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub code: Option<RdjsonCode>,
    #[serde(default)]
    pub suggestions: Vec<RdjsonSuggestion>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RdjsonCode {
    pub value: String,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RdjsonLocation {
    pub path: String,
    #[serde(default)]
    pub range: Option<RdjsonRange>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RdjsonRange {
    pub start: RdjsonPosition,
    #[serde(default)]
    pub end: Option<RdjsonPosition>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RdjsonPosition {
    pub line: u32,
    #[serde(default)]
    pub column: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RdjsonSuggestion {
    pub range: RdjsonRange,
    pub text: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Rdjson;

impl Parser for Rdjson {
    fn parse(&self, plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let rdjson_output: RdjsonOutput = serde_json::from_str(output)?;
        let default_severity = rdjson_output.severity;

        Ok(rdjson_output
            .diagnostics
            .into_iter()
            .map(|diagnostic| {
                let severity = diagnostic
                    .severity
                    .as_deref()
                    .or(default_severity.as_deref());

                Issue {
                    tool: plugin_name.into(),
                    rule_key: diagnostic
                        .code
                        .as_ref()
                        .map(|code| code.value.clone())
                        .unwrap_or_default(),
                    message: diagnostic.message.clone(),
                    documentation_url: diagnostic
                        .code
                        .as_ref()
                        .and_then(|code| code.url.clone())
                        .unwrap_or_default(),
                    category: Category::Lint.into(),
                    level: severity_to_level(severity).into(),
                    location: Some(Location {
                        path: diagnostic.location.path.clone(),
                        range: diagnostic.location.range.as_ref().map(build_range),
                    }),
                    suggestions: build_suggestions(
                        &diagnostic.suggestions,
                        &diagnostic.location.path,
                    ),
                    ..Default::default()
                }
            })
            .collect())
    }
}

fn build_range(range: &RdjsonRange) -> Range {
    let end = range.end.as_ref().unwrap_or(&range.start);

    Range {
        start_line: range.start.line,
        start_column: range.start.column.unwrap_or(0),
        end_line: end.line,
        end_column: end.column.unwrap_or(0),
        ..Default::default()
    }
}

// Rdformat does not carry fix applicability, and tools may include fixes
// they consider unsafe (Biome does), so suggestions are marked unsafe and
// only apply with --unsafe.
fn build_suggestions(suggestions: &[RdjsonSuggestion], path: &str) -> Vec<Suggestion> {
    suggestions
        .iter()
        .map(|suggestion| Suggestion {
            source: SuggestionSource::Tool.into(),
            r#unsafe: true,
            replacements: vec![Replacement {
                data: suggestion.text.clone(),
                location: Some(Location {
                    path: path.into(),
                    range: Some(build_range(&suggestion.range)),
                }),
            }],
            ..Default::default()
        })
        .collect()
}

fn severity_to_level(severity: Option<&str>) -> Level {
    match severity {
        Some("ERROR") => Level::High,
        Some("WARNING") => Level::Medium,
        Some("INFO") => Level::Low,
        _ => Level::Medium,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = r###"
        {
            "source": { "name": "Biome", "url": "https://biomejs.dev" },
            "diagnostics": [
                {
                "code": {
                    "url": "https://biomejs.dev/linter/rules/no-unused-variables",
                    "value": "lint/correctness/noUnusedVariables"
                },
                "location": {
                    "path": "basic.in.ts",
                    "range": {
                    "end": { "column": 10, "line": 6 },
                    "start": { "column": 7, "line": 6 }
                    }
                },
                "message": "This variable foo is unused.",
                "suggestions": [
                    {
                    "range": {
                        "end": { "column": 9, "line": 13 },
                        "start": { "column": 7, "line": 6 }
                    },
                    "text": "_foo = (bar: Bar) => {\n  switch (bar) {\n    case Bar.Baz:\n      foobar();\n      barfoo();\n      break;\n  }\n  { !_foo"
                    }
                ]
                },
                {
                "code": {
                    "url": "https://biomejs.dev/linter/rules/no-unused-variables",
                    "value": "lint/correctness/noUnusedVariables"
                },
                "location": {
                    "path": "basic.in.ts",
                    "range": {
                    "end": { "column": 9, "line": 16 },
                    "start": { "column": 6, "line": 16 }
                    }
                },
                "message": "This variable Foo is unused."
                },
                {
                "code": {
                    "url": "https://biomejs.dev/linter/rules/use-enum-initializers",
                    "value": "lint/style/useEnumInitializers"
                },
                "location": {
                    "path": "basic.in.ts",
                    "range": {
                    "end": { "column": 9, "line": 4 },
                    "start": { "column": 6, "line": 4 }
                    }
                },
                "message": "This enum declaration contains members that are implicitly initialized.",
                "suggestions": [
                    {
                    "range": {
                        "end": { "column": 16, "line": 4 },
                        "start": { "column": 16, "line": 4 }
                    },
                    "text": "= 0 "
                    }
                ]
                }
            ]
        }
        "###;

        let issues = Rdjson::default().parse("biome", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r###"
        - tool: biome
          ruleKey: lint/correctness/noUnusedVariables
          message: This variable foo is unused.
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://biomejs.dev/linter/rules/no-unused-variables"
          location:
            path: basic.in.ts
            range:
              startLine: 6
              startColumn: 7
              endLine: 6
              endColumn: 10
          suggestions:
            - unsafe: true
              source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: "_foo = (bar: Bar) => {\n  switch (bar) {\n    case Bar.Baz:\n      foobar();\n      barfoo();\n      break;\n  }\n  { !_foo"
                  location:
                    path: basic.in.ts
                    range:
                      startLine: 6
                      startColumn: 7
                      endLine: 13
                      endColumn: 9
        - tool: biome
          ruleKey: lint/correctness/noUnusedVariables
          message: This variable Foo is unused.
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://biomejs.dev/linter/rules/no-unused-variables"
          location:
            path: basic.in.ts
            range:
              startLine: 16
              startColumn: 6
              endLine: 16
              endColumn: 9
        - tool: biome
          ruleKey: lint/style/useEnumInitializers
          message: This enum declaration contains members that are implicitly initialized.
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://biomejs.dev/linter/rules/use-enum-initializers"
          location:
            path: basic.in.ts
            range:
              startLine: 4
              startColumn: 6
              endLine: 4
              endColumn: 9
          suggestions:
            - unsafe: true
              source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: "= 0 "
                  location:
                    path: basic.in.ts
                    range:
                      startLine: 4
                      startColumn: 16
                      endLine: 4
                      endColumn: 16
        "###);
    }

    #[test]
    fn parse_severities() {
        let input = r###"
        {
            "source": { "name": "example" },
            "severity": "INFO",
            "diagnostics": [
                {
                "message": "explicit severity",
                "severity": "ERROR",
                "location": { "path": "a.ts", "range": { "start": { "line": 1, "column": 1 } } }
                },
                {
                "message": "inherits result severity",
                "location": { "path": "a.ts", "range": { "start": { "line": 2, "column": 1 } } }
                },
                {
                "message": "no range",
                "location": { "path": "a.ts" }
                }
            ]
        }
        "###;

        let issues = Rdjson::default().parse("example", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r###"
        - tool: example
          message: explicit severity
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: a.ts
            range:
              startLine: 1
              startColumn: 1
              endLine: 1
              endColumn: 1
        - tool: example
          message: inherits result severity
          level: LEVEL_LOW
          category: CATEGORY_LINT
          location:
            path: a.ts
            range:
              startLine: 2
              startColumn: 1
              endLine: 2
              endColumn: 1
        - tool: example
          message: no range
          level: LEVEL_LOW
          category: CATEGORY_LINT
          location:
            path: a.ts
        "###);
    }
}
