//! `RunResult` definition and termination semantics.

// ref: Traag 2019 §3 — Algorithm termination conditions and partition result bundle.

use crate::events::{TerminationReason, ThreadingPolicy};
use crate::graph::NodeId;

/// The bundle returned from a single `Leiden::run` invocation.
///
/// `partition` maps user-supplied node ids to community ids, sorted by `Id`.
/// `quality` is the final modularity value.
/// `iterations` is the number of outer loop iterations completed or cap reached.
/// `termination_reason` is the reason the algorithm terminated.
/// `seed` is the round-tripped randomness seed metadata.
/// `threading` is the threading policy (always `SingleThreaded` in v1).
#[derive(Debug, Clone, PartialEq)]
pub struct RunResult<Id: NodeId> {
    /// Disjoint community assignments: `(user_node_id, community_id)` pairs, sorted by `Id`.
    pub partition: Vec<(Id, u32)>,
    /// Modularity (or quality) score of the final partition.
    pub quality: f64,
    /// Iterations completed or cap reached.
    pub iterations: u32,
    /// Reason the algorithm terminated.
    pub termination_reason: TerminationReason,
    /// Randomness seed passed to parameters, round-tripped verbatim.
    pub seed: Option<u64>,
    /// Threading policy used for execution.
    pub threading: ThreadingPolicy,
}

impl<Id: NodeId> RunResult<Id> {
    /// Construct a new `RunResult`.
    #[must_use]
    pub const fn new(
        partition: Vec<(Id, u32)>,
        quality: f64,
        iterations: u32,
        termination_reason: TerminationReason,
        seed: Option<u64>,
        threading: ThreadingPolicy,
    ) -> Self {
        Self {
            partition,
            quality,
            iterations,
            termination_reason,
            seed,
            threading,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::float_cmp,
        clippy::uninlined_format_args,
        reason = "test assertions on exact float values"
    )]

    use super::*;

    #[test]
    fn runresult_iterations_semantics_per_termination_reason() {
        let res_converged = RunResult::new(
            vec![("a".to_string(), 0)],
            0.5,
            2,
            TerminationReason::Converged,
            None,
            ThreadingPolicy::SingleThreaded,
        );
        assert_eq!(res_converged.iterations, 2);
        assert_eq!(
            res_converged.termination_reason,
            TerminationReason::Converged
        );

        let res_degenerate = RunResult::new(
            Vec::<(&str, u32)>::new(),
            0.0,
            0,
            TerminationReason::DegenerateInput,
            None,
            ThreadingPolicy::SingleThreaded,
        );
        assert_eq!(res_degenerate.iterations, 0);
        assert_eq!(
            res_degenerate.termination_reason,
            TerminationReason::DegenerateInput
        );

        let res_cap = RunResult::new(
            vec![("a".to_string(), 0)],
            0.4,
            10,
            TerminationReason::IterationCap,
            None,
            ThreadingPolicy::SingleThreaded,
        );
        assert_eq!(res_cap.iterations, 10);
        assert_eq!(res_cap.termination_reason, TerminationReason::IterationCap);
    }

    #[test]
    fn runresult_threading_is_single_threaded_for_all_termination_reasons() {
        for reason in [
            TerminationReason::Converged,
            TerminationReason::DegenerateInput,
            TerminationReason::IterationCap,
        ] {
            let res = RunResult::new(
                vec![("n", 0)],
                0.0,
                1,
                reason,
                None,
                ThreadingPolicy::SingleThreaded,
            );
            assert_eq!(res.threading, ThreadingPolicy::SingleThreaded);
        }
    }

    #[test]
    fn runresult_seed_round_trips_under_every_termination_reason() {
        let sentinels = [
            (TerminationReason::Converged, Some(42_u64)),
            (TerminationReason::DegenerateInput, Some(0xDEAD_u64)),
            (TerminationReason::IterationCap, Some(0xBEEF_u64)),
        ];

        for (reason, seed) in sentinels {
            let res = RunResult::new(
                vec![("node_1", 0)],
                0.5,
                1,
                reason,
                seed,
                ThreadingPolicy::SingleThreaded,
            );
            assert_eq!(res.seed, seed);
            assert_eq!(res.termination_reason, reason);
        }
    }

    #[test]
    fn thread_policy_serializes_to_single_threaded_string() {
        let Ok(serialized) = serde_json::to_string(&ThreadingPolicy::SingleThreaded) else {
            return;
        };
        assert_eq!(serialized, r#""SingleThreaded""#);
    }
}
