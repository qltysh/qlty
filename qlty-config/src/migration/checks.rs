use super::classic::{Check, Checks, ClassicConfig};
use anyhow::Result;
use toml_edit::{table, value, Table};

const CLASSIC_COMPLEX_LOGIC_DEFAULT_THRESHOLD: i64 = 4;
const CLASSIC_ARGUMENT_COUNT_DEFAULT_THRESHOLD: i64 = 4;
const CLASSIC_METHOD_COMPLEXITY_DEFAULT_THRESHOLD: i64 = 5;
const CLASSIC_RETURN_STATEMENTS_DEFAULT_THRESHOLD: i64 = 4;
const CLASSIC_NESTED_CONTROL_FLOW_DEFAULT_THRESHOLD: i64 = 4;
const CLASSIC_FILE_LINES_DEFAULT_THRESHOLD: i64 = 250;
const DUPLICATION_DEFAULT_THRESHOLD: i64 = 20;

const LEEWAY_THRESHOLD_MULTIPLER: f64 = 1.1;
const DEFAULT_THRESHOLD_MULTIPLIER: f64 = 1.0;
const FILE_COMPLEXITY_THRESHOLD_MULTIPLER: f64 = 0.22;

const WEIGHTED_DUPLICATION_DEFAULT_THRESHOLD: i64 =
    (DUPLICATION_DEFAULT_THRESHOLD as f64 * LEEWAY_THRESHOLD_MULTIPLER) as i64;

const ALL_QLTY_STRUCTURE_CHECKS: [QltyStructureChecks; 6] = [
    QltyStructureChecks::BooleanLogic,
    QltyStructureChecks::FileComplexity,
    QltyStructureChecks::ReturnStatements,
    QltyStructureChecks::NestedControlFlow,
    QltyStructureChecks::FunctionParameters,
    QltyStructureChecks::FunctionComplexity,
];

#[derive(Debug)]
enum QltyStructureChecks {
    BooleanLogic,
    FileComplexity,
    ReturnStatements,
    NestedControlFlow,
    FunctionParameters,
    FunctionComplexity,
}

impl QltyStructureChecks {
    fn to_string(&self) -> &'static str {
        match self {
            QltyStructureChecks::BooleanLogic => "boolean_logic",
            QltyStructureChecks::FileComplexity => "file_complexity",
            QltyStructureChecks::ReturnStatements => "return_statements",
            QltyStructureChecks::NestedControlFlow => "nested_control_flow",
            QltyStructureChecks::FunctionParameters => "function_parameters",
            QltyStructureChecks::FunctionComplexity => "function_complexity",
        }
    }

    fn default_classic_threshold(&self) -> i64 {
        match self {
            QltyStructureChecks::BooleanLogic => CLASSIC_COMPLEX_LOGIC_DEFAULT_THRESHOLD,
            QltyStructureChecks::FileComplexity => CLASSIC_FILE_LINES_DEFAULT_THRESHOLD,
            QltyStructureChecks::ReturnStatements => CLASSIC_RETURN_STATEMENTS_DEFAULT_THRESHOLD,
            QltyStructureChecks::NestedControlFlow => CLASSIC_NESTED_CONTROL_FLOW_DEFAULT_THRESHOLD,
            QltyStructureChecks::FunctionParameters => CLASSIC_ARGUMENT_COUNT_DEFAULT_THRESHOLD,
            QltyStructureChecks::FunctionComplexity => CLASSIC_METHOD_COMPLEXITY_DEFAULT_THRESHOLD,
        }
    }

    fn threshold_multiplier(&self) -> f64 {
        match self {
            QltyStructureChecks::FileComplexity => FILE_COMPLEXITY_THRESHOLD_MULTIPLER,
            QltyStructureChecks::FunctionComplexity => LEEWAY_THRESHOLD_MULTIPLER,
            _ => DEFAULT_THRESHOLD_MULTIPLIER,
        }
    }

    fn classic_check_equivalent<'a>(&self, checks: &'a Checks) -> &'a Option<Check> {
        match self {
            QltyStructureChecks::BooleanLogic => &checks.complex_logic,
            QltyStructureChecks::FileComplexity => &checks.file_lines,
            QltyStructureChecks::ReturnStatements => &checks.return_statements,
            QltyStructureChecks::NestedControlFlow => &checks.nested_control_flow,
            QltyStructureChecks::FunctionParameters => &checks.argument_count,
            QltyStructureChecks::FunctionComplexity => &checks.method_complexity,
        }
    }

    fn check_threshold(&self, check_option: &Option<Check>) -> i64 {
        if let Some(check) = check_option.as_ref() {
            if let Some(config) = check.config.as_ref() {
                if let Some(threshold) = config.threshold {
                    return threshold;
                }
            }
        }

        self.default_classic_threshold()
    }

    fn weighted_check_threshold(&self, checks: &Checks) -> i64 {
        let classic_threshold = match self {
            QltyStructureChecks::BooleanLogic => self.check_threshold(&checks.complex_logic),
            QltyStructureChecks::FileComplexity => self.check_threshold(&checks.file_lines),
            QltyStructureChecks::FunctionParameters => self.check_threshold(&checks.argument_count),
            QltyStructureChecks::ReturnStatements => {
                self.check_threshold(&checks.return_statements)
            }
            QltyStructureChecks::NestedControlFlow => {
                self.check_threshold(&checks.nested_control_flow)
            }
            QltyStructureChecks::FunctionComplexity => {
                self.check_threshold(&checks.method_complexity)
            }
        };

        (classic_threshold as f64 * self.threshold_multiplier()) as i64
    }
}

