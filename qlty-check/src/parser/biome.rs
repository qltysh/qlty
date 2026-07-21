use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{
    Category, Issue, Level, Location, Range, Replacement, Suggestion, SuggestionSource,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BiomeOutput {
    pub diagnostics: Vec<BiomeDiagnostic>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BiomeDiagnostic {
    pub category: String,
    pub severity: String,
    pub description: String,
    pub location: BiomeLocation,
    #[serde(default)]
    pub advices: Option<BiomeAdvices>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BiomeLocation {
    pub path: BiomePath,
    pub span: Option<Vec<u64>>,
    #[serde(rename = "sourceCode")]
    pub source_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BiomePath {
    pub file: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BiomeAdvices {
    pub advices: Vec<BiomeAdvice>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BiomeAdvice {
    #[serde(default)]
    pub diff: Option<BiomeDiff>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BiomeDiff {
    pub dictionary: String,
    pub ops: Vec<BiomeDiffOp>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BiomeDiffOp {
    #[serde(default, rename = "diffOp")]
    pub diff_op: Option<DiffOperationWrapper>,
    #[serde(default, rename = "equalLines")]
    pub equal_lines: Option<EqualLines>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffOperationWrapper {
    pub equal: Option<DiffRange>,
    pub insert: Option<DiffRange>,
    pub delete: Option<DiffRange>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EqualLines {
    pub line_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffRange {
    pub range: Vec<u64>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Biome;

impl Parser for Biome {
    fn parse(&self, plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let biome_output: BiomeOutput = serde_json::from_str(output)?;

        for diagnostic in biome_output.diagnostics {
            let suggestions = self.build_suggestions(&diagnostic);

            // Range is a bit tricky to calculate.
            let range = if let Some(source_code) = &diagnostic.location.source_code {
                let span = diagnostic.location.span.clone().unwrap_or_default();

                let (start_line, start_column, end_line, end_column) = if let Some(start_offset) =
                    span.first()
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
                tool: plugin_name.into(),
                rule_key: diagnostic.category.clone(),
                message: diagnostic.description.clone(),
                category: Category::Lint.into(),
                level: severity_to_level(&diagnostic.severity).into(),
                location: Some(Location {
                    path: diagnostic.location.path.file.clone(),
                    range,
                }),
                suggestions,
                ..Default::default()
            };

            issues.push(issue);
        }

        Ok(issues)
    }
}

impl Biome {
    fn build_suggestions(&self, diagnostic: &BiomeDiagnostic) -> Vec<Suggestion> {
        if let Some(advices) = &diagnostic.advices {
            if let Some(source_code) = &diagnostic.location.source_code {
                advices
                    .advices
                    .iter()
                    .filter_map(|advice| {
                        let replacements = self.build_replacements(&advice.diff, source_code);

                        if replacements.is_empty() {
                            None
                        } else {
                            Some(Suggestion {
                                source: SuggestionSource::Tool.into(),
                                replacements,
                                ..Default::default()
                            })
                        }
                    })
                    .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    fn build_replacements(&self, diff: &Option<BiomeDiff>, source_code: &str) -> Vec<Replacement> {
        if let Some(diff) = diff {
            let mut cumulative_offset = 0u64; // Tracks the offset caused by equalLines.
            let mut line_iter = source_code.lines().enumerate(); // Line iterator with line numbers.
            let mut current_line = 0; // Tracks the current line for equalLines offset calculation.
            let mut last_end_offset = 0u64; // Tracks the last end offset for diffOps

            diff.ops
                .iter()
                .filter_map(|op| {
                    if let Some(equal_lines) = &op.equal_lines {
                        current_line = get_end_line_from_range(source_code, last_end_offset);
                        // Update cumulative_offset using equalLines.
                        cumulative_offset += calculate_equal_lines_offset(
                            &mut line_iter,
                            equal_lines.line_count,
                            current_line,
                        );
                        current_line += equal_lines.line_count as usize; // Move to the next line after equalLines.
                        None
                    } else if let Some(diff_op) = &op.diff_op {
                        if let Some(range) = &diff_op.insert {
                            last_end_offset = range.range[1] + cumulative_offset;

                            build_insert_replacement(
                                diff.dictionary.clone(),
                                &range.range,
                                source_code,
                                cumulative_offset,
                            )
                        } else if let Some(range) = &diff_op.delete {
                            let start_offset = range.range[0] + cumulative_offset;
                            let end_offset = range.range[1] + cumulative_offset;
                            last_end_offset = end_offset;

                            build_delete_replacement(source_code, &[start_offset, end_offset])
                        } else if let Some(range) = &diff_op.equal {
                            let end_offset = range.range[1] + cumulative_offset;
                            last_end_offset = end_offset;

                            None
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }
}

fn calculate_equal_lines_offset(
    line_iter: &mut std::iter::Enumerate<std::str::Lines>,
    line_count: u32,
    current_line: usize,
) -> u64 {
    let mut offset = 0u64;

    for (line_idx, line) in line_iter {
        if line_idx >= current_line && line_idx < current_line + line_count as usize {
            offset += line.len() as u64 + 1; // Include newline character
        } else if line_idx >= current_line + line_count as usize {
            break;
        }
    }

    offset + 1
}

fn get_end_line_from_range(source_code: &str, end_offset: u64) -> usize {
    let mut current_offset = 0u64;

    for (line_idx, line) in source_code.lines().enumerate() {
        let line_length = line.chars().count() as u64 + 1; // +1 for the newline character.
        if current_offset > end_offset {
            return line_idx;
        }
        current_offset += line_length;
    }

    source_code.lines().count() // Default to the last line if out of bounds.
}

fn build_insert_replacement(
    dictionary: String,
    range: &[u64],
    source_code: &str,
    cumulative_offset: u64,
) -> Option<Replacement> {
    let (start_offset, end_offset) = match range {
        [start, end] => (*start, *end),
        _ => {
            return None;
        }
    };

    let sliced_data = dictionary
        .get(start_offset as usize..end_offset as usize)
        .unwrap_or("")
        .to_string();

    let (start_line, start_column, _end_line, _end_column) = calculate_line_and_column(
        source_code,
        start_offset + cumulative_offset,
        end_offset + cumulative_offset,
    );

    Some(Replacement {
        data: sliced_data,
        location: Some(Location {
            path: "".into(),
            range: Some(Range {
                start_line,
                start_column,
                end_line: start_line,
                end_column: start_column,
                ..Default::default()
            }),
        }),
    })
}

fn build_delete_replacement(source_code: &str, range: &[u64]) -> Option<Replacement> {
    // Extract the start and end indices from the range.
    let (start_offset, end_offset) = match range {
        [start, end] => (*start as usize, *end as usize),
        _ => {
            return None;
        }
    };

    let (start_line, start_column, end_line, end_column) =
        calculate_line_and_column(source_code, start_offset as u64, end_offset as u64);

    Some(Replacement {
        data: "".to_string(),
        location: Some(Location {
            path: "".into(), // Biome doesn't provide file-specific replacement paths here.
            range: Some(Range {
                start_line,
                start_column,
                end_line,
                end_column,
                ..Default::default()
            }),
        }),
    })
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

fn severity_to_level(severity: &str) -> Level {
    match severity {
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
                "duration": { "secs": 0, "nanos": 46965834 },
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
                        "sourceCode": "const foobar = () => { }\nconst barfoo = () => { }\n\nenum Bar { Baz };\n\nconst foo = (bar: Bar) => {\n  switch (bar) {\n    case Bar.Baz:\n      foobar();\n      barfoo();\n      break;\n  }\n  { !foo ? null : 1 }\n}\n\nenum Foo { Bae };\n"
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
                        "dictionary": "const foobar = () => { }\nconst barfoo = () => { }\n\nenum Bar { Baz = 0 };\n\nconst foo = (bar: Bar) => {\nenum Foo { Bae };\n",
                        "ops": [
                            { "diffOp": { "equal": { "range": [0, 62] } } },
                            { "diffOp": { "equal": { "range": [62, 66] } } },
                            { "diffOp": { "insert": { "range": [66, 70] } } },
                            { "diffOp": { "equal": { "range": [70, 101] } } },
                            { "equalLines": { "line_count": 8 } },
                            { "diffOp": { "equal": { "range": [101, 120] } } }
                        ]
                        }
                    }
                    ]
                },
                "verboseAdvices": { "advices": [] },
                "location": {
                    "path": { "file": "basic.in.ts" },
                    "span": [56, 59],
                    "sourceCode": "const foobar = () => { }\nconst barfoo = () => { }\n\nenum Bar { Baz };\n\nconst foo = (bar: Bar) => {\n  switch (bar) {\n    case Bar.Baz:\n      foobar();\n      barfoo();\n      break;\n  }\n  { !foo ? null : 1 }\n}\n\nenum Foo { Bae };\n"
                },
                "tags": ["fixable"],
                "source": null
                },
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
                        "span": [218, 221],
                        "sourceCode": "const foobar = () => { }\nconst barfoo = () => { }\n\nenum Bar { Baz };\n\nconst foo = (bar: Bar) => {\n  switch (bar) {\n    case Bar.Baz:\n      foobar();\n      barfoo();\n      break;\n  }\n  { !foo ? null : 1 }\n}\n\nenum Foo { Bae };\n"
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
                        "dictionary": "const foobar = () => { }\nconst barfoo = () => { }\n}\n\nenum Foo { Bae = 0 };\n",
                        "ops": [
                            { "diffOp": { "equal": { "range": [0, 50] } } },
                            { "equalLines": { "line_count": 10 } },
                            { "diffOp": { "equal": { "range": [50, 64] } } },
                            { "diffOp": { "equal": { "range": [64, 68] } } },
                            { "diffOp": { "insert": { "range": [68, 72] } } },
                            { "diffOp": { "equal": { "range": [72, 75] } } }
                        ]
                        }
                    }
                    ]
                },
                "verboseAdvices": { "advices": [] },
                "location": {
                    "path": { "file": "basic.in.ts" },
                    "span": [212, 215],
                    "sourceCode": "const foobar = () => { }\nconst barfoo = () => { }\n\nenum Bar { Baz };\n\nconst foo = (bar: Bar) => {\n  switch (bar) {\n    case Bar.Baz:\n      foobar();\n      barfoo();\n      break;\n  }\n  { !foo ? null : 1 }\n}\n\nenum Foo { Bae };\n"
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
                        "dictionary": "const foobar = () => { }\nconst barfoo = () => { }\n\nenum Bar { Baz };\n\nconst foo = (bar: Bar) => {\n  switch (bar) {\n    case Bar.Baz:      barfoo();\n      break;\n  }\n  { !foo ? null : 1 }\n}\n\nenum Foo { Bae };\n",
                        "ops": [
                            { "diffOp": { "equal": { "range": [0, 97] } } },
                            { "diffOp": { "equal": { "range": [97, 132] } } },
                            { "equalLines": { "line_count": 1 } },
                            { "diffOp": { "equal": { "range": [132, 164] } } },
                            { "diffOp": { "delete": { "range": [164, 169] } } },
                            { "diffOp": { "equal": { "range": [169, 185] } } },
                            { "diffOp": { "delete": { "range": [185, 186] } } },
                            { "diffOp": { "equal": { "range": [186, 208] } } }
                        ]
                        }
                    }
                    ]
                },
                "verboseAdvices": { "advices": [] },
                "location": {
                    "path": { "file": "basic.in.ts" },
                    "span": [184, 203],
                    "sourceCode": "const foobar = () => { }\nconst barfoo = () => { }\n\nenum Bar { Baz };\n\nconst foo = (bar: Bar) => {\n  switch (bar) {\n    case Bar.Baz:\n      foobar();\n      barfoo();\n      break;\n  }\n  { !foo ? null : 1 }\n}\n\nenum Foo { Bae };\n"
                },
                "tags": ["fixable"],
                "source": null
                }
            ],
            "command": "lint"
            }
        "###;

        let issues = Biome::default().parse("biome", input);
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
          suggestions:
            - source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: "= 0 "
                  location:
                    range:
                      startLine: 4
                      startColumn: 16
                      endLine: 4
                      endColumn: 16
        - tool: biome
          ruleKey: lint/style/useEnumInitializers
          message: This enum declaration contains members that are implicitly initialized.
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.ts
            range:
              startLine: 16
              startColumn: 6
              endLine: 16
              endColumn: 9
          suggestions:
            - source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: "= 0 "
                  location:
                    range:
                      startLine: 16
                      startColumn: 16
                      endLine: 16
                      endColumn: 16
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
          suggestions:
            - source: SUGGESTION_SOURCE_TOOL
              replacements:
                - location:
                    range:
                      startLine: 12
                      startColumn: 4
                      endLine: 13
                      endColumn: 5
                - location:
                    range:
                      startLine: 13
                      startColumn: 21
                      endLine: 13
                      endColumn: 22
        "###);
    }
}
