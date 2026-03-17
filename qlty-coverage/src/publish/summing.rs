use qlty_types::tests::v1::{CoverageSummary, FileCoverage};
use std::collections::HashMap;

pub struct DeduplicatedCoverages(Vec<FileCoverage>);

impl DeduplicatedCoverages {
    pub fn as_slice(&self) -> &[FileCoverage] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn into_inner(self) -> Vec<FileCoverage> {
        self.0
    }
}

fn merge_hits(existing: &mut Vec<i64>, other: &[i64]) {
    let min_len = existing.len().min(other.len());
    existing.truncate(min_len);

    for i in 0..min_len {
        if existing[i] < 0 || other[i] < 0 {
            existing[i] = existing[i].min(other[i]);
        } else {
            existing[i] += other[i];
        }
    }
}

fn compute_summary(hits: &[i64]) -> CoverageSummary {
    let mut covered: i64 = 0;
    let mut missed: i64 = 0;
    let mut omit: i64 = 0;

    for &hit in hits {
        match hit {
            -1 => omit += 1,
            0 => missed += 1,
            _ => covered += 1,
        }
    }

    CoverageSummary {
        covered,
        missed,
        omit,
        total: covered + missed + omit,
    }
}

pub fn sum_file_coverages(file_coverages: Vec<FileCoverage>) -> DeduplicatedCoverages {
    let mut map: HashMap<String, FileCoverage> = HashMap::new();

    for fc in file_coverages {
        match map.get_mut(&fc.path) {
            Some(existing) => {
                merge_hits(&mut existing.hits, &fc.hits);
            }
            None => {
                map.insert(fc.path.clone(), fc);
            }
        }
    }

    DeduplicatedCoverages(
        map.into_values()
            .map(|mut fc| {
                fc.summary = Some(compute_summary(&fc.hits));
                fc
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sum_hits(hits_arrays: &[Vec<i64>]) -> Vec<i64> {
        if hits_arrays.is_empty() {
            return vec![];
        }

        let mut result = hits_arrays[0].clone();
        for other in &hits_arrays[1..] {
            merge_hits(&mut result, other);
        }
        result
    }

    mod spec_tests {
        use super::*;
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct Spec {
            expectations: Vec<Expectation>,
        }

        #[derive(Deserialize)]
        struct Expectation {
            id: String,
            inputs: Vec<Vec<i64>>,
            expected: Vec<i64>,
        }

        #[test]
        fn test_all_spec_expectations() {
            let spec_json = include_str!("../../tests/fixtures/coverage_summation_spec.json");
            let spec: Spec = serde_json::from_str(spec_json).unwrap();

            assert_eq!(spec.expectations.len(), 26);

            for expectation in &spec.expectations {
                let result = sum_hits(&expectation.inputs);
                assert_eq!(
                    result, expectation.expected,
                    "failed for spec: {}",
                    expectation.id
                );
            }
        }
    }

    mod sum_hits_tests {
        use super::*;

        #[test]
        fn zero_arrays() {
            assert_eq!(sum_hits(&[]), Vec::<i64>::new());
        }
    }

    mod sum_file_coverages_tests {
        use super::*;

        fn make_fc(path: &str, hits: Vec<i64>) -> FileCoverage {
            FileCoverage {
                path: path.to_string(),
                hits,
                ..Default::default()
            }
        }

        #[test]
        fn no_duplicates_passes_through() {
            let input = vec![make_fc("a.rs", vec![1, 0, -1]), make_fc("b.rs", vec![0, 1])];

            let result = sum_file_coverages(input).into_inner();
            assert_eq!(result.len(), 2);

            let a = result.iter().find(|fc| fc.path == "a.rs").unwrap();
            assert_eq!(a.hits, vec![1, 0, -1]);

            let b = result.iter().find(|fc| fc.path == "b.rs").unwrap();
            assert_eq!(b.hits, vec![0, 1]);
        }

        #[test]
        fn duplicates_merged() {
            let input = vec![
                make_fc("a.rs", vec![1, 0, -1]),
                make_fc("a.rs", vec![2, 1, -1]),
            ];

            let result = sum_file_coverages(input).into_inner();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].hits, vec![3, 1, -1]);
        }

        #[test]
        fn summary_recomputed_after_merge() {
            let input = vec![
                make_fc("a.rs", vec![1, 0, -1, 0]),
                make_fc("a.rs", vec![0, 1, -1, 0]),
            ];

            let result = sum_file_coverages(input).into_inner();
            let summary = result[0].summary.unwrap();
            assert_eq!(summary.covered, 2);
            assert_eq!(summary.missed, 1);
            assert_eq!(summary.omit, 1);
            assert_eq!(summary.total, 4);
        }

        #[test]
        fn metadata_preserved_from_first_entry() {
            let mut fc1 = make_fc("a.rs", vec![1]);
            fc1.build_id = "build-1".to_string();
            fc1.commit_sha = Some("abc123".to_string());

            let fc2 = make_fc("a.rs", vec![2]);

            let result = sum_file_coverages(vec![fc1, fc2]).into_inner();
            assert_eq!(result[0].build_id, "build-1");
            assert_eq!(result[0].commit_sha, Some("abc123".to_string()));
        }

        #[test]
        fn mixed_duplicates_and_unique() {
            let input = vec![
                make_fc("a.rs", vec![1, 0]),
                make_fc("b.rs", vec![0, 1]),
                make_fc("a.rs", vec![0, 1]),
            ];

            let result = sum_file_coverages(input).into_inner();
            assert_eq!(result.len(), 2);

            let a = result.iter().find(|fc| fc.path == "a.rs").unwrap();
            assert_eq!(a.hits, vec![1, 1]);

            let b = result.iter().find(|fc| fc.path == "b.rs").unwrap();
            assert_eq!(b.hits, vec![0, 1]);
        }
    }
}
