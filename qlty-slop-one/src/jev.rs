//! Language-neutral Jev assessments, cached by exact request content.
//!
//! A file is split into consecutive excerpts (`chunk`), each excerpt is sent
//! to Jev with the seven fixed questions (`client`), keyed on disk by
//! slopdetect's canonical digest of the request (`canonical`, `cache`),
//! validated into one value per question (`response`), charged against a
//! per-invocation budget (`budget`), and combined with byte-weighted means.

mod budget;
mod cache;
mod canonical;
mod chunk;
mod client;
mod response;

use std::collections::BTreeMap;

use serde::Serialize;

use crate::measure::LineSpan;

pub use budget::{Budget, Reservation, MAX_REQUEST_TOKENS, PRICE_PER_MILLION_USD};
pub use canonical::{canonical, digest, utf8_length};
pub use chunk::{chunks, MAX_CHUNK_BYTES};
pub use client::{aggregate, request_key, Jev, RequestState};
pub use response::{validate, Answers};

/// The Jev version every request names and every answer must come from.
pub const JEV_MODEL: &str = "jev-1.13.0";

/// Where Jev requests go. Exact parity with slopdetect is claimed for
/// TypeSafe only.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JevProvider {
    /// TypeSafe's own API, `typesafe/jev-1.13.0`, keyed by `TYPESAFE_API_KEY`.
    TypeSafe,
    /// The Vercel AI Gateway, `vercel/jev`, keyed by `AI_GATEWAY_API_KEY`.
    Vercel,
}

/// One excerpt's answer to one question.
#[derive(Clone, Debug, Serialize)]
pub struct Excerpt {
    pub start_line: u32,
    pub end_line: u32,
    pub value: f64,
    pub weight_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ranges: Option<Vec<LineSpan>>,
}

/// Per-invocation API accounting, in slopdetect's JSON shape.
#[derive(Clone, Debug, Serialize)]
pub struct Usage {
    pub requests: u64,
    pub cost_upper_bound_usd: f64,
    pub input_tokens: u64,
    pub unconfirmed_requests: u64,
    pub budget_usd: f64,
}

/// The seven Jev features and their evidence for one analyzed file.
#[derive(Clone, Debug)]
pub struct JevFeatures {
    /// Byte-weighted means in `features::questions()` order.
    pub values: Vec<f64>,
    pub means: BTreeMap<String, f64>,
    pub assessments: BTreeMap<String, Vec<Excerpt>>,
    pub model: String,
    pub excerpts: usize,
}
