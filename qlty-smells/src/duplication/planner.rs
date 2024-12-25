use super::transformers::InclusionPathMatcher;
use super::{LanguagePlan, Plan, Settings};
use anyhow::Result;
use qlty_analysis::code::File;
use qlty_analysis::utils::fs::path_to_string;
use qlty_config::config::issue_transformer::IssueTransformer;
use qlty_config::config::smells::{Duplication, IdenticalCode, SimilarCode};
use qlty_config::config::Ignore;
use qlty_config::{
    config::{IssueMode, Language},
    QltyConfig,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct Planner {
    config: QltyConfig,
    settings: Settings,
    files: Vec<Arc<File>>,
}

impl Planner {
    pub fn new(config: &QltyConfig, settings: &Settings, files: Vec<Arc<File>>) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            settings: settings.clone(),
            files,
        })
    }

    fn extract_filter_patterns(&self, language: &Language) -> Vec<String> {
        // Check for filter patterns in the language configuration
        if let Some(smells) = &language.smells {
            if let Some(duplication) = &smells.duplication {
                if !duplication.filter_patterns.is_empty() {
                    return duplication.filter_patterns.clone();
                }
            }
        }

        // Check for filter patterns in the QltyConfig
        if let Some(smells) = &self.config.smells {
            if let Some(duplication) = &smells.duplication {
                if !duplication.filter_patterns.is_empty() {
                    return duplication.filter_patterns.clone();
                }
            }
        }

        // Return default filter patterns if none found
        Duplication::default().filter_patterns
    }

    fn extract_nodes_threshold(&self, language: &Language) -> usize {
        if let Some(smells) = &language.smells {
            if let Some(duplication) = &smells.duplication {
                if let Some(nodes_threshold) = duplication.nodes_threshold {
                    return nodes_threshold;
                }
            }
        }

        if let Some(smells) = &self.config.smells {
            if let Some(duplication) = &smells.duplication {
                if let Some(nodes_threshold) = duplication.nodes_threshold {
                    return nodes_threshold;
                }
            }
        }

        Duplication::default().nodes_threshold.unwrap()
    }

    fn extract_identical_lines_threshold(&self, language: &Language) -> Option<usize> {
        if let Some(smells) = &language.smells {
            if let Some(identical_code) = &smells.identical_code {
                if identical_code.enabled {
                    if let Some(lines_threshold) = identical_code.threshold {
                        return Some(lines_threshold);
                    }
                } else {
                    return None;
                }
            }
        }

        if let Some(smells) = &self.config.smells {
            if let Some(identical_code) = &smells.identical_code {
                if identical_code.enabled {
                    if let Some(lines_threshold) = identical_code.threshold {
                        return Some(lines_threshold);
                    }
                } else {
                    return None;
                }
            }
        }

        Some(IdenticalCode::default().threshold.unwrap())
    }

    fn extract_similar_lines_threshold(&self, language: &Language) -> Option<usize> {
        if let Some(smells) = &language.smells {
            if let Some(similar_code) = &smells.similar_code {
                if similar_code.enabled {
                    if let Some(lines_threshold) = similar_code.threshold {
                        return Some(lines_threshold);
                    }
                } else {
                    return None;
                }
            }
        }

        if let Some(smells) = &self.config.smells {
            if let Some(similar_code) = &smells.similar_code {
                if similar_code.enabled {
                    if let Some(lines_threshold) = similar_code.threshold {
                        return Some(lines_threshold);
                    }
                } else {
                    return None;
                }
            }
        }

        Some(SimilarCode::default().threshold.unwrap())
    }

    pub fn compute(&self) -> Result<Plan> {
        let mut languages = HashMap::new();

        for (name, language_settings) in &self.config.language {
            let mut filter_patterns = self.extract_filter_patterns(language_settings);

            if !self.settings.include_tests {
                filter_patterns.extend(language_settings.test_syntax_patterns.clone());
            }

            let language_plan = LanguagePlan {
                filters: filter_patterns,
                nodes_threshold: self.extract_nodes_threshold(language_settings),
                similar_lines_threshold: self.extract_similar_lines_threshold(language_settings),
                identical_lines_threshold: self
                    .extract_identical_lines_threshold(language_settings),

                issue_mode: IssueMode::extract_issue_mode_from_smells(
                    language_settings,
                    &self.config,
                ),
            };

            // Skip language if issue mode is disabled or both thresholds are None
            if language_plan.issue_mode == IssueMode::Disabled
                || (language_plan.similar_lines_threshold.is_none()
                    && language_plan.identical_lines_threshold.is_none())
            {
                continue;
            }

            languages.insert(name.to_string(), language_plan);
        }

        let mut source_files = self.files.clone();

        if !self.settings.include_tests && !self.config.test_patterns.is_empty() {
            let ignore = Ignore {
                file_patterns: self.config.test_patterns.clone(),
                ..Default::default()
            };

            ignore.initialize_globset();

            source_files.retain(|file| !ignore.matches_path(&path_to_string(file.path.clone())));
        }

        Ok(Plan {
            languages,
            transformers: self.compute_transformers()?,
            source_files,
        })
    }

    fn compute_transformers(&self) -> Result<Vec<Box<dyn IssueTransformer>>> {
        let mut transformers: Vec<Box<dyn IssueTransformer>> = Vec::new();

        if !self.settings.paths.is_empty() {
            transformers.push(Box::new(InclusionPathMatcher::new(
                self.settings.paths.clone(),
            )?));
        }

        Ok(transformers)
    }
}

#[cfg(test)]
mod test {
    use qlty_config::Workspace;

    #[test]
    fn test_javascript_language_contains_filter_patterns_by_default() {
        let workspace = Workspace::new().unwrap();
        workspace.fetch_sources().unwrap();
        let config = workspace.config().unwrap();
        assert!(!&config.language["javascript"]
            .smells
            .as_ref()
            .unwrap()
            .duplication
            .as_ref()
            .unwrap()
            .filter_patterns
            .is_empty());
    }
}
