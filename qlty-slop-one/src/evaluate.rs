//! Orchestrate one file's evaluation.

use std::path::{Path, PathBuf};

use serde_json::json;

use crate::error::{Error, Result};
use crate::jev::{Jev, JevProvider, Usage};
use crate::measure::Measurement;
use crate::model::{Model, ModelInfo, Observed};
use crate::report::{Evaluated, FileOutcome, Skipped};
use crate::scope::Exclusion;
use crate::scope::Scope;
use crate::source::{source_text, PreparedSource};

/// Options for one invocation.
#[derive(Clone, Debug)]
pub struct Options {
    pub cache_dir: PathBuf,
    pub budget_usd: f64,
    pub provider: JevProvider,
    pub offline: bool,
    pub include_tests: bool,
    pub include_excluded: bool,
}

pub struct Evaluator {
    model: &'static Model,
    jev: Jev,
    options: Options,
}

impl Evaluator {
    pub fn new(options: Options) -> Result<Self> {
        let model = Model::shared()?;
        let jev = Jev::try_new(
            options.cache_dir.clone(),
            options.budget_usd,
            options.provider,
            options.offline,
        )?;
        Ok(Self {
            model,
            jev,
            options,
        })
    }

    pub fn model_info(&self) -> ModelInfo {
        self.model.info()
    }

    pub fn usage(&self) -> Usage {
        self.jev.usage()
    }

    /// Evaluate one file. Analysis failures become `FileOutcome::Failed`.
    pub fn evaluate(&self, path: &Path) -> FileOutcome {
        match self.evaluate_file(path) {
            Ok(outcome) => outcome,
            Err(error) => FileOutcome::failed(path, &error),
        }
    }

    fn evaluate_file(&self, path: &Path) -> Result<FileOutcome> {
        if !path.is_file() {
            return Err(Error::NotASourceFile(path.to_path_buf()));
        }
        let include_tests = self.options.include_tests || self.options.include_excluded;
        let include_excluded = self.options.include_excluded;
        let scope = Scope::try_new(path)?;
        if !include_tests {
            if let Some(exclusion) = scope.test_exclusion() {
                return Ok(FileOutcome::Skipped(Skipped {
                    path: display(path),
                    language: None,
                    passed: None,
                    score: None,
                    skipped: true,
                    reason: format!(
                        "Test file excluded: {} matches {}. Use --include-tests to evaluate it.",
                        kebab(&exclusion.kind())?,
                        python_repr(exclusion.matched())
                    ),
                    source_scope: json!({"policy": "excluded-test-file", "exclusion": exclusion}),
                }));
            }
        }
        let mut exclusion = if include_excluded {
            None
        } else {
            scope.source_exclusion(None)?
        };
        let mut text = None;
        if exclusion.is_none() {
            let contents = source_text(path)?;
            if !include_excluded {
                exclusion = scope.source_exclusion(Some(&contents))?;
            }
            text = Some(contents);
        }
        if let Some(exclusion) = exclusion {
            return Ok(FileOutcome::Skipped(Skipped {
                path: display(path),
                language: None,
                passed: None,
                score: None,
                skipped: true,
                reason: format!(
                    "{} file excluded: {} matches {}. Use --include-excluded to evaluate it.",
                    capitalize(&category_name(&exclusion)?),
                    kebab(&exclusion.kind())?,
                    python_repr(exclusion.matched())
                ),
                source_scope: json!({"policy": "excluded-source-file", "exclusion": exclusion}),
            }));
        }
        let text = text.expect("source text is read whenever no exclusion applied");
        let suffix = suffix_of(path);
        let prepared = PreparedSource::prepare(&text, &suffix, include_tests)?;
        if !prepared.has_code() {
            return Ok(FileOutcome::Skipped(Skipped {
                path: display(path),
                language: Some("Rust"),
                passed: None,
                score: None,
                skipped: true,
                reason: "No code remains after excluding inline test modules.".to_string(),
                source_scope: serde_json::to_value(prepared.info())?,
            }));
        }
        let measurement = Measurement::analyze(prepared.text(), &suffix)?;
        let jev = self.jev.extract(prepared.text(), measurement.language)?;
        let mut values = measurement.values.clone();
        values.extend_from_slice(&jev.values);
        let mut explanation = self.model.explain(&values, &measurement.summaries, &jev)?;
        for factor in &mut explanation.factors {
            match &mut factor.observed {
                Observed::Qlty(summary) => prepared.restore_spans(&mut summary.locations)?,
                Observed::Jev(observed) => prepared.restore_excerpts(&mut observed.excerpts)?,
            }
        }
        Ok(FileOutcome::Evaluated(Evaluated {
            path: display(path),
            language: measurement.language.display_name(),
            source_scope: serde_json::to_value(prepared.info())?,
            explanation,
        }))
    }
}

/// The kebab-case name serde gives an enum, as the Python dicts spelled it.
fn kebab(value: &impl serde::Serialize) -> Result<String> {
    match serde_json::to_value(value)? {
        serde_json::Value::String(text) => Ok(text),
        other => Ok(other.to_string()),
    }
}

fn category_name(exclusion: &Exclusion) -> Result<String> {
    match exclusion.category() {
        Some(category) => kebab(&category),
        None => Ok(String::new()),
    }
}

fn display(path: &Path) -> String {
    path.display().to_string()
}

/// `pathlib.Path(path).suffix`: the final extension including its dot, or "".
fn suffix_of(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    match name.rfind('.') {
        Some(index) if index > 0 && index + 1 < name.len() => name[index..].to_string(),
        _ => String::new(),
    }
}

/// Python `str.capitalize()` for the ASCII category names.
fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first
            .to_uppercase()
            .chain(chars.flat_map(char::to_lowercase))
            .collect(),
        None => String::new(),
    }
}

/// Python `repr()` of a string, as slopdetect's `{matched!r}` produced it.
fn python_repr(text: &str) -> String {
    let quote = if text.contains('\'') && !text.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::new();
    out.push(quote);
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\x{:02x}", c as u32))
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffix_is_the_final_extension_with_its_dot() {
        assert_eq!(suffix_of(Path::new("src/Input.d.ts")), ".ts");
        assert_eq!(suffix_of(Path::new("Makefile")), "");
        assert_eq!(suffix_of(Path::new(".bashrc")), "");
    }

    #[test]
    fn capitalize_matches_python() {
        assert_eq!(capitalize("generated"), "Generated");
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn python_repr_quotes_like_python() {
        assert_eq!(python_repr("tests"), "'tests'");
        assert_eq!(python_repr("it's"), "\"it's\"");
        assert_eq!(python_repr("a\\b"), "'a\\\\b'");
    }
}