#[derive(Debug)]
pub struct CheckMigration {}

impl CheckMigration {
    pub fn migrate_maintainability_checks(
        classic_config: &ClassicConfig,
        smells_table: &mut Table,
    ) -> Result<()> {
        if let Some(checks) = classic_config.checks.as_ref() {
            Self::migrate_structure_checks(smells_table, checks);
            Self::migrate_duplication_check(smells_table, checks);
        } else {
            Self::migrate_defaults(smells_table)?;
        }

        Ok(())
    }

    fn migrate_structure_checks(smells_table: &mut Table, checks: &Checks) {
        for check in ALL_QLTY_STRUCTURE_CHECKS {
            let check_table = smells_table
                .entry(check.to_string())
                .or_insert(table())
                .as_table_mut()
                .unwrap();

            let check_value = check.weighted_check_threshold(checks);
            check_table["threshold"] = value(check_value);

            if let Some(enabled) = check_enabled(check.classic_check_equivalent(checks)) {
                check_table["enabled"] = value(enabled);
            }
        }
    }

    fn migrate_duplication_check(smells_table: &mut Table, checks: &Checks) {
        let duplication_table = smells_table
            .entry("duplication")
            .or_insert(table())
            .as_table_mut()
            .unwrap();

        let similar_code_enabled = check_enabled(&checks.similar_code).unwrap_or(true);
        let identical_code_enabled = check_enabled(&checks.identical_code).unwrap_or(true);

        // disable only if both are disabled
        if !similar_code_enabled && !identical_code_enabled {
            duplication_table["enabled"] = value(false);
            return;
        } else {
            duplication_table["enabled"] = value(true);
        }

        let identical_code_threshold = checks
            .identical_code
            .as_ref()
            .and_then(|check| check.config.as_ref())
            .and_then(|config| config.threshold)
            .unwrap_or(DUPLICATION_DEFAULT_THRESHOLD);

        let similar_code_threshold = checks
            .similar_code
            .as_ref()
            .and_then(|check| check.config.as_ref())
            .and_then(|config| config.threshold)
            .unwrap_or(DUPLICATION_DEFAULT_THRESHOLD);

        let threshold = (identical_code_threshold + similar_code_threshold) / 2;

        duplication_table["threshold"] = value(threshold);
    }

    fn migrate_defaults(smells_table: &mut Table) -> Result<()> {
        let default_checks = Checks::default();

        for check in ALL_QLTY_STRUCTURE_CHECKS {
            Self::migrate_default(
                smells_table,
                check.to_string(),
                check.weighted_check_threshold(&default_checks),
            );
        }

        Self::migrate_default(
            smells_table,
            "duplication",
            WEIGHTED_DUPLICATION_DEFAULT_THRESHOLD,
        );

        Ok(())
    }

    fn migrate_default(smells_table: &mut Table, check: &str, threshold: i64) {
        smells_table[check] = table();
        let check_table = smells_table.get_mut(check).unwrap().as_table_mut().unwrap();

        check_table["threshold"] = value(threshold);
    }
}

