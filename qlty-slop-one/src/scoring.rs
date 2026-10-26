//! The versioned 1–10 score scale with a fixed decision boundary at 5.
//!
//! The trained risk and its cutoff remain unchanged. A piecewise linear
//! mapping preserves the endpoints and maps the old score cutoff to 5 exactly.
//! Every expression keeps slopdetect's operand order so results are
//! bit-identical.

use crate::error::{Error, Result};

pub const SCORE_SCALE: &str = "threshold-centered-1-10-v1";
pub const SCORE_CUTOFF: f64 = 5.0;

/// Maps risks and legacy `10 - 9 * risk` scores onto the 1–10 scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScoreScale {
    risk_threshold: f64,
    legacy_cutoff: f64,
    lower_slope: f64,
    upper_slope: f64,
}

impl ScoreScale {
    pub fn try_new(risk_threshold: f64) -> Result<Self> {
        if !risk_threshold.is_finite() || risk_threshold <= 0.0 || risk_threshold >= 1.0 {
            return Err(Error::InvalidModel(
                "Score scaling requires a risk cutoff strictly between 0 and 1".to_string(),
            ));
        }
        let legacy_cutoff = 10.0 - 9.0 * risk_threshold;
        Ok(Self {
            risk_threshold,
            legacy_cutoff,
            lower_slope: 4.0 / (legacy_cutoff - 1.0),
            upper_slope: 5.0 / (10.0 - legacy_cutoff),
        })
    }

    pub fn risk_threshold(&self) -> f64 {
        self.risk_threshold
    }

    /// The legacy score (`10 - 9 * risk`) at the risk cutoff.
    pub fn legacy_cutoff(&self) -> f64 {
        self.legacy_cutoff
    }

    pub fn from_legacy(&self, score: f64) -> f64 {
        if score <= self.legacy_cutoff {
            return at_most_cutoff(1.0 + (score - 1.0) * self.lower_slope);
        }
        // Keep a representable passing value above the boundary even at one ULP.
        above_cutoff(5.0 + (score - self.legacy_cutoff) * self.upper_slope)
    }

    pub fn from_risk(&self, risk: f64) -> f64 {
        if risk >= self.risk_threshold {
            return at_most_cutoff(1.0 + 4.0 * (1.0 - risk) / (1.0 - self.risk_threshold));
        }
        above_cutoff(10.0 - 5.0 * risk / self.risk_threshold)
    }

    /// Positive multiplier for an additive explanation between two old scores.
    ///
    /// This is an endpoint rescaling of the original model attribution, not a
    /// new claim about the causal effect of any individual feature.
    pub fn secant(&self, before: f64, after: f64) -> f64 {
        let (low, high) = if after < before {
            (after, before)
        } else {
            (before, after)
        };
        if high <= self.legacy_cutoff {
            return self.lower_slope;
        }
        if low >= self.legacy_cutoff {
            return self.upper_slope;
        }
        let upper_share = (high - self.legacy_cutoff) / (high - low);
        self.lower_slope * (1.0 - upper_share) + self.upper_slope * upper_share
    }
}

/// Python `min(SCORE_CUTOFF, value)`: the first argument wins unless the second is smaller.
fn at_most_cutoff(value: f64) -> f64 {
    if value < SCORE_CUTOFF {
        value
    } else {
        SCORE_CUTOFF
    }
}

/// Python `max(nextafter(SCORE_CUTOFF, 10), value)`: the first argument wins unless the second is larger.
fn above_cutoff(value: f64) -> f64 {
    let floor = SCORE_CUTOFF.next_up();
    if value > floor {
        value
    } else {
        floor
    }
}

