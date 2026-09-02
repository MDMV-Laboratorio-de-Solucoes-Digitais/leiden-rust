//! Structured observability events emitted by the Leiden algorithm.

use serde::{Deserialize, Serialize};

use crate::error::LeidenError;

/// Algorithm phase identifier for per-phase events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Greedy local-moving phase (Traag 2019 §3, Algorithm A.2 lines 8-25).
    LocalMoving,
    /// Refinement phase (Traag 2019 §3, Algorithm A.2 lines 33-42).
    Refinement,
    /// Aggregation phase building the aggregate graph (Traag 2019 §3, lines 44-48).
    Aggregation,
}

/// Reason the orchestrator stopped iterating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminationReason {
    /// Stable iteration: no node moved and refinement did not split.
    Converged,
    /// User-supplied iteration cap reached; best-so-far partition returned.
    IterationCap,
    /// Degenerate input (zero nodes or zero total edge weight); short-circuit.
    DegenerateInput,
}

/// Policy describing threading used for a run.
///
/// In v1 only [`ThreadingPolicy::SingleThreaded`] is ever produced; the run
/// is strictly sequential. `ThreadPoolSize` is reserved for a future
/// multi-threaded variant gated behind a Constitution amendment (FR-012).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreadingPolicy {
    /// Single-threaded execution (v1 only).
    SingleThreaded,
    /// Thread-pool size reserved for a future parallel variant.
    ThreadPoolSize(std::num::NonZeroU32),
}

/// Structured observability event emitted by every algorithm phase.
///
/// Emitted via `tracing` (library) and also sent over an `mpsc::Sender` to
/// the TUI's render loop (binary). `tracing` events are the source of truth;
/// the TUI consumes the same payload via a custom subscriber layer.
#[derive(Debug, Clone)]
pub enum LeidenEvent {
    /// Graph successfully loaded and validated.
    GraphLoaded {
        /// Number of distinct nodes.
        nodes: usize,
        /// Number of undirected edges (CSR entries / 2).
        edges: usize,
        /// Sum of all edge weights (`m`).
        total_weight: f64,
    },
    /// Start of an outer-loop iteration.
    IterationStarted {
        /// Zero-based iteration index.
        index: u32,
        /// Phase being entered.
        phase: Phase,
    },
    /// Progress within the local-moving phase.
    LocalMovingProgress {
        /// Outer-loop iteration index.
        iteration: u32,
        /// Number of nodes moved so far in this pass.
        moved_nodes: u32,
    },
    /// Modularity delta for a single node move.
    LocalMovingDelta {
        /// Outer-loop iteration index.
        iteration: u32,
        /// Delta modularity (`ΔQ`) of the move.
        delta_q: f64,
    },
    /// Refinement merged two subsets.
    RefinementMerged {
        /// Outer-loop iteration index.
        iteration: u32,
        /// Source community id.
        from: u32,
        /// Target community id.
        to: u32,
    },
    /// Aggregation built the aggregate graph.
    Aggregation {
        /// Outer-loop iteration index.
        iteration: u32,
        /// Number of nodes in the aggregate graph.
        aggregate_nodes: usize,
    },
    /// Quality (modularity) computed for an iteration.
    QualityComputed {
        /// Outer-loop iteration index.
        iteration: u32,
        /// Total modularity `Q`.
        quality: f64,
    },
    /// End of an outer-loop iteration.
    IterationFinished {
        /// Zero-based iteration index.
        index: u32,
        /// Total modularity `Q` at iteration end.
        quality: f64,
        /// Optional snapshot of the partition.
        partition: Option<crate::partition::Partition>,
    },
    /// Algorithm terminated.
    Terminated {
        /// Total iterations executed.
        iterations: u32,
        /// Why the loop stopped.
        reason: TerminationReason,
        /// Final modularity `Q`.
        quality: f64,
    },
    /// Back-pressure indicator: channel full, events dropped.
    Throttled {
        /// Number of events dropped since last `Throttled`.
        dropped: u64,
    },
}

