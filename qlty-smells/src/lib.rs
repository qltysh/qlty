//! Code smells expose numeric `properties.threshold` and `properties.actual` in
//! both JSON and SARIF. The threshold is the effective rule configuration;
//! actual is the full measurement in the same units. Findings are inclusive of
//! the threshold. Existing `value`, `value_delta`, and locations remain unchanged.
//!
//! Boolean logic reports maximum logical-operator depth across the enclosing
//! expression, including all findings within that expression. Nested control
//! flow reports the deepest control level under each finding, including its
//! enclosing control levels. Both follow the existing language visitor semantics.
//! Parameters and returns use their respective counts; file and function
//! complexity use cognitive complexity. Duplication uses the representative
//! block's line count used by the detector (the existing `value`); `mass` remains
//! a separate syntax-tree measurement.

pub mod duplication;
pub mod metrics;
pub mod structure;
