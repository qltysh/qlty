use crate::{issue_muter::IssueMuter, workspace_reader::WorkspaceReader};
use console::style;
use qlty_config::{
    config::{Match, Set, Triage},
    issue_transformer::IssueTransformer,
    warn_once, QltyConfig,
};
use qlty_types::{category_from_str, level_from_str};
use std::path::Path;

pub trait BasicTransformations {
    fn compute_transformers(
        &self,
        workspace_root: &Path,
        qlty_config: &QltyConfig,
    ) -> Vec<Box<dyn IssueTransformer>> {
        let mut transformers: Vec<Box<dyn IssueTransformer>> = vec![];

        for ignore in &qlty_config.ignore {
            transformers.push(Box::new(ignore.clone()));
        }

        transformers.push(Box::new(IssueMuter::new(WorkspaceReader::new(
            workspace_root.to_path_buf(),
        ))));

        // keep triage last
        let triages = self.build_triages(qlty_config);
        for issue_triage in &triages {
            transformers.push(Box::new(issue_triage.clone()));
        }

        transformers
    }

    fn build_triages(&self, qlty_config: &QltyConfig) -> Vec<Triage> {
        let mut triages = qlty_config.triage.clone();

        if !qlty_config.overrides.is_empty() {
            warn_once(&format!(
                "{} The `{}` field in qlty.toml is deprecated. Please use `{}` instead.",
                style("WARNING:").bold().yellow(),
                style("[[override]]").bold(),
                style("[[triage]]").bold()
            ));

            for issue_override in &qlty_config.overrides {
                triages.push(Triage {
                    set: Set {
                        level: issue_override.level.as_ref().map(|l| level_from_str(l)),
                        category: issue_override
                            .category
                            .as_ref()
                            .map(|c| category_from_str(c)),
                        mode: issue_override.mode,
                        ..Default::default()
                    },
                    r#match: Match {
                        plugins: issue_override.plugins.clone(),
                        rules: issue_override.rules.clone(),
                        file_patterns: issue_override.file_patterns.clone(),
                        ..Default::default()
                    },
                });
            }
        }

        triages
    }

    fn apply_basic_issue_transformations(
        &mut self,
        workspace_root: &Path,
        qlty_config: &QltyConfig,
    );
}