impl LeidenEvent {
    /// Emit this event via `tracing::info!` with structured fields.
    pub fn emit(&self) {
        match self {
            Self::GraphLoaded {
                nodes,
                edges,
                total_weight,
            } => {
                tracing::info!(nodes = %nodes, edges = %edges, total_weight = %total_weight, "GraphLoaded");
            }
            Self::IterationStarted { index, phase } => {
                tracing::info!(index = %index, phase = ?phase, "IterationStarted");
            }
            Self::LocalMovingProgress {
                iteration,
                moved_nodes,
            } => {
                tracing::info!(iteration = %iteration, moved_nodes = %moved_nodes, "LocalMovingProgress");
            }
            Self::LocalMovingDelta { iteration, delta_q } => {
                tracing::info!(iteration = %iteration, delta_q = %delta_q, "LocalMovingDelta");
            }
            Self::RefinementMerged {
                iteration,
                from,
                to,
            } => {
                tracing::info!(iteration = %iteration, from = %from, to = %to, "RefinementMerged");
            }
            Self::Aggregation {
                iteration,
                aggregate_nodes,
            } => {
                tracing::info!(iteration = %iteration, aggregate_nodes = %aggregate_nodes, "Aggregation");
            }
            Self::QualityComputed { iteration, quality } => {
                tracing::info!(iteration = %iteration, quality = %quality, "QualityComputed");
            }
            Self::IterationFinished { index, quality, .. } => {
                tracing::info!(index = %index, quality = %quality, "IterationFinished");
            }
            Self::Terminated {
                iterations,
                reason,
                quality,
            } => {
                tracing::info!(iterations = %iterations, reason = ?reason, quality = %quality, "Terminated");
            }
            Self::Throttled { dropped } => {
                tracing::info!(dropped = %dropped, "Throttled");
            }
        }
    }
}

/// Map a [`LeidenError`] to a structured tracing error event.
pub fn emit_error(error: &LeidenError) {
    tracing::error!(error = %error, "LeidenError");
}

#[cfg(test)]
mod tests {
    use super::{LeidenEvent, Phase, TerminationReason};

    #[test]
    fn leidenevent_debug_and_clone() {
        let events = vec![
            LeidenEvent::GraphLoaded {
                nodes: 10,
                edges: 20,
                total_weight: 30.0,
            },
            LeidenEvent::IterationStarted {
                index: 0,
                phase: Phase::LocalMoving,
            },
            LeidenEvent::LocalMovingProgress {
                iteration: 1,
                moved_nodes: 5,
            },
            LeidenEvent::LocalMovingDelta {
                iteration: 1,
                delta_q: 0.01,
            },
            LeidenEvent::RefinementMerged {
                iteration: 1,
                from: 0,
                to: 1,
            },
            LeidenEvent::Aggregation {
                iteration: 1,
                aggregate_nodes: 4,
            },
            LeidenEvent::QualityComputed {
                iteration: 1,
                quality: 0.42,
            },
            LeidenEvent::IterationFinished {
                index: 1,
                quality: 0.42,
                partition: None,
            },
            LeidenEvent::Terminated {
                iterations: 2,
                reason: TerminationReason::Converged,
                quality: 0.42,
            },
            LeidenEvent::Throttled { dropped: 3 },
        ];
        for event in &events {
            let debug = format!("{event:?}");
            assert!(!debug.is_empty());
            let cloned = event.clone();
            let cloned_debug = format!("{cloned:?}");
            assert_eq!(debug, cloned_debug);
            let result = match event {
                LeidenEvent::GraphLoaded { .. }
                | LeidenEvent::IterationStarted { .. }
                | LeidenEvent::LocalMovingProgress { .. }
                | LeidenEvent::LocalMovingDelta { .. }
                | LeidenEvent::RefinementMerged { .. }
                | LeidenEvent::Aggregation { .. }
                | LeidenEvent::QualityComputed { .. }
                | LeidenEvent::IterationFinished { .. }
                | LeidenEvent::Terminated { .. }
                | LeidenEvent::Throttled { .. } => "matched",
            };
            assert_eq!(result, "matched");
        }
    }

