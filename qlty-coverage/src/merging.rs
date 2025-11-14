use qlty_types::tests::v1::FileCoverage;

/// Merges file coverages with duplicate paths in-place.
///
/// When multiple FileCoverage entries share the same path, they are merged into a single entry
/// following these rules:
/// - If both hits[i] are non-negative (>= 0), sum them
/// - If either hits[i] is negative, result is -1
/// - If arrays are different sizes, use the element that exists
/// - Missing elements are treated as -1
///
/// Time: O(n log n) for sort + O(n * m) for merge where m = avg hits length
/// Space: O(1) extra space (excluding input vector and temporary merged hits)
pub fn merge_file_coverages(file_coverages: &mut Vec<FileCoverage>) {
    if file_coverages.len() <= 1 {
        return;
    }

    // Step 1: Sort by path to group duplicates together
    file_coverages.sort_by(|a, b| a.path.cmp(&b.path));

    // Step 2: Merge consecutive entries with same path
    let mut write_idx = 0;
    let mut read_idx = 0;

    while read_idx < file_coverages.len() {
        // Find range of entries with same path
        let start_idx = read_idx;
        let current_path = file_coverages[start_idx].path.clone();

        while read_idx < file_coverages.len() && file_coverages[read_idx].path == current_path {
            read_idx += 1;
        }
        let end_idx = read_idx;

        // Merge range [start_idx..end_idx) into position write_idx
        if end_idx - start_idx == 1 {
            // Single entry, just move it if needed
            if write_idx != start_idx {
                file_coverages.swap(write_idx, start_idx);
            }
        } else {
            // Multiple entries need merging
            merge_hits_at_index(file_coverages, start_idx, end_idx, write_idx);
        }

        write_idx += 1;
    }

    // Step 3: Truncate vector to remove merged entries
    file_coverages.truncate(write_idx);
}

