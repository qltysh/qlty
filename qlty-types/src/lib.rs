use pbjson_types::{value::Kind, Struct, Value};
use std::cmp::Ordering;
use std::{
    collections::HashMap,
    ops::{Add, AddAssign, RangeInclusive},
    path::{Path, PathBuf},
};

pub mod analysis {
    pub mod v1 {
        include!("protos/qlty.analysis.v1.rs");
    }
}

pub mod tests {
    pub mod v1 {
        include!("protos/qlty.tests.v1.rs");
    }
}

impl From<tree_sitter::Range> for analysis::v1::Range {
    fn from(range: tree_sitter::Range) -> Self {
        analysis::v1::Range {
            start_line: (range.start_point.row + 1) as u32,
            start_column: (range.start_point.column + 1) as u32,
            end_line: (range.end_point.row + 1) as u32,
            end_column: (range.end_point.column + 1) as u32,
            start_byte: Some(range.start_byte as u32),
            end_byte: Some(range.end_byte as u32),
        }
    }
}

impl Eq for analysis::v1::Issue {}

impl Ord for analysis::v1::Issue {
    fn cmp(&self, other: &Self) -> Ordering {
        let path_cmp = self.path().cmp(&other.path());
        if path_cmp != Ordering::Equal {
            return path_cmp;
        }

        let mut self_start_line = 0;

        if let Some(self_location) = &self.location {
            if let Some(self_start_range) = &self_location.range {
                let self_start_line_range = self_start_range.line_range();
                self_start_line = *self_start_line_range.start();
            }
        }

        let mut other_start_line = 0;

        if let Some(other_location) = &other.location {
            if let Some(other_start_range) = &other_location.range {
                let other_start_line_range = other_start_range.line_range();
                other_start_line = *other_start_line_range.start();
            }
        }

        let line_cmp = self_start_line.cmp(&other_start_line);
        if line_cmp != Ordering::Equal {
            return line_cmp;
        }

        let level_cmp = other.level.cmp(&self.level);
        if level_cmp != Ordering::Equal {
            return level_cmp;
        }

        self.message.cmp(&other.message)
    }
}

impl PartialOrd for analysis::v1::Issue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl analysis::v1::Issue {
    pub fn rule_id(&self) -> String {
        format!("{}:{}", self.tool, self.rule_key)
    }

    pub fn set_property_string(&mut self, key: &str, value: String) {
        self.properties
            .get_or_insert_with(Struct::default)
            .fields
            .insert(
                key.to_string(),
                Value {
                    kind: Some(Kind::StringValue(value)),
                },
            );
    }

    pub fn set_property_number(&mut self, key: &str, value: f64) {
        self.properties
            .get_or_insert_with(Struct::default)
            .fields
            .insert(
                key.to_string(),
                Value {
                    kind: Some(Kind::NumberValue(value)),
                },
            );
    }

    pub fn set_property_bool(&mut self, key: &str, value: bool) {
        self.properties
            .get_or_insert_with(Struct::default)
            .fields
            .insert(
                key.to_string(),
                Value {
                    kind: Some(Kind::BoolValue(value)),
                },
            );
    }

    pub fn get_property_string(&self, key: &str) -> String {
        match self.get_property_kind(key) {
            Kind::StringValue(value) => value.clone(),
            _ => panic!("Expected string value for property {}", key),
        }
    }

    pub fn get_property_number(&self, key: &str) -> f64 {
        match self.get_property_kind(key) {
            Kind::NumberValue(value) => *value,
            _ => panic!("Expected number value for property {}", key),
        }
    }

    pub fn get_property_bool(&self, key: &str) -> bool {
        match self.get_property_kind(key) {
            Kind::BoolValue(value) => *value,
            _ => panic!("Expected bool value for property {}", key),
        }
    }

    fn get_property_kind(&self, key: &str) -> &Kind {
        self.property_fields()
            .get(key)
            .unwrap()
            .kind
            .as_ref()
            .unwrap()
    }

    fn property_fields(&self) -> &HashMap<String, pbjson_types::Value> {
        &self.properties.as_ref().unwrap().fields
    }

    pub fn path(&self) -> Option<String> {
        if let Some(location) = self.location() {
            Some(location.path)
        } else {
            None
        }
    }

    pub fn line_range(&self) -> Option<RangeInclusive<usize>> {
        self.range().map(|range| range.line_range())
    }

    pub fn range(&self) -> Option<analysis::v1::Range> {
        if let Some(location) = self.location() {
            location.range()
        } else {
            None
        }
    }

    pub fn location(&self) -> Option<analysis::v1::Location> {
        self.location.clone()
    }
}

impl analysis::v1::Replacement {
    pub fn location(&self) -> analysis::v1::Location {
        self.location.as_ref().unwrap().clone()
    }

    pub fn range(&self) -> analysis::v1::Range {
        self.location().range().unwrap()
    }
}

impl analysis::v1::Location {
    pub fn line_range(&self) -> Option<RangeInclusive<usize>> {
        self.range().map(|range| range.line_range())
    }

    pub fn range(&self) -> Option<analysis::v1::Range> {
        self.range.clone()
    }

    pub fn relative_path(&self, base: &Path) -> String {
        let path = PathBuf::from(&self.path);

        // If it doesn't start with a slash, it is already
        // relative so return it as-is
        if path.is_relative() {
            return self.path.clone();
        }

        path.strip_prefix(base)
            .expect("Path not relative to base path")
            .to_string_lossy()
            .to_string()
    }
}

impl std::hash::Hash for analysis::v1::Location {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
        self.range.hash(state);
    }
}

impl Eq for analysis::v1::Location {}

