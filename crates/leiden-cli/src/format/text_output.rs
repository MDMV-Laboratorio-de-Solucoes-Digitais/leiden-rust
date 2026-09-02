//! Tab-separated plain text partition output per `cli-schema.md §1.4.2`.

use std::fmt::Write;

use leiden::RunResult;

/// Format the partition assignments as sorted tab-separated `<node>\t<community>` lines.
#[must_use]
pub fn render_text_output(result: &RunResult<String>) -> String {
    let mut sorted_partition = result.partition.clone();
    sorted_partition.sort_by(|a, b| a.0.cmp(&b.0));

    let mut output = String::new();
    for (node, comm) in sorted_partition {
        let _ = writeln!(output, "{node}\t{comm}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use leiden::{TerminationReason, ThreadingPolicy};

    #[test]
    fn text_output_sorted_tab_separated() {
        let result = RunResult {
            partition: vec![("z".to_string(), 1), ("a".to_string(), 0)],
            quality: 0.5,
            iterations: 1,
            termination_reason: TerminationReason::Converged,
            seed: Some(0),
            threading: ThreadingPolicy::SingleThreaded,
        };

        let text = render_text_output(&result);
        assert_eq!(text, "a\t0\nz\t1\n");
    }
}