fn merge_hits_at_index(
    file_coverages: &mut [FileCoverage],
    start_idx: usize,
    end_idx: usize,
    target_idx: usize,
) {
    // Find max length across all hits arrays in this range
    let max_len = file_coverages[start_idx..end_idx]
        .iter()
        .map(|fc| fc.hits.len())
        .max()
        .unwrap_or(0);

    // Create merged hits array
    let mut merged_hits = Vec::with_capacity(max_len);

    for i in 0..max_len {
        let mut result: Option<i64> = None;

        for fc_idx in start_idx..end_idx {
            let hit = file_coverages[fc_idx].hits.get(i).copied();

            result = match (result, hit) {
                // First value we encounter
                (None, Some(h)) => Some(h),
                (None, None) => Some(-1), // Out of bounds treated as -1

                // Either is negative → result is -1
                (Some(a), Some(b)) if a < 0 || b < 0 => Some(-1),
                (Some(a), None) if a < 0 => Some(-1),
                (Some(_), None) => Some(-1), // Missing element treated as -1

                // Both non-negative → sum them
                (Some(a), Some(b)) => Some(a + b),
            };
        }

        merged_hits.push(result.unwrap_or(-1));
    }

    // Move first entry to target position and update its hits
    if target_idx != start_idx {
        file_coverages.swap(target_idx, start_idx);
    }
    file_coverages[target_idx].hits = merged_hits;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_file_coverage(path: &str, hits: Vec<i64>) -> FileCoverage {
        FileCoverage {
            path: path.to_string(),
            hits,
            ..Default::default()
        }
    }

    #[test]
    fn test_no_duplicates() {
        let mut coverages = vec![
            create_file_coverage("src/a.rs", vec![1, 2, 3]),
            create_file_coverage("src/b.rs", vec![4, 5, 6]),
        ];

        merge_file_coverages(&mut coverages);

        assert_eq!(coverages.len(), 2);
        assert_eq!(coverages[0].path, "src/a.rs");
        assert_eq!(coverages[0].hits, vec![1, 2, 3]);
        assert_eq!(coverages[1].path, "src/b.rs");
        assert_eq!(coverages[1].hits, vec![4, 5, 6]);
    }

    #[test]
    fn test_single_duplicate_both_non_negative_sum() {
        let mut coverages = vec![
            create_file_coverage("src/main.rs", vec![1, 2, 0]),
            create_file_coverage("src/main.rs", vec![3, 0, 4]),
        ];

        merge_file_coverages(&mut coverages);

        assert_eq!(coverages.len(), 1);
        assert_eq!(coverages[0].path, "src/main.rs");
        assert_eq!(coverages[0].hits, vec![4, 2, 4]); // [1+3, 2+0, 0+4]
    }

    #[test]
    fn test_either_negative_results_in_negative_one() {
        let mut coverages = vec![
            create_file_coverage("src/main.rs", vec![1, -1, 5]),
            create_file_coverage("src/main.rs", vec![2, 3, -1]),
        ];

        merge_file_coverages(&mut coverages);

        assert_eq!(coverages.len(), 1);
        assert_eq!(coverages[0].path, "src/main.rs");
        assert_eq!(coverages[0].hits, vec![3, -1, -1]); // [1+2, either neg, either neg]
    }

    #[test]
    fn test_both_negative_results_in_negative_one() {
        let mut coverages = vec![
            create_file_coverage("src/main.rs", vec![-1, -1]),
            create_file_coverage("src/main.rs", vec![-1, -1]),
        ];

        merge_file_coverages(&mut coverages);

        assert_eq!(coverages.len(), 1);
        assert_eq!(coverages[0].path, "src/main.rs");
        assert_eq!(coverages[0].hits, vec![-1, -1]);
    }

    #[test]
    fn test_different_array_sizes_use_existing_elements() {
        let mut coverages = vec![
            create_file_coverage("src/main.rs", vec![1, 2, 3, 4]),
            create_file_coverage("src/main.rs", vec![5, 6]),
        ];

        merge_file_coverages(&mut coverages);

        assert_eq!(coverages.len(), 1);
        assert_eq!(coverages[0].path, "src/main.rs");
        assert_eq!(coverages[0].hits, vec![6, 8, -1, -1]); // [1+5, 2+6, 3+missing, 4+missing]
    }

    #[test]
    fn test_shorter_array_first() {
        let mut coverages = vec![
            create_file_coverage("src/main.rs", vec![1, 2]),
            create_file_coverage("src/main.rs", vec![3, 4, 5, 6]),
        ];

        merge_file_coverages(&mut coverages);

        assert_eq!(coverages.len(), 1);
        assert_eq!(coverages[0].path, "src/main.rs");
        assert_eq!(coverages[0].hits, vec![4, 6, -1, -1]); // [1+3, 2+4, missing+5, missing+6]
    }

    #[test]
    fn test_three_way_merge() {
        let mut coverages = vec![
            create_file_coverage("src/main.rs", vec![1, 0, 0]),
            create_file_coverage("src/main.rs", vec![0, 2, 0]),
            create_file_coverage("src/main.rs", vec![0, 0, 3]),
        ];

        merge_file_coverages(&mut coverages);

        assert_eq!(coverages.len(), 1);
        assert_eq!(coverages[0].path, "src/main.rs");
        assert_eq!(coverages[0].hits, vec![1, 2, 3]); // [1+0+0, 0+2+0, 0+0+3]
    }

    #[test]
    fn test_three_way_merge_with_negatives() {
        let mut coverages = vec![
            create_file_coverage("src/main.rs", vec![1, 2, 3]),
            create_file_coverage("src/main.rs", vec![4, -1, 6]),
            create_file_coverage("src/main.rs", vec![7, 8, 9]),
        ];

        merge_file_coverages(&mut coverages);

        assert_eq!(coverages.len(), 1);
        assert_eq!(coverages[0].path, "src/main.rs");
        assert_eq!(coverages[0].hits, vec![12, -1, 18]); // [1+4+7, 2+(-1)+8, 3+6+9]
    }

    #[test]
    fn test_multiple_files_with_duplicates() {
        let mut coverages = vec![
            create_file_coverage("src/a.rs", vec![1, 2]),
            create_file_coverage("src/b.rs", vec![3, 4]),
            create_file_coverage("src/a.rs", vec![5, 6]),
            create_file_coverage("src/c.rs", vec![7, 8]),
            create_file_coverage("src/b.rs", vec![9, 10]),
        ];

        merge_file_coverages(&mut coverages);

        assert_eq!(coverages.len(), 3);

        // Results should be sorted by path
        assert_eq!(coverages[0].path, "src/a.rs");
        assert_eq!(coverages[0].hits, vec![6, 8]); // [1+5, 2+6]

        assert_eq!(coverages[1].path, "src/b.rs");
        assert_eq!(coverages[1].hits, vec![12, 14]); // [3+9, 4+10]

        assert_eq!(coverages[2].path, "src/c.rs");
        assert_eq!(coverages[2].hits, vec![7, 8]);
    }

    #[test]
    fn test_empty_vector() {
        let mut coverages: Vec<FileCoverage> = vec![];
        merge_file_coverages(&mut coverages);
        assert_eq!(coverages.len(), 0);
    }

    #[test]
    fn test_single_entry() {
        let mut coverages = vec![create_file_coverage("src/main.rs", vec![1, 2, 3])];
        merge_file_coverages(&mut coverages);

        assert_eq!(coverages.len(), 1);
        assert_eq!(coverages[0].path, "src/main.rs");
        assert_eq!(coverages[0].hits, vec![1, 2, 3]);
    }

    #[test]
    fn test_empty_hits_arrays() {
        let mut coverages = vec![
            create_file_coverage("src/main.rs", vec![]),
            create_file_coverage("src/main.rs", vec![]),
        ];

        merge_file_coverages(&mut coverages);

        assert_eq!(coverages.len(), 1);
        assert_eq!(coverages[0].path, "src/main.rs");
        assert_eq!(coverages[0].hits, Vec::<i64>::new());
    }

    #[test]
    fn test_one_empty_one_with_data() {
        let mut coverages = vec![
            create_file_coverage("src/main.rs", vec![]),
            create_file_coverage("src/main.rs", vec![1, 2, 3]),
        ];

        merge_file_coverages(&mut coverages);

        assert_eq!(coverages.len(), 1);
        assert_eq!(coverages[0].path, "src/main.rs");
        assert_eq!(coverages[0].hits, vec![-1, -1, -1]); // Empty treated as missing
    }

    #[test]
    fn test_preserves_other_fields() {
        let mut coverages = vec![
            FileCoverage {
                path: "src/main.rs".to_string(),
                hits: vec![1, 2],
                build_id: "build-123".to_string(),
                blob_oid: "abc123".to_string(),
                ..Default::default()
            },
            FileCoverage {
                path: "src/main.rs".to_string(),
                hits: vec![3, 4],
                build_id: "build-456".to_string(),
                blob_oid: "def456".to_string(),
                ..Default::default()
            },
        ];

        merge_file_coverages(&mut coverages);

        assert_eq!(coverages.len(), 1);
        assert_eq!(coverages[0].hits, vec![4, 6]);
        // Should preserve fields from the first entry
        assert_eq!(coverages[0].build_id, "build-123");
        assert_eq!(coverages[0].blob_oid, "abc123");
    }
}
