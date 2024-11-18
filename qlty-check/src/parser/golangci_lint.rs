use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{
    Category, Issue, Level, Location, Range, Replacement, Suggestion, SuggestionSource,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
struct GolangciLintOutput {
    #[serde(rename = "Issues")]
    issues: Vec<GolangciLintIssue>,
}

#[derive(Debug, Deserialize, Clone)]
struct GolangciLintIssue {
    #[serde(rename = "FromLinter")]
    from_linter: String,
    #[serde(rename = "Text")]
    text: String,
    #[serde(rename = "Severity")]
    severity: String,
    #[serde(rename = "Replacement")]
    replacement: Option<GolangciLintReplacement>,
    #[serde(rename = "Pos")]
    pos: Pos,
    #[serde(rename = "LineRange")]
    line_range: Option<LineRange>,
    #[serde(rename = "SourceLines")]
    source_lines: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct GolangciLintReplacement {
    #[serde(rename = "NewLines")]
    new_lines: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct Pos {
    #[serde(rename = "Filename")]
    filename: String,
    #[serde(rename = "Line")]
    line: u32,
    #[serde(rename = "Column")]
    column: u32,
}

#[derive(Debug, Deserialize, Clone)]
struct LineRange {
    #[serde(rename = "From")]
    from: u32,
    #[serde(rename = "To")]
    to: u32,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct GolangciLint {}

impl Parser for GolangciLint {
    fn parse(&self, plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let golangcilint_output: GolangciLintOutput = serde_json::from_str(output)?;

        for golangcilint_issue in golangcilint_output.issues {
            let suggestions = build_suggestions(&golangcilint_issue);

            let issue = Issue {
                tool: plugin_name.into(),
                message: golangcilint_issue.text,
                category: Category::Lint.into(),
                level: level_to_issue_level(golangcilint_issue.severity).into(),
                rule_key: golangcilint_issue.from_linter,
                location: Some(Location {
                    path: golangcilint_issue.pos.filename.clone(),
                    range: Some(Range {
                        start_line: golangcilint_issue.pos.line,
                        start_column: golangcilint_issue.pos.column,
                        ..Default::default()
                    }),
                }),
                suggestions,
                ..Default::default()
            };

            issues.push(issue);
        }

        Ok(issues)
    }
}

fn level_to_issue_level(severity: String) -> Level {
    match severity.as_str() {
        "ignore" => Level::Low,
        "warn" => Level::Medium,
        "error" => Level::High,
        _ => Level::Medium,
    }
}

fn build_suggestions(
    golangcilint_issue: &GolangciLintIssue,
) -> Vec<qlty_types::analysis::v1::Suggestion> {
    let mut suggestions = vec![];

    if let Some(replacement) = &golangcilint_issue.replacement {
        let replacement_text = replacement.new_lines.join("\n");

        let (start_line, end_line) = if let Some(line_range) = &golangcilint_issue.line_range {
            (line_range.from, line_range.to)
        } else {
            (golangcilint_issue.pos.line, golangcilint_issue.pos.line)
        };

        suggestions.push(Suggestion {
            source: SuggestionSource::Tool.into(),
            replacements: vec![Replacement {
                location: Some(Location {
                    path: golangcilint_issue.pos.filename.clone(),
                    range: Some(Range {
                        start_line,
                        end_line,
                        end_column: golangcilint_issue
                            .source_lines
                            .last()
                            .map_or(0, |l| (l.len() + 1) as u32),
                        ..Default::default()
                    }),
                }),
                data: replacement_text.clone(),
            }],
            ..Default::default()
        });
    }

    suggestions
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = r###"
        {
            "Issues": [
                {
                "FromLinter": "errcheck",
                "Text": "Error return value of `time.Parse` is not checked",
                "Severity": "",
                "SourceLines": ["\ttime.Parse(\"asdf\", \"\")"],
                "Replacement": null,
                "Pos": {
                    "Filename": "basic.in.go",
                    "Offset": 217,
                    "Line": 12,
                    "Column": 12
                },
                "ExpectNoLint": false,
                "ExpectedNoLintLinter": ""
                },
                {
                "FromLinter": "godot",
                "Text": "Comment should end in a period",
                "Severity": "",
                "SourceLines": ["// this is the main function 🏃"],
                "Replacement": {
                    "NeedOnlyDelete": false,
                    "NewLines": ["// this is the main function 🏃."],
                    "Inline": null
                },
                "Pos": {
                    "Filename": "basic.in.go",
                    "Offset": 42,
                    "Line": 7,
                    "Column": 34
                },
                "ExpectNoLint": false,
                "ExpectedNoLintLinter": ""
                },
                {
                "FromLinter": "goimports",
                "Text": "File is not `goimports`-ed",
                "Severity": "",
                "SourceLines": ["import \"time\"", "import \"fmt\""],
                "Replacement": {
                    "NeedOnlyDelete": false,
                    "NewLines": ["import (", "\t\"fmt\"", "\t\"time\"", ")"],
                    "Inline": null
                },
                "LineRange": { "From": 3, "To": 4 },
                "Pos": { "Filename": "basic.in.go", "Offset": 0, "Line": 3, "Column": 0 },
                "ExpectNoLint": false,
                "ExpectedNoLintLinter": ""
                }
            ],
            "Report": {
                "Linters": [
                { "Name": "asasalint" },
                { "Name": "asciicheck", "Enabled": true },
                { "Name": "bidichk" },
                { "Name": "bodyclose", "Enabled": true },
                { "Name": "canonicalheader" },
                { "Name": "containedctx" },
                { "Name": "contextcheck" },
                { "Name": "copyloopvar" },
                { "Name": "cyclop" },
                { "Name": "decorder" },
                { "Name": "deadcode" },
                { "Name": "depguard", "Enabled": true },
                { "Name": "dogsled", "Enabled": true },
                { "Name": "dupl" },
                { "Name": "dupword" },
                { "Name": "durationcheck" },
                { "Name": "errcheck", "Enabled": true, "EnabledByDefault": true },
                { "Name": "errchkjson" },
                { "Name": "errname" },
                { "Name": "errorlint" },
                { "Name": "execinquery" },
                { "Name": "exhaustive" },
                { "Name": "exhaustivestruct" },
                { "Name": "exhaustruct" },
                { "Name": "exportloopref", "Enabled": true },
                { "Name": "forbidigo" },
                { "Name": "forcetypeassert" },
                { "Name": "fatcontext" },
                { "Name": "funlen" },
                { "Name": "gci" },
                { "Name": "ginkgolinter" },
                { "Name": "gocheckcompilerdirectives" },
                { "Name": "gochecknoglobals" },
                { "Name": "gochecknoinits", "Enabled": true },
                { "Name": "gochecksumtype" },
                { "Name": "gocognit" },
                { "Name": "goconst" },
                { "Name": "gocritic" },
                { "Name": "gocyclo" },
                { "Name": "godot", "Enabled": true },
                { "Name": "godox" },
                { "Name": "err113" },
                { "Name": "gofmt", "Enabled": true },
                { "Name": "gofumpt" },
                { "Name": "goheader", "Enabled": true },
                { "Name": "goimports", "Enabled": true },
                { "Name": "golint" },
                { "Name": "mnd" },
                { "Name": "gomnd" },
                { "Name": "gomoddirectives" },
                { "Name": "gomodguard" },
                { "Name": "goprintffuncname", "Enabled": true },
                { "Name": "gosec", "Enabled": true },
                { "Name": "gosimple", "Enabled": true, "EnabledByDefault": true },
                { "Name": "gosmopolitan" },
                { "Name": "govet", "Enabled": true, "EnabledByDefault": true },
                { "Name": "grouper" },
                { "Name": "ifshort" },
                { "Name": "importas" },
                { "Name": "inamedparam" },
                { "Name": "ineffassign", "Enabled": true, "EnabledByDefault": true },
                { "Name": "interfacebloat" },
                { "Name": "interfacer" },
                { "Name": "intrange" },
                { "Name": "ireturn" },
                { "Name": "lll" },
                { "Name": "loggercheck" },
                { "Name": "maintidx" },
                { "Name": "makezero" },
                { "Name": "maligned" },
                { "Name": "mirror" },
                { "Name": "misspell", "Enabled": true },
                { "Name": "musttag" },
                { "Name": "nakedret", "Enabled": true },
                { "Name": "nestif" },
                { "Name": "nilerr" },
                { "Name": "nilnil" },
                { "Name": "nlreturn" },
                { "Name": "noctx" },
                { "Name": "nonamedreturns" },
                { "Name": "nosnakecase" },
                { "Name": "nosprintfhostport" },
                { "Name": "paralleltest" },
                { "Name": "perfsprint" },
                { "Name": "prealloc" },
                { "Name": "predeclared" },
                { "Name": "promlinter" },
                { "Name": "protogetter" },
                { "Name": "reassign" },
                { "Name": "revive" },
                { "Name": "rowserrcheck", "Enabled": true },
                { "Name": "sloglint" },
                { "Name": "scopelint" },
                { "Name": "sqlclosecheck" },
                { "Name": "spancheck" },
                { "Name": "staticcheck", "Enabled": true, "EnabledByDefault": true },
                { "Name": "structcheck" },
                { "Name": "stylecheck", "Enabled": true },
                { "Name": "tagalign" },
                { "Name": "tagliatelle" },
                { "Name": "tenv" },
                { "Name": "testableexamples" },
                { "Name": "testifylint" },
                { "Name": "testpackage" },
                { "Name": "thelper" },
                { "Name": "tparallel" },
                { "Name": "typecheck", "Enabled": true, "EnabledByDefault": true },
                { "Name": "unconvert", "Enabled": true },
                { "Name": "unparam" },
                { "Name": "unused", "Enabled": true, "EnabledByDefault": true },
                { "Name": "usestdlibvars" },
                { "Name": "varcheck" },
                { "Name": "varnamelen" },
                { "Name": "wastedassign" },
                { "Name": "whitespace", "Enabled": true },
                { "Name": "wrapcheck" },
                { "Name": "wsl" },
                { "Name": "zerologlint" },
                { "Name": "nolintlint", "Enabled": true }
                ]
            }
        }
        "###;

        let issues = GolangciLint::default().parse("golangcilint", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r###"
        - tool: golangcilint
          ruleKey: errcheck
          message: "Error return value of `time.Parse` is not checked"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: basic.in.go
            range:
              startLine: 12
              startColumn: 12
        - tool: golangcilint
          ruleKey: godot
          message: Comment should end in a period
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: basic.in.go
            range:
              startLine: 7
              startColumn: 34
          suggestions:
            - source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: // this is the main function 🏃.
                  location:
                    path: basic.in.go
                    range:
                      startLine: 7
                      endLine: 7
                      endColumn: 34
        - tool: golangcilint
          ruleKey: goimports
          message: "File is not `goimports`-ed"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: basic.in.go
            range:
              startLine: 3
          suggestions:
            - source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: "import (\n\t\"fmt\"\n\t\"time\"\n)"
                  location:
                    path: basic.in.go
                    range:
                      startLine: 3
                      endLine: 4
                      endColumn: 13
        "###);
    }
}
