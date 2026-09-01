//! Orchestrator driving the iterative Leiden algorithm.

// ref: Traag 2019 §3 — Leiden algorithm outer loop (Algorithm A.2 lines 1–48).

pub mod result;

use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;

use crate::aggregation::aggregation;

/// Control flags for pausing, stepping, or aborting the algorithm.
#[derive(Debug, Default, Clone)]
pub struct ControlFlags {
    /// Flag indicating whether the orchestrator is paused.
    pub paused: Arc<AtomicBool>,
    /// Flag indicating whether the orchestrator should step once.
    pub step: Arc<AtomicBool>,
    /// Flag indicating whether the orchestrator should abort execution.
    pub abort: Arc<AtomicBool>,
}
use crate::error::LeidenError;
use crate::events::{LeidenEvent, Phase, TerminationReason, ThreadingPolicy};
use crate::graph::{CsrGraph, NodeId};
use crate::local_moving::local_moving;
use crate::params::LeidenParameters;
use crate::partition::Partition;
use crate::quality::{Modularity, QualityFunction};
use crate::refinement::refinement;

pub use result::RunResult;

/// Orchestrator entry point for running the Leiden community detection algorithm.
#[derive(Debug, Default, Clone)]
pub struct Leiden {
    params: LeidenParameters,
    event_sink: Option<Sender<LeidenEvent>>,
    threads: Option<NonZeroU32>,
    control_flags: Option<Arc<ControlFlags>>,
}

impl Leiden {
    /// Construct a new `Leiden` orchestrator with default parameters.
    #[must_use]
    pub fn new() -> Self {
        Self {
            params: LeidenParameters::default(),
            event_sink: None,
            threads: None,
            control_flags: None,
        }
    }

    /// Override the algorithm parameters.
    #[must_use]
    pub const fn with_parameters(mut self, params: LeidenParameters) -> Self {
        self.params = params;
        self
    }

    /// Attach an event sink channel for receiving observability events.
    #[must_use]
    pub fn with_event_sink(mut self, tx: Sender<LeidenEvent>) -> Self {
        self.event_sink = Some(tx);
        self
    }

    /// Set the thread pool size (accepted for forward compatibility in v1).
    #[must_use]
    pub const fn with_threads(mut self, threads: NonZeroU32) -> Self {
        self.threads = Some(threads);
        self
    }

    /// Attach control flags for pausing, stepping, or aborting execution.
    #[must_use]
    pub fn with_control_flags(mut self, flags: Arc<ControlFlags>) -> Self {
        self.control_flags = Some(flags);
        self
    }

    fn emit_event(&self, event: LeidenEvent) {
        match &event {
            LeidenEvent::GraphLoaded {
                nodes,
                edges,
                total_weight,
            } => {
                tracing::debug!(
                    nodes = nodes,
                    edges = edges,
                    total_weight = total_weight,
                    "graph_loaded"
                );
            }
            LeidenEvent::IterationStarted { index, phase } => {
                tracing::debug!(iteration = index, phase = ?phase, "iteration_started");
            }
            LeidenEvent::LocalMovingProgress {
                iteration,
                moved_nodes,
            } => {
                tracing::debug!(
                    iteration = iteration,
                    moved_nodes = moved_nodes,
                    "local_moving_progress"
                );
            }
            LeidenEvent::LocalMovingDelta { iteration, delta_q } => {
                tracing::debug!(
                    iteration = iteration,
                    delta_q = delta_q,
                    "local_moving_delta"
                );
            }
            LeidenEvent::RefinementMerged {
                iteration,
                from,
                to,
            } => {
                tracing::debug!(
                    iteration = iteration,
                    from = from,
                    to = to,
                    "refinement_merged"
                );
            }
            LeidenEvent::Aggregation {
                iteration,
                aggregate_nodes,
            } => {
                tracing::debug!(
                    iteration = iteration,
                    aggregate_nodes = aggregate_nodes,
                    "aggregation"
                );
            }
            LeidenEvent::QualityComputed { iteration, quality } => {
                tracing::debug!(iteration = iteration, quality = quality, "quality_computed");
            }
            LeidenEvent::IterationFinished { index, quality, .. } => {
                tracing::debug!(iteration = index, quality = quality, "iteration_finished");
            }
            LeidenEvent::Terminated {
                iterations,
                reason,
                quality,
            } => {
                tracing::debug!(
                    iterations = iterations,
                    reason = ?reason,
                    quality = quality,
                    "terminated"
                );
            }
            LeidenEvent::Throttled { dropped } => {
                tracing::warn!(dropped = dropped, "throttled");
            }
        }
        if let Some(ref sink) = self.event_sink {
            let _ = sink.send(event);
        }
    }

