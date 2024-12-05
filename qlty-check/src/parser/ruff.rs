use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{
    Category, Issue, Level, Location, Range, Replacement, Suggestion, SuggestionSource,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct RuffIssue {
    code: Option<String>,
    filename: String,
    location: RuffLocation,
    end_location: RuffLocation,
    message: String,
    url: Option<String>,
    fix: Option<RuffFix>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct RuffLocation {
    column: u32,
    row: u32,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct RuffFix {
    applicability: String,
    message: String,
    edits: Vec<RuffEdit>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct RuffEdit {
    content: String,
    location: RuffLocation,
    end_location: RuffLocation,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Ruff {}

impl Parser for Ruff {
    fn parse(&self, _plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let ruff_issues: Vec<RuffIssue> = serde_json::from_str(output)?;

        for ruff_issue in ruff_issues {
            let issue = Issue {
                tool: "ruff".to_string(),
                rule_key: ruff_issue.code.unwrap_or("error".to_string()),
                message: ruff_issue.message,
                category: Category::Lint.into(),
                level: Level::Medium.into(),
                documentation_url: ruff_issue.url.unwrap_or_default(),
                location: Some(Location {
                    path: ruff_issue.filename.clone(),
                    range: Some(Range {
                        start_line: ruff_issue.location.row,
                        start_column: ruff_issue.location.column,
                        end_line: ruff_issue.end_location.row,
                        end_column: ruff_issue.end_location.column,
                        ..Default::default()
                    }),
                }),
                suggestions: self.build_suggestions(&ruff_issue.fix, ruff_issue.filename),
                ..Default::default()
            };
            issues.push(issue);
        }

        Ok(issues)
    }
}

impl Ruff {
    fn build_suggestions(&self, fix: &Option<RuffFix>, path: String) -> Vec<Suggestion> {
        let mut replacements = vec![];

        if let Some(fix) = fix {
            for edit in &fix.edits {
                let replacement = Replacement {
                    data: edit.content.clone(),
                    location: Some(Location {
                        path: path.clone(),
                        range: Some(Range {
                            start_line: edit.location.row,
                            start_column: edit.location.column,
                            end_line: edit.end_location.row,
                            end_column: edit.end_location.column,
                            ..Default::default()
                        }),
                    }),
                };
                replacements.push(replacement);
            }
        }

        if replacements.is_empty() {
            vec![]
        } else {
            vec![Suggestion {
                source: SuggestionSource::Tool.into(),
                replacements,
                ..Default::default()
            }]
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = r###"
        [
        {
        "cell": null,
        "code": null,
        "end_location": {
            "column": 12,
            "row": 42
        },
        "filename": "/private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_odriQA/basic.in.pyy",
        "fix": null,
        "location": {
            "column": 11,
            "row": 42
        },
        "message": "SyntaxError: Simple statements must be separated by newlines or semicolons",
        "noqa_row": null,
        "url": null
        },
        {
            "cell": null,
            "code": "E402",
            "end_location": {
                "column": 11,
                "row": 7
            },
            "filename": "/private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_odriQA/basic.in.py",
            "fix": null,
            "location": {
                "column": 1,
                "row": 7
            },
            "message": "Module level import not at top of file",
            "noqa_row": 7,
            "url": "https://docs.astral.sh/ruff/rules/module-import-not-at-top-of-file"
        },
        {
            "cell": null,
            "code": "F401",
            "end_location": {
                "column": 11,
                "row": 7
            },
            "filename": "/private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_odriQA/basic.in.py",
            "fix": {
                "applicability": "safe",
                "edits": [
                    {
                        "content": "",
                        "end_location": {
                            "column": 1,
                            "row": 8
                        },
                        "location": {
                            "column": 1,
                            "row": 7
                        }
                    }
                ],
                "message": "Remove unused import: `sys`"
            },
            "location": {
                "column": 8,
                "row": 7
            },
            "message": "`sys` imported but unused",
            "noqa_row": 7,
            "url": "https://docs.astral.sh/ruff/rules/unused-import"
        },
        {
            "cell": null,
            "code": "E402",
            "end_location": {
                "column": 12,
                "row": 9
            },
            "filename": "/private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_odriQA/basic.in.py",
            "fix": null,
            "location": {
                "column": 1,
                "row": 9
            },
            "message": "Module level import not at top of file",
            "noqa_row": 9,
            "url": "https://docs.astral.sh/ruff/rules/module-import-not-at-top-of-file"
        },
        {
            "cell": null,
            "code": "F401",
            "end_location": {
                "column": 12,
                "row": 9
            },
            "filename": "/private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_odriQA/basic.in.py",
            "fix": {
                "applicability": "safe",
                "edits": [
                    {
                        "content": "",
                        "end_location": {
                            "column": 1,
                            "row": 10
                        },
                        "location": {
                            "column": 1,
                            "row": 9
                        }
                    }
                ],
                "message": "Remove unused import: `json`"
            },
            "location": {
                "column": 8,
                "row": 9
            },
            "message": "`json` imported but unused",
            "noqa_row": 9,
            "url": "https://docs.astral.sh/ruff/rules/unused-import"
        }
        ]
        "###;

        let issues = Ruff::default().parse("Ruff", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r###"
        - tool: ruff
          ruleKey: error
          message: "SyntaxError: Simple statements must be separated by newlines or semicolons"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: /private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_odriQA/basic.in.pyy
            range:
              startLine: 42
              startColumn: 11
              endLine: 42
              endColumn: 12
        - tool: ruff
          ruleKey: E402
          message: Module level import not at top of file
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://docs.astral.sh/ruff/rules/module-import-not-at-top-of-file"
          location:
            path: /private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_odriQA/basic.in.py
            range:
              startLine: 7
              startColumn: 1
              endLine: 7
              endColumn: 11
        - tool: ruff
          ruleKey: F401
          message: "`sys` imported but unused"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://docs.astral.sh/ruff/rules/unused-import"
          location:
            path: /private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_odriQA/basic.in.py
            range:
              startLine: 7
              startColumn: 8
              endLine: 7
              endColumn: 11
          suggestions:
            - source: SUGGESTION_SOURCE_TOOL
              replacements:
                - location:
                    path: /private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_odriQA/basic.in.py
                    range:
                      startLine: 7
                      startColumn: 1
                      endLine: 8
                      endColumn: 1
        - tool: ruff
          ruleKey: E402
          message: Module level import not at top of file
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://docs.astral.sh/ruff/rules/module-import-not-at-top-of-file"
          location:
            path: /private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_odriQA/basic.in.py
            range:
              startLine: 9
              startColumn: 1
              endLine: 9
              endColumn: 12
        - tool: ruff
          ruleKey: F401
          message: "`json` imported but unused"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://docs.astral.sh/ruff/rules/unused-import"
          location:
            path: /private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_odriQA/basic.in.py
            range:
              startLine: 9
              startColumn: 8
              endLine: 9
              endColumn: 12
          suggestions:
            - source: SUGGESTION_SOURCE_TOOL
              replacements:
                - location:
                    path: /private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_odriQA/basic.in.py
                    range:
                      startLine: 9
                      startColumn: 1
                      endLine: 10
                      endColumn: 1
        "###);
    }
}
