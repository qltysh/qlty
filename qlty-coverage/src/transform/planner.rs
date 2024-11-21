use crate::transform::{Plan, Settings};
use crate::transformer::AddPrefix;
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

        if let Some(prefix) = self.settings.strip_prefix.clone() {
            transformers.push(Box::new(StripPrefix::new(prefix)));
        } else {
            transformers.push(Box::<StripPrefix>::default());
        }

        transformers.push(Box::new(StripDotSlashPrefix));

        if let Some(prefix) = self.settings.add_prefix.clone() {
            transformers.push(Box::new(AddPrefix::new(&prefix)));
        }

        Ok(transformers)
    }
}
