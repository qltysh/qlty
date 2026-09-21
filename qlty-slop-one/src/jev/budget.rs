//! An in-memory spending ledger for one invocation. Every attempt reserves
//! the maximum request cost first and replaces it with the confirmed cost
//! once the provider reports usage; a failed or unconfirmed attempt keeps its
//! full reservation.

use serde_json::Value;

use super::Usage;
use crate::error::{Error, Result};
use crate::fsum::fsum;

/// TypeSafe's price for Jev input, in dollars per million tokens.
pub const PRICE_PER_MILLION_USD: f64 = 0.042;

/// The most input tokens one Jev request can carry.
pub const MAX_REQUEST_TOKENS: u64 = 64000;

/// The dollar cost of a request at the token ceiling.
fn maximum_request_cost() -> f64 {
    MAX_REQUEST_TOKENS as f64 * PRICE_PER_MILLION_USD / 1e6
}

fn token_cost(tokens: u64) -> f64 {
    tokens as f64 * PRICE_PER_MILLION_USD / 1e6
}

#[derive(Clone, Debug)]
struct Call {
    amount_usd: f64,
    tokens: Option<u64>,
}

/// One reserved request. Consumed by [`Budget::complete`].
#[derive(Debug)]
#[must_use = "an unfinished reservation keeps the maximum request cost"]
pub struct Reservation {
    index: usize,
}

/// The per-invocation API budget.
#[derive(Clone, Debug)]
pub struct Budget {
    limit_usd: f64,
    calls: Vec<Call>,
}

impl Budget {
    pub fn try_new(limit_usd: f64) -> Result<Self> {
        if !limit_usd.is_finite() || limit_usd < 0.0 {
            return Err(Error::Jev(
                "API budget must be a finite nonnegative dollar amount.".to_owned(),
            ));
        }
        Ok(Self {
            limit_usd,
            calls: Vec::new(),
        })
    }

    /// Reserves the maximum request cost, or fails when it would exceed the
    /// limit.
    pub fn reserve(&mut self) -> Result<Reservation> {
        let amount = maximum_request_cost();
        let used = fsum(self.calls.iter().map(|call| call.amount_usd));
        if used + amount > self.limit_usd {
            return Err(Error::BudgetExhausted);
        }
        self.calls.push(Call {
            amount_usd: amount,
            tokens: None,
        });
        Ok(Reservation {
            index: self.calls.len() - 1,
        })
    }

    /// Replaces a reservation with the cost of `input_tokens`, the raw
    /// provider usage value. Anything but an integer in
    /// `0..=MAX_REQUEST_TOKENS` keeps the full reservation and fails.
    pub fn complete(
        &mut self,
        reservation: Reservation,
        input_tokens: Option<&Value>,
    ) -> Result<()> {
        let tokens = input_tokens
            .and_then(Value::as_u64)
            .filter(|tokens| *tokens <= MAX_REQUEST_TOKENS)
            .ok_or_else(|| {
                Error::Jev(
                    "Jev returned invalid usage; the maximum request cost remains reserved."
                        .to_owned(),
                )
            })?;
        let call = &mut self.calls[reservation.index];
        call.amount_usd = token_cost(tokens);
        call.tokens = Some(tokens);
        Ok(())
    }

    pub fn summary(&self) -> Usage {
        Usage {
            requests: self.calls.len() as u64,
            cost_upper_bound_usd: fsum(self.calls.iter().map(|call| call.amount_usd)),
            input_tokens: self.calls.iter().filter_map(|call| call.tokens).sum(),
            unconfirmed_requests: self
                .calls
                .iter()
                .filter(|call| call.tokens.is_none())
                .count() as u64,
            budget_usd: self.limit_usd,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn reserve_and_complete(usage: Option<Value>) -> (Budget, Result<()>) {
        let mut budget = Budget::try_new(0.003).unwrap();
        let reservation = budget.reserve().unwrap();
        let outcome = budget.complete(reservation, usage.as_ref());
        (budget, outcome)
    }

    #[test]
    fn rejects_negative_and_non_finite_limits() {
        assert!(Budget::try_new(-0.01).is_err());
        assert!(Budget::try_new(f64::NAN).is_err());
        assert!(Budget::try_new(f64::INFINITY).is_err());
    }

    #[test]
    fn reservation_costs_the_token_ceiling() {
        let mut budget = Budget::try_new(1.0).unwrap();
        let _reservation = budget.reserve().unwrap();
        assert_eq!(budget.summary().cost_upper_bound_usd, 0.002688);
    }

    #[test]
    fn confirmed_usage_replaces_the_reservation() {
        let (budget, outcome) = reserve_and_complete(Some(json!(6364)));
        outcome.unwrap();
        let summary = budget.summary();
        assert_eq!(summary.requests, 1);
        assert_eq!(summary.input_tokens, 6364);
        assert_eq!(summary.unconfirmed_requests, 0);
        assert_eq!(summary.cost_upper_bound_usd, 6364.0 * 0.042 / 1e6);
    }

    #[test]
    fn missing_usage_keeps_the_reservation() {
        let (budget, outcome) = reserve_and_complete(None);
        assert!(matches!(outcome, Err(Error::Jev(_))));
        assert_eq!(budget.summary().unconfirmed_requests, 1);
        assert_eq!(budget.summary().cost_upper_bound_usd, 0.002688);
    }

    #[test]
    fn null_usage_keeps_the_reservation() {
        let (budget, outcome) = reserve_and_complete(Some(Value::Null));
        assert!(outcome.is_err());
        assert_eq!(budget.summary().unconfirmed_requests, 1);
    }

    #[test]
    fn negative_usage_keeps_the_reservation() {
        let (budget, outcome) = reserve_and_complete(Some(json!(-1)));
        assert!(outcome.is_err());
        assert_eq!(budget.summary().unconfirmed_requests, 1);
    }

    #[test]
    fn oversized_usage_keeps_the_reservation() {
        let (budget, outcome) = reserve_and_complete(Some(json!(64001)));
        assert!(outcome.is_err());
        assert_eq!(budget.summary().unconfirmed_requests, 1);
    }

    #[test]
    fn float_usage_keeps_the_reservation() {
        let (budget, outcome) = reserve_and_complete(Some(json!(100.0)));
        assert!(outcome.is_err());
        assert_eq!(budget.summary().unconfirmed_requests, 1);
    }

    #[test]
    fn boolean_usage_keeps_the_reservation() {
        let (budget, outcome) = reserve_and_complete(Some(json!(true)));
        assert!(outcome.is_err());
        assert_eq!(budget.summary().unconfirmed_requests, 1);
    }

    #[test]
    fn unconfirmed_reservation_exhausts_a_tight_budget() {
        let (mut budget, _outcome) = reserve_and_complete(None);
        assert!(matches!(budget.reserve(), Err(Error::BudgetExhausted)));
        assert_eq!(budget.summary().requests, 1);
    }

    #[test]
    fn zero_budget_refuses_the_first_request() {
        let mut budget = Budget::try_new(0.0).unwrap();
        assert!(matches!(budget.reserve(), Err(Error::BudgetExhausted)));
    }

    #[test]
    fn summary_reports_the_limit() {
        assert_eq!(Budget::try_new(0.5).unwrap().summary().budget_usd, 0.5);
    }
}
