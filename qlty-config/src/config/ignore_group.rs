use super::Ignore;

#[derive(Debug)]
pub struct IgnoreGroup {
    pub ignores: Vec<String>,
    pub negate: bool,
}

impl IgnoreGroup {
    pub fn build_from_ignores(ignores: &Vec<&Ignore>) -> Vec<Self> {
        let mut ignore_groups = vec![];

        let start_with_negated = ignores
            .first()
            .and_then(|ignore| ignore.file_patterns.first())
            .is_some_and(|pattern| pattern.starts_with('!'));

        let mut current_ignore_group = IgnoreGroup {
            ignores: vec![],
            negate: start_with_negated,
        };

        for ignore in ignores {
            if ignore.file_patterns.is_empty() {
                continue;
            } else if !ignore.rules.is_empty() {
                // If specific rules are defined, this is not a blanket ignore group.
                // It is only ignored in the context of a specific rule, but we still need to
                // process the files.
                continue;
            }

            for pattern in &ignore.file_patterns {
                if let Some(pattern) = pattern.strip_prefix('!') {
                    if current_ignore_group.negate {
                        current_ignore_group.ignores.push(pattern.to_string());
                    } else {
                        // Push previous group before switching negation
                        ignore_groups.push(current_ignore_group);
                        current_ignore_group = IgnoreGroup {
                            ignores: vec![pattern.to_string()],
                            negate: true,
                        };
                    }
                } else if current_ignore_group.negate {
                    ignore_groups.push(current_ignore_group);
                    current_ignore_group = IgnoreGroup {
                        ignores: vec![pattern.to_string()],
                        negate: false,
                    };
                } else {
                    current_ignore_group.ignores.push(pattern.to_string());
                }
            }
        }

        // Ensure the last group is added
        if !current_ignore_group.ignores.is_empty() {
            ignore_groups.push(current_ignore_group);
        }

        ignore_groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create Ignore instances with default values
    fn build_ignore(file_patterns: Vec<&str>) -> Ignore {
        Ignore {
            file_patterns: file_patterns.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    fn build_ignore_with_rules(file_patterns: Vec<&str>, rules: Vec<String>) -> Ignore {
        Ignore {
            file_patterns: file_patterns.iter().map(|s| s.to_string()).collect(),
            rules,
            ..Default::default()
        }
    }

    #[test]
    fn test_empty_ignores() {
        let ignores: Vec<&Ignore> = vec![];
        let result = IgnoreGroup::build_from_ignores(&ignores);
        assert!(
            result.is_empty(),
            "Expected empty result for empty ignores input"
        );
    }

    #[test]
    fn test_single_non_negated_ignore() {
        let ignore1 = build_ignore(vec!["src/", "target/"]);

        let ignores: Vec<&Ignore> = vec![&ignore1];
        let result = IgnoreGroup::build_from_ignores(&ignores);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].ignores, vec!["src/", "target/"]);
        assert_eq!(result[0].negate, false);
    }

    #[test]
    fn test_single_negated_ignore() {
        let ignore1 = build_ignore(vec!["!src/", "!target/"]);

        let ignores: Vec<&Ignore> = vec![&ignore1];
        let result = IgnoreGroup::build_from_ignores(&ignores);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].ignores, vec!["src/", "target/"]);
        assert_eq!(result[0].negate, true);
    }

    #[test]
    fn test_mixed_negated_and_non_negated_patterns() {
        let ignore1 = build_ignore(vec!["src/", "!target/"]);
        let ignore2 = build_ignore(vec!["bin/", "!out/"]);

        let ignores: Vec<&Ignore> = vec![&ignore1, &ignore2];
        let result = IgnoreGroup::build_from_ignores(&ignores);

        assert_eq!(result.len(), 4);
        assert_eq!(result[0].ignores, vec!["src/"]);
        assert_eq!(result[0].negate, false);

        assert_eq!(result[1].ignores, vec!["target/"]);
        assert_eq!(result[1].negate, true);

        assert_eq!(result[2].ignores, vec!["bin/"]);
        assert_eq!(result[2].negate, false);

        assert_eq!(result[3].ignores, vec!["out/"]);
        assert_eq!(result[3].negate, true);
    }

    #[test]
    fn test_multiple_negated_blocks() {
        let ignore1 = build_ignore(vec!["!foo/", "!bar/"]);
        let ignore2 = build_ignore(vec!["baz/"]);
        let ignore3 = build_ignore(vec!["!qux/"]);

        let ignores: Vec<&Ignore> = vec![&ignore1, &ignore2, &ignore3];
        let result = IgnoreGroup::build_from_ignores(&ignores);

        assert_eq!(result.len(), 3);

        assert_eq!(result[0].ignores, vec!["foo/", "bar/"]);
        assert_eq!(result[0].negate, true);

        assert_eq!(result[1].ignores, vec!["baz/"]);
        assert_eq!(result[1].negate, false);

        assert_eq!(result[2].ignores, vec!["qux/"]);
        assert_eq!(result[2].negate, true);
    }

    #[test]
    fn test_ignore_with_rules() {
        let ignore1 = build_ignore_with_rules(vec!["src/"], vec!["rule1".to_string()]);
        let ignore2 = build_ignore(vec!["target/"]);

        let ignores: Vec<&Ignore> = vec![&ignore1, &ignore2];
        let result = IgnoreGroup::build_from_ignores(&ignores);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].ignores, vec!["target/"]);
        assert_eq!(result[0].negate, false);
    }
}
