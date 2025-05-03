use crate::publish::{metrics::CoverageMetrics, Plan, Report, Results};
use anyhow::Result;
use qlty_types::tests::v1::FileCoverage;
use std::collections::HashSet;
use std::path::PathBuf;

pub struct Processor {
    plan: Plan,
    results: Results,
}

impl Processor {
    pub fn new(plan: &Plan, results: Results) -> Self {
        Self {
            plan: plan.clone(),
            results,
        }
    }

    pub fn compute(&self) -> Result<Report> {
        let mut report_files = self.results.report_files.clone();

        report_files.iter_mut().for_each(|f| {
            f.build_id = self.plan.metadata.build_id.clone();
            f.tag = self.plan.metadata.tag.clone();
        });

        let mut transformed_file_coverages = self
            .results
            .file_coverages
            .iter()
            .filter_map(|file_coverage| self.transform(file_coverage.to_owned()))
            .collect::<Vec<_>>();

        let mut found_files = HashSet::new();
        let mut missing_files = HashSet::new();

        if self.plan.skip_missing_files {
            transformed_file_coverages.retain(|file_coverage| {
                match PathBuf::from(&file_coverage.path).try_exists() {
                    Ok(true) => {
                        found_files.insert(file_coverage.path.clone());
                        true
                    }
                    _ => {
                        missing_files.insert(file_coverage.path.clone());
                        false
                    }
                }
            });
        } else {
            for file_coverage in &transformed_file_coverages {
                match PathBuf::from(&file_coverage.path).try_exists() {
                    Ok(true) => {
                        found_files.insert(file_coverage.path.clone());
                    }
                    _ => {
                        missing_files.insert(file_coverage.path.clone());
                    }
                }
            }
        }

        let totals = CoverageMetrics::calculate(&transformed_file_coverages);

        Ok(Report {
            metadata: self.plan.metadata.clone(),
            report_files,
            file_coverages: transformed_file_coverages,
            totals,
            missing_files,
            found_files,
        })
    }

    fn transform(&self, file_coverage: FileCoverage) -> Option<FileCoverage> {
        let mut file_coverage: Option<FileCoverage> = Some(file_coverage.clone());

        for transformer in self.plan.transformers.iter() {
            if file_coverage.is_some() {
                file_coverage = transformer.transform(file_coverage.unwrap());
            } else {
                return None;
            }
        }

        file_coverage
    }
}
