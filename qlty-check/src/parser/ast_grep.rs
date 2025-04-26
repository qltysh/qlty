use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AstGrep {}

impl Parser for AstGrep {
    fn parse(&self, plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        return Ok(vec![]); // TODO: Implement parsing logic
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
    "text": "Promise.all([await foo()])",
    "range": {
      "byteOffset": {
        "start": 25,
        "end": 51
      },
      "start": {
        "line": 2,
        "column": 0
      },
      "end": {
        "line": 2,
        "column": 26
      }
    },
    "file": "sample.ts",
    "lines": "Promise.all([await foo()]);",
    "charCount": {
      "leading": 0,
      "trailing": 1
    },
    "language": "TypeScript",
    "metaVariables": {
      "single": {
        "A": {
          "text": "[await foo()]",
          "range": {
            "byteOffset": {
              "start": 37,
              "end": 50
            },
            "start": {
              "line": 2,
              "column": 12
            },
            "end": {
              "line": 2,
              "column": 25
            }
          }
        }
      },
      "multi": {
        "secondary": [
          {
            "text": "await foo()",
            "range": {
              "byteOffset": {
                "start": 38,
                "end": 49
              },
              "start": {
                "line": 2,
                "column": 13
              },
              "end": {
                "line": 2,
                "column": 24
              }
            }
          }
        ]
      },
      "transformed": {}
    },
    "ruleId": "no-await-in-promise-all",
    "severity": "hint",
    "note": null,
    "message": "",
    "labels": [
      {
        "text": "await foo()",
        "range": {
          "byteOffset": {
            "start": 38,
            "end": 49
          },
          "start": {
            "line": 2,
            "column": 13
          },
          "end": {
            "line": 2,
            "column": 24
          }
        }
      }
    ]
  }
  ]
"###;
        let issues = AstGrep::default().parse("ast-grep", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), "");
    }
}
