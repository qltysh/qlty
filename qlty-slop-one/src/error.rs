use std::path::PathBuf;

use thiserror::Error;

/// A file could not be evaluated. This is never a maintainability failure.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Not a source file: {0}")]
    NotASourceFile(PathBuf),
    #[error("Source file exceeds the 16 MiB limit: {0}")]
    SourceTooLarge(PathBuf),
    #[error("Source must be UTF-8: {0}")]
    SourceNotUtf8(PathBuf),
    #[error("Source is empty or contains binary data: {0}")]
    SourceEmptyOrBinary(PathBuf),
    #[error("Qlty could not identify the source language.")]
    UnknownLanguage,
    #[error("Qlty did not parse this source file. Its language may be unsupported.")]
    Unparsed,
    #[error("Qlty analysis failed: {0}")]
    Analysis(String),
    #[error("Qlty analysis failed: {0}")]
    Qlty(#[source] anyhow::Error),
    #[error("Cannot safely identify inline test modules: Rust parsing failed. Use --include-tests to analyze the original file.")]
    RustParse,
    #[error("Rust test-module exclusion did not preserve valid syntax.")]
    RustFilter,
    #[error("Analysis returned a source location outside the analyzed code.")]
    LocationOutOfRange,
    #[error("Invalid model artifact: {0}")]
    InvalidModel(String),
    #[error("Invalid model input features.")]
    InvalidInput,
    #[error("Model explanation did not reconcile with its score.")]
    Unreconciled,
    #[error("{0}")]
    Jev(String),
    #[error("API spending limit reached. Cached results are still available.")]
    BudgetExhausted,
    #[error("No cached Jev answers for this file; run once without --offline.")]
    Offline,
    #[error("Set {0} in the environment.")]
    MissingCredential(&'static str),
    #[error("{0}")]
    Cache(String),
    #[error("{0}")]
    Scope(String),
    #[error("Could not read repository source exclusions from git attributes.")]
    GitAttributes(#[source] git2::Error),
    #[error("Could not read qlty workspace patterns from {path}")]
    WorkspaceConfig {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("Invalid pattern {pattern:?} in {path}")]
    WorkspacePattern {
        path: PathBuf,
        pattern: String,
        #[source]
        source: globset::Error,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
