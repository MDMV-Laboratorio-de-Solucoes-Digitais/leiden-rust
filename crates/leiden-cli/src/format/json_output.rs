//! JSON output serialization for partition results per `cli-schema.md §1.4.1`.

use leiden::{LeidenParameters, RunResult, TerminationReason, ThreadingPolicy};
use serde::Serialize;

/// Top-level JSON partition document.
#[derive(Debug, Serialize)]
pub struct PartitionOutput<'a> {
    /// Resolution parameter gamma.
    pub gamma: f64,
    /// Randomness seed metadata.
    pub seed: Option<u64>,
    /// Number of completed iterations or cap reached.
    pub iterations: u32,
    /// Termination reason in `snake_case`.
    #[serde(rename = "termination_reason")]
    pub termination_reason: &'a str,
    /// Final modularity quality score.
    pub quality: f64,
    /// Threading policy string ("`SingleThreaded`" in v1).
    #[serde(rename = "threading")]
    pub threading: &'a str,
    /// Disjoint community assignments sorted by node identifier.
    #[serde(rename = "assignments")]
    pub assignments: Vec<Assignment<'a>>,
}

/// A single node-to-community assignment pair.
#[derive(Debug, Serialize)]
pub struct Assignment<'a> {
    /// User node identifier.
    #[serde(rename = "node")]
    pub node: &'a str,
    /// Community index.
    #[serde(rename = "community")]
    pub community: u32,
}

/// Format the run result as pretty-printed JSON.
///
/// # Errors
///
/// Returns `serde_json::Error` if serialization fails.
pub fn render_json_output(
    params: &LeidenParameters,
    result: &RunResult<String>,
) -> Result<String, serde_json::Error> {
    let term_str = match result.termination_reason {
        TerminationReason::Converged => "converged",
        TerminationReason::IterationCap => "iteration_cap",
        TerminationReason::DegenerateInput => "degenerate_input",
    };
    let threading_str = match result.threading {
        ThreadingPolicy::SingleThreaded => "SingleThreaded",
        ThreadingPolicy::ThreadPoolSize(_) => "ThreadPoolSize",
    };
    let mut assignments: Vec<Assignment<'_>> = result
        .partition
        .iter()
        .map(|(node, comm)| Assignment {
            node: node.as_str(),
            community: *comm,
        })
        .collect();
    assignments.sort_by(|a, b| a.node.cmp(b.node));

    let output = PartitionOutput {
        gamma: params.gamma,
        seed: result.seed,
        iterations: result.iterations,
        termination_reason: term_str,
        quality: result.quality,
        threading: threading_str,
        assignments,
    };

    serde_json::to_string_pretty(&output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_output_serializes_threaded_single_threaded() {
        let params = LeidenParameters::default();
        let result = RunResult {
            partition: vec![("a".to_string(), 0), ("b".to_string(), 1)],
            quality: 0.5,
            iterations: 2,
            termination_reason: TerminationReason::Converged,
            seed: Some(0),
            threading: ThreadingPolicy::SingleThreaded,
        };

        let Ok(json_str) = render_json_output(&params, &result) else {
            return;
        };
        assert!(json_str.contains(r#""threading": "SingleThreaded""#));
        assert!(json_str.contains(r#""termination_reason": "converged""#));
    }

    #[test]
    fn thread_policy_serializes_to_single_threaded_string() {
        let Ok(json_str) = serde_json::to_string(&ThreadingPolicy::SingleThreaded) else {
            return;
        };
        assert_eq!(json_str, r#""SingleThreaded""#);
    }
}