    #[test]
    fn phase_debug_and_clone() {
        let phases = [Phase::LocalMoving, Phase::Refinement, Phase::Aggregation];
        for phase in phases {
            let debug = format!("{phase:?}");
            assert!(!debug.is_empty());
            let cloned = phase;
            assert_eq!(format!("{cloned:?}"), debug);
        }
    }

    fn assert_tracing_contains(event: &LeidenEvent, expected_substrings: &[&str]) {
        use std::io::Write;

        let buffer = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
        let buffer_clone = std::sync::Arc::clone(&buffer);
        let writer = move || {
            struct BufferWriter {
                buffer: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
            }
            impl Write for BufferWriter {
                fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                    self.buffer
                        .lock()
                        .map_err(|_| std::io::Error::other("mutex poisoned"))?
                        .extend_from_slice(buf);
                    Ok(buf.len())
                }
                fn flush(&mut self) -> std::io::Result<()> {
                    Ok(())
                }
            }
            impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for BufferWriter {
                type Writer = Self;
                fn make_writer(&'a self) -> Self::Writer {
                    Self {
                        buffer: std::sync::Arc::clone(&self.buffer),
                    }
                }
            }
            BufferWriter {
                buffer: std::sync::Arc::clone(&buffer_clone),
            }
        };

        let subscriber = tracing_subscriber::fmt::Subscriber::builder()
            .with_writer(writer)
            .with_ansi(false)
            .without_time()
            .with_level(false)
            .with_target(false)
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            event.emit();
        });

        let output = buffer
            .lock()
            .map(|guard| String::from_utf8_lossy(&guard).into_owned())
            .unwrap_or_default();

        for substring in expected_substrings {
            assert!(
                output.contains(substring),
                "expected tracing output for {event:?} to contain `{substring}`, got: {output:?}"
            );
        }
    }

    #[test]
    fn leidenevent_variants_emit_named_fields() {
        assert_tracing_contains(
            &LeidenEvent::GraphLoaded {
                nodes: 10,
                edges: 5,
                total_weight: 12.5,
            },
            &["nodes=", "edges=", "total_weight="],
        );
        assert_tracing_contains(
            &LeidenEvent::IterationStarted {
                index: 0,
                phase: Phase::LocalMoving,
            },
            &["index=", "phase="],
        );
        assert_tracing_contains(
            &LeidenEvent::LocalMovingProgress {
                iteration: 1,
                moved_nodes: 7,
            },
            &["iteration=", "moved_nodes="],
        );
        assert_tracing_contains(
            &LeidenEvent::LocalMovingDelta {
                iteration: 2,
                delta_q: 0.123,
            },
            &["iteration=", "delta_q="],
        );
        assert_tracing_contains(
            &LeidenEvent::RefinementMerged {
                iteration: 1,
                from: 0,
                to: 2,
            },
            &["iteration=", "from=", "to="],
        );
        assert_tracing_contains(
            &LeidenEvent::Aggregation {
                iteration: 1,
                aggregate_nodes: 4,
            },
            &["iteration=", "aggregate_nodes="],
        );
        assert_tracing_contains(
            &LeidenEvent::QualityComputed {
                iteration: 1,
                quality: 0.42,
            },
            &["iteration=", "quality="],
        );
        assert_tracing_contains(
            &LeidenEvent::IterationFinished {
                index: 1,
                quality: 0.5,
                partition: None,
            },
            &["index=", "quality="],
        );
        assert_tracing_contains(
            &LeidenEvent::Terminated {
                iterations: 3,
                reason: TerminationReason::Converged,
                quality: 0.6,
            },
            &["iterations=", "reason=", "quality="],
        );
        assert_tracing_contains(&LeidenEvent::Throttled { dropped: 10 }, &["dropped="]);
    }

    #[test]
    fn iteration_finished_carries_partition() {
        let event = LeidenEvent::IterationFinished {
            index: 1,
            quality: 0.5,
            partition: None::<crate::partition::Partition>,
        };
        assert!(
            matches!(
                event,
                LeidenEvent::IterationFinished {
                    partition: None,
                    ..
                }
            ),
            "wrong event"
        );
    }
}
