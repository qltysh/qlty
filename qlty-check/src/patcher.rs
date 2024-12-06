use crate::{
    executor::staging_area::StagingArea, results::FixedResult, source_reader::SourceReader as _,
};
use anyhow::Result;
use itertools::Itertools;
use qlty_types::analysis::v1::Issue;
use std::{borrow::BorrowMut, collections::HashSet, ops::RangeInclusive, path::PathBuf};
use tracing::{debug, error, trace, warn};

const UNSAFE_RULES: [&str; 3] = [
    "eslint:@typescript-eslint/no-explicit-any",
    "eslint:@typescript-eslint/no-empty-interface",
    "eslint:@typescript-eslint/no-non-null-assertion",
];

#[derive(Debug, Clone)]
pub struct Patcher {
    staging_area: StagingArea,
}

impl Patcher {
    pub fn filter_issues(issues: &[Issue], allow_unsafe: bool) -> Vec<Issue> {
        issues
            .iter()
            .filter(|issue| Patcher::is_patchable(issue, allow_unsafe))
            .cloned()
            .collect()
    }

    pub fn is_patchable(issue: &Issue, allow_unsafe: bool) -> bool {
        !issue.suggestions.is_empty()
            && !issue.suggestions[0].patch.is_empty()
            && issue.location.is_some()
            && (allow_unsafe || Patcher::issue_is_safe(issue))
    }

    fn issue_is_safe(issue: &Issue) -> bool {
        let full_rule_key = format!("{}:{}", issue.tool, issue.rule_key);
        !UNSAFE_RULES.contains(&full_rule_key.as_str())
    }

    pub fn new(staging_area: &StagingArea) -> Self {
        Self {
            staging_area: staging_area.clone(),
        }
    }

    pub fn try_apply(&self, issues: &[Issue], allow_unsafe: bool) -> HashSet<FixedResult> {
        let mut fixed: HashSet<FixedResult> = HashSet::new();

        let issues_by_path = issues
            .iter()
            .into_group_map_by(|issue| issue.path().map(PathBuf::from));

        for (path, issues) in issues_by_path {
            if path.is_none() {
                continue;
            }

            let path = path.unwrap();
            let path_buf = PathBuf::from(&path);

            if let Err(err) =
                self.apply_to_file(path_buf, &issues, allow_unsafe, fixed.borrow_mut())
            {
                warn!("Failed to apply patch to {:?}: {}", path, err);
            }
        }

        fixed
    }

