use super::{LanguagePlan, Plan};
use anyhow::Result;
use qlty_analysis::code::File;
use qlty_config::{
    config::{
        smells::{
            BooleanLogic, FileComplexity, FunctionComplexity, FunctionParameters,
            NestedControlFlow, ReturnStatements,
        },
        IssueMode, Language,
    },
    QltyConfig,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct Planner {
    config: QltyConfig,
    files: Vec<Arc<File>>,
}

impl Planner {
    pub fn new(config: &QltyConfig, files: Vec<Arc<File>>) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            files,
        })
    }

    // Simple readable extract functions
    // There is some code duplication here, but removing it makes code less readable
    fn extract_boolean_logic(&self, language: &Language) -> Option<usize> {
        if let Some(smells) = &language.smells {
            if let Some(boolean_logic) = &smells.boolean_logic {
                if boolean_logic.enabled {
                    if boolean_logic.threshold.is_some() {
                        return boolean_logic.threshold;
                    }
                } else {
                    return None;
                }
            }
        }

        if let Some(smells) = &self.config.smells {
            if let Some(boolean_logic) = &smells.boolean_logic {
                if boolean_logic.enabled {
                    if boolean_logic.threshold.is_some() {
                        return boolean_logic.threshold;
                    }
                } else {
                    return None;
                }
            }
        }

        BooleanLogic::default().threshold
    }

    fn extract_file_complexity(&self, language: &Language) -> Option<usize> {
        if let Some(smells) = &language.smells {
            if let Some(file_complexity) = &smells.file_complexity {
                if file_complexity.enabled {
                    if file_complexity.threshold.is_some() {
                        return file_complexity.threshold;
                    }
                } else {
                    return None;
                }
            }
        }

        if let Some(smells) = &self.config.smells {
            if let Some(file_complexity) = &smells.file_complexity {
                if file_complexity.enabled {
                    if file_complexity.threshold.is_some() {
                        return file_complexity.threshold;
                    }
                } else {
                    return None;
                }
            }
        }

        FileComplexity::default().threshold
    }

    fn extract_function_complexity(&self, language: &Language) -> Option<usize> {
        if let Some(smells) = &language.smells {
            if let Some(function_complexity) = &smells.function_complexity {
                if function_complexity.enabled {
                    if function_complexity.threshold.is_some() {
                        return function_complexity.threshold;
                    }
                } else {
                    return None;
                }
            }
        }

        if let Some(smells) = &self.config.smells {
            if let Some(function_complexity) = &smells.function_complexity {
                if function_complexity.enabled {
                    if function_complexity.threshold.is_some() {
                        return function_complexity.threshold;
                    }
                } else {
                    return None;
                }
            }
        }

        FunctionComplexity::default().threshold
    }

    fn extract_nested_control_flow(&self, language: &Language) -> Option<usize> {
        if let Some(smells) = &language.smells {
            if let Some(nested_control_flow) = &smells.nested_control_flow {
                if nested_control_flow.enabled {
                    if nested_control_flow.threshold.is_some() {
                        return nested_control_flow.threshold;
                    }
                } else {
                    return None;
                }
            }
        }

        if let Some(smells) = &self.config.smells {
            if let Some(nested_control_flow) = &smells.nested_control_flow {
                if nested_control_flow.enabled {
                    if nested_control_flow.threshold.is_some() {
                        return nested_control_flow.threshold;
                    }
                } else {
                    return None;
                }
            }
        }

        NestedControlFlow::default().threshold
    }

    fn extract_function_parameters(&self, language: &Language) -> Option<usize> {
        if let Some(smells) = &language.smells {
            if let Some(function_parameters) = &smells.function_parameters {
                if function_parameters.enabled {
                    if function_parameters.threshold.is_some() {
                        return function_parameters.threshold;
                    }
                } else {
                    return None;
                }
            }
        }

        if let Some(smells) = &self.config.smells {
            if let Some(function_parameters) = &smells.function_parameters {
                if function_parameters.enabled {
                    if function_parameters.threshold.is_some() {
                        return function_parameters.threshold;
                    }
                } else {
                    return None;
                }
            }
        }

        FunctionParameters::default().threshold
    }

    fn extract_return_statements(&self, language: &Language) -> Option<usize> {
        if let Some(smells) = &language.smells {
            if let Some(return_statements) = &smells.return_statements {
                if return_statements.enabled {
                    if return_statements.threshold.is_some() {
                        return return_statements.threshold;
                    }
                } else {
                    return None;
                }
            }
        }

        if let Some(smells) = &self.config.smells {
            if let Some(return_statements) = &smells.return_statements {
                if return_statements.enabled {
                    if return_statements.threshold.is_some() {
                        return return_statements.threshold;
                    }
                } else {
                    return None;
                }
            }
        }

        ReturnStatements::default().threshold
    }

    pub fn compute(&self) -> Result<Plan> {
        let mut languages = HashMap::new();

        for (name, language_settings) in &self.config.language {
            let language_plan = LanguagePlan {
                boolean_logic: self.extract_boolean_logic(language_settings),
                file_complexity: self.extract_file_complexity(language_settings),
                function_complexity: self.extract_function_complexity(language_settings),
                nested_control: self.extract_nested_control_flow(language_settings),
                parameters: self.extract_function_parameters(language_settings),
                returns: self.extract_return_statements(language_settings),
                issue_mode: IssueMode::extract_issue_mode_from_smells(
                    language_settings,
                    &self.config,
                ),
            };

            if language_plan.issue_mode == IssueMode::Disabled {
                continue;
            }

            languages.insert(name.to_string(), language_plan);
        }

        Ok(Plan {
            languages,
            source_files: self.files.clone(),
        })
    }
}
