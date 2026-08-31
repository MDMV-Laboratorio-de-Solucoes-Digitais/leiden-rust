//! Algorithm parameters for a single Leiden run.

use crate::error::LeidenError;

/// User-supplied parameters for one algorithm run.
///
/// `gamma` defaults to `1.0`; `gamma <= 0.0` is rejected by `validate`
/// (FR-003). `iteration_cap` defaults to `10` (FR-003a, matching Traag et al.
/// 2019). `seed` is `None` for the v1 deterministic variant (FR-004): v1
/// does not consume `seed` for any algorithm decision; tie-breaks use the
/// lowest internal node id. `seed` is carried through `RunResult.seed`
/// verbatim for forward compatibility only (see `spec.md` Assumptions
/// "Algorithm variant" and Clarifications 2026-08-30 (seed field)). Stochastic
/// variants, if added later, MUST be gated behind a Cargo feature flag AND
/// require a Constitution amendment.
#[derive(Debug, Clone)]
pub struct LeidenParameters {
    /// Resolution parameter controlling community granularity.
    pub gamma: f64,
    /// Optional randomness seed; not consumed in v1, round-tripped verbatim.
    pub seed: Option<u64>,
    /// Maximum number of outer-loop iterations.
    pub iteration_cap: u32,
}

impl LeidenParameters {
    /// Default resolution parameter value.
    #[must_use]
    pub const fn default_gamma() -> f64 {
        1.0
    }

    /// Default iteration cap value.
    #[must_use]
    pub const fn default_iteration_cap() -> u32 {
        10
    }

    /// Validate parameters, returning a typed error on violation.
    ///
    /// # Errors
    ///
    /// Returns `LeidenError::InvalidGamma` when `gamma` is non-finite or `<= 0`.
    /// Returns `LeidenError::InvalidIterationCap` when `iteration_cap < 1`.
    pub fn validate(&self) -> Result<(), LeidenError> {
        if !self.gamma.is_finite() || self.gamma <= 0.0 {
            return Err(LeidenError::InvalidGamma(self.gamma));
        }
        if self.iteration_cap < 1 {
            return Err(LeidenError::InvalidIterationCap(self.iteration_cap));
        }
        Ok(())
    }
}

impl Default for LeidenParameters {
    fn default() -> Self {
        Self {
            gamma: Self::default_gamma(),
            seed: None,
            iteration_cap: Self::default_iteration_cap(),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test code: validates error paths; production-code expect ban does not apply per Constitution §III"
)]
mod tests {
    use super::LeidenParameters;

    #[test]
    fn default_values_match_spec() {
        let params = LeidenParameters::default();
        assert!((params.gamma - 1.0).abs() < f64::EPSILON);
        assert!(params.seed.is_none());
        assert_eq!(params.iteration_cap, 10);
    }

    #[test]
    fn validate_accepts_defaults() {
        let params = LeidenParameters::default();
        assert!(params.validate().is_ok());
    }

    #[test]
    fn validate_rejects_gamma_zero() {
        let params = LeidenParameters {
            gamma: 0.0,
            seed: None,
            iteration_cap: 10,
        };
        let err = params.validate().expect_err("gamma == 0 must be rejected");
        assert!(matches!(
            err,
            crate::error::LeidenError::InvalidGamma(v) if v.abs() < f64::EPSILON
        ));
    }

    #[test]
    fn validate_rejects_gamma_negative() {
        let params = LeidenParameters {
            gamma: -0.5,
            seed: None,
            iteration_cap: 10,
        };
        let err = params.validate().expect_err("gamma < 0 must be rejected");
        assert!(matches!(
            err,
            crate::error::LeidenError::InvalidGamma(v) if (v - -0.5).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn validate_rejects_gamma_nan() {
        let params = LeidenParameters {
            gamma: f64::NAN,
            seed: None,
            iteration_cap: 10,
        };
        let err = params.validate().expect_err("gamma NaN must be rejected");
        assert!(matches!(err, crate::error::LeidenError::InvalidGamma(v) if v.is_nan()));
    }

    #[test]
    fn validate_rejects_gamma_infinite() {
        let params = LeidenParameters {
            gamma: f64::INFINITY,
            seed: None,
            iteration_cap: 10,
        };
        let err = params.validate().expect_err("gamma inf must be rejected");
        assert!(matches!(err, crate::error::LeidenError::InvalidGamma(v) if v.is_infinite()));
    }

    #[test]
    fn validate_rejects_iteration_cap_zero() {
        let params = LeidenParameters {
            gamma: 1.0,
            seed: None,
            iteration_cap: 0,
        };
        let err = params
            .validate()
            .expect_err("iteration_cap == 0 must be rejected");
        assert!(matches!(
            err,
            crate::error::LeidenError::InvalidIterationCap(0)
        ));
    }

    #[test]
    fn validate_accepts_iteration_cap_one() {
        let params = LeidenParameters {
            gamma: 1.0,
            seed: None,
            iteration_cap: 1,
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn validate_accepts_custom_gamma() {
        let params = LeidenParameters {
            gamma: 2.5,
            seed: Some(42),
            iteration_cap: 20,
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    #[expect(
        clippy::redundant_clone,
        clippy::unnecessary_struct_initialization,
        reason = "explicitly testing that Clone produces an equal value; struct rebuild is the comparison oracle"
    )]
    fn clone_preserves_values() {
        let params = LeidenParameters {
            gamma: 1.5,
            seed: Some(99),
            iteration_cap: 5,
        };
        let other = LeidenParameters {
            gamma: params.gamma,
            seed: params.seed,
            iteration_cap: params.iteration_cap,
        };
        let cloned_via_clone = params.clone();
        assert!((other.gamma - cloned_via_clone.gamma).abs() < f64::EPSILON);
        assert_eq!(other.seed, cloned_via_clone.seed);
        assert_eq!(other.iteration_cap, cloned_via_clone.iteration_cap);
        assert!((other.gamma - 1.5).abs() < f64::EPSILON);
        assert_eq!(other.seed, Some(99));
        assert_eq!(other.iteration_cap, 5);
    }
}