/// Formats a score with `decimals` places without rounding a failing score up
/// to the visible passing boundary. `None` renders as an em dash.
pub fn format_score(
    value: Option<f64>,
    decimals: usize,
    passed: Option<bool>,
    threshold: f64,
) -> String {
    let Some(value) = value else {
        return "—".to_string();
    };
    let passed = passed.unwrap_or(value > threshold);
    let text = format!("{value:.decimals$}");
    let rounded: f64 = text
        .parse()
        .expect("a formatted f64 should parse back as f64");
    if !passed && rounded >= threshold {
        let below = threshold - 10f64.powf(-(decimals as f64));
        return format!("{below:.decimals$}");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    const THRESHOLD: f64 = 0.6073781128742494;

    fn assert_anchors_ordering_and_classification(threshold: f64) {
        let scale = ScoreScale::try_new(threshold).unwrap();
        assert_eq!(scale.from_risk(0.0), 10.0);
        assert_eq!(scale.from_risk(threshold), 5.0);
        assert_eq!(scale.from_risk(1.0), 1.0);
        let mut risks: Vec<f64> = (0..=1000).map(|i| f64::from(i) / 1000.0).collect();
        risks.extend([threshold, threshold.next_down(), threshold.next_up()]);
        risks.sort_by(f64::total_cmp);
        let scores: Vec<f64> = risks.iter().map(|risk| scale.from_risk(*risk)).collect();
        assert!(scores.windows(2).all(|pair| pair[0] >= pair[1]));
        assert!(risks
            .iter()
            .zip(&scores)
            .all(|(risk, score)| (*score > 5.0) == (*risk < threshold)));
        assert_eq!(scale.from_legacy(scale.legacy_cutoff()), 5.0);
        assert!(scale.from_legacy(scale.legacy_cutoff().next_up()) > 5.0);
    }

    #[test]
    fn anchors_ordering_and_classification_hold_at_low_threshold() {
        assert_anchors_ordering_and_classification(0.1);
    }

    #[test]
    fn anchors_ordering_and_classification_hold_at_half() {
        assert_anchors_ordering_and_classification(0.5);
    }

    #[test]
    fn anchors_ordering_and_classification_hold_at_model_threshold() {
        assert_anchors_ordering_and_classification(0.607184960734721);
    }

    #[test]
    fn anchors_ordering_and_classification_hold_at_high_threshold() {
        assert_anchors_ordering_and_classification(0.9);
    }

    fn assert_additive_rescaling_reconciles(before: f64, after: f64) {
        let scale = ScoreScale::try_new(0.607184960734721).unwrap();
        let rescaled = (after - before) * scale.secant(before, after);
        let expected = scale.from_legacy(after) - scale.from_legacy(before);
        assert!((rescaled - expected).abs() <= 1e-13);
    }

    #[test]
    fn additive_rescaling_reconciles_across_the_cutoff() {
        assert_additive_rescaling_reconciles(1.0, 10.0);
        assert_additive_rescaling_reconciles(3.0, 8.0);
        assert_additive_rescaling_reconciles(8.0, 3.0);
    }

    #[test]
    fn additive_rescaling_reconciles_on_one_side() {
        assert_additive_rescaling_reconciles(2.0, 4.0);
        assert_additive_rescaling_reconciles(6.0, 8.0);
        assert_additive_rescaling_reconciles(4.0, 4.0);
    }

    #[test]
    fn rejects_cutoffs_outside_the_open_unit_interval() {
        assert!(ScoreScale::try_new(0.0).is_err());
        assert!(ScoreScale::try_new(1.0).is_err());
        assert!(ScoreScale::try_new(f64::NAN).is_err());
        assert!(ScoreScale::try_new(2.0).is_err());
    }

    #[test]
    fn derived_slopes_match_python() {
        let scale = ScoreScale::try_new(THRESHOLD).unwrap();
        assert_eq!(scale.legacy_cutoff(), 4.533596984131756);
        assert_eq!(scale.lower_slope, 1.131991004622969);
        assert_eq!(scale.upper_slope, 0.9146782601805359);
    }

    #[test]
    fn from_risk_matches_python_bit_for_bit() {
        let scale = ScoreScale::try_new(THRESHOLD).unwrap();
        assert_eq!(scale.from_risk(0.1), 9.176789565837518);
        assert_eq!(scale.from_risk(0.3553892033050768), 7.074398996505691);
        assert_eq!(scale.from_risk(THRESHOLD.next_down()), 5.000000000000001);
        assert_eq!(scale.from_risk(THRESHOLD.next_up()), 4.999999999999998);
        assert_eq!(scale.from_risk(0.7), 4.0563757124820174);
    }

    #[test]
    fn from_legacy_matches_python_bit_for_bit() {
        let scale = ScoreScale::try_new(THRESHOLD).unwrap();
        assert_eq!(scale.from_legacy(3.0), 3.263982009245938);
        assert_eq!(scale.from_legacy(4.53359698413215), 5.000000000000361);
        assert_eq!(scale.from_legacy(5.0), 5.42660869909732);
        assert_eq!(scale.from_legacy(8.0), 8.170643479638928);
    }

    #[test]
    fn secant_matches_python_bit_for_bit() {
        let scale = ScoreScale::try_new(THRESHOLD).unwrap();
        assert_eq!(scale.secant(1.0, 10.0), 1.0);
        assert_eq!(scale.secant(3.0, 8.0), 0.981332294078598);
        assert_eq!(scale.secant(8.0, 3.0), 0.981332294078598);
        assert_eq!(scale.secant(2.0, 4.0), 1.131991004622969);
        assert_eq!(scale.secant(6.0, 8.0), 0.9146782601805359);
        assert_eq!(scale.secant(4.5342, 7.1), 0.9146782601805359);
    }

    #[test]
    fn display_rounding_does_not_disguise_a_failing_score() {
        assert_eq!(format_score(Some(4.999), 1, None, SCORE_CUTOFF), "4.9");
        assert_eq!(format_score(Some(5.0), 1, None, SCORE_CUTOFF), "4.9");
        assert_eq!(format_score(Some(5.001), 1, None, SCORE_CUTOFF), "5.0");
        assert_eq!(format_score(Some(4.999), 2, None, SCORE_CUTOFF), "4.99");
        assert_eq!(format_score(Some(4.94), 1, None, SCORE_CUTOFF), "4.9");
        assert_eq!(format_score(None, 1, None, SCORE_CUTOFF), "—");
    }

    #[test]
    fn display_rounding_honors_an_explicit_pass_flag() {
        assert_eq!(format_score(Some(5.0), 1, Some(true), SCORE_CUTOFF), "5.0");
        assert_eq!(format_score(Some(7.0), 1, Some(false), SCORE_CUTOFF), "4.9");
    }

    #[test]
    fn display_rounding_is_half_even_like_python() {
        assert_eq!(format_score(Some(2.5), 0, None, SCORE_CUTOFF), "2");
        assert_eq!(format_score(Some(3.5), 0, None, SCORE_CUTOFF), "4");
        assert_eq!(format_score(Some(5.0), 0, None, SCORE_CUTOFF), "4");
        assert_eq!(format_score(Some(9.96), 1, None, SCORE_CUTOFF), "10.0");
    }
}
