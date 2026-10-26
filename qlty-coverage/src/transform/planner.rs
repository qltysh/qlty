use crate::transform::{Plan, Settings};
use crate::transformer::AddPrefix;
use crate::transformer::DefaultPathFixer;
use crate::transformer::StripDotSlashPrefix;
use crate::transformer::StripPrefix;
use crate::utils::extract_path_and_format;
use crate::Transformer;
use anyhow::Result;
use qlty_analysis::utils::fs::path_to_string;
use qlty_types::tests::v1::ReportFile;

#[derive(Debug, Clone)]
pub struct Planner {
    settings: Settings,
}

impl Planner {
    pub fn new(settings: &Settings) -> Self {
        Self {
            settings: settings.clone(),
        }
    }

    pub fn compute(&self) -> Result<Plan> {
        Ok(Plan {
            report_file: self.compute_report_file()?,
            transformers: self.compute_transformers()?,
        })
    }

    fn compute_report_file(&self) -> Result<ReportFile> {
        let (path, format) =
            extract_path_and_format(&self.settings.path, self.settings.report_format)?;

        Ok(ReportFile {
            path: path_to_string(path),
            format: format.to_string(),
            ..Default::default()
        })
    }

    fn compute_transformers(&self) -> Result<Vec<Box<dyn Transformer>>> {
        let mut transformers: Vec<Box<dyn Transformer>> = vec![];

        // Check if user provided any manual path fixing options
        let has_manual_path_fixing =
            self.settings.strip_prefix.is_some() || self.settings.add_prefix.is_some();

        if let Some(prefix) = self.settings.strip_prefix.clone() {
            transformers.push(Box::new(StripPrefix::new(prefix)));
        } else {
            transformers.push(Box::new(StripPrefix::new_from_git_root()?));
        }

        // Apply default path fixing only if no manual path fixing options are provided
        if !has_manual_path_fixing {
            transformers.push(Box::new(DefaultPathFixer::new()?));
        }

        transformers.push(Box::new(StripDotSlashPrefix));

        if let Some(prefix) = self.settings.add_prefix.clone() {
            transformers.push(Box::new(AddPrefix::new(&prefix)));
        }

        Ok(transformers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_path_fixer_added_when_no_manual_options() {
        let settings = Settings {
            path: "coverage.json".to_string(),
            report_format: None,
            strip_prefix: None,
            add_prefix: None,
        };

        let planner = Planner::new(&settings);
        let transformers = planner.compute_transformers().unwrap();

        // Should contain DefaultPathFixer when no manual options are set
        let has_default_fixer = transformers
            .iter()
            .any(|t| format!("{:?}", t).contains("DefaultPathFixer"));
        assert!(
            has_default_fixer,
            "DefaultPathFixer should be added when no manual path fixing options are provided"
        );
    }

    #[test]
    fn test_default_path_fixer_not_added_with_strip_prefix() {
        let settings = Settings {
            path: "coverage.json".to_string(),
            report_format: None,
            strip_prefix: Some("/home/user/project".to_string()),
            add_prefix: None,
        };

        let planner = Planner::new(&settings);
        let transformers = planner.compute_transformers().unwrap();

        // Should NOT contain DefaultPathFixer when strip_prefix is set
        let has_default_fixer = transformers
            .iter()
            .any(|t| format!("{:?}", t).contains("DefaultPathFixer"));
        assert!(
            !has_default_fixer,
            "DefaultPathFixer should not be added when strip_prefix is provided"
        );
    }

    #[test]
    fn test_default_path_fixer_not_added_with_add_prefix() {
        let settings = Settings {
            path: "coverage.json".to_string(),
            report_format: None,
            strip_prefix: None,
            add_prefix: Some("src/".to_string()),
        };

        let planner = Planner::new(&settings);
        let transformers = planner.compute_transformers().unwrap();

        // Should NOT contain DefaultPathFixer when add_prefix is set
        let has_default_fixer = transformers
            .iter()
            .any(|t| format!("{:?}", t).contains("DefaultPathFixer"));
        assert!(
            !has_default_fixer,
            "DefaultPathFixer should not be added when add_prefix is provided"
        );
    }

    #[test]
    fn test_default_path_fixer_not_added_with_both_options() {
        let settings = Settings {
            path: "coverage.json".to_string(),
            report_format: None,
            strip_prefix: Some("/home/user/project".to_string()),
            add_prefix: Some("src/".to_string()),
        };

        let planner = Planner::new(&settings);
        let transformers = planner.compute_transformers().unwrap();

        // Should NOT contain DefaultPathFixer when both options are set
        let has_default_fixer = transformers
            .iter()
            .any(|t| format!("{:?}", t).contains("DefaultPathFixer"));
        assert!(!has_default_fixer, "DefaultPathFixer should not be added when both strip_prefix and add_prefix are provided");
    }
}
