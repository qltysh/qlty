use super::Parser;
use anyhow::Result;
use path_absolutize::Absolutize;
use qlty_analysis::utils::fs::path_to_string;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use tracing::info;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SarifFile {
    pub runs: Vec<SarifRun>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SarifRun {
    pub tool: SarifTool,
    pub results: Vec<SarifResult>,
    #[serde(alias = "originalUriBaseIds")]
    pub original_uri_base_ids: Option<OriginalUriBaseIds>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SarifTool {
    pub driver: SarifDriver,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SarifDriver {
    #[serde(default)]
    pub rules: Vec<SarifRule>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SarifRule {
    pub id: String,
    #[serde(alias = "helpUri")]
    pub help_uri: Option<String>,
    #[serde(alias = "defaultConfiguration")]
    pub default_configuration: Option<RuleDefaultConfiguration>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RuleDefaultConfiguration {
    #[serde(alias = "level")]
    pub level: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OriginalUriBaseIds {
    #[serde(alias = "ROOTPATH", alias = "%SRCROOT%")]
    pub root_path: RootPath,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RootPath {
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SarifResult {
    #[serde(alias = "ruleId")]
    pub rule_id: Option<String>,
    pub message: SarifMessage,
    pub level: Option<String>,
    #[serde(default)]
    pub locations: Vec<SarifLocation>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub suppressions: Option<Vec<Suppression>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Suppression {
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SarifLocation {
    #[serde(alias = "physicalLocation")]
    pub physical_location: SarifPhysicalLocation,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SarifPhysicalLocation {
    #[serde(alias = "artifactLocation")]
    pub artifact_location: SarifArtifactLocation,
    pub region: Option<SarifRegion>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SarifArtifactLocation {
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SarifRegion {
    #[serde(alias = "startColumn")]
    pub start_column: Option<u32>,
    #[serde(alias = "startLine")]
    pub start_line: Option<u32>,
    #[serde(alias = "endColumn")]
    pub end_column: Option<u32>,
    #[serde(alias = "endLine")]
    pub end_line: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SarifMessage {
    pub text: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Sarif {
    pub level: Option<Level>,
    pub category: Option<Category>,
}

impl Sarif {
    pub fn new(level: Option<Level>, category: Option<Category>) -> Self {
        Self { level, category }
    }

    fn get_location(
        locations: Vec<SarifLocation>,
        original_uri_base_ids: Option<OriginalUriBaseIds>,
    ) -> Option<Location> {
        if let Some(location) = locations.first() {
            let range = match location.physical_location.region.as_ref() {
                Some(region) => {
                    let start_line = region.start_line.unwrap_or(1);
                    let start_column = region.start_column.unwrap_or(1);

                    Some(Range {
                        start_line,
                        start_column,
                        end_line: region.end_line.unwrap_or(start_line),
                        end_column: region.end_column.unwrap_or(start_column),
                        ..Default::default()
                    })
                }
                None => None,
            };

            let artifact_location = &location.physical_location.artifact_location.uri;
            let path = if let Some(original_uri_base) = original_uri_base_ids {
                Sarif::merge_paths(&original_uri_base.root_path.uri, artifact_location)
            } else {
                artifact_location.into()
            };
            return Some(Location { path, range });
        }

        None
    }

    fn get_level(&self, result: &SarifResult, rule_info: &HashMap<String, &SarifRule>) -> Level {
        if let Some(level) = self.level {
            level
        } else if let Some(level) = &result.level {
            Sarif::level_to_level(Some(level.clone()))
        } else if let Some(rule_id) = &result.rule_id {
            rule_info
                .get(rule_id)
                .map(|rule| {
                    rule.default_configuration
                        .as_ref()
                        .map(|config| Sarif::level_to_level(config.level.clone()))
                        .unwrap_or(Level::Medium)
                })
                .unwrap()
        } else {
            Level::Medium
        }
    }

    fn level_to_level(level: Option<String>) -> Level {
        match level.as_deref() {
            Some("error") => Level::High,
            Some("warning") => Level::Medium,
            Some("note") => Level::Low,
            _ => Level::Medium,
        }
    }

    fn merge_paths(base_uri: &str, relative_path: &str) -> String {
        let base_path = Path::new(base_uri.strip_prefix("file://").unwrap_or(base_uri));
        let relative_path = Path::new(relative_path);

        if base_path.ends_with(relative_path) {
            Self::strip_trailing_slash(&path_to_string(base_path))
        } else {
            Self::strip_trailing_slash(&path_to_string(
                relative_path
                    .strip_prefix("/")
                    .unwrap_or(relative_path)
                    .absolutize_from(base_path)
                    .unwrap(),
            ))
        }
    }

    fn strip_trailing_slash(path_string: &str) -> String {
        if path_string.ends_with('/') {
            path_string.strip_suffix('/').unwrap().into()
        } else {
            path_string.into()
        }
    }

    fn suppressed_issue(suppressions: &Option<Vec<Suppression>>) -> bool {
        if let Some(suppressions) = suppressions {
            // since suppressions is an array, we should check if all of them are accepted
            suppressions.iter().all(|suppression| {
                // honor status if it exists otherwise default to accepted
                suppression.status.is_none() || suppression.status == Some("accepted".to_string())
            })
        } else {
            false
        }
    }
}

impl Parser for Sarif {
    fn parse(&self, plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let mut rule_info = HashMap::new();
        let sarif: SarifFile = serde_json::from_str(output)?;

        for run in &sarif.runs {
            run.tool.driver.rules.iter().for_each(|rule| {
                rule_info.insert(rule.id.clone(), rule);
            });
        }

        for run in &sarif.runs {
            for result in &run.results {
                if Sarif::suppressed_issue(&result.suppressions) {
                    continue;
                }

                let location = Sarif::get_location(
                    result.locations.clone(),
                    run.original_uri_base_ids.clone(),
                );

                if let Some(kind) = &result.kind {
                    if !(kind == "fail" || kind == "review") {
                        info!("Skipping issue with kind: {}, issue: {:?}", kind, result);
                        continue;
                    }
                }

                let rule_key = result
                    .rule_id
                    .clone()
                    .unwrap_or_else(|| result.message.text.clone());

                let issue = Issue {
                    documentation_url: rule_info
                        .get(&rule_key)
                        .map(|rule| rule.help_uri.clone().unwrap_or("".into()))
                        .unwrap_or("".into()),
                    tool: plugin_name.into(),
                    rule_key,
                    message: result.message.text.clone(),
                    category: self.category.unwrap_or(Category::Lint).into(),
                    level: self.get_level(result, &rule_info).into(),
                    location,
                    ..Default::default()
                };

                issues.push(issue);
            }
        }

        Ok(issues)
    }
}

// qlty-ignore: +ripgrep
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = r###"
        {
          "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
          "version": "2.1.0",
          "runs": [
            {
              "results": [
                {
                  "level": "warning",
                  "locations": [
                    {
                      "physicalLocation": {
                        "artifactLocation": {
                          "uri": "/dev/shm/sandbox/detekt_test_repo/example.kt"
                        },
                        "region": {
                          "startColumn": 12,
                          "startLine": 18
                        }
                      }
                    }
                  ],
                  "message": {
                    "text": "A class should always override hashCode when overriding equals and the other way around."
                  },
                  "ruleId": "detekt.potential-bugs.EqualsWithHashCodeExist"
                },
                {
                    "level": "error",
                    "kind": "informational",
                    "locations": [
                      {
                        "physicalLocation": {
                          "artifactLocation": {
                            "uri": "/dev/shm/sandbox/detekt_test_repo/example.kt"
                          },
                          "region": {
                            "startColumn": 12,
                            "startLine": 18
                          }
                        }
                      }
                    ],
                    "message": {
                      "text": "A class should always override hashCode when overriding equals and the other way around."
                    },
                    "ruleId": "detekt.potential-bugs.EqualsWithHashCodeExist"
                }
              ],
              "tool": {
                "driver": {
                  "downloadUri": "https://github.com/detekt/detekt/releases/download/v1.19.0/detekt",
                  "fullName": "detekt",
                  "guid": "022ca8c2-f6a2-4c95-b107-bb72c43263f3",
                  "informationUri": "https://detekt.github.io/detekt",
                  "language": "en",
                  "name": "detekt",
                  "workspace": "detekt",
                  "semanticVersion": "1.19.0",
                  "version": "1.19.0"
                }
              }
            }
          ]
        }
        "###;

        let issues = Sarif::default().parse("sarif", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r"
        - tool: sarif
          ruleKey: detekt.potential-bugs.EqualsWithHashCodeExist
          message: A class should always override hashCode when overriding equals and the other way around.
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: /dev/shm/sandbox/detekt_test_repo/example.kt
            range:
              startLine: 18
              startColumn: 12
              endLine: 18
              endColumn: 12
        ");
    }

    #[test]
    fn parse_tflint() {
        let input = r###"
        {
            "version": "2.1.0",
            "$schema": "https://json.schemastore.org/sarif-2.1.0-rtm.5.json",
            "runs": [
              {
                "tool": {
                  "driver": {
                    "name": "tflint",
                    "version": "0.51.1",
                    "informationUri": "https://github.com/terraform-linters/tflint",
                    "rules": [
                      {
                        "id": "terraform_required_version",
                        "shortDescription": {
                          "text": ""
                        },
                        "helpUri": "https://github.com/terraform-linters/tflint-ruleset-terraform/blob/v0.7.0/docs/rules/terraform_required_version.md"
                      },
                      {
                        "id": "terraform_typed_variables",
                        "shortDescription": {
                          "text": ""
                        },
                        "helpUri": "https://github.com/terraform-linters/tflint-ruleset-terraform/blob/v0.7.0/docs/rules/terraform_typed_variables.md"
                      },
                      {
                        "id": "terraform_unused_declarations",
                        "shortDescription": {
                          "text": ""
                        },
                        "helpUri": "https://github.com/terraform-linters/tflint-ruleset-terraform/blob/v0.7.0/docs/rules/terraform_unused_declarations.md"
                      }
                    ]
                  }
                },
                "results": [
                  {
                    "ruleId": "terraform_required_version",
                    "level": "warning",
                    "message": {
                      "text": "terraform \"required_version\" attribute is required"
                    }
                  },
                  {
                    "ruleId": "terraform_typed_variables",
                    "level": "warning",
                    "message": {
                      "text": "`foo` variable has no type"
                    },
                    "locations": [
                      {
                        "physicalLocation": {
                          "artifactLocation": {
                            "uri": "aws.in.tf"
                          },
                          "region": {
                            "startLine": 10,
                            "startColumn": 1,
                            "endLine": 10,
                            "endColumn": 15
                          }
                        }
                      }
                    ]
                  },
                  {
                    "ruleId": "terraform_unused_declarations",
                    "level": "warning",
                    "message": {
                      "text": "variable \"foo\" is declared but not used"
                    },
                    "locations": [
                      {
                        "physicalLocation": {
                          "artifactLocation": {
                            "uri": "aws.in.tf"
                          },
                          "region": {
                            "startLine": 10,
                            "startColumn": 1,
                            "endLine": 10,
                            "endColumn": 15
                          }
                        }
                      }
                    ]
                  },
                  {
                    "ruleId": "terraform_unused_declarations",
                    "level": "warning",
                    "message": {
                      "text": "variable \"region\" is declared but not used"
                    },
                    "locations": [
                      {
                        "physicalLocation": {
                          "artifactLocation": {
                            "uri": "aws.in.tf"
                          },
                          "region": {
                            "startLine": 1,
                            "startColumn": 1,
                            "endLine": 1,
                            "endColumn": 18
                          }
                        }
                      }
                    ]
                  }
                ]
              },
              {
                "tool": {
                  "driver": {
                    "name": "tflint-errors",
                    "version": "0.51.1",
                    "informationUri": "https://github.com/terraform-linters/tflint"
                  }
                },
                "results": []
              }
            ]
          }
        "###;

        let issues = Sarif::default().parse("sarif", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: sarif
          ruleKey: terraform_required_version
          message: "terraform \"required_version\" attribute is required"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://github.com/terraform-linters/tflint-ruleset-terraform/blob/v0.7.0/docs/rules/terraform_required_version.md"
        - tool: sarif
          ruleKey: terraform_typed_variables
          message: "`foo` variable has no type"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://github.com/terraform-linters/tflint-ruleset-terraform/blob/v0.7.0/docs/rules/terraform_typed_variables.md"
          location:
            path: aws.in.tf
            range:
              startLine: 10
              startColumn: 1
              endLine: 10
              endColumn: 15
        - tool: sarif
          ruleKey: terraform_unused_declarations
          message: "variable \"foo\" is declared but not used"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://github.com/terraform-linters/tflint-ruleset-terraform/blob/v0.7.0/docs/rules/terraform_unused_declarations.md"
          location:
            path: aws.in.tf
            range:
              startLine: 10
              startColumn: 1
              endLine: 10
              endColumn: 15
        - tool: sarif
          ruleKey: terraform_unused_declarations
          message: "variable \"region\" is declared but not used"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://github.com/terraform-linters/tflint-ruleset-terraform/blob/v0.7.0/docs/rules/terraform_unused_declarations.md"
          location:
            path: aws.in.tf
            range:
              startLine: 1
              startColumn: 1
              endLine: 1
              endColumn: 18
        "#);
    }

    #[test]
    fn parse_srcroot_root_path() {
        let input = r###"
            {
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [
                {
                "originalUriBaseIds": {
                    "%SRCROOT%": {
                    "uri": "file:///path/to/test"
                    }
                },
                "results": [
                    {
                    "level": "error",
                    "locations": [
                        {
                        "physicalLocation": {
                            "artifactLocation": {
                            "uri": "/path/to/src/Main.kt",
                            "uriBaseId": "%SRCROOT%"
                            },
                            "region": {
                            "startColumn": 37,
                            "startLine": 1
                            }
                        }
                        }
                    ],
                    "message": {
                        "text": "First line of body expression fits on same line as function signature"
                    },
                    "ruleId": "standard:function-signature"
                    },
                    {
                    "level": "error",
                    "locations": [
                        {
                        "physicalLocation": {
                            "artifactLocation": {
                            "uri": "/path/to/src/Main.kt",
                            "uriBaseId": "%SRCROOT%"
                            },
                            "region": {
                            "startColumn": 1,
                            "startLine": 5
                            }
                        }
                        }
                    ],
                    "message": {
                        "text": "Class body should not start with blank line"
                    },
                    "ruleId": "standard:no-empty-first-line-in-class-body"
                    }
                ],
                "tool": {
                    "driver": {
                    "downloadUri": "https://github.com/pinterest/ktlint/releases/tag/1.5.0",
                    "fullName": "ktlint",
                    "informationUri": "https://github.com/pinterest/ktlint/",
                    "language": "en",
                    "name": "ktlint",
                    "organization": "pinterest",
                    "rules": [
                    ],
                    "semanticVersion": "1.5.0",
                    "version": "1.5.0"
                    }
                }
                }
            ]
        }
        "###;

        let issues = Sarif::default().parse("sarif", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: sarif
          ruleKey: "standard:function-signature"
          message: First line of body expression fits on same line as function signature
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: /path/to/test/path/to/src/Main.kt
            range:
              startLine: 1
              startColumn: 37
              endLine: 1
              endColumn: 37
        - tool: sarif
          ruleKey: "standard:no-empty-first-line-in-class-body"
          message: Class body should not start with blank line
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: /path/to/test/path/to/src/Main.kt
            range:
              startLine: 5
              startColumn: 1
              endLine: 5
              endColumn: 1
        "#);
    }

    #[test]
    fn parse_rule_level() {
        let input = r###"
        {
          "$schema": "https://docs.oasis-open.org/sarif/sarif/v2.1.0/os/schemas/sarif-schema-2.1.0.json",
          "runs": [
            {
              "invocations": [
                {
                  "executionSuccessful": true,
                  "toolExecutionNotifications": []
                }
              ],
              "results": [
                {
                  "fingerprints": {
                    "matchBasedId/v1": "3437f3fbb99391b3b9d42722a007e8b9164d6fd5946c48f031e0c09bc8dc2579fb279bb78ad3c5264a02ea686111273862137545c36aed784cf54e4b86abf396_0"
                  },
                  "locations": [
                    {
                      "physicalLocation": {
                        "artifactLocation": {
                          "uri": "remix/app/routes/auth.github.callback/route.tsx",
                          "uriBaseId": "%SRCROOT%"
                        },
                        "region": {
                          "endColumn": 2,
                          "endLine": 71,
                          "snippet": {
                            "text": "export async function loader({ request }: LoaderFunctionArgs) {\n  let returnTo = (await returnToCookie.parse(request.headers.get(\"Cookie\"))) ?? \"/workspaces\";\n\n  if (!returnTo.startsWith(\"/\")) {\n    throw new ParametersError(\"Invalid returnTo\");\n  }\n\n  // TODO: Since we are migrating users from Quality Classic, we need to do some custom logic for associating\n  // the activation token with the user. This will be removed once we are done migrating users.\n  // and replaced with:\n  // return authenticator.authenticate(\"github\", request, {\n  //     successRedirect: returnTo,\n  //     throwOnError: true,\n  // })\n  try {\n    const user =  await authenticator.authenticate(\"github\", request);\n    if (!user) throw new Error(\"Authentication failed\");\n    const activationToken = await activationTokenCookie.parse(request.headers.get(\"Cookie\"));\n\n    if (activationToken && !validateToken(activationToken)) {\n      return redirect(\"/login?activationTokenError=invalid\", {\n        headers: {\n          \"Set-Cookie\": await activationTokenCookie.serialize(null, { maxAge: -1 }) // Clear the cookie\n        }\n      })\n    }\n\n    if (activationToken && !user.activationToken) {\n      await updateUser(user, { activationToken });\n    }\n\n    const session = await getSession();\n    session.set(\"user\", user);\n    session.set(\"strategy\", \"github\");\n\n    return redirect(returnTo, {\n      headers: {\n        \"Set-Cookie\": await commitSession(session),\n      },\n\n    });\n  } catch (error) {\n    // Because redirects work by throwing a Response, you need to check if the\n    // caught error is a response and return it or throw it again\n    if (error instanceof Response) return error;\n\n    if (error instanceof PrismaClientKnownRequestError) {\n      const { code, meta } = error as PrismaClientKnownRequestError;\n      const targets: string[] = meta?.target as string[] ?? [];\n\n      if (code === \"P2002\" && targets.includes(\"activationToken\")) {\n        return redirect(\"/login?activationTokenError=inUse\", {\n          headers: {\n            \"Set-Cookie\": await activationTokenCookie.serialize(null, { maxAge: -1 }) // Clear the cookie\n          }\n        });\n      }\n    }\n    logger.error(error);\n    return redirect(\"/login\");\n  }\n}"
                          },
                          "startColumn": 8,
                          "startLine": 10
                        }
                      }
                    }
                  ],
                  "message": {
                    "text": "loader must call `await authorize(...)`"
                  },
                  "properties": {},
                  "ruleId": "loader-requires-auth"
                }
              ],
              "tool": {
                "driver": {
                  "name": "Semgrep OSS",
                  "rules": [
                    {
                      "defaultConfiguration": {
                        "level": "error"
                      },
                      "fullDescription": {
                        "text": "loader must call `await authorize(...)`"
                      },
                      "help": {
                        "markdown": "loader must call `await authorize(...)`",
                        "text": "loader must call `await authorize(...)`"
                      },
                      "id": "loader-requires-auth",
                      "name": "loader-requires-auth",
                      "properties": {
                        "precision": "very-high",
                        "tags": []
                      },
                      "shortDescription": {
                        "text": "Semgrep Finding: loader-requires-auth"
                      }
                    }
                  ],
                  "semanticVersion": "1.68.0"
                }
              }
            }
          ],
          "version": "2.1.0"
        }

        "###;

        let issues = Sarif::default().parse("sarif", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: sarif
          ruleKey: loader-requires-auth
          message: "loader must call `await authorize(...)`"
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: remix/app/routes/auth.github.callback/route.tsx
            range:
              startLine: 10
              startColumn: 8
              endLine: 71
              endColumn: 2
        "#);
    }

    #[test]
    fn test_merge_paths_basic() {
        assert_eq!(Sarif::merge_paths("/dir", "foo.tf"), "/dir/foo.tf");
    }

    #[test]
    fn test_merge_paths_basic_with_slash() {
        assert_eq!(Sarif::merge_paths("/dir", "/foo.tf"), "/dir/foo.tf");
    }

    #[test]
    fn test_merge_paths_protocol() {
        assert_eq!(Sarif::merge_paths("file:///dir", "foo.tf"), "/dir/foo.tf");
    }

    #[test]
    fn test_merge_paths_same() {
        assert_eq!(
            Sarif::merge_paths("/dir/foo.tf", "/dir/foo.tf"),
            "/dir/foo.tf"
        );
    }

    #[test]
    fn test_merge_paths_trailing_slash_right() {
        assert_eq!(
            Sarif::merge_paths("/dir/foo.tf", "/dir/foo.tf/"),
            "/dir/foo.tf"
        );
    }

    #[test]
    fn test_merge_paths_trailing_slash_left() {
        assert_eq!(
            Sarif::merge_paths("/dir/foo.tf/", "/dir/foo.tf"),
            "/dir/foo.tf"
        );
    }

    #[test]
    fn test_merge_paths_trailing_slash_both() {
        assert_eq!(
            Sarif::merge_paths("/dir/foo.tf/", "/dir/foo.tf/"),
            "/dir/foo.tf"
        );
    }

    #[test]
    fn test_merge_paths_append() {
        assert_eq!(Sarif::merge_paths("/dir/", "foo/bar.tf"), "/dir/foo/bar.tf");
    }

    #[test]
    fn test_merge_paths_relative() {
        assert_eq!(Sarif::merge_paths("/dir/", "../foo.tf"), "/foo.tf");
    }

    #[test]
    fn test_merge_paths_original_test_windows() {
        assert_eq!(
            Sarif::merge_paths("file://C:/path/to/dir/", "/other_file.rs"),
            "C:/path/to/dir/other_file.rs"
        );
    }

    #[test]
    fn parse_suppressed_issue() {
        let input = r###"
{
    "version": "2.1.0",
    "$schema": "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0-rtm.5.json",
    "runs": [
      {
        "tool": {
          "driver": {
            "name": "Brakeman",
            "informationUri": "https://brakemanscanner.org",
            "semanticVersion": "6.1.2",
            "rules": [
              {
                "id": "BRAKE0120",
                "name": "EOLRails/Unmaintained Dependency",
                "fullDescription": {
                  "text": "Checks for unsupported versions of Rails."
                },
                "helpUri": "https://brakemanscanner.org/docs/warning_types/unmaintained_dependency/",
                "help": {
                  "text": "More info: https://brakemanscanner.org/docs/warning_types/unmaintained_dependency/.",
                  "markdown": "[More info](https://brakemanscanner.org/docs/warning_types/unmaintained_dependency/)."
                },
                "properties": {
                  "tags": [
                    "EOLRails"
                  ]
                }
              },
              {
                "id": "BRAKE0013",
                "name": "Evaluation/Dangerous Eval",
                "fullDescription": {
                  "text": "Searches for evaluation of user input."
                },
                "helpUri": "https://brakemanscanner.org/docs/warning_types/dangerous_eval/",
                "help": {
                  "text": "More info: https://brakemanscanner.org/docs/warning_types/dangerous_eval/.",
                  "markdown": "[More info](https://brakemanscanner.org/docs/warning_types/dangerous_eval/)."
                },
                "properties": {
                  "tags": [
                    "Evaluation"
                  ]
                }
              }
            ]
          }
        },
        "results": [
          {
            "ruleId": "BRAKE0120",
            "ruleIndex": 0,
            "level": "error",
            "message": {
              "text": "Support for Rails 5.2.8.1 ended on 2022-06-01."
            },
            "locations": [
              {
                "physicalLocation": {
                  "artifactLocation": {
                    "uri": "Gemfile.lock",
                    "uriBaseId": "%SRCROOT%"
                  },
                  "region": {
                    "startLine": 105
                  }
                }
              }
            ],
            "suppressions": [
              {
                "status": "rejected"
              }
            ]
          },
          {
            "ruleId": "BRAKE0013",
            "ruleIndex": 1,
            "level": "error",
            "message": {
              "text": "User input in eval."
            },
            "locations": [
              {
                "physicalLocation": {
                  "artifactLocation": {
                    "uri": "app/helpers/users_helper.rb",
                    "uriBaseId": "%SRCROOT%"
                  },
                  "region": {
                    "startLine": 3
                  }
                }
              }
            ],
            "suppressions": [
              {
                "kind": "external",
                "justification": "",
                "location": {
                  "physicalLocation": {
                    "artifactLocation": {
                      "uri": "config/brakeman.ignore",
                      "uriBaseId": "%SRCROOT%"
                    }
                  }
                }
              }
            ]
          }
        ]
      }
    ]
  }
        "###;
        let issues = Sarif::default().parse("sarif", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: sarif
          ruleKey: BRAKE0120
          message: Support for Rails 5.2.8.1 ended on 2022-06-01.
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          documentationUrl: "https://brakemanscanner.org/docs/warning_types/unmaintained_dependency/"
          location:
            path: Gemfile.lock
            range:
              startLine: 105
              startColumn: 1
              endLine: 105
              endColumn: 1
        "#);
    }

    #[test]
    fn parse_rule_id_null_issue() {
        let input = r###"
{
    "version": "2.1.0",
    "$schema": "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0-rtm.5.json",
    "runs": [
      {
        "tool": {
          "driver": {
            "name": "Brakeman",
            "informationUri": "https://brakemanscanner.org",
            "semanticVersion": "6.1.2",
            "rules": []
          }
        },
        "results": [
          {
            "level": "error",
            "locations": [
              {
                "physicalLocation": {
                  "artifactLocation": {
                    "uri": "file:///home/runner/work/portal-admin/partnership_instant_contact/tests/test_models.py"
                  },
                  "region": {
                    "endColumn": 14,
                    "endLine": 21,
                    "startColumn": 13,
                    "startLine": 21
                  }
                }
              }
            ],
            "message": {
              "text": "SyntaxError"
            },
            "ruleId": null
          }
        ]
      }
    ]
  }
        "###;
        let issues = Sarif::default().parse("sarif", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: sarif
          ruleKey: SyntaxError
          message: SyntaxError
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: "file:///home/runner/work/portal-admin/partnership_instant_contact/tests/test_models.py"
            range:
              startLine: 21
              startColumn: 13
              endLine: 21
              endColumn: 14
        "#);
    }
}