impl analysis::v1::Range {
    pub fn line_range_u32(&self) -> RangeInclusive<u32> {
        self.start_line..=self.end_line
    }

    pub fn line_range(&self) -> RangeInclusive<usize> {
        (self.start_line as usize)..=(self.end_line as usize)
    }
}

impl std::hash::Hash for analysis::v1::Range {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.start_line.hash(state);
        self.start_column.hash(state);
        self.end_line.hash(state);
        self.end_column.hash(state);
        self.start_byte.hash(state);
        self.end_byte.hash(state);
    }
}

pub fn level_from_str(level: &str) -> analysis::v1::Level {
    match level.to_lowercase().as_str() {
        "high" => analysis::v1::Level::High,
        "medium" => analysis::v1::Level::Medium,
        "low" => analysis::v1::Level::Low,
        "fmt" => analysis::v1::Level::Fmt,
        _ => analysis::v1::Level::Unspecified,
    }
}

pub fn category_from_str(category: &str) -> analysis::v1::Category {
    match category.to_lowercase().as_str() {
        "bug" => analysis::v1::Category::Bug,
        "vulnerability" => analysis::v1::Category::Vulnerability,
        "structure" => analysis::v1::Category::Structure,
        "duplication" => analysis::v1::Category::Duplication,
        "security_hotspot" => analysis::v1::Category::SecurityHotspot,
        "performance" => analysis::v1::Category::Performance,
        "documentation" => analysis::v1::Category::Documentation,
        "type_check" => analysis::v1::Category::TypeCheck,
        "style" => analysis::v1::Category::Style,
        "anti_pattern" => analysis::v1::Category::AntiPattern,
        "accessibility" => analysis::v1::Category::Accessibility,
        _ => analysis::v1::Category::Unspecified,
    }
}

impl analysis::v1::Level {
    pub fn as_lower_str_name(&self) -> &'static str {
        match self {
            analysis::v1::Level::Unspecified => "unspecified",
            analysis::v1::Level::Note => "note",
            analysis::v1::Level::Fmt => "fmt",
            analysis::v1::Level::Low => "low",
            analysis::v1::Level::Medium => "medium",
            analysis::v1::Level::High => "high",
        }
    }
}

impl clap::ValueEnum for analysis::v1::Level {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            analysis::v1::Level::Unspecified,
            analysis::v1::Level::Fmt,
            analysis::v1::Level::Low,
            analysis::v1::Level::Medium,
            analysis::v1::Level::High,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(clap::builder::PossibleValue::new(self.as_lower_str_name()))
    }
}

pub fn language_enum_from_name(name: &str) -> analysis::v1::Language {
    match name {
        "php" => analysis::v1::Language::Php,
        "kotlin" => analysis::v1::Language::Kotlin,
        "go" => analysis::v1::Language::Go,
        "java" => analysis::v1::Language::Java,
        "javascript" => analysis::v1::Language::Javascript,
        "jsx" => analysis::v1::Language::Jsx,
        "python" => analysis::v1::Language::Python,
        "ruby" => analysis::v1::Language::Ruby,
        "rust" => analysis::v1::Language::Rust,
        "tsx" => analysis::v1::Language::Tsx,
        "typescript" => analysis::v1::Language::Typescript,
        _ => panic!("Unrecognized language name: {}", name),
    }
}

pub fn calculate_effort_minutes(
    value_delta: u32,
    base_minutes: u32,
    minutes_per_delta: u32,
) -> u32 {
    base_minutes + (value_delta * minutes_per_delta)
}

impl Add for analysis::v1::Stats {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            name: self.name,
            fully_qualified_name: self.fully_qualified_name,
            workspace_id: self.workspace_id,
            project_id: self.project_id,
            kind: self.kind,
            path: self.path,
            language: self.language,
            analyzed_at: self.analyzed_at,
            files: Some(self.files.unwrap_or(0) + other.files.unwrap_or(0)),
            functions: Some(self.functions.unwrap_or(0) + other.functions.unwrap_or(0)),
            classes: Some(self.classes.unwrap_or(0) + other.classes.unwrap_or(0)),
            fields: Some(self.fields.unwrap_or(0) + other.fields.unwrap_or(0)),
            lines: Some(self.lines.unwrap_or(0) + other.lines.unwrap_or(0)),
            code_lines: Some(self.code_lines.unwrap_or(0) + other.code_lines.unwrap_or(0)),
            blank_lines: Some(self.blank_lines.unwrap_or(0) + other.blank_lines.unwrap_or(0)),
            comment_lines: Some(self.comment_lines.unwrap_or(0) + other.comment_lines.unwrap_or(0)),
            complexity: Some(self.complexity.unwrap_or(0) + other.complexity.unwrap_or(0)),
            cyclomatic: Some(self.cyclomatic.unwrap_or(0) + other.cyclomatic.unwrap_or(0)),
            lcom4: Some(self.lcom4.unwrap_or(0) + other.lcom4.unwrap_or(0)),
            reference: self.reference,
            build_id: self.build_id,
            commit_sha: self.commit_sha,
            pull_request_number: self.pull_request_number,
            tracked_branch_id: self.tracked_branch_id,
        }
    }
}

impl tests::v1::CoverageSummary {
    pub fn percent(&self) -> f64 {
        self.covered as f64 / (self.covered + self.missed) as f64 * 100.0
    }
}

impl Add for tests::v1::CoverageSummary {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            covered: self.covered + other.covered,
            missed: self.missed + other.missed,
            omit: self.omit + other.omit,
            total: self.total + other.total,
        }
    }
}

impl AddAssign for tests::v1::CoverageSummary {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}
