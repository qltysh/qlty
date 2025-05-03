use qlty_types::tests::v1::FileCoverage;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize)]
pub struct CoverageMetrics {
    pub covered_lines: u64,
    pub uncovered_lines: u64,
    pub omitted_lines: u64,
    pub total_lines: u64,
    pub coverage_percentage: f64,
}

impl CoverageMetrics {
    pub fn calculate(file_coverages: &[FileCoverage]) -> Self {
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
        let mut omitted_lines = 0;

        for hits in combined_hits.values() {
            for &hit in hits {
                if hit > 0 {
                    covered_lines += 1;
                } else if hit == 0 {
                    uncovered_lines += 1;
                } else {
                    omitted_lines += 1;
                }
            }
        }

        let total_lines = covered_lines + uncovered_lines + omitted_lines;
        let coverable_lines = covered_lines + uncovered_lines;

        let coverage_percentage = if coverable_lines > 0 {
            (covered_lines as f64 / coverable_lines as f64) * 100.0
        } else {
            0.0
        };

        Self {
            covered_lines,
            uncovered_lines,
            omitted_lines,
            total_lines,
            coverage_percentage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_coverage() {
        let metrics = CoverageMetrics::calculate(&[]);

        assert_eq!(metrics.covered_lines, 0);
        assert_eq!(metrics.uncovered_lines, 0);
        assert_eq!(metrics.omitted_lines, 0);
        assert_eq!(metrics.total_lines, 0);
        assert_eq!(metrics.coverage_percentage, 0.0);
    }

    #[test]
    fn test_single_file_coverage() {
        let file_coverage = FileCoverage {
            path: "src/main.rs".to_string(),
            hits: vec![1, 0, -1, 2],
            ..Default::default()
        };

        let metrics = CoverageMetrics::calculate(&[file_coverage]);

        assert_eq!(metrics.covered_lines, 2);
        assert_eq!(metrics.uncovered_lines, 1);
        assert_eq!(metrics.omitted_lines, 1);
        assert_eq!(metrics.total_lines, 4);
        assert_eq!(metrics.coverage_percentage, 2.0 / 3.0 * 100.0);
    }

    #[test]
    fn test_multiple_coverages_different_files() {
        let file_coverage1 = FileCoverage {
            path: "src/main.rs".to_string(),
            hits: vec![1, 0, 3],
            ..Default::default()
        };

        let file_coverage2 = FileCoverage {
            path: "src/lib.rs".to_string(),
            hits: vec![0, 1, 0, -1],
            ..Default::default()
        };

        let metrics = CoverageMetrics::calculate(&[file_coverage1, file_coverage2]);

        assert_eq!(metrics.covered_lines, 3);
        assert_eq!(metrics.uncovered_lines, 3);
        assert_eq!(metrics.omitted_lines, 1);
        assert_eq!(metrics.total_lines, 7);
        assert_eq!(metrics.coverage_percentage, 3.0 / 6.0 * 100.0);
    }

    #[test]
    fn test_combining_coverages_same_file() {
        let file_coverage1 = FileCoverage {
            path: "src/main.rs".to_string(),
            hits: vec![1, 0, 0],
            ..Default::default()
        };

        let file_coverage2 = FileCoverage {
            path: "src/main.rs".to_string(),
            hits: vec![0, 1, 2],
            ..Default::default()
        };

        let metrics = CoverageMetrics::calculate(&[file_coverage1, file_coverage2]);

        assert_eq!(metrics.covered_lines, 3);
        assert_eq!(metrics.uncovered_lines, 0);
        assert_eq!(metrics.omitted_lines, 0);
        assert_eq!(metrics.total_lines, 3);
        assert_eq!(metrics.coverage_percentage, 100.0);
    }

    #[test]
    fn test_combining_coverages_different_lengths() {
        let file_coverage1 = FileCoverage {
            path: "src/main.rs".to_string(),
            hits: vec![1, 0, 0],
            ..Default::default()
        };

        let file_coverage2 = FileCoverage {
            path: "src/main.rs".to_string(),
            hits: vec![0, 1, 2, 3, 0],
            ..Default::default()
        };

        let metrics = CoverageMetrics::calculate(&[file_coverage1, file_coverage2]);

        assert_eq!(metrics.covered_lines, 4);
        assert_eq!(metrics.uncovered_lines, 1);
        assert_eq!(metrics.omitted_lines, 0);
        assert_eq!(metrics.total_lines, 5);
        assert_eq!(metrics.coverage_percentage, 4.0 / 5.0 * 100.0);
    }

    #[test]
    fn test_only_uncoverable_lines() {
        let file_coverage = FileCoverage {
            path: "src/main.rs".to_string(),
            hits: vec![-1, -1, -1],
            ..Default::default()
        };

        let metrics = CoverageMetrics::calculate(&[file_coverage]);

        assert_eq!(metrics.covered_lines, 0);
        assert_eq!(metrics.uncovered_lines, 0);
        assert_eq!(metrics.omitted_lines, 3);
        assert_eq!(metrics.total_lines, 3);
        assert_eq!(metrics.coverage_percentage, 0.0);
    }
}
