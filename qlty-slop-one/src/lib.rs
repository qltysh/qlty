//! SlopOne: explainable source-file maintainability scoring with Qlty smells
//! and seven Jev assessments, ported from `lithoscomputer/slopdetect`.
//!
//! The pipeline for one file is: select the source (`scope`, `source`),
//! measure eight Qlty smells in-process (`measure`), ask Jev seven questions
//! (`jev`), average 640 logistic components (`model`), map risk to a 1–10
//! score (`scoring`), and render the result (`report`).

mod error;
mod evaluate;
pub mod features;
pub mod fsum;
mod grammar;
pub mod jev;
pub mod language;
pub mod measure;
pub mod model;
mod pylines;
mod report;
pub mod scope;
pub mod scoring;
pub mod source;

pub use error::Error;
pub use evaluate::{Evaluator, Options};
pub use jev::{JevProvider, Usage};
pub use model::{Model, ModelInfo};
pub use report::{render_summary, render_text, Document, FileOutcome, Summary};
