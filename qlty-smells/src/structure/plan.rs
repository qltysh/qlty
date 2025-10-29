use qlty_analysis::code::File;
use qlty_config::{config::IssueMode, issue_transformer::IssueTransformer};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Debug)]
pub struct Plan {
    pub languages: HashMap<String, LanguagePlan>,
    pub source_files: Vec<Arc<File>>,
    pub transformers: Vec<Box<dyn IssueTransformer>>,
}

impl Plan {
    pub fn get_language(&self, language: &str) -> LanguagePlan {
        self.languages
            .get(language)
            .unwrap_or(&LanguagePlan::default())
            .clone()
    }
}

#[derive(Clone, Debug, Default)]
pub struct LanguagePlan {
    pub boolean_logic: Option<usize>,
    pub file_complexity: Option<usize>,
    pub function_complexity: Option<usize>,
    pub nested_control: Option<usize>,
    pub parameters: Option<usize>,
    pub returns: Option<usize>,
    pub issue_mode: IssueMode,
}
