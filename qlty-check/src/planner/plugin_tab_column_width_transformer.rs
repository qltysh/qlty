use crate::source_reader::SourceReader;
use qlty_config::issue_transformer::IssueTransformer;
use qlty_types::analysis::v1::Issue;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct PluginTabColumnWidthTransformer {
    pub source_reader: Arc<dyn SourceReader>,
    pub tab_column_width: usize,
    pub plugin_name: String,
}

impl IssueTransformer for PluginTabColumnWidthTransformer {
    fn transform(&self, mut issue: Issue) -> Option<Issue> {
        if issue.tool == self.plugin_name {
            if let Some(location) = issue.location.as_mut() {
                if let Some(range) = location.range.as_mut() {
                    if let Some([start_column, end_column]) =
                        self.get_correct_columns_for_path_and_range(&location.path, *range)
                    {
                        range.start_column = start_column;
                        range.end_column = end_column;
                    }
                }
            }

            for suggestion in &mut issue.suggestions {
                for replacement in &mut suggestion.replacements {
                    if let Some(location) = replacement.location.as_mut() {
                        if let Some(range) = location.range.as_mut() {
                            if let Some([start_column, end_column]) =
                                self.get_correct_columns_for_path_and_range(&location.path, *range)
                            {
                                range.start_column = start_column;
                                range.end_column = end_column;
                            }
                        }
                    }
                }
            }
        }
        Some(issue)
    }

    fn clone_box(&self) -> Box<dyn IssueTransformer> {
        Box::new(self.clone())
    }
}

impl PluginTabColumnWidthTransformer {
    fn get_correct_columns_for_path_and_range(
        &self,
        file_path: &str,
        range: qlty_types::analysis::v1::Range,
    ) -> Option<[u32; 2]> {
        let contents = self.source_reader.read(file_path.into());
        let (mut start_column, mut end_column) = (range.start_column, range.end_column);
        if let Ok(data) = &contents {
            if let Some(target_line) = data.split('\n').nth(range.start_line as usize - 1) {
                let tabs_before_start = target_line
                    .chars()
                    .take((range.start_column as usize).saturating_sub(1))
                    .filter(|&c| c == '\t')
                    .count();
                let tabs_before_end = target_line
                    .chars()
                    .take((range.end_column as usize).saturating_sub(1))
                    .filter(|&c| c == '\t')
                    .count();
                if tabs_before_start > 0 {
                    start_column = start_column
                        .saturating_sub((tabs_before_start * (self.tab_column_width - 1)) as u32);
                }
                if tabs_before_end > 0 {
                    end_column = end_column
                        .saturating_sub((tabs_before_end * (self.tab_column_width - 1)) as u32);
                }
            }
        }
        Some([start_column, end_column])
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        parser::{shellcheck::Shellcheck, Parser},
        source_reader::SourceReaderFs,
    };

    fn transformed_issues(
        issues: Vec<Issue>,
        plugin_tab_column_width_transformer: &PluginTabColumnWidthTransformer,
    ) -> Vec<Issue> {
        issues
            .iter()
            .map(|issue| plugin_tab_column_width_transformer.transform(issue.clone()))
            .filter(Option::is_some)
            .map(Option::unwrap)
            .collect()
    }

    #[test]
    fn parse_shellcheck_tabs() {
        let input = include_str!(
            "../../tests/fixtures/planner/plugin_tab_column_width_transformer/parse.01.output.txt"
        );

        let plugin_tab_column_width_transformer = PluginTabColumnWidthTransformer {
            source_reader: Arc::new(SourceReaderFs::with_cache(
            [(
                "/tmp/src/main.sh".into(),
                include_str!("../../tests/fixtures/planner/plugin_tab_column_width_transformer/parse.01.input.sh").into(),
            )]
            .into(),
        )),
            tab_column_width: 8,
            plugin_name: "shellcheck".to_string(),
        };

        let issues = Shellcheck {}.parse("shellcheck", input).ok().unwrap();
        let issues = transformed_issues(issues, &plugin_tab_column_width_transformer);

        insta::assert_yaml_snapshot!(issues, @r###"
        - tool: shellcheck
          ruleKey: "2086"
          message: Double quote to prevent globbing and word splitting.
          level: LEVEL_LOW
          category: CATEGORY_LINT
          location:
            path: /tmp/src/main.sh
            range:
              startLine: 5
              startColumn: 12
              endLine: 5
              endColumn: 23
          suggestions:
            - source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: "\""
                  location:
                    path: /tmp/src/main.sh
                    range:
                      startLine: 5
                      startColumn: 12
                      endLine: 5
                      endColumn: 12
                - data: "\""
                  location:
                    path: /tmp/src/main.sh
                    range:
                      startLine: 5
                      startColumn: 23
                      endLine: 5
                      endColumn: 23
        "###);
    }
}
