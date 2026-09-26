//! SlopOne: explainable source-file maintainability scoring with Qlty smells
//! and seven Jev assessments, ported from `lithoscomputer/slopdetect`.
//!
//! The pipeline for one file is: select the source (`scope`, `source`),
//! measure eight Qlty smells in-process (`measure`), ask Jev seven questions
//! (`jev`), average 640 logistic components (`model`), map risk to a 1–10
//! score (`scoring`), and render the result (`report`).

mod compare;
pub mod content;
mod error;
mod evaluate;
pub mod features;
pub mod fsum;
mod grammar;
pub mod jev;
pub mod language;
pub mod measure;
pub mod model;
mod prompt;
pub mod pylines;
mod report;
pub mod scope;
pub mod scoring;
pub mod source;

pub use compare::{
    Base, BaseVersion, Basis, Change, ComparedFile, Comparison, ComparisonSummary, Contents,
    Decline, FactorImpact, FactorImpacts, Verdict,
};
pub use content::{
    content_key, score_content, score_content_cached, ContentScore, MeasurementCache,
};
pub use error::Error;
pub use evaluate::{Evaluator, Options};
pub use jev::{JevProvider, Usage};
pub use model::{Model, ModelInfo};
pub use prompt::{render_comparisons, Framing, PromptMode, RevisionPair, Revisions, TextOptions};
pub use report::{render_summary, render_text, Document, FileOutcome, Summary};
