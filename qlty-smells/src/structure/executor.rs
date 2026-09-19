use super::{checks, Plan};
use qlty_analysis::code::File;
use qlty_analysis::Report;
use qlty_types::analysis::v1::Issue;
use rayon::prelude::*;
use std::sync::Arc;

#[derive(Debug)]
pub struct Executor {
    plan: Plan,
    pub issues: Vec<Issue>,
}

impl Executor {
    pub fn new(plan: &Plan) -> Self {
        Self {
            plan: plan.clone(),
            issues: vec![],
        }
    }

    pub fn execute(&mut self) {
        self.issues = self
            .plan
            .source_files
            .clone()
            .into_par_iter()
            .flat_map(|source_file| self.check(source_file))
            .collect();
    }

    pub fn report(&self) -> Report {
        Report {
            issues: self.issues.clone(),
            ..Default::default()
        }
    }

    pub fn check(&self, source_file: Arc<File>) -> Vec<Issue> {
        let mut issues = vec![];
        let tree = source_file.parse();

        let language = self.plan.get_language(&source_file.language_name);

        if let Some(threshold) = language.parameters {
            issues.extend(checks::parameters::check(
                threshold,
                source_file.clone(),
                &tree,
            ));
        }

        if let Some(threshold) = language.returns {
            issues.extend(checks::returns::check(
                threshold,
                source_file.clone(),
                &tree,
            ));
        }

        if let Some(threshold) = language.nested_control {
            issues.extend(checks::nested_control::check(
                threshold,
                source_file.clone(),
                &tree,
            ));
        }

        if let Some(threshold) = language.boolean_logic {
            issues.extend(checks::boolean_logic::check(
                threshold,
                source_file.clone(),
                &tree,
            ));
        }

        if let Some(threshold) = language.file_complexity {
            issues.extend(checks::file_complexity::check(
                threshold,
                source_file.clone(),
                &tree,
            ));
        }

        if let Some(threshold) = language.function_complexity {
            issues.extend(checks::function_complexity::check(
                threshold,
                source_file.clone(),
                &tree,
            ));
        }

        for issue in &mut issues {
            issue.mode = language.issue_mode as i32;
        }

        issues
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::structure::Planner;
    use qlty_config::config::smells::{BooleanLogic, Smells};
    use qlty_config::config::Language;
    use qlty_config::QltyConfig;
    use std::collections::HashMap;

    #[test]
    fn measurements_use_language_threshold_override() {
        let source_file = Arc::new(File::from_string(
            "rust",
            "fn f() { a || b || c || d || e; }",
        ));
        let config = QltyConfig {
            smells: Some(Smells {
                boolean_logic: Some(BooleanLogic {
                    enabled: true,
                    threshold: Some(10),
                }),
                ..Default::default()
            }),
            language: HashMap::from([(
                "rust".to_string(),
                Language {
                    smells: Some(Smells {
                        boolean_logic: Some(BooleanLogic {
                            enabled: true,
                            threshold: Some(3),
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        let plan = Planner::new(&config, vec![source_file])
            .unwrap()
            .compute()
            .unwrap();
        let mut executor = Executor::new(&plan);
        executor.execute();

        assert_eq!(executor.issues.len(), 1);
        assert_eq!(executor.issues[0].get_property_number("threshold"), 3.0);
        assert_eq!(executor.issues[0].get_property_number("actual"), 4.0);
    }
}
