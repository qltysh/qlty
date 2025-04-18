use super::Exclude;

#[derive(Debug)]
pub struct ExcludeGroup {
    pub excludes: Vec<String>,
    pub negate: bool,
}

impl ExcludeGroup {
    pub fn build_from_excludes(excludes: &Vec<&Exclude>) -> Vec<Self> {
        let mut exclude_groups = vec![];

        let start_with_negated = excludes
            .first()
            .and_then(|exclude| exclude.file_patterns.first())
            .is_some_and(|pattern| pattern.starts_with('!'));

        let mut current_exclude_group = ExcludeGroup {
            excludes: vec![],
            negate: start_with_negated,
        };

        for exclude in excludes {
            if exclude.file_patterns.is_empty() {
                continue;
            } else if !exclude.plugins.is_empty() {
                // Skip this exclude if it has plugins
                // We have PluginSpecificExcludeMatcher for that
                continue;
            }

            for pattern in &exclude.file_patterns {
                if let Some(pattern) = pattern.strip_prefix('!') {
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
                        excludes: vec![pattern.to_string()],
                        negate: false,
                    };
                } else {
                    current_exclude_group.excludes.push(pattern.to_string());
                }
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

    // Helper function to create Exclude instances with default values
    fn build_exclude(file_patterns: Vec<&str>) -> Exclude {
        Exclude {
            file_patterns: file_patterns.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn test_empty_excludes() {
        let excludes: Vec<&Exclude> = vec![];
        let result = ExcludeGroup::build_from_excludes(&excludes);
        assert!(
            result.is_empty(),
            "Expected empty result for empty excludes input"
        );
    }

    #[test]
    fn test_single_non_negated_exclude() {
        let exclude1 = build_exclude(vec!["src/", "target/"]);

        let excludes: Vec<&Exclude> = vec![&exclude1];
        let result = ExcludeGroup::build_from_excludes(&excludes);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].excludes, vec!["src/", "target/"]);
        assert_eq!(result[0].negate, false);
    }

    #[test]
    fn test_single_negated_exclude() {
        let exclude1 = build_exclude(vec!["!src/", "!target/"]);

        let excludes: Vec<&Exclude> = vec![&exclude1];
        let result = ExcludeGroup::build_from_excludes(&excludes);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].excludes, vec!["src/", "target/"]);
        assert_eq!(result[0].negate, true);
    }

    #[test]
    fn test_mixed_negated_and_non_negated_patterns() {
        let exclude1 = build_exclude(vec!["src/", "!target/"]);
        let exclude2 = build_exclude(vec!["bin/", "!out/"]);

        let excludes: Vec<&Exclude> = vec![&exclude1, &exclude2];
        let result = ExcludeGroup::build_from_excludes(&excludes);

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
        let exclude1 = build_exclude(vec!["!foo/", "!bar/"]);
        let exclude2 = build_exclude(vec!["baz/"]);
        let exclude3 = build_exclude(vec!["!qux/"]);

        let excludes: Vec<&Exclude> = vec![&exclude1, &exclude2, &exclude3];
        let result = ExcludeGroup::build_from_excludes(&excludes);

        assert_eq!(result.len(), 3);

        assert_eq!(result[0].excludes, vec!["foo/", "bar/"]);
        assert_eq!(result[0].negate, true);

        assert_eq!(result[1].excludes, vec!["baz/"]);
        assert_eq!(result[1].negate, false);

        assert_eq!(result[2].excludes, vec!["qux/"]);
        assert_eq!(result[2].negate, true);
    }
}
