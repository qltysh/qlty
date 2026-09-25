//! The fixed feature space: eight Qlty smells times four encodings, plus seven
//! Jev questions. Names and order are part of the model contract.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

pub const RULES: [Rule; 8] = [
    Rule::BooleanLogic,
    Rule::NestedControlFlow,
    Rule::FunctionParameters,
    Rule::ReturnStatements,
    Rule::FileComplexity,
    Rule::FunctionComplexity,
    Rule::IdenticalCode,
    Rule::SimilarCode,
];

pub const FIELDS: [Field; 4] = [
    Field::LogCount,
    Field::CountPer100SourceLines,
    Field::LogMaxMagnitude,
    Field::LineCoverageFraction,
];

pub const QLTY_FEATURE_COUNT: usize = RULES.len() * FIELDS.len();

/// One of the eight Qlty smell rules SlopOne measures.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Rule {
    BooleanLogic,
    NestedControlFlow,
    FunctionParameters,
    ReturnStatements,
    FileComplexity,
    FunctionComplexity,
    IdenticalCode,
    SimilarCode,
}

impl Rule {
    /// The Qlty rule key, as in `qlty:boolean-logic`.
    pub fn key(self) -> &'static str {
        match self {
            Self::BooleanLogic => "boolean-logic",
            Self::NestedControlFlow => "nested-control-flow",
            Self::FunctionParameters => "function-parameters",
            Self::ReturnStatements => "return-statements",
            Self::FileComplexity => "file-complexity",
            Self::FunctionComplexity => "function-complexity",
            Self::IdenticalCode => "identical-code",
            Self::SimilarCode => "similar-code",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        RULES.into_iter().find(|rule| rule.key() == key)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::BooleanLogic => "Boolean logic",
            Self::NestedControlFlow => "Nested control flow",
            Self::FunctionParameters => "Function parameters",
            Self::ReturnStatements => "Return statements",
            Self::FileComplexity => "File complexity",
            Self::FunctionComplexity => "Function complexity",
            Self::IdenticalCode => "Identical code",
            Self::SimilarCode => "Similar code",
        }
    }

    /// The unit of `properties.actual` for this rule.
    pub fn magnitude_unit(self) -> &'static str {
        match self {
            Self::BooleanLogic => "Boolean depth",
            Self::NestedControlFlow => "Nesting depth",
            Self::FunctionParameters => "Parameters",
            Self::ReturnStatements => "Return statements",
            Self::FileComplexity | Self::FunctionComplexity => "Cyclomatic complexity",
            Self::IdenticalCode | Self::SimilarCode => "Duplicated lines",
        }
    }

    pub fn is_duplication(self) -> bool {
        matches!(self, Self::IdenticalCode | Self::SimilarCode)
    }

    /// The SlopOne detection threshold. Rust uses 5 for Boolean logic.
    pub fn threshold(self, language: crate::language::Language) -> usize {
        match self {
            Self::BooleanLogic if language == crate::language::Language::Rust => 5,
            Self::BooleanLogic => 4,
            Self::NestedControlFlow => 5,
            Self::FunctionParameters | Self::ReturnStatements => 6,
            Self::FileComplexity => 50,
            Self::FunctionComplexity => 18,
            Self::IdenticalCode | Self::SimilarCode => 15,
        }
    }

    fn snake(self) -> String {
        self.key().replace('-', "_")
    }
}

/// One of the four encodings of a smell's findings.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Field {
    LogCount,
    CountPer100SourceLines,
    LogMaxMagnitude,
    LineCoverageFraction,
}

impl Field {
    pub fn id(self) -> &'static str {
        match self {
            Self::LogCount => "log_count",
            Self::CountPer100SourceLines => "count_per_100_source_lines",
            Self::LogMaxMagnitude => "log_max_magnitude",
            Self::LineCoverageFraction => "line_coverage_fraction",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        FIELDS.into_iter().find(|field| field.id() == id)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::LogCount => "Finding count",
            Self::CountPer100SourceLines => "Findings per 100 source lines",
            Self::LogMaxMagnitude => "Maximum magnitude",
            Self::LineCoverageFraction => "Source-line coverage",
        }
    }

    pub fn transform(self) -> &'static str {
        match self {
            Self::LogCount | Self::LogMaxMagnitude => "log1p",
            Self::CountPer100SourceLines | Self::LineCoverageFraction => "identity",
        }
    }

    /// The short label used in text output.
    pub fn short_label(self) -> &'static str {
        match self {
            Self::LogCount => "count",
            Self::CountPer100SourceLines => "density",
            Self::LogMaxMagnitude => "magnitude",
            Self::LineCoverageFraction => "coverage",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum QuestionKind {
    Noul,
    Score,
}

/// One of the seven fixed Jev questions, exactly as sent on the wire.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Question {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: QuestionKind,
    pub instructions: String,
    pub criteria: Vec<String>,
}

impl Question {
    pub fn label(&self) -> &'static str {
        match self.id.as_str() {
            "direct_unmaintainable" => "Overall difficulty of maintenance",
            "boilerplate" => "Routine boilerplate",
            "deep_reasoning" => "Interacting conditions and state",
            "control_flow" => "Difficulty following execution paths",
            "duplicated_logic" => "Duplicated logic",
            "mirrored_state" => "Manually synchronized state",
            "hand_enumerated_data" => "Hand-enumerated data",
            _ => "Jev assessment",
        }
    }
}

const QUESTIONS_JSON: &str = include_str!("assets/questions.json");

/// The seven questions, in feature order.
pub fn questions() -> &'static [Question] {
    static QUESTIONS: OnceLock<Vec<Question>> = OnceLock::new();
    QUESTIONS.get_or_init(|| {
        serde_json::from_str(QUESTIONS_JSON).expect("the bundled questions.json should be valid")
    })
}

/// The 39 feature names, in model input order.
pub fn feature_names() -> Vec<String> {
    let mut names: Vec<String> = RULES
        .iter()
        .flat_map(|rule| {
            FIELDS
                .iter()
                .map(move |field| format!("smell_{}_{}", rule.snake(), field.id()))
        })
        .collect();
    names.extend(
        questions()
            .iter()
            .map(|question| format!("{}_mean", question.id)),
    );
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_names_have_thirty_nine_entries_in_model_order() {
        let names = feature_names();
        assert_eq!(names.len(), 39);
        assert_eq!(names[0], "smell_boolean_logic_log_count");
        assert_eq!(names[31], "smell_similar_code_line_coverage_fraction");
        assert_eq!(names[32], "direct_unmaintainable_mean");
        assert_eq!(names[38], "hand_enumerated_data_mean");
    }

    #[test]
    fn questions_carry_their_kinds_and_criteria() {
        let questions = questions();
        assert_eq!(questions.len(), 7);
        assert_eq!(questions[0].kind, QuestionKind::Noul);
        assert!(questions[0].criteria.is_empty());
        assert_eq!(questions[3].kind, QuestionKind::Score);
        assert_eq!(questions[3].criteria.len(), 3);
    }

    #[test]
    fn rust_boolean_threshold_is_five() {
        assert_eq!(
            Rule::BooleanLogic.threshold(crate::language::Language::Rust),
            5
        );
        assert_eq!(
            Rule::BooleanLogic.threshold(crate::language::Language::Python),
            4
        );
    }
}
