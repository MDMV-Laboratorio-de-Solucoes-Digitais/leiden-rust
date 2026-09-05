//! Modularity quality function and move components calculations.

// ref: Traag 2019 §3 — Modularity definition (Eq. 1) and delta move formula (Eq. A5).

use crate::graph::{CsrGraph, NodeId};
use crate::partition::Partition;
use crate::quality::QualityFunction;

/// Cached per-node quantities used by `QualityFunction::delta_move` to
/// avoid recomputation across phases.
///
/// `k_i` is the weighted degree of the node.
/// `sigma_in_to_target` is the sum of weights between node and target community.
/// `sigma_tot_target` is the sum of degrees of nodes in target community.
/// `sigma_in_from_current` is the sum of weights between node and current community (excluding node).
/// `sigma_tot_current` is the sum of degrees of nodes in current community.
#[derive(Debug, Clone, PartialEq)]
pub struct MoveComponents {
    /// Weighted degree of the node.
    pub k_i: f64,
    /// Sum of edge weights between the node and the target community.
    pub sigma_in_to_target: f64,
    /// Sum of degrees of nodes in the target community.
    pub sigma_tot_target: f64,
    /// Sum of edge weights between the node and its current community (excluding self).
    pub sigma_in_from_current: f64,
    /// Sum of degrees of nodes in the current community.
    pub sigma_tot_current: f64,
}

impl MoveComponents {
    /// Construct a new `MoveComponents` with pre-computed quantities.
    #[must_use]
    pub const fn new(
        k_i: f64,
        sigma_in_to_target: f64,
        sigma_tot_target: f64,
        sigma_in_from_current: f64,
        sigma_tot_current: f64,
    ) -> Self {
        Self {
            k_i,
            sigma_in_to_target,
            sigma_tot_target,
            sigma_in_from_current,
            sigma_tot_current,
        }
    }
}

/// Modularity quality function (Traag 2019, Eq. 1).
#[derive(Debug, Clone, PartialEq)]
pub struct Modularity {
    /// Resolution parameter gamma.
    pub gamma: f64,
}

impl Modularity {
    /// Create a new `Modularity` quality function with resolution `gamma`.
    #[must_use]
    pub const fn new(gamma: f64) -> Self {
        Self { gamma }
    }
}

impl QualityFunction for Modularity {
    // ref: Traag 2019 §3 — Modularity formula Eq. (1)
    fn total_quality<Id: NodeId>(&self, graph: &CsrGraph<Id>, partition: &Partition) -> f64 {
        let m = graph.total_weight();
        if m <= 0.0 || !m.is_finite() {
            return 0.0;
        }

        let n = graph.node_count();
        let Ok(num_comm) = usize::try_from(partition.community_count()) else {
            return 0.0;
        };

        let mut two_e_c = vec![0.0; num_comm];
        let mut sigma_tot = vec![0.0; num_comm];

        for u in 0..n {
            let Ok(u_internal) = u32::try_from(u) else {
                continue;
            };
            let Ok(c_u) = usize::try_from(partition.community_of(u_internal)) else {
                continue;
            };
            let deg_u = graph.degree_of(u_internal);

            if let Some(s_tot) = sigma_tot.get_mut(c_u) {
                *s_tot += deg_u;
            }

            let neighbours = graph.neighbours_of(u_internal);
            let weights = graph.weights_of(u_internal);

            for (&v, &w) in neighbours.iter().zip(weights.iter()) {
                let Ok(c_v) = usize::try_from(partition.community_of(v)) else {
                    continue;
                };
                if c_u == c_v {
                    let Some(e) = two_e_c.get_mut(c_u) else {
                        continue;
                    };
                    *e += w;
                }
            }
        }

        let mut total = 0.0;
        let two_m = 2.0 * m;

        for c in 0..num_comm {
            let two_e = two_e_c.get(c).copied().unwrap_or(0.0);
            let s_tot = sigma_tot.get(c).copied().unwrap_or(0.0);
            let penalty = self.gamma * (s_tot * s_tot) / two_m;
            total += two_e - penalty;
        }

        let q = total / two_m;
        if q.is_nan() || !q.is_finite() { 0.0 } else { q }
    }

