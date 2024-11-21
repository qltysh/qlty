use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct BiomeOutput {
    pub diagnostics: Vec<BiomeDiagnostic>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct BiomeDiagnostic {
    pub category: String,
    pub severity: String,
    pub description: String,
    pub location: BiomeLocation,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct BiomeLocation {
    pub path: BiomePath,
    pub span: Option<Vec<u64>>,
    #[serde(rename = "sourceCode")]
    pub source_code: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct BiomePath {
    pub file: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Biome {}

impl Parser for Biome {
    fn parse(&self, _plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let biome_output: BiomeOutput = serde_json::from_str(output)?;

        for diagnostic in biome_output.diagnostics {
            // Skip format issues.
            if diagnostic.category == "format" {
                continue;
            }

            // Range is a bit tricky to calculate.
            let range = if let Some(source_code) = diagnostic.location.source_code {
                let span = diagnostic.location.span.unwrap_or_default();

                let (start_line, start_column, end_line, end_column) = if let Some(start_offset) =
                    span.get(0)
                {
                    if let Some(end_offset) = span.get(1) {
                        calculate_line_and_column(source_code.as_str(), *start_offset, *end_offset)
                    } else {
                        (0, 0, 0, 0)
                    }
                } else {
                    (0, 0, 0, 0)
                };

                Some(Range {
                    start_line,
                    start_column,
                    end_line,
                    end_column,
                    ..Default::default()
                })
            } else {
                None
            };

            let issue = Issue {
                tool: "biome".to_string(),
                rule_key: diagnostic.category,
                message: diagnostic.description,
                category: Category::Lint.into(),
                level: severity_to_level(diagnostic.severity).into(),
                location: Some(Location {
                    path: diagnostic.location.path.file,
                    range,
                }),
                ..Default::default()
            };

            issues.push(issue);
        }

        Ok(issues)
    }
}

fn calculate_line_and_column(
    source_code: &str,
    start_offset: u64,
    end_offset: u64,
) -> (u32, u32, u32, u32) {
    let mut current_offset: u64 = 0;
    let mut start_line: Option<u32> = None;
    let mut end_line: Option<u32> = None;
    let mut start_column: Option<u32> = None;
    let mut end_column: Option<u32> = None;

    for (line_number, line) in source_code.lines().enumerate() {
        let line_length = line.len() as u64 + 1; // +1 accounts for the newline character.

        // Check if the start_offset falls in this line.
        if start_line.is_none()
            && current_offset <= start_offset
            && start_offset < current_offset + line_length
        {
            start_line = Some(line_number as u32 + 1);
            start_column = Some((start_offset - current_offset + 1) as u32);
        }

        // Check if the end_offset falls in this line.
        if end_line.is_none()
            && current_offset <= end_offset
            && end_offset < current_offset + line_length
        {
            end_line = Some(line_number as u32 + 1);
            end_column = Some((end_offset - current_offset + 1) as u32);
        }

        current_offset += line_length;

        // Continue iterating to find both start and end positions.
        if start_line.is_some() && end_line.is_some() {
            break;
        }
    }

    (
        start_line.unwrap_or(0),
        start_column.unwrap_or(0),
        end_line.unwrap_or(0),
        end_column.unwrap_or(0),
    )
}

fn severity_to_level(severity: String) -> Level {
    match severity.as_str() {
        "warning" => Level::Medium,
        "error" => Level::High,
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
            "summary": {
                "changed": 0,
                "unchanged": 1,
                "duration": { "secs": 0, "nanos": 26938833 },
                "errors": 3,
                "warnings": 0,
                "skipped": 0,
                "suggestedFixesSkipped": 0,
                "diagnosticsNotPrinted": 0
            },
            "diagnostics": [
                {
                "category": "lint/style/useEnumInitializers",
                "severity": "error",
                "description": "This enum declaration contains members that are implicitly initialized.",
                "message": [
                    { "elements": [], "content": "This " },
                    { "elements": ["Emphasis"], "content": "enum declaration" },
                    {
                        "elements": [],
                        "content": " contains members that are implicitly initialized."
                    }
                ],
                "advices": {
                    "advices": [
                    {
                        "log": [
                        "info",
                        [
                            { "elements": [], "content": "This " },
                            { "elements": ["Emphasis"], "content": "enum member" },
                            {
                                "elements": [],
                                "content": " should be explicitly initialized."
                            }
                        ]
                        ]
                    },
                    {
                        "frame": {
                            "path": null,
                            "span": [62, 65],
                            "sourceCode": "const foobar = () => { }\nconst barfoo = () => { }\n\nenum Bar { Baz };\n\nconst foo = (bar: Bar) => {\n  switch (bar) {\n    case Bar.Baz:\n      foobar();\n      barfoo();\n      break;\n  }\n  { !foo ? null : 1 }\n}\n"
                        }
                    },
                    {
                        "log": [
                        "info",
                        [
                            {
                            "elements": [],
                            "content": "Allowing implicit initializations for enum members can cause bugs if enum declarations are modified over time."
                            }
                        ]
                        ]
                    },
                    {
                        "log": [
                        "info",
                        [
                            {
                            "elements": [],
                            "content": "Safe fix: Initialize all enum members."
                            }
                        ]
                        ]
                    },
                    {
                        "diff": {
                        "dictionary": "const foobar = () => { }\nconst barfoo = () => { }\n\nenum Bar { Baz = 0 };\n\nconst foo = (bar: Bar) => {  { !foo ? null : 1 }\n}\n",
                        "ops": [
                            { "diffOp": { "equal": { "range": [0, 62] } } },
                            { "diffOp": { "equal": { "range": [62, 66] } } },
                            { "diffOp": { "insert": { "range": [66, 70] } } },
                            { "diffOp": { "equal": { "range": [70, 101] } } },
                            { "equalLines": { "line_count": 6 } },
                            { "diffOp": { "equal": { "range": [101, 125] } } }
                        ]
                        }
                    }
                    ]
                },
                "verboseAdvices": { "advices": [] },
                "location": {
                    "path": { "file": "basic.in.ts" },
                    "span": [56, 59],
                    "sourceCode": "const foobar = () => { }\nconst barfoo = () => { }\n\nenum Bar { Baz };\n\nconst foo = (bar: Bar) => {\n  switch (bar) {\n    case Bar.Baz:\n      foobar();\n      barfoo();\n      break;\n  }\n  { !foo ? null : 1 }\n}\n"
                },
                "tags": ["fixable"],
                "source": null
                },
                {
                "category": "lint/complexity/noUselessLoneBlockStatements",
                "severity": "error",
                "description": "This block statement doesn't serve any purpose and can be safely removed.",
                "message": [
                    {
                    "elements": [],
                    "content": "This block statement doesn't serve any purpose and can be safely removed."
                    }
                ],
                "advices": {
                    "advices": [
                    {
                        "log": [
                        "info",
                        [
                            {
                            "elements": [],
                            "content": "Standalone block statements without any block-level declarations are redundant in JavaScript and can be removed to simplify the code."
                            }
                        ]
                        ]
                    },
                    {
                        "log": [
                        "info",
                        [
                            {
                            "elements": [],
                            "content": "Safe fix: Remove redundant block."
                            }
                        ]
                        ]
                    },
                    {
                        "diff": {
                        "dictionary": "const foobar = () => { }\nconst barfoo = () => { }\n\nenum Bar { Baz };\n\nconst foo = (bar: Bar) => {\n  switch (bar) {\n    case Bar.Baz:      barfoo();\n      break;\n  }\n  { !foo ? null : 1 }\n}\n",
                        "ops": [
                            { "diffOp": { "equal": { "range": [0, 97] } } },
                            { "diffOp": { "equal": { "range": [97, 132] } } },
                            { "equalLines": { "line_count": 1 } },
                            { "diffOp": { "equal": { "range": [132, 164] } } },
                            { "diffOp": { "delete": { "range": [164, 169] } } },
                            { "diffOp": { "equal": { "range": [169, 185] } } },
                            { "diffOp": { "delete": { "range": [185, 186] } } },
                            { "diffOp": { "equal": { "range": [186, 189] } } }
                        ]
                        }
                    }
                    ]
                },
                "verboseAdvices": { "advices": [] },
                "location": {
                    "path": { "file": "basic.in.ts" },
                    "span": [184, 203],
                    "sourceCode": "const foobar = () => { }\nconst barfoo = () => { }\n\nenum Bar { Baz };\n\nconst foo = (bar: Bar) => {\n  switch (bar) {\n    case Bar.Baz:\n      foobar();\n      barfoo();\n      break;\n  }\n  { !foo ? null : 1 }\n}\n"
                },
                "tags": ["fixable"],
                "source": null
                },
                {
                "category": "format",
                "severity": "error",
                "description": "Formatter would have printed the following content:",
                "message": [
                    {
                    "elements": [],
                    "content": "Formatter would have printed the following content:"
                    }
                ],
                "advices": {
                    "advices": [
                    {
                        "diff": {
                        "dictionary": "const foobar = () => { };\nconst barfoo = () => {\n\nenum Bar {\n\tBaz,\n\n\nconst foo = (bar: Bar) => {\n  \tswitch (bar) {\n    \t\tcase Bar.Baz:\n      \t\t\tfoobar();\nbarfoo();\nbreak;\n}\n{\n\t\t!foo ? null : 1;\n\t}\n}",
                        "ops": [
                            { "diffOp": { "equal": { "range": [0, 22] } } },
                            { "diffOp": { "delete": { "range": [22, 23] } } },
                            { "diffOp": { "equal": { "range": [23, 24] } } },
                            { "diffOp": { "insert": { "range": [24, 25] } } },
                            { "diffOp": { "equal": { "range": [25, 48] } } },
                            { "diffOp": { "delete": { "range": [22, 23] } } },
                            { "diffOp": { "equal": { "range": [23, 24] } } },
                            { "diffOp": { "insert": { "range": [24, 25] } } },
                            { "diffOp": { "equal": { "range": [48, 60] } } },
                            { "diffOp": { "delete": { "range": [22, 23] } } },
                            { "diffOp": { "insert": { "range": [60, 62] } } },
                            { "diffOp": { "equal": { "range": [62, 65] } } },
                            { "diffOp": { "delete": { "range": [22, 23] } } },
                            { "diffOp": { "insert": { "range": [65, 67] } } },
                            { "diffOp": { "equal": { "range": [23, 24] } } },
                            { "diffOp": { "delete": { "range": [24, 25] } } },
                            { "diffOp": { "equal": { "range": [67, 97] } } },
                            { "diffOp": { "delete": { "range": [97, 99] } } },
                            { "diffOp": { "insert": { "range": [99, 100] } } },
                            { "diffOp": { "equal": { "range": [100, 115] } } },
                            { "diffOp": { "delete": { "range": [115, 119] } } },
                            { "diffOp": { "insert": { "range": [119, 121] } } },
                            { "diffOp": { "equal": { "range": [121, 135] } } },
                            { "diffOp": { "delete": { "range": [135, 141] } } },
                            { "diffOp": { "insert": { "range": [141, 144] } } },
                            { "diffOp": { "equal": { "range": [144, 154] } } },
                            { "diffOp": { "delete": { "range": [135, 141] } } },
                            { "diffOp": { "insert": { "range": [141, 144] } } },
                            { "diffOp": { "equal": { "range": [154, 164] } } },
                            { "diffOp": { "delete": { "range": [135, 141] } } },
                            { "diffOp": { "insert": { "range": [141, 144] } } },
                            { "diffOp": { "equal": { "range": [164, 171] } } },
                            { "diffOp": { "delete": { "range": [135, 137] } } },
                            { "diffOp": { "insert": { "range": [141, 142] } } },
                            { "diffOp": { "equal": { "range": [171, 173] } } },
                            { "diffOp": { "delete": { "range": [97, 99] } } },
                            { "diffOp": { "insert": { "range": [141, 142] } } },
                            { "diffOp": { "equal": { "range": [173, 174] } } },
                            { "diffOp": { "delete": { "range": [97, 98] } } },
                            { "diffOp": { "insert": { "range": [174, 177] } } },
                            { "diffOp": { "equal": { "range": [177, 192] } } },
                            { "diffOp": { "delete": { "range": [97, 98] } } },
                            { "diffOp": { "insert": { "range": [192, 195] } } },
                            { "diffOp": { "equal": { "range": [195, 198] } } },
                            { "diffOp": { "insert": { "range": [192, 193] } } },
                            { "diffOp": { "equal": { "range": [48, 49] } } }
                        ]
                        }
                    }
                    ]
                },
                "verboseAdvices": { "advices": [] },
                "location": {
                    "path": { "file": "basic.in.ts" },
                    "span": null,
                    "sourceCode": null
                },
                "tags": [],
                "source": null
                }
            ],
            "command": "check"
        }
        "###;

        let issues = Biome::default().parse("bandit", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r###"
        - tool: biome
          ruleKey: lint/style/useEnumInitializers
          message: This enum declaration contains members that are implicitly initialized.
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.ts
            range:
              startLine: 4
              startColumn: 6
              endLine: 4
              endColumn: 9
        - tool: biome
          ruleKey: lint/complexity/noUselessLoneBlockStatements
          message: "This block statement doesn't serve any purpose and can be safely removed."
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.ts
            range:
              startLine: 13
              startColumn: 3
              endLine: 13
              endColumn: 22
        "###);
    }
}
