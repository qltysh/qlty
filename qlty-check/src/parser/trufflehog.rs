use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Output {
    source_metadata: SourceMetadata,
    detector_name: String,
    redacted: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SourceMetadata {
    #[serde(alias = "Data")]
    data: MetadataData,
}

#[derive(Serialize, Deserialize, Debug)]
struct MetadataData {
    #[serde(alias = "Filesystem")]
    filesystem: FilesystemData,
}

#[derive(Serialize, Deserialize, Debug)]
struct FilesystemData {
    file: String,
    line: Option<u32>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Trufflehog {}

impl Parser for Trufflehog {
    fn parse(&self, _plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];

        output.trim().lines().for_each(|trufflehog_source| {
            let parsed_data: Output =
                serde_json::from_str(trufflehog_source.trim()).expect("Error parsing JSON data");

            let range = parsed_data.source_metadata.data.filesystem.line.map(|line| Range {
                    start_line: line + 1,
                    ..Default::default()
                });

            let issue = Issue {
                tool: "trufflehog".into(),
                message: format!("Secret detected {}", parsed_data.redacted),
                category: Category::Vulnerability.into(),
                level: Level::High.into(),
                rule_key: parsed_data.detector_name,
                location: Some(Location {
                    path: parsed_data.source_metadata.data.filesystem.file,
                    range,
                }),
                ..Default::default()
            };

            issues.push(issue);
        });

        Ok(issues)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = r###"
        {"SourceMetadata":{"Data":{"Filesystem":{"file":"secrets.in.py","line":7}}},"SourceID":1,"SourceType":15,"SourceName":"trufflehog - filesystem","DetectorType":17,"DetectorName":"URI","DecoderName":"PLAIN","Verified":true,"Raw":"https://admin:admin@the-internet.herokuapp.com","RawV2":"https://admin:admin@the-internet.herokuapp.com/basic_auth","Redacted":"https://admin:********@the-internet.herokuapp.com","ExtraData":null,"StructuredData":null}
        {"SourceMetadata":{"Data":{"Filesystem":{"file":"second.in.py","line":7}}},"SourceID":1,"SourceType":15,"SourceName":"trufflehog - filesystem","DetectorType":17,"DetectorName":"URI","DecoderName":"PLAIN","Verified":true,"Raw":"https://admin:admin@the-internet.herokuapp.com","RawV2":"https://admin:admin@the-internet.herokuapp.com/basic_auth","Redacted":"https://admin:********@the-internet.herokuapp.com","ExtraData":null,"StructuredData":null}
        {"SourceMetadata":{"Data":{"Filesystem":{"file":"thrid.in.py"}}},"SourceID":1,"SourceType":15,"SourceName":"trufflehog - filesystem","DetectorType":17,"DetectorName":"URI","DecoderName":"PLAIN","Verified":true,"Raw":"https://admin:admin@the-internet.herokuapp.com","RawV2":"https://admin:admin@the-internet.herokuapp.com/basic_auth","Redacted":"https://admin:********@the-internet.herokuapp.com","ExtraData":null,"StructuredData":null}
        "###;

        let issues = Trufflehog::default().parse("trufflehog", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: trufflehog
          ruleKey: URI
          message: "Secret detected https://admin:********@the-internet.herokuapp.com"
          level: LEVEL_HIGH
          category: CATEGORY_VULNERABILITY
          location:
            path: secrets.in.py
            range:
              startLine: 8
        - tool: trufflehog
          ruleKey: URI
          message: "Secret detected https://admin:********@the-internet.herokuapp.com"
          level: LEVEL_HIGH
          category: CATEGORY_VULNERABILITY
          location:
            path: second.in.py
            range:
              startLine: 8
        - tool: trufflehog
          ruleKey: URI
          message: "Secret detected https://admin:********@the-internet.herokuapp.com"
          level: LEVEL_HIGH
          category: CATEGORY_VULNERABILITY
          location:
            path: thrid.in.py
        "#);
    }
}