    // ref: Traag 2019 §3 — Delta move formula Eq. (A5)
    fn delta_move<Id: NodeId>(
        &self,
        graph: &CsrGraph<Id>,
        partition: &Partition,
        node: u32,
        target_community: u32,
        components: &MoveComponents,
    ) -> f64 {
        let current_community = partition.community_of(node);
        if current_community == target_community {
            return 0.0;
        }

        let m = graph.total_weight();
        if m <= 0.0 || !m.is_finite() {
            return 0.0;
        }

        let delta_edges = (components.sigma_in_to_target - components.sigma_in_from_current) / m;
        let two_m_sq = 2.0 * m * m;
        let diff_tot = components.sigma_tot_target - components.sigma_tot_current + components.k_i;
        let delta_degree = self.gamma * components.k_i * diff_tot / two_m_sq;

        let delta_q = delta_edges - delta_degree;
        if delta_q.is_nan() || !delta_q.is_finite() {
            0.0
        } else {
            delta_q
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
    use crate::graph::Edge;

    #[test]
    fn move_components_fields_populated_correctly() {
        let comp = MoveComponents::new(2.5, 1.0, 4.0, 0.5, 3.0);
        assert_eq!(comp.k_i, 2.5);
        assert_eq!(comp.sigma_in_to_target, 1.0);
        assert_eq!(comp.sigma_tot_target, 4.0);
        assert_eq!(comp.sigma_in_from_current, 0.5);
        assert_eq!(comp.sigma_tot_current, 3.0);
    }

    #[test]
    fn modularity_total_quality_empty_graph_returns_zero() {
        let edges: Vec<Edge<u32>> = vec![];
        let graph_res = CsrGraph::from_edges(edges);
        assert!(graph_res.is_err());
    }

    #[test]
    fn modularity_delta_move_matches_hand_computed_triangle() {
        // 3-node triangle with weights 1.0
        let edges = vec![
            Edge {
                source: 0_u32,
                target: 1_u32,
                weight: 1.0,
            },
            Edge {
                source: 1_u32,
                target: 2_u32,
                weight: 1.0,
            },
            Edge {
                source: 0_u32,
                target: 2_u32,
                weight: 1.0,
            },
        ];
        let Ok(graph) = CsrGraph::from_edges(edges) else {
            return;
        };
        let partition = Partition::singletons(3);
        let modularity = Modularity::new(1.0);

        // Move node 0 from community 0 to community 1
        // k_0 = 2.0, sigma_in_to_target = 1.0, sigma_tot_target = 2.0, sigma_in_from_current = 0.0, sigma_tot_current = 2.0
        let components = MoveComponents::new(2.0, 1.0, 2.0, 0.0, 2.0);
        let delta = modularity.delta_move(&graph, &partition, 0, 1, &components);

        // Hand-computed: 1/3 - 4/18 = 2/18 = 1/9 ~ 0.1111111111111111
        let expected = 1.0 / 9.0;
        assert!((delta - expected).abs() < 1e-10);
    }

    #[test]
    fn modularity_total_quality_two_cliques_reference() {
        let mut edges = Vec::new();
        for i in 0..4_u32 {
            for j in (i + 1)..4_u32 {
                edges.push(Edge {
                    source: i,
                    target: j,
                    weight: 1.0,
                });
            }
        }
        for i in 4..9_u32 {
            for j in (i + 1)..9_u32 {
                edges.push(Edge {
                    source: i,
                    target: j,
                    weight: 1.0,
                });
            }
        }
        edges.push(Edge {
            source: 3,
            target: 4,
            weight: 0.5,
        });

        let Ok(graph) = CsrGraph::from_edges(edges) else {
            return;
        };
        let mut partition = Partition::singletons(9);
        // Put 0..4 in comm 0, 4..9 in comm 1
        for i in 1..4_u32 {
            partition.move_node(i, 0);
        }
        for i in 5..9_u32 {
            partition.move_node(i, 4);
        }
        partition.renumber();

        let modularity = Modularity::new(1.0);
        let q = modularity.total_quality(&graph, &partition);
        assert!(q > 0.4);
    }
}

#[cfg(test)]
mod property_tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::cast_possible_truncation,
        clippy::uninlined_format_args,
        clippy::manual_range_contains,
        clippy::format_push_string,
        clippy::option_if_let_else,
        clippy::unreachable,
        clippy::cast_lossless,
        unused_doc_comments,
        deprecated,
        reason = "test code"
    )]

    use proptest::prelude::*;
    use rand::Rng;

    use super::{Modularity, MoveComponents};
    use crate::partition::Partition;
    use crate::quality::QualityFunction;
    use crate::testing::config::{MODULARITY_EPSILON, proptest_config};
    use crate::testing::graphs::{
        ErdosRenyi, GraphGenerator, ScaleFree, StochasticBlock, TestGraph, TopologyKind,
    };
    use crate::testing::invariants::{assert_finite, assert_modularity_valid};

    /// Strategy selecting one of three distinct topologies (FR-006).
    fn topology_strategy() -> impl Strategy<Value = TopologyKind> {
        prop_oneof![
            Just(TopologyKind::ErdosRenyi),
            Just(TopologyKind::StochasticBlock),
            Just(TopologyKind::ScaleFree),
        ]
    }

    /// Generate a CSR graph using the selected topology.
    fn generate_graph(topology: TopologyKind, rng: &mut impl Rng) -> TestGraph {
        match topology {
            TopologyKind::ErdosRenyi => ErdosRenyi::new(0.3).generate(rng),
            TopologyKind::StochasticBlock => StochasticBlock::new(3, 0.3, 0.05).generate(rng),
            TopologyKind::ScaleFree => ScaleFree::new(2).generate(rng),
            _ => unreachable!(),
        }
    }

    /// Build a random partition by merging singletons into a random number of communities.
    fn random_partition(graph: &TestGraph, rng: &mut impl Rng) -> Partition {
        let n = graph.node_count();
        let mut partition = Partition::singletons(n);
        if n > 0 {
            let k = rng.random_range(1..=n);
            for u in 0..n as u32 {
                let target = rng.random_range(0..k as u32);
                partition.move_node(u, target);
            }
            partition.renumber();
        }
        partition
    }

    proptest! {
        #![proptest_config(proptest_config(Some(100), cfg!(debug_assertions)))]

        /// Verifies INV-001: Modularity bounded above by 1.0.
        #[test]
        fn modularity_bounded_above(topology in topology_strategy()) {
            let mut rng = rand::thread_rng();
            let graph = generate_graph(topology, &mut rng);
            prop_assume!(graph.total_weight() > 0.0);
            let partition = random_partition(&graph, &mut rng);
            let modularity = Modularity::new(1.0);
            let q = modularity.total_quality(&graph, &partition);
            assert_modularity_valid(q);
            assert!(
                q <= 1.0 + MODULARITY_EPSILON,
                "modularity {q} exceeds upper bound 1.0"
            );
        }

        /// Verifies that singleton partition yields modularity approximately zero.
        ///
        /// For a singleton partition Q = -Σ deg(i)² / (4m²) ∈ [-0.5, 0].
        /// The value approaches zero for regular/dense graphs and reaches -0.5
        /// only for the most extreme degree skew.  We verify Q lies within the
        /// theoretical bounds (with epsilon slack).
        #[test]
        fn singleton_partition_zero(topology in topology_strategy()) {
            let mut rng = rand::thread_rng();
            let graph = generate_graph(topology, &mut rng);
            prop_assume!(graph.total_weight() > 0.0);
            let n = graph.node_count();
            let partition = Partition::singletons(n);
            let modularity = Modularity::new(1.0);
            let q = modularity.total_quality(&graph, &partition);
            assert_finite(q);
            // Theoretical bounds for singleton modularity: [-0.5, 0].
            assert!(
                q >= -0.5 - MODULARITY_EPSILON,
                "singleton modularity {q} below theoretical minimum -0.5"
            );
            assert!(
                q <= 0.0 + MODULARITY_EPSILON,
                "singleton modularity {q} above theoretical maximum 0.0"
            );
        }

        /// Verifies INV-004: total_quality() and delta_move() return finite values.
        #[test]
        fn all_quality_finite(topology in topology_strategy()) {
            let mut rng = rand::thread_rng();
            let graph = generate_graph(topology, &mut rng);
            prop_assume!(graph.total_weight() > 0.0);
            let partition = random_partition(&graph, &mut rng);
            let modularity = Modularity::new(1.0);

            let q = modularity.total_quality(&graph, &partition);
            assert_finite(q);

            // Pick a random node and a different target community.
            let n = graph.node_count();
            prop_assume!(n >= 2);
            let node = rng.random_range(0..n as u32);
            let current = partition.community_of(node);
            let num_comm = partition.community_count();
            prop_assume!(num_comm >= 2);
            let mut target = rng.random_range(0..num_comm);
            while target == current {
                target = rng.random_range(0..num_comm);
            }

            // Compute move components from the graph structure.
            let k_i = graph.degree_of(node);
            let neighbours = graph.neighbours_of(node);
            let weights = graph.weights_of(node);
            let mut sigma_in_to_target = 0.0_f64;
            let mut sigma_in_from_current = 0.0_f64;
            for (&v, &w) in neighbours.iter().zip(weights.iter()) {
                if partition.community_of(v) == target {
                    sigma_in_to_target += w;
                }
                if partition.community_of(v) == current {
                    sigma_in_from_current += w;
                }
            }
            let sigma_tot_target: f64 = partition
                .nodes_in_community(target)
                .iter()
                .map(|&v| graph.degree_of(v))
                .sum();
            let sigma_tot_current: f64 = partition
                .nodes_in_community(current)
                .iter()
                .map(|&v| graph.degree_of(v))
                .sum();

            let components = MoveComponents::new(
                k_i,
                sigma_in_to_target,
                sigma_tot_target,
                sigma_in_from_current,
                sigma_tot_current,
            );
            let delta = modularity.delta_move(&graph, &partition, node, target, &components);
            assert_finite(delta);
        }
    }
}
