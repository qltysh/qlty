#[derive(Debug)]
pub struct ExcludeGroup {
    pub excludes: Vec<String>,
    pub negate: bool,
}

impl ExcludeGroup {
    pub fn build_from_exclude_patterns(exclude_patterns: &Vec<String>) -> Vec<Self> {
        let mut exclude_groups = vec![];

        let start_with_negated = exclude_patterns
            .first()
            .is_some_and(|pattern| pattern.starts_with('!'));

        let mut current_exclude_group = ExcludeGroup {
            excludes: vec![],
            negate: start_with_negated,
        };

        for exclude_pattern in exclude_patterns {
            if exclude_pattern.is_empty() {
                continue;
            }

            if let Some(pattern) = exclude_pattern.strip_prefix('!') {
                if current_exclude_group.negate {
                    current_exclude_group.excludes.push(pattern.to_string());
                } else {
                    // Push previous group before switching negation
                    exclude_groups.push(current_exclude_group);
                    current_exclude_group = ExcludeGroup {
                        excludes: vec![pattern.to_string()],
                        negate: true,
                    };
                }
            } else if current_exclude_group.negate {
                exclude_groups.push(current_exclude_group);
                current_exclude_group = ExcludeGroup {
                    excludes: vec![exclude_pattern.to_string()],
                    negate: false,
                };
            } else {
                current_exclude_group
                    .excludes
                    .push(exclude_pattern.to_string());
            }
        }

        // Ensure the last group is added
        if !current_exclude_group.excludes.is_empty() {
            exclude_groups.push(current_exclude_group);
        }

        exclude_groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_excludes() {
        let exclude_patterns: Vec<String> = vec![];
        let result = ExcludeGroup::build_from_exclude_patterns(&exclude_patterns);
        assert!(
            result.is_empty(),
            "Expected empty result for empty excludes input"
        );
    }

    #[test]
    fn test_single_non_negated_exclude() {
        let exclude_patterns = vec!["src/".to_string(), "target/".to_string()];

        let result = ExcludeGroup::build_from_exclude_patterns(&exclude_patterns);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].excludes, vec!["src/", "target/"]);
        assert_eq!(result[0].negate, false);
    }

    #[test]
    fn test_single_negated_exclude() {
        let exclude_patterns = vec!["!src/".to_string(), "!target/".to_string()];

        let result = ExcludeGroup::build_from_exclude_patterns(&exclude_patterns);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].excludes, vec!["src/", "target/"]);
        assert_eq!(result[0].negate, true);
    }

    #[test]
    fn test_mixed_negated_and_non_negated_patterns() {
        let exclude_patterns = vec![
            "src/".to_string(),
            "!target/".to_string(),
            "bin/".to_string(),
            "!out/".to_string(),
        ];

        let result = ExcludeGroup::build_from_exclude_patterns(&exclude_patterns);

        assert_eq!(result.len(), 4);
        assert_eq!(result[0].excludes, vec!["src/"]);
        assert_eq!(result[0].negate, false);

        assert_eq!(result[1].excludes, vec!["target/"]);
        assert_eq!(result[1].negate, true);

        assert_eq!(result[2].excludes, vec!["bin/"]);
        assert_eq!(result[2].negate, false);

        assert_eq!(result[3].excludes, vec!["out/"]);
        assert_eq!(result[3].negate, true);
    }

    #[test]
    fn test_multiple_negated_blocks() {
        let exclude_patterns = vec![
            "!foo/".to_string(),
            "!bar/".to_string(),
            "baz/".to_string(),
            "!qux/".to_string(),
        ];

        let result = ExcludeGroup::build_from_exclude_patterns(&exclude_patterns);

        assert_eq!(result.len(), 3);

        assert_eq!(result[0].excludes, vec!["foo/", "bar/"]);
        assert_eq!(result[0].negate, true);

        assert_eq!(result[1].excludes, vec!["baz/"]);
        assert_eq!(result[1].negate, false);

        assert_eq!(result[2].excludes, vec!["qux/"]);
        assert_eq!(result[2].negate, true);
    }
}
