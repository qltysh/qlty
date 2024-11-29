use super::{sarif::Sarif, Parser};
use anyhow::{Ok, Result};
use qlty_types::analysis::v1::{Category, Issue};
use serde::{Deserialize, Serialize};

const VALID_MESSAGE_LINES_PREFIX: [&str; 5] = [
    "Package:",
    "Installed Version:",
    "Fixed Version:",
    "Match:",
    "Secret",
];

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct TrivySarif {
    pub category: Option<Category>,
}

impl TrivySarif {
    pub fn new(category: Option<Category>) -> Self {
        Self { category }
    }
}

impl Parser for TrivySarif {
    fn parse(&self, plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let issues = Sarif::default()
            .parse(plugin_name, output)?
            .into_iter()
            .map(|mut issue| {
                issue.category = self.category.unwrap_or(Category::Vulnerability).into();
                issue.documentation_url = extract_link(&issue.message);
                issue.message = sanitize_message(&issue.message);
                issue
            })
            .collect();

        Ok(issues)
    }
}

fn extract_link(message: &str) -> String {
    if message.contains("Link: ") {
        message
            .split("Link: ")
            .nth(1)
            .and_then(|link_section| link_section.split(')').next())
            .and_then(|link_text| link_text.split('(').nth(1))
            .map(|link| link.to_string())
            .unwrap_or_default()
    } else {
        "".to_string()
    }
}

fn sanitize_message(message: &str) -> String {
    if message.contains("Message: ") {
        // for config scan
        message
            .split("Message: ")
            .nth(1)
            .and_then(|msg| msg.lines().next())
            .unwrap_or_default()
            .to_string()
    } else {
        // for vuln and secrets scan
        message
            .lines()
            .filter(|line| {
                VALID_MESSAGE_LINES_PREFIX
                    .iter()
                    .any(|prefix| line.starts_with(prefix))
            })
            .collect::<Vec<&str>>()
            .join("\n")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_MESSAGES: [&str; 5] = [
        "Artifact: load-balancing.tf\nType: terraform\nVulnerability AVD-AWS-0052\nSeverity: HIGH\nMessage: Application load balancer is not set to drop invalid headers.\nLink: [AVD-AWS-0052](https://avd.aquasec.com/misconfig/avd-aws-0052)",
        "Artifact: main.tf\nType: terraform\nVulnerability AVD-AWS-0017\nSeverity: LOW\nMessage: Log group is not encrypted.\nLink: [AVD-AWS-0017](https://avd.aquasec.com/misconfig/avd-aws-0017)",
        "Artifact: ecs.tf\nType: terraform\nVulnerability AVD-AWS-0034\nSeverity: LOW\nMessage: Cluster does not have container insights enabled.\nLink: [AVD-AWS-0034](https://avd.aquasec.com/misconfig/avd-aws-0034)",
        "Package: github.com/go-gitea/gitea\nInstalled Version: 1.2.3\nVulnerability CVE-2020-13246\nSeverity: HIGH\nFixed Version: 1.12.0\nLink: [CVE-2020-13246](https://avd.aquasec.com/nvd/cve-2020-13246)",
        "Artifact: /basic.in.py\nType: \nSecret Asymmetric Private Key\nSeverity: HIGH\nMatch: BEGIN OPENSSH PRIVATE KEY-----*******-----END OPENSSH PRI",
    ];

    #[test]
    fn test_parse() {
        let input = r###"
        {
            "version": "2.1.0",
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "runs": [
            {
                "tool": {
                    "driver": {
                        "fullName": "Trivy Vulnerability Scanner",
                        "informationUri": "https://github.com/aquasecurity/trivy",
                        "name": "Trivy",
                        "rules": [],
                        "version": "0.50.1"
                    }
                },
                "results": [
                {
                    "ruleId": "AVD-AWS-0034",
                    "ruleIndex": 0,
                    "level": "note",
                    "message": {
                    "text": "Artifact: ecs.tf\nType: terraform\nVulnerability AVD-AWS-0034\nSeverity: LOW\nMessage: Cluster does not have container insights enabled.\nLink: [AVD-AWS-0034](https://avd.aquasec.com/misconfig/avd-aws-0034)"
                    },
                    "locations": [
                    {
                        "physicalLocation": {
                        "artifactLocation": {
                            "uri": "ecs.tf",
                            "uriBaseId": "ROOTPATH"
                        },
                        "region": {
                            "startLine": 6,
                            "startColumn": 1,
                            "endLine": 6,
                            "endColumn": 1
                        }
                        },
                        "message": {
                        "text": "ecs.tf"
                        }
                    }
                    ]
                }
                ],
                "columnKind": "utf16CodeUnits",
                "originalUriBaseIds": {
                "ROOTPATH": {
                    "uri": "file:///private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/qlty/AJ89q1iO-rw/terraform/modules/retool/regional/retool/ecs.tf/"
                }
                }
            }
            ]
        }
        "###;

        let issues = TrivySarif::default().parse("trivy", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: trivy
          ruleKey: AVD-AWS-0034
          message: Cluster does not have container insights enabled.
          level: LEVEL_LOW
          category: CATEGORY_VULNERABILITY
          documentationUrl: "https://avd.aquasec.com/misconfig/avd-aws-0034"
          location:
            path: /private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/qlty/AJ89q1iO-rw/terraform/modules/retool/regional/retool/ecs.tf
            range:
              startLine: 6
              startColumn: 1
              endLine: 6
              endColumn: 1
        "#);
    }

    #[test]
    fn test_sanitize_message() {
        let test_results =[
            "Application load balancer is not set to drop invalid headers.",
            "Log group is not encrypted.",
            "Cluster does not have container insights enabled.",
            "Package: github.com/go-gitea/gitea\nInstalled Version: 1.2.3\nFixed Version: 1.12.0",
            "Secret Asymmetric Private Key\nMatch: BEGIN OPENSSH PRIVATE KEY-----*******-----END OPENSSH PRI"
        ];

        for i in 0..TEST_MESSAGES.len() {
            let sanitized_message = sanitize_message(TEST_MESSAGES[i]);
            assert_eq!(sanitized_message, test_results[i]);
        }
    }

    #[test]
    fn test_extract_link() {
        let test_results = [
            "https://avd.aquasec.com/misconfig/avd-aws-0052",
            "https://avd.aquasec.com/misconfig/avd-aws-0017",
            "https://avd.aquasec.com/misconfig/avd-aws-0034",
            "https://avd.aquasec.com/nvd/cve-2020-13246",
            "",
        ];

        for i in 0..TEST_MESSAGES.len() {
            let sanitized_message = extract_link(TEST_MESSAGES[i]);
            assert_eq!(sanitized_message, test_results[i]);
        }
    }
}