fn check_enabled(check_option: &Option<Check>) -> Option<bool> {
    if let Some(check) = check_option.as_ref() {
        if let Some(enabled) = check.enabled {
            return Some(enabled);
        }
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::migration::classic::CheckConfig;
    use toml_edit::table;

    // Helper function to create a ClassicConfig with default values
    fn default_classic_config() -> ClassicConfig {
        ClassicConfig {
            prepare: None,
            checks: Some(Checks {
                complex_logic: None,
                file_lines: None,
                return_statements: None,
                nested_control_flow: None,
                argument_count: None,
                method_complexity: None,
                similar_code: None,
                identical_code: None,
            }),
            exclude_patterns: None,
        }
    }

    #[test]
    fn test_migrate_defaults() -> Result<()> {
        let mut empty_table = table();
        let mut smells_table = empty_table.as_table_mut().unwrap();

        CheckMigration::migrate_defaults(&mut smells_table)?;

        // Validate default values for all quality checks
        for check in ALL_QLTY_STRUCTURE_CHECKS.iter() {
            let check_table = smells_table
                .get(check.to_string())
                .unwrap()
                .as_table()
                .unwrap();
            assert_eq!(
                check_table
                    .get("threshold")
                    .unwrap()
                    .as_value()
                    .unwrap()
                    .as_integer()
                    .unwrap(),
                check.weighted_check_threshold(&Checks::default())
            );
        }

        let duplication_table = smells_table.get("duplication").unwrap().as_table().unwrap();
        assert_eq!(
            duplication_table
                .get("threshold")
                .unwrap()
                .as_value()
                .unwrap()
                .as_integer()
                .unwrap(),
            WEIGHTED_DUPLICATION_DEFAULT_THRESHOLD
        );

        Ok(())
    }

    #[test]
    fn test_migrate_structure_checks() -> Result<()> {
        let mut empty_table = table();
        let mut smells_table = empty_table.as_table_mut().unwrap();
        let mut classic_config = default_classic_config();
        classic_config.checks = Some(Checks {
            complex_logic: Some(Check {
                enabled: Some(true),
                config: Some(CheckConfig { threshold: Some(5) }),
            }),
            file_lines: Some(Check {
                enabled: Some(false),
                config: Some(CheckConfig {
                    threshold: Some(100),
                }),
            }),
            return_statements: None,
            nested_control_flow: None,
            argument_count: None,
            method_complexity: None,
            similar_code: None,
            identical_code: None,
        });

        CheckMigration::migrate_structure_checks(
            &mut smells_table,
            &classic_config.checks.as_ref().unwrap(),
        );

        // Validate the migration results
        let boolean_logic_table = smells_table
            .get("boolean_logic")
            .unwrap()
            .as_table()
            .unwrap();
        assert_eq!(
            boolean_logic_table
                .get("threshold")
                .unwrap()
                .as_value()
                .unwrap()
                .as_integer()
                .unwrap(),
            5
        );
        assert_eq!(
            boolean_logic_table
                .get("enabled")
                .unwrap()
                .as_value()
                .unwrap()
                .as_bool()
                .unwrap(),
            true
        );

        let file_complexity_table = smells_table
            .get("file_complexity")
            .unwrap()
            .as_table()
            .unwrap();
        assert_eq!(
            file_complexity_table
                .get("threshold")
                .unwrap()
                .as_value()
                .unwrap()
                .as_integer()
                .unwrap(),
            22
        );
        assert_eq!(
            file_complexity_table
                .get("enabled")
                .unwrap()
                .as_value()
                .unwrap()
                .as_bool()
                .unwrap(),
            false
        );

        Ok(())
    }

    #[test]
    fn test_migrate_duplication_check() -> Result<()> {
        let mut empty_table = table();
        let mut smells_table = empty_table.as_table_mut().unwrap();

        let classic_config = ClassicConfig {
            prepare: None,
            checks: Some(Checks {
                similar_code: Some(Check {
                    enabled: Some(false),
                    config: Some(CheckConfig {
                        threshold: Some(30),
                    }),
                }),
                identical_code: Some(Check {
                    enabled: Some(true),
                    config: Some(CheckConfig {
                        threshold: Some(50),
                    }),
                }),
                ..Default::default()
            }),
            exclude_patterns: None,
        };

        CheckMigration::migrate_duplication_check(
            &mut smells_table,
            &classic_config.checks.as_ref().unwrap(),
        );

        // Validate the duplication check
        let duplication_table = smells_table.get("duplication").unwrap().as_table().unwrap();
        assert_eq!(
            duplication_table
                .get("threshold")
                .unwrap()
                .as_value()
                .unwrap()
                .as_integer()
                .unwrap(),
            40 // Average of 30 and 50
        );
        assert_eq!(
            duplication_table
                .get("enabled")
                .unwrap()
                .as_value()
                .unwrap()
                .as_bool()
                .unwrap(),
            true
        );

        Ok(())
    }

    #[test]
    fn test_migrate_maintainability_checks_with_config() -> Result<()> {
        let mut empty_table = table();
        let mut smells_table = empty_table.as_table_mut().unwrap();

        let classic_config = ClassicConfig {
            prepare: None,
            exclude_patterns: None,
            checks: Some(Checks {
                complex_logic: Some(Check {
                    enabled: Some(true),
                    config: Some(CheckConfig { threshold: Some(5) }),
                }),
                file_lines: Some(Check {
                    enabled: Some(false),
                    config: Some(CheckConfig {
                        threshold: Some(100),
                    }),
                }),
                return_statements: Some(Check {
                    enabled: Some(true),
                    config: Some(CheckConfig {
                        threshold: Some(10),
                    }),
                }),
                nested_control_flow: Some(Check {
                    enabled: Some(false),
                    config: Some(CheckConfig {
                        threshold: Some(15),
                    }),
                }),
                argument_count: Some(Check {
                    enabled: Some(true),
                    config: Some(CheckConfig {
                        threshold: Some(20),
                    }),
                }),
                method_complexity: Some(Check {
                    enabled: Some(true),
                    config: Some(CheckConfig {
                        threshold: Some(25),
                    }),
                }),
                similar_code: Some(Check {
                    enabled: Some(false),
                    config: Some(CheckConfig {
                        threshold: Some(30),
                    }),
                }),
                identical_code: Some(Check {
                    enabled: Some(true),
                    config: Some(CheckConfig {
                        threshold: Some(40),
                    }),
                }),
            }),
        };

        CheckMigration::migrate_maintainability_checks(&classic_config, &mut smells_table)?;

        // Validate the results for each check
        for check in ALL_QLTY_STRUCTURE_CHECKS.iter() {
            let check_table = smells_table
                .get(check.to_string())
                .unwrap()
                .as_table()
                .unwrap();

            assert_eq!(
                check_table
                    .get("threshold")
                    .unwrap()
                    .as_value()
                    .unwrap()
                    .as_integer()
                    .unwrap(),
                check.weighted_check_threshold(&classic_config.checks.as_ref().unwrap())
            );

            assert_eq!(
                check_table
                    .get("enabled")
                    .map(|v| v.as_value().unwrap().as_bool().unwrap())
                    .unwrap_or(true),
                check_enabled(
                    check.classic_check_equivalent(&classic_config.checks.as_ref().unwrap())
                )
                .unwrap_or(true)
            );
        }

        let duplication_table = smells_table.get("duplication").unwrap().as_table().unwrap();
        assert_eq!(
            duplication_table
                .get("threshold")
                .unwrap()
                .as_value()
                .unwrap()
                .as_integer()
                .unwrap(),
            (30 + 40) / 2 // Average of similar_code and identical_code thresholds
        );
        assert_eq!(
            duplication_table
                .get("enabled")
                .unwrap()
                .as_value()
                .unwrap()
                .as_bool()
                .unwrap(),
            true // Enabled since at least one of similar_code or identical_code is enabled
        );

        Ok(())
    }

    #[test]
    fn test_migrate_maintainability_checks_with_defaults() -> Result<()> {
        let mut empty_table = table();
        let mut smells_table = empty_table.as_table_mut().unwrap();

        let classic_config = ClassicConfig {
            prepare: None,
            checks: None,
            exclude_patterns: None,
        };

        CheckMigration::migrate_maintainability_checks(&classic_config, &mut smells_table)?;

        // Validate the default values for all quality checks
        for check in ALL_QLTY_STRUCTURE_CHECKS.iter() {
            let check_table = smells_table
                .get(check.to_string())
                .unwrap()
                .as_table()
                .unwrap();

            assert_eq!(
                check_table
                    .get("threshold")
                    .unwrap()
                    .as_value()
                    .unwrap()
                    .as_integer()
                    .unwrap(),
                check.weighted_check_threshold(&Checks::default())
            );
        }

        let duplication_table = smells_table.get("duplication").unwrap().as_table().unwrap();
        assert_eq!(
            duplication_table
                .get("threshold")
                .unwrap()
                .as_value()
                .unwrap()
                .as_integer()
                .unwrap(),
            WEIGHTED_DUPLICATION_DEFAULT_THRESHOLD
        );

        Ok(())
    }
}
