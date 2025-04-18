use crate::publish::{Plan, Report, Results};
use anyhow::Result;
use qlty_types::tests::v1::FileCoverage;
use serde::Serialize;
use std::collections::HashMap;

pub struct Processor {
    plan: Plan,
    results: Results,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CoverageMetrics {
    pub covered_lines: i64,
    pub uncovered_lines: i64,
    pub total_lines: i64,
    pub coverage_percentage: f64,
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

        let transformed_file_coverages = self
            .results
            .file_coverages
            .iter()
            .filter_map(|file_coverage| self.transform(file_coverage.to_owned()))
            .collect::<Vec<_>>();

        let coverage_metrics = self.calculate_coverage_metrics(&transformed_file_coverages);

        Ok(Report {
            metadata: self.plan.metadata.clone(),
            report_files,
            file_coverages: transformed_file_coverages,
            coverage_metrics,
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

    fn calculate_coverage_metrics(&self, file_coverages: &[FileCoverage]) -> CoverageMetrics {
        // Group file coverages by path
        let mut path_hits_map: HashMap<String, Vec<Vec<i64>>> = HashMap::new();

        // First collect all the file coverages by path
        for file_coverage in file_coverages {
            path_hits_map
                .entry(file_coverage.path.clone())
                .or_default()
                .push(file_coverage.hits.clone());
        }

        // Then combine the hits arrays for each path by summing at each index
        let mut combined_hits: HashMap<String, Vec<i64>> = HashMap::new();

        for (path, hits_arrays) in path_hits_map {
            // Skip if there are no hits arrays
            if hits_arrays.is_empty() {
                continue;
            }

            // Find the maximum length to handle arrays of different lengths
            let max_len = hits_arrays.iter().map(|arr| arr.len()).max().unwrap_or(0);

            // Create a combined array initialized with zeros
            let mut combined = vec![0; max_len];

            // Sum the hits at each index
            for hits_array in hits_arrays {
                for (i, &hit) in hits_array.iter().enumerate() {
                    combined[i] += hit;
                }
            }

            combined_hits.insert(path, combined);
        }

        let mut covered_lines = 0;
        let mut uncovered_lines = 0;

        for hits in combined_hits.values() {
            for &hit in hits {
                if hit > 0 {
                    covered_lines += 1;
                } else if hit == 0 {
                    uncovered_lines += 1;
                }
                // Negative values are skipped lines
            }
        }

        let total_lines = covered_lines + uncovered_lines;

        let coverage_percentage = if total_lines > 0 {
            (covered_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        CoverageMetrics {
            covered_lines,
            uncovered_lines,
            total_lines,
            coverage_percentage,
        }
    }
}