    fn apply_to_file(
        &self,
        path: PathBuf,
        issues: &[&Issue],
        allow_unsafe: bool,
        fixed: &mut HashSet<FixedResult>,
    ) -> Result<()> {
        let original_source = self.staging_area.read(path.clone())?;
        let mut modified_source = original_source.clone();

        for issue in issues.iter() {
            if !Patcher::is_patchable(issue, allow_unsafe) {
                // guard to avoid issues with no suggestions. this does not guard against unsafe rules
                continue;
            }

            let patch_string = issue.suggestions[0].patch.clone();
            let full_rule_key = format!("{}:{}", issue.tool, issue.rule_key);
            let display_location = format!(
                "{}:{}",
                path.display(),
                issue
                    .line_range()
                    .unwrap_or(RangeInclusive::new(1, 1))
                    .start()
            );
            trace!(
                "Applying patch for {} ({}):\n{}",
                display_location,
                full_rule_key,
                &patch_string
            );

            if let Ok(patch) = diffy::Patch::from_str(&patch_string) {
                match diffy::apply(&modified_source, &patch) {
                    Ok(patched_source) => {
                        if patched_source != modified_source {
                            debug!(
                                "Successfully applied patch for {} ({}):\n{}",
                                display_location, full_rule_key, &patch,
                            );

                            fixed.insert(FixedResult {
                                rule_key: issue.rule_key.clone(),
                                location: issue.location().unwrap().clone(),
                            });

                            modified_source = patched_source;
                        } else {
                            warn!(
                                "Patch produced no change to contents {}:\n{}",
                                path.display(),
                                &patch
                            );
                        }
                    }
                    Err(error) => {
                        error!(
                            "Failed to apply patch for {} ({}): {}\n{}",
                            display_location,
                            full_rule_key,
                            error.to_string(),
                            &patch
                        );
                    }
                }
            } else {
                error!(
                    "Failed to parse patch for {} ({}):\n{}",
                    display_location, full_rule_key, patch_string
                );
            }
        }

        self.staging_area
            .write_to_source(path.as_path(), modified_source)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::executor::staging_area::{Mode, StagingArea};
    use qlty_types::analysis::v1::{Location, Suggestion};
    use tempfile::tempdir;

    #[test]
    fn test_try_apply() {
        let temp_dir = tempdir().unwrap();
        let temp_source = temp_dir.path().to_path_buf();
        let staging_area = StagingArea::generate(Mode::Source, temp_source.clone(), None);
        let file = &temp_source.join("main.rs");

        std::fs::write(file, "A\nB\nC\n\nD\nE\nF").ok();

        let issues = [
            Issue {
                rule_key: "fixable".to_string(),
                location: Some(Location {
                    path: file.to_str().unwrap().to_string(),
                    ..Default::default()
                }),
                suggestions: vec![
                    Suggestion {
                        patch: diffy::create_patch("A\nB\nC\n\nD\nE\nF", "X\nB\nC\n\nD\nE\nF")
                            .to_string(),
                        ..Default::default()
                    },
                    Suggestion {
                        patch: diffy::create_patch("A\nB\nC\n\nD\nE\nF", "NOT_APPLIED").to_string(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Issue {
                rule_key: "other_fixable".to_string(),
                location: Some(Location {
                    path: file.to_str().unwrap().to_string(),
                    ..Default::default()
                }),
                suggestions: vec![Suggestion {
                    patch: diffy::create_patch("A\nB\nC\n\nD\nE\nF", "A\nB\nC\n\nD\nE\nX")
                        .to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Issue {
                tool: "eslint".to_string(),
                rule_key: "@typescript-eslint/no-empty-interface".to_string(),
                location: Some(Location {
                    path: file.to_str().unwrap().to_string(),
                    ..Default::default()
                }),
                suggestions: vec![Suggestion {
                    patch: diffy::create_patch("A\nB\nC\n\nD\nE\nF", "A\nB\nX\n\nD\nE\nF")
                        .to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Issue {
                rule_key: "unfixable".to_string(),
                location: Some(Location {
                    path: file.to_str().unwrap().to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            Issue {
                rule_key: "unfixable".to_string(),
                location: Some(Location {
                    path: "another_file.rs".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ];

        let patcher = Patcher::new(&staging_area);
        let fixed = patcher.try_apply(&issues, false);

        assert_eq!(
            staging_area.read(file.clone()).unwrap(),
            "X\nB\nC\n\nD\nE\nX"
        );
        assert_eq!(fixed.len(), 2);
        assert!(fixed.contains(&FixedResult {
            rule_key: "fixable".to_string(),
            location: issues.first().unwrap().location().unwrap().clone()
        }));
    }

    #[test]
    fn test_try_apply_unsafe() {
        let temp_dir = tempdir().unwrap();
        let temp_source = temp_dir.path().to_path_buf();
        let staging_area = StagingArea::generate(Mode::Source, temp_source.clone(), None);
        let file = &temp_source.join("main.rs");

        std::fs::write(file, "A\nB\nC\n\nD\nE\nF").ok();

        let issues = [Issue {
            tool: "eslint".to_string(),
            rule_key: "@typescript-eslint/no-empty-interface".to_string(),
            location: Some(Location {
                path: file.to_str().unwrap().to_string(),
                ..Default::default()
            }),
            suggestions: vec![Suggestion {
                patch: diffy::create_patch("A\nB\nC\n\nD\nE\nF", "A\nB\nX\n\nD\nE\nF").to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }];

        let patcher = Patcher::new(&staging_area);
        let fixed = patcher.try_apply(&issues, false);
        assert_eq!(
            staging_area.read(file.clone()).unwrap(),
            "A\nB\nC\n\nD\nE\nF"
        );
        assert_eq!(fixed.len(), 0);

        let fixed = patcher.try_apply(&issues, true);
        assert_eq!(
            staging_area.read(file.clone()).unwrap(),
            "A\nB\nX\n\nD\nE\nF"
        );
        assert_eq!(fixed.len(), 1);
    }

    #[test]
    fn test_is_patchable() {
        struct TestData {
            issue: Issue,
            allow_unsafe: bool,
            expected: bool,
        }

        let tests = vec![
            TestData {
                issue: Issue {
                    tool: "tool".to_string(),
                    rule_key: "no_suggestions".to_string(),
                    location: Some(Location::default()),
                    suggestions: vec![],
                    ..Default::default()
                },
                allow_unsafe: false,
                expected: false,
            },
            TestData {
                issue: Issue {
                    tool: "tool".to_string(),
                    rule_key: "no_location".to_string(),
                    location: None,
                    suggestions: vec![Suggestion {
                        patch: diffy::create_patch("A\nB\nC\n\nD\nE\nF", "X\nB\nC\n\nD\nE\nF")
                            .to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                allow_unsafe: false,
                expected: false,
            },
            TestData {
                issue: Issue {
                    tool: "tool".to_string(),
                    rule_key: "fixable".to_string(),
                    location: Some(Location::default()),
                    suggestions: vec![Suggestion {
                        patch: "PATCH".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                allow_unsafe: true,
                expected: true,
            },
            TestData {
                issue: Issue {
                    tool: "eslint".to_string(),
                    rule_key: "@typescript-eslint/no-empty-interface".to_string(),
                    location: Some(Location::default()),
                    suggestions: vec![Suggestion {
                        patch: "PATCH".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                allow_unsafe: false,
                expected: false,
            },
            TestData {
                issue: Issue {
                    tool: "eslint".to_string(),
                    rule_key: "@typescript-eslint/no-empty-interface".to_string(),
                    location: Some(Location::default()),
                    suggestions: vec![Suggestion {
                        patch: "PATCH".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                allow_unsafe: true,
                expected: true,
            },
        ];
        for test in tests.iter() {
            assert_eq!(
                Patcher::is_patchable(&test.issue, test.allow_unsafe),
                test.expected,
                "rule_key: {}",
                test.issue.rule_key
            );
        }
    }

    #[test]
    fn test_filter_issues() {
        let issues = vec![
            Issue {
                rule_key: "fixable".to_string(),
                location: Some(Location::default()),
                suggestions: vec![Suggestion {
                    patch: "PATCH".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Issue {
                rule_key: "other_fixable".to_string(),
                location: Some(Location::default()),
                suggestions: vec![Suggestion {
                    patch: "PATCH".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Issue {
                tool: "eslint".to_string(),
                rule_key: "@typescript-eslint/no-empty-interface".to_string(),
                location: Some(Location::default()),
                suggestions: vec![Suggestion {
                    patch: "PATCH".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
        ];

        let filtered = Patcher::filter_issues(&issues, false);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].rule_key, "fixable");
        assert_eq!(filtered[1].rule_key, "other_fixable");

        let filtered = Patcher::filter_issues(&issues, true);
        assert_eq!(filtered.len(), 3);
    }
}
