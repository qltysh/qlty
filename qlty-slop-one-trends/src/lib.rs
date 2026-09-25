//! SlopOne trend reports: weekly and monthly code quality history for one
//! repository, with file-level drilldowns and score explanations.
//!
//! The pipeline is `periods` (calendar snapshot selection), `snapshot`
//! (freeze a run manifest from Git history), `scoring` (score each unique
//! content with SlopOne), `change` (group files across snapshots and attribute
//! the period's quality change), `export` (the report data contracts),
//! `explain` (per-row score explanations), and `render` (SVG chart and the
//! standalone HTML report).

pub mod change;
pub mod chart_data;
mod error;
pub mod explain;
pub mod export;
pub mod periods;
pub mod pipeline;
pub mod render;
pub mod report;
pub mod run;
pub mod scoring;
pub mod snapshot;

pub use error::Error;
