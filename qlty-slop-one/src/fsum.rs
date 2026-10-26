//! Exactly rounded summation, matching CPython's `math.fsum`.
//!
//! This is the Shewchuk partials algorithm as CPython 3.13 implements it in
//! `Modules/mathmodule.c` (`math_fsum_impl`), including the final half-even
//! rounding correction across partials that a plain Shewchuk sum lacks. The
//! only differences are in the non-finite paths, where Python raises: an
//! intermediate overflow returns the infinite partial sum, and `-inf + inf`
//! returns NaN.

/// The exactly rounded sum of `values`, bit for bit as `math.fsum` computes it.
pub fn fsum(values: impl IntoIterator<Item = f64>) -> f64 {
    let mut partials: Vec<f64> = Vec::new();
    let mut special_sum = 0.0;
    let mut inf_sum = 0.0;

    for value in values {
        let mut x = value;
        let mut kept = 0;
        for index in 0..partials.len() {
            let mut y = partials[index];
            if x.abs() < y.abs() {
                std::mem::swap(&mut x, &mut y);
            }
            let hi = x + y;
            let lo = y - (hi - x);
            if lo != 0.0 {
                partials[kept] = lo;
                kept += 1;
            }
            x = hi;
        }
        partials.truncate(kept);

        if x == 0.0 {
            continue;
        }
        if x.is_finite() {
            partials.push(x);
            continue;
        }
        if value.is_finite() {
            return x;
        }
        if value.is_infinite() {
            inf_sum += value;
        }
        special_sum += value;
        partials.clear();
    }

    if special_sum != 0.0 {
        return if inf_sum.is_nan() {
            f64::NAN
        } else {
            special_sum
        };
    }

    let Some(mut hi) = partials.pop() else {
        return 0.0;
    };
    let mut lo = 0.0;
    while let Some(y) = partials.pop() {
        let x = hi;
        hi = x + y;
        lo = y - (hi - x);
        if lo != 0.0 {
            break;
        }
    }
    if let Some(&below) = partials.last() {
        if (lo < 0.0 && below < 0.0) || (lo > 0.0 && below > 0.0) {
            let y = lo * 2.0;
            let x = hi + y;
            if y == x - hi {
                hi = x;
            }
        }
    }
    hi
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_sum_is_zero() {
        assert_eq!(fsum([]), 0.0);
    }

    #[test]
    fn rounds_half_even_across_partials() {
        assert_eq!(fsum([1e16, 1.0, 1e-16]), 1.0000000000000002e16);
    }

    #[test]
    fn half_even_correction_ignores_input_order() {
        assert_eq!(fsum([1.0, 1e16, 1e-16]), 1.0000000000000002e16);
    }

    #[test]
    fn rounds_odd_tie_upward() {
        assert_eq!(fsum([1e16, 3.0, 1e-16]), 1.0000000000000004e16);
    }

    #[test]
    fn rounds_negative_tie_half_even() {
        assert_eq!(fsum([-1e16, -1.0, -1e-16]), -1.0000000000000002e16);
    }

    #[test]
    fn correction_applies_with_two_small_partials() {
        assert_eq!(fsum([1e16, 1.0, 1e-16, 1e-16]), 1.0000000000000002e16);
    }

    #[test]
    fn recovers_values_lost_to_cancellation() {
        assert_eq!(fsum([1.0, 1e100, 1.0, -1e100]), 2.0);
    }

    #[test]
    fn differs_from_kahan_summation() {
        assert_eq!(
            fsum([1e100, 1.0, -1e100, 1e-100, 1e50, -1.0, -1e50]),
            1e-100
        );
    }

    #[test]
    fn ten_tenths_sum_to_one() {
        assert_eq!(fsum([0.1; 10]), 1.0);
    }

    #[test]
    fn many_small_values_are_exact() {
        let values = std::iter::repeat_n(1e-16, 100_000).chain([1.0]);
        assert_eq!(fsum(values), 1.00000000001);
    }

    #[test]
    fn alternating_reciprocals_cancel_exactly() {
        let values = (1..=1000)
            .flat_map(|i| [1.0 / f64::from(i), -1.0 / f64::from(i)])
            .chain([0.3]);
        assert_eq!(fsum(values), 0.3);
    }

    #[test]
    fn mixed_powers_of_two_match_python() {
        let values = (-60..60)
            .step_by(7)
            .map(|e| 2f64.powi(e))
            .chain((-60..60).step_by(11).map(|e| -(2f64.powi(e))));
        assert_eq!(fsum(values), 5.798733634139597e17);
    }

    #[test]
    fn random_wide_range_values_match_python() {
        let values = [
            -3.5233447033367526e-06,
            -2.10353007153653e-12,
            -8.551274266649145e+19,
            -8.117399161206348e+22,
            -8.840021504505864e+17,
            -5.706036383286767e-10,
            -1.3270863267522828e-11,
            -5.1867399974594996e+20,
            -1.5096162171497208e+21,
            -0.7523960777007087,
            2.612518314634741e+22,
            8.954178849140112e+21,
            1.7108284528077347e-12,
            9.525102111858403e-13,
            1.1332979587418516e-07,
            -4.207814273366475e-06,
            8.137177106428495e+20,
            -0.00038303635179613124,
            -7.938885751128173e+21,
            277826937.85236824,
            -8.051388480105332e-11,
            1.2873658626677329e+24,
            -5.880825743613469e+19,
            -144815.38866119424,
            -68796268320651.96,
            -2.768352881108674,
            5.887589630449824,
            -83628.99784084603,
            503930.0762290298,
            4588.905788784353,
            2.1791803807280725e-11,
            -763868443490.0758,
            -6700757.927128535,
            -6.960309306789904e+16,
            -1.5660329104651136e-11,
            5.291417324256262e+21,
            578188.3429807099,
            -31975527.56176089,
            1.8873975421003696e+22,
            5.937839516431888e-11,
            679.9355610250828,
            -5.180332516071107e-12,
            -87866.11448055606,
            29425770905533.758,
            -4308089358.117016,
            77408058.44761837,
            -954874143888822.9,
            -2.8907178091930796e+24,
            -7.658084110365363e-12,
            -5635.844503606411,
        ];
        assert_eq!(fsum(values), -1.6345799172544025e+24);
    }

    #[test]
    fn small_decimal_sums_match_python() {
        assert_eq!(fsum([0.1, 0.2, 0.3]), 0.6);
        assert_eq!(fsum([0.1, 0.2, -0.3]), 2.7755575615628914e-17);
    }

    #[test]
    fn negative_zero_inputs_sum_to_positive_zero() {
        assert!(fsum([-0.0, -0.0]).is_sign_positive());
    }

    #[test]
    fn infinity_dominates_finite_values() {
        assert_eq!(fsum([1.0, f64::INFINITY]), f64::INFINITY);
        assert_eq!(fsum([f64::INFINITY, -1.0, f64::INFINITY]), f64::INFINITY);
    }

    #[test]
    fn nan_input_gives_nan() {
        assert!(fsum([1.0, f64::NAN]).is_nan());
    }

    #[test]
    fn opposite_infinities_give_nan() {
        assert!(fsum([f64::INFINITY, f64::NEG_INFINITY]).is_nan());
    }

    #[test]
    fn intermediate_overflow_gives_infinity() {
        assert_eq!(fsum([1e308, 1e308, -1e308]), f64::INFINITY);
    }
}