    fn project_on_g0(n: usize, flat_mapping: &[u32], moved_partition: &Partition) -> Partition {
        let mut g0_assignment = vec![0_u32; n];
        for i in 0..n {
            let curr_node = flat_mapping.get(i).copied().unwrap_or(0);
            let comm = moved_partition.community_of(curr_node);
            if let Some(slot) = g0_assignment.get_mut(i) {
                *slot = comm;
            }
        }
        let mut g0_partition = Partition::singletons(n);
        g0_partition.assignment = g0_assignment;
        g0_partition.renumber();
        g0_partition
    }

    fn handle_trivial_graph<Id: NodeId>(&self, graph: &CsrGraph<Id>) -> Option<RunResult<Id>> {
        let n = graph.node_count();
        let m = graph.total_weight();

        if n == 0 {
            let result = RunResult::new(
                Vec::new(),
                0.0,
                0,
                TerminationReason::DegenerateInput,
                self.params.seed,
                ThreadingPolicy::SingleThreaded,
            );
            self.emit_event(LeidenEvent::Terminated {
                iterations: 0,
                reason: TerminationReason::DegenerateInput,
                quality: 0.0,
            });
            return Some(result);
        }

        if n == 1 {
            let mut partition_vec = Vec::new();
            if let Some(id) = graph.node_id(0) {
                partition_vec.push((id.clone(), 0));
            }
            let result = RunResult::new(
                partition_vec,
                0.0,
                0,
                TerminationReason::Converged,
                self.params.seed,
                ThreadingPolicy::SingleThreaded,
            );
            self.emit_event(LeidenEvent::Terminated {
                iterations: 0,
                reason: TerminationReason::Converged,
                quality: 0.0,
            });
            return Some(result);
        }

        if m <= 0.0 {
            let mut partition_vec = Vec::with_capacity(n);
            for i in 0..n {
                let Ok(u_idx) = u32::try_from(i) else {
                    continue;
                };
                if let Some(id) = graph.node_id(u_idx) {
                    partition_vec.push((id.clone(), u_idx));
                }
            }
            partition_vec.sort_by(|a, b| a.0.cmp(&b.0));
            let result = RunResult::new(
                partition_vec,
                0.0,
                0,
                TerminationReason::Converged,
                self.params.seed,
                ThreadingPolicy::SingleThreaded,
            );
            self.emit_event(LeidenEvent::Terminated {
                iterations: 0,
                reason: TerminationReason::Converged,
                quality: 0.0,
            });
            return Some(result);
        }

        None
    }

