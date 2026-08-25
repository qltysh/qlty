use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct Report {
    // Unused files (knip <6 reports them at the top level)
    #[serde(default)]
    pub files: Vec<String>,

    // Issues within files
    pub issues: Vec<KnipIssue>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct KnipIssue {
    pub file: String,

    // Unused files (knip >=6 reports them per issue row)
    #[serde(default)]
    pub files: Vec<KnipIdentifier>,

    // Unused dependencies
    #[serde(default)]
    pub dependencies: Vec<KnipIdentifier>,
    #[serde(default, alias = "devDependencies")]
    pub dev_dependencies: Vec<KnipIdentifier>,
    #[serde(default, alias = "optionalPeerDependencies")]
    pub optional_peer_dependencies: Vec<KnipIdentifier>,

    // Unlisted dependencies
    #[serde(default)]
    pub unlisted: Vec<KnipIdentifier>,

    // Unlisted binaries
    #[serde(default)]
    pub binaries: Vec<KnipIdentifier>,

    // Unused exports
    #[serde(default)]
    pub exports: Vec<KnipIdentifier>,
    #[serde(alias = "nsExports")]
    pub ns_exports: Option<Vec<KnipIdentifier>>,

    // Unused exported types
    #[serde(default)]
    pub types: Vec<KnipIdentifier>,
    #[serde(alias = "nsTypes")]
    pub ns_types: Option<Vec<KnipIdentifier>>,

    // Unresolved imports
    #[serde(default)]
    pub unresolved: Vec<KnipIdentifier>,

    // Unused exported enum or class members
    // (knip <6 groups them by parent symbol, knip >=6 uses a flat list
    // with the parent recorded on each identifier's `namespace`)
    #[serde(alias = "enumMembers")]
    pub enum_members: Option<KnipMembers>,
    #[serde(alias = "classMembers")]
    pub class_members: Option<KnipMembers>,

    // Duplicate exports
    #[serde(default)]
    pub duplicates: Vec<Vec<KnipIdentifier>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum KnipMembers {
    Grouped(HashMap<String, Vec<KnipIdentifier>>),
    Flat(Vec<KnipIdentifier>),
}

impl KnipMembers {
    fn into_pairs(self) -> Vec<(String, KnipIdentifier)> {
        match self {
            KnipMembers::Grouped(map) => map
                .into_iter()
                .flat_map(|(parent, members)| {
                    members
                        .into_iter()
                        .map(move |member| (parent.clone(), member))
                })
                .collect(),
            KnipMembers::Flat(members) => members
                .into_iter()
                .map(|member| (member.namespace.clone().unwrap_or_default(), member))
                .collect(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct KnipIdentifier {
    pub name: String,
    pub namespace: Option<String>,
    pub line: Option<i32>,
    pub col: Option<i32>,
    pub pos: Option<i32>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Knip {}

impl Parser for Knip {
    fn parse(&self, _plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let report: Report = serde_json::from_str(output)?;

        for file in report.files {
            issues.push(build_issue("unused-file", "Unused file".into(), file));
        }

        for issue in report.issues {
            for file in issue.files {
                issues.push(build_issue("unused-file", "Unused file".into(), file.name));
            }

            for dependency in issue.dependencies {
                issues.push(build_issue(
                    "unused-dependency",
                    format!("Unused dependency: {}", dependency.name),
                    issue.file.clone(),
                ));
            }

            for dev_dependency in issue.dev_dependencies {
                issues.push(build_issue(
                    "unused-dev-dependency",
                    format!("Unused dev dependency: {}", dev_dependency.name),
                    issue.file.clone(),
                ));
            }

            for optional_peer_dependency in issue.optional_peer_dependencies {
                issues.push(build_issue(
                    "referenced-optional-peer-dependency",
                    format!(
                        "Referenced optional peer dependency: {}",
                        optional_peer_dependency.name
                    ),
                    issue.file.clone(),
                ));
            }

            for unlisted in issue.unlisted {
                issues.push(build_issue(
                    "unlisted-dependency",
                    format!("Unlisted dependency: {}", unlisted.name),
                    issue.file.clone(),
                ));
            }

            for binary in issue.binaries {
                issues.push(build_issue(
                    "unlisted-binary",
                    format!("Unlisted binary: {}", binary.name),
                    issue.file.clone(),
                ));
            }

            for unresolved in issue.unresolved {
                issues.push(build_issue(
                    "unresolved-import",
                    format!("Unresolved import: {}", unresolved.name),
                    issue.file.clone(),
                ));
            }

            for export in issue.exports {
                issues.push(build_issue(
                    "unused-export",
                    format!("Unused export: {}", export.name),
                    issue.file.clone(),
                ));
            }

            for ns_export in issue.ns_exports.unwrap_or_default() {
                issues.push(build_issue(
                    "export-in-used-namespace",
                    format!("Export in used namespace: {}", ns_export.name),
                    issue.file.clone(),
                ));
            }

            for export_type in issue.types {
                issues.push(build_issue(
                    "unused-exported-type",
                    format!("Unused exported type: {}", export_type.name),
                    issue.file.clone(),
                ));
            }

            for ns_type in issue.ns_types.unwrap_or_default() {
                issues.push(build_issue(
                    "exported-type-in-used-namespace",
                    format!("Exported type in used namespace: {}", ns_type.name),
                    issue.file.clone(),
                ));
            }

            for (enum_name, member) in issue
                .enum_members
                .map(KnipMembers::into_pairs)
                .unwrap_or_default()
            {
                issues.push(build_issue(
                    "unused-exported-enum-member",
                    format!("Unused exported enum member: {}.{}", enum_name, member.name),
                    issue.file.clone(),
                ));
            }

            for (class_name, member) in issue
                .class_members
                .map(KnipMembers::into_pairs)
                .unwrap_or_default()
            {
                issues.push(build_issue(
                    "unused-exported-class-member",
                    format!(
                        "Unused exported class member: {}.{}",
                        class_name, member.name
                    ),
                    issue.file.clone(),
                ));
            }

            for duplicate in issue.duplicates {
                let mut message = "Duplicate export: ".to_string();
                for instance in duplicate {
                    message.push_str(format!("Exporting '{}'", instance.name).as_str());

                    if let Some(line) = instance.line {
                        message.push_str(format!(" line: {}", line).as_str());
                    }

                    if let Some(col) = instance.col {
                        message.push_str(format!(" col: {}", col).as_str());
                    }

                    message.push_str(". ");
                }
                issues.push(build_issue(
                    "duplicate-export",
                    message.trim().into(),
                    issue.file.clone(),
                ));
            }
        }

        Ok(issues)
    }
}

fn build_issue(rule_key: &str, message: String, path: String) -> Issue {
    Issue {
        tool: "knip".into(),
        message,
        category: Category::DeadCode.into(),
        level: Level::Medium.into(),
        rule_key: rule_key.into(),
        location: Some(Location {
            path,
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = r###"
        {
            "files": ["src/hello.js"],
            "issues": [
                {
                "file": "package.json",
                "dependencies": [
                    { "name": "@aws-sdk/client-batch" },
                    { "name": "@aws-sdk/client-cloudwatch-logs" }
                ],
                "devDependencies": [],
                "optionalPeerDependencies": [],
                "unlisted": [],
                "binaries": [],
                "unresolved": [],
                "exports": [],
                "types": [],
                "enumMembers": {},
                "duplicates": []
                },
                {
                "file": "index.js",
                "dependencies": [],
                "devDependencies": [],
                "optionalPeerDependencies": [],
                "unlisted": [{ "name": "src/mistake.js" }],
                "binaries": [],
                "unresolved": [],
                "exports": [],
                "types": [],
                "enumMembers": {},
                "duplicates": []
                },
                {
                "file": "refresher.js",
                "dependencies": [],
                "devDependencies": [],
                "optionalPeerDependencies": [],
                "unlisted": [],
                "binaries": [],
                "unresolved": [],
                "exports": [],
                "types": [],
                "enumMembers": {},
                "duplicates":[[{"name":"Refresher","line":29,"col":5,"pos":835},{"name":"default","line":223,"col":15,"pos":7052}]]
                }
            ]
        }
        "###;

        let issues = Knip::default().parse("knip", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: knip
          ruleKey: unused-file
          message: Unused file
          level: LEVEL_MEDIUM
          category: CATEGORY_DEAD_CODE
          location:
            path: src/hello.js
        - tool: knip
          ruleKey: unused-dependency
          message: "Unused dependency: @aws-sdk/client-batch"
          level: LEVEL_MEDIUM
          category: CATEGORY_DEAD_CODE
          location:
            path: package.json
        - tool: knip
          ruleKey: unused-dependency
          message: "Unused dependency: @aws-sdk/client-cloudwatch-logs"
          level: LEVEL_MEDIUM
          category: CATEGORY_DEAD_CODE
          location:
            path: package.json
        - tool: knip
          ruleKey: unlisted-dependency
          message: "Unlisted dependency: src/mistake.js"
          level: LEVEL_MEDIUM
          category: CATEGORY_DEAD_CODE
          location:
            path: index.js
        - tool: knip
          ruleKey: duplicate-export
          message: "Duplicate export: Exporting 'Refresher' line: 29 col: 5. Exporting 'default' line: 223 col: 15."
          level: LEVEL_MEDIUM
          category: CATEGORY_DEAD_CODE
          location:
            path: refresher.js
        "#);
    }

    #[test]
    fn parse_v6() {
        let input = r###"
        {
            "issues": [
                {
                "file": "src/hello.js",
                "binaries": [],
                "catalog": [],
                "dependencies": [],
                "devDependencies": [],
                "duplicates": [],
                "enumMembers": [],
                "exports": [],
                "files": [{ "name": "src/hello.js" }],
                "namespaceMembers": [],
                "optionalPeerDependencies": [],
                "types": [],
                "unlisted": [],
                "unresolved": []
                },
                {
                "file": "package.json",
                "binaries": [],
                "catalog": [],
                "dependencies": [{ "name": "@aws-sdk/client-batch" }],
                "devDependencies": [],
                "duplicates": [],
                "enumMembers": [],
                "exports": [],
                "files": [],
                "namespaceMembers": [],
                "optionalPeerDependencies": [],
                "types": [],
                "unlisted": [],
                "unresolved": []
                },
                {
                "file": "colors.ts",
                "binaries": [],
                "catalog": [],
                "dependencies": [],
                "devDependencies": [],
                "duplicates": [],
                "enumMembers": [{ "namespace": "Colors", "name": "BLUE", "line": 3, "col": 3, "pos": 40 }],
                "exports": [],
                "files": [],
                "namespaceMembers": [],
                "optionalPeerDependencies": [],
                "types": [],
                "unlisted": [],
                "unresolved": []
                }
            ]
        }
        "###;

        let issues = Knip::default().parse("knip", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: knip
          ruleKey: unused-file
          message: Unused file
          level: LEVEL_MEDIUM
          category: CATEGORY_DEAD_CODE
          location:
            path: src/hello.js
        - tool: knip
          ruleKey: unused-dependency
          message: "Unused dependency: @aws-sdk/client-batch"
          level: LEVEL_MEDIUM
          category: CATEGORY_DEAD_CODE
          location:
            path: package.json
        - tool: knip
          ruleKey: unused-exported-enum-member
          message: "Unused exported enum member: Colors.BLUE"
          level: LEVEL_MEDIUM
          category: CATEGORY_DEAD_CODE
          location:
            path: colors.ts
        "#);
    }
}