    fn run_outer_loop<Id: NodeId>(
        &self,
        graph: &CsrGraph<Id>,
        quality_fn: &Modularity,
    ) -> (Vec<u32>, f64, u32, TerminationReason) {
        let n = graph.node_count();
        let mut curr_graph = graph.to_u32_graph();
        let mut curr_partition = Partition::singletons_from_graph(&curr_graph);
        let mut flat_mapping: Vec<u32> =
            u32::try_from(n).map_or_else(|_| Vec::new(), |count| (0..count).collect());

        let initial_quality = quality_fn.total_quality(graph, &curr_partition);
        let mut best_partition_assignment: Vec<u32> = curr_partition.assignment.clone();
        let mut best_quality = initial_quality;
        let mut best_comm_count = curr_partition.community_count();

        let mut completed_passes = 0_u32;
        let mut termination_reason = TerminationReason::IterationCap;

        for iter in 0..self.params.iteration_cap {
            if let Some(flags) = &self.control_flags {
                if flags.abort.load(Ordering::SeqCst) {
                    break;
                }
                while flags.paused.load(Ordering::SeqCst) {
                    if flags.abort.load(Ordering::SeqCst) {
                        break;
                    }
                    if flags.step.load(Ordering::SeqCst) {
                        flags.step.store(false, Ordering::SeqCst);
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                if flags.abort.load(Ordering::SeqCst) {
                    break;
                }
            }

            self.emit_event(LeidenEvent::IterationStarted {
                index: iter,
                phase: Phase::LocalMoving,
            });

            let (moved_partition, moved_any, moved_count) =
                local_moving(&curr_graph, curr_partition, quality_fn);

            self.emit_event(LeidenEvent::LocalMovingProgress {
                iteration: iter,
                moved_nodes: moved_count,
            });

            self.emit_event(LeidenEvent::IterationStarted {
                index: iter,
                phase: Phase::Refinement,
            });

            let refined_partition = refinement(&curr_graph, &moved_partition, quality_fn);
            let g0_partition = Self::project_on_g0(n, &flat_mapping, &moved_partition);
            let current_q = quality_fn.total_quality(graph, &g0_partition);
            let current_comm_count = g0_partition.community_count();

            self.emit_event(LeidenEvent::QualityComputed {
                iteration: iter,
                quality: current_q,
            });
            self.emit_event(LeidenEvent::IterationFinished {
                index: iter,
                quality: current_q,
                partition: Some(g0_partition.clone()),
            });

            let is_better = current_q > best_quality + f64::EPSILON;
            let is_tied = (current_q - best_quality).abs() <= f64::EPSILON;
            if is_better || (is_tied && current_comm_count < best_comm_count) {
                best_quality = current_q;
                best_partition_assignment.clone_from(&g0_partition.assignment);
                best_comm_count = current_comm_count;
            }

            let Ok(curr_nodes_u32) = u32::try_from(curr_graph.node_count()) else {
                break;
            };
            let is_stable = !moved_any
                || (refined_partition.community_count() == curr_nodes_u32
                    && refined_partition == moved_partition);

            if is_stable {
                completed_passes = iter.saturating_add(1);
                termination_reason = TerminationReason::Converged;
                break;
            }

            if iter.saturating_add(1) == self.params.iteration_cap {
                completed_passes = self.params.iteration_cap;
                termination_reason = TerminationReason::IterationCap;
                break;
            }

            self.emit_event(LeidenEvent::IterationStarted {
                index: iter,
                phase: Phase::Aggregation,
            });

            let Ok((next_graph, next_partition)) =
                aggregation(&curr_graph, &refined_partition, &moved_partition)
            else {
                completed_passes = iter.saturating_add(1);
                termination_reason = TerminationReason::Converged;
                break;
            };

            self.emit_event(LeidenEvent::Aggregation {
                iteration: iter,
                aggregate_nodes: next_graph.node_count(),
            });

            for slot in &mut flat_mapping {
                *slot = refined_partition.community_of(*slot);
            }

            curr_graph = next_graph;
            curr_partition = next_partition;
            completed_passes = iter.saturating_add(1);
        }

        (
            best_partition_assignment,
            best_quality,
            completed_passes,
            termination_reason,
        )
    }

    /// Run the Leiden algorithm on `graph`.
    ///
    /// # Errors
    ///
    /// Returns `LeidenError::InvalidGamma` if `gamma <= 0`.
    /// Returns `LeidenError::InvalidIterationCap` if `iteration_cap < 1`.
    /// Returns `LeidenError` on graph or aggregation failures.
    pub fn run<Id: NodeId>(self, graph: &CsrGraph<Id>) -> Result<RunResult<Id>, LeidenError> {
        self.params.validate()?;

        self.emit_event(LeidenEvent::GraphLoaded {
            nodes: graph.node_count(),
            edges: graph.edge_count(),
            total_weight: graph.total_weight(),
        });

        if let Some(trivial_result) = self.handle_trivial_graph(graph) {
            return Ok(trivial_result);
        }

        let n = graph.node_count();
        let quality_fn = Modularity::new(self.params.gamma);

        let (best_assignment, best_quality, completed_passes, termination_reason) =
            self.run_outer_loop(graph, &quality_fn);

        let iterations = match termination_reason {
            TerminationReason::Converged => completed_passes,
            TerminationReason::IterationCap => self.params.iteration_cap,
            TerminationReason::DegenerateInput => 0,
        };

        let mut final_partition_vec = Vec::with_capacity(n);
        for i in 0..n {
            let Ok(u_idx) = u32::try_from(i) else {
                continue;
            };
            if let Some(id) = graph.node_id(u_idx) {
                let comm = best_assignment.get(i).copied().unwrap_or(0);
                final_partition_vec.push((id.clone(), comm));
            }
        }
        final_partition_vec.sort_by(|a, b| a.0.cmp(&b.0));

        self.emit_event(LeidenEvent::Terminated {
            iterations,
            reason: termination_reason,
            quality: best_quality,
        });

        Ok(RunResult::new(
            final_partition_vec,
            best_quality,
            iterations,
            termination_reason,
            self.params.seed,
            ThreadingPolicy::SingleThreaded,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{CsrGraph, Edge};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_orchestrator_control_flags() {
        let paused = Arc::new(AtomicBool::new(true));
        let step = Arc::new(AtomicBool::new(false));
        let abort = Arc::new(AtomicBool::new(false));

        let control_flags = Arc::new(ControlFlags {
            paused: paused.clone(),
            step: step.clone(),
            abort: abort.clone(),
        });

        let orchestrator = Leiden::new().with_control_flags(control_flags);

        let edges = vec![Edge {
            source: 1_u32,
            target: 2_u32,
            weight: 1.0,
        }];
        let Ok(graph) = CsrGraph::from_edges(edges) else {
            return;
        };

        // We run it in a background thread to allow unpausing
        let handle = std::thread::spawn(move || {
            let _ = orchestrator.run(&graph);
        });

        // The orchestrator is paused, we unpause it by setting step
        step.store(true, Ordering::SeqCst);

        // We abort
        abort.store(true, Ordering::SeqCst);

        let _ = handle.join();
    }
}
