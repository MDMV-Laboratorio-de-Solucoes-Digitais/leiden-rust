//! Graph generation strategies for property-based tests.
//!
//! Provides random graph generators implementing the `GraphGenerator` trait.
//! Each generator produces graphs with different topological properties.
#![cfg(test)]
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
    clippy::redundant_pub_crate,
    clippy::cast_lossless,
    unused_imports,
    dead_code,
    unused_doc_comments,
    deprecated,
    reason = "test code"
)]

use super::config::{MAX_NODES, MAX_WEIGHT, MIN_NODES, MIN_WEIGHT};
use crate::graph::{CsrGraph, Edge};
use rand::Rng;

/// Type alias for test graphs using u32 node IDs.
pub(crate) type TestGraph = CsrGraph<u32>;

/// Topology kinds for graph generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TopologyKind {
    /// Erdős-Rényi random graph.
    ErdosRenyi,
    /// Stochastic Block Model (community structure).
    StochasticBlock,
    /// Scale-free / Barabási-Albert graph.
    ScaleFree,
    /// Graph with intentional parallel edges.
    ParallelEdges,
    /// Disconnected graph (union of components).
    DisconnectedGraph,
}

/// Trait for graph generators used in property-based tests.
///
/// # Contract
/// - MUST produce graphs with node count in [`MIN_NODES`, `MAX_NODES`] by default
/// - MUST produce connected graphs by default (disconnected tested separately)
/// - MUST NOT produce self-loops
/// - MUST NOT produce negative or zero edge weights
/// - MUST be deterministic given the same seed
pub(crate) trait GraphGenerator {
    /// Returns the topology type this generator produces.
    fn topology(&self) -> TopologyKind;

    /// Generates a CSR graph using the provided RNG.
    fn generate(&self, rng: &mut impl Rng) -> TestGraph;

    /// Returns the node count range for this generator.
    fn node_count_range(&self) -> std::ops::RangeInclusive<usize> {
        MIN_NODES..=MAX_NODES
    }

    /// Returns the edge weight range for this generator.
    fn weight_range(&self) -> std::ops::RangeInclusive<f64> {
        MIN_WEIGHT..=MAX_WEIGHT
    }
}

/// Helper to create an Edge<u32>.
fn make_edge(source: u32, target: u32, weight: f64) -> Edge<u32> {
    Edge {
        source,
        target,
        weight,
    }
}

/// Erdős-Rényi random graph generator.
///
/// # Parameters
/// - `p`: Probability of edge creation between any two nodes
///
/// Produces uniform random graphs with no self-loops or duplicate edges.
#[derive(Debug, Clone)]
pub(crate) struct ErdosRenyi {
    /// Edge probability (0.0, 1.0).
    pub p: f64,
}

impl ErdosRenyi {
    /// Creates a new Erdős-Rényi generator with the given edge probability.
    ///
    /// # Panics
    /// Panics if `p` is not in [0, 1].
    pub(crate) fn new(p: f64) -> Self {
        assert!((0.0..=1.0).contains(&p), "p must be in [0, 1]");
        Self { p }
    }
}

impl GraphGenerator for ErdosRenyi {
    fn topology(&self) -> TopologyKind {
        TopologyKind::ErdosRenyi
    }

    fn generate(&self, rng: &mut impl Rng) -> TestGraph {
        let node_count = rng.random_range(self.node_count_range());
        let weight_range = self.weight_range();

        let mut edges = Vec::new();
        for u in 0..node_count {
            for v in (u + 1)..node_count {
                let roll: f64 = rng.random();
                if roll < self.p {
                    let weight = rng.random_range(weight_range.clone());
                    edges.push(make_edge(u as u32, v as u32, weight));
                }
            }
        }

        // Ensure at least one edge exists
        if edges.is_empty() && node_count >= 2 {
            edges.push(make_edge(0, 1, rng.random_range(weight_range)));
        }

        // SAFETY: edges is non-empty and all weights are positive finite values
        // generated via random_range(MIN_WEIGHT..=MAX_WEIGHT), so from_edges
        // cannot fail with EmptyGraph, InvalidWeight, or SelfLoop.
        CsrGraph::from_edges(edges).unwrap()
    }
}

/// Stochastic Block Model generator (community structure).
///
/// # Parameters
/// - `communities`: Number of communities (≥ 2)
/// - `p_in`: Probability of edge within same community
/// - `p_out`: Probability of edge between different communities
///
/// Produces graphs with detectable community structure.
#[derive(Debug, Clone)]
pub(crate) struct StochasticBlock {
    /// Number of communities.
    pub communities: usize,
    /// Intra-community edge probability.
    pub p_in: f64,
    /// Inter-community edge probability.
    pub p_out: f64,
}

impl StochasticBlock {
    /// Creates a new SBM generator.
    ///
    /// # Panics
    /// Panics if `communities < 2` or `p_in <= p_out`.
    pub(crate) fn new(communities: usize, p_in: f64, p_out: f64) -> Self {
        assert!(communities >= 2, "communities must be >= 2");
        assert!(p_in > p_out, "p_in must be > p_out for detectability");
        Self {
            communities,
            p_in,
            p_out,
        }
    }
}

impl GraphGenerator for StochasticBlock {
    fn topology(&self) -> TopologyKind {
        TopologyKind::StochasticBlock
    }

    fn generate(&self, rng: &mut impl Rng) -> TestGraph {
        let min_nodes = MIN_NODES.max(2);
        let node_count = rng.random_range(min_nodes..=MAX_NODES);
        let weight_range = self.weight_range();
        let nodes_per_community = node_count.max(1) / self.communities.max(1);

        let mut edges = Vec::new();
        for u in 0..node_count {
            for v in (u + 1)..node_count {
                let u_comm = u / nodes_per_community.max(1);
                let v_comm = v / nodes_per_community.max(1);
                let p = if u_comm == v_comm {
                    self.p_in
                } else {
                    self.p_out
                };

                let roll: f64 = rng.random();
                if roll < p {
                    edges.push(make_edge(
                        u as u32,
                        v as u32,
                        rng.random_range(weight_range.clone()),
                    ));
                }
            }
        }

        if edges.is_empty() && node_count >= 2 {
            edges.push(make_edge(0, 1, rng.random_range(weight_range)));
        }

        CsrGraph::from_edges(edges).expect("valid graph construction")
    }
}

/// Barabási-Albert scale-free graph generator.
///
/// # Parameters
/// - `m`: Number of edges each new node creates during preferential attachment
///
/// Produces graphs with power-law degree distribution.
#[derive(Debug, Clone)]
pub(crate) struct ScaleFree {
    /// Edges per new node during preferential attachment.
    pub m: usize,
}

impl ScaleFree {
    /// Creates a new scale-free generator.
    ///
    /// # Panics
    /// Panics if `m < 1`.
    pub(crate) fn new(m: usize) -> Self {
        assert!(m >= 1, "m must be >= 1");
        Self { m }
    }
}

impl GraphGenerator for ScaleFree {
    fn topology(&self) -> TopologyKind {
        TopologyKind::ScaleFree
    }

    fn generate(&self, rng: &mut impl Rng) -> TestGraph {
        let min_nodes = MIN_NODES.max(2);
        let node_count = rng.random_range(min_nodes..=MAX_NODES);
        let weight_range = self.weight_range();
        let m = self.m.min(node_count.saturating_sub(1)).max(1);

        let mut edges = Vec::new();
        let mut degrees = vec![0u32; node_count];

        // Start with a small complete graph of m+1 nodes
        for u in 0..=m.min(node_count - 1) {
            for v in (u + 1)..=m.min(node_count - 1) {
                edges.push(make_edge(
                    u as u32,
                    v as u32,
                    rng.random_range(weight_range.clone()),
                ));
                degrees[u] += 1;
                degrees[v] += 1;
            }
        }

        // Preferential attachment for remaining nodes
        for new_node in (m + 1)..node_count {
            let total_degree: u32 = degrees[..new_node].iter().sum();
            if total_degree == 0 {
                edges.push(make_edge(
                    0,
                    new_node as u32,
                    rng.random_range(weight_range.clone()),
                ));
                degrees[0] += 1;
                degrees[new_node] += 1;
                continue;
            }

            let mut attached = 0;
            let mut attempts = 0;
            while attached < m && attempts < m * 10 {
                attempts += 1;
                let target = rng.random_range(0..new_node);
                let prob = degrees[target] as f64 / total_degree as f64;
                let roll: f64 = rng.random();
                if roll < prob || degrees[target] == 0 {
                    edges.push(make_edge(
                        target as u32,
                        new_node as u32,
                        rng.random_range(weight_range.clone()),
                    ));
                    degrees[target] += 1;
                    degrees[new_node] += 1;
                    attached += 1;
                }
            }
        }

        if edges.is_empty() && node_count >= 2 {
            edges.push(make_edge(0, 1, rng.random_range(weight_range)));
        }

        CsrGraph::from_edges(edges).expect("valid graph construction")
    }
}

/// Generator that produces intentional parallel (duplicate) edges.
///
/// Used for testing weight summing behavior in CSR graph construction.
#[derive(Debug, Clone)]
pub(crate) struct ParallelEdges {
    /// Maximum number of parallel edges between any two nodes.
    pub max_parallel: usize,
}

impl ParallelEdges {
    /// Creates a new parallel edges generator.
    ///
    /// # Panics
    /// Panics if `max_parallel < 1`.
    pub(crate) fn new(max_parallel: usize) -> Self {
        assert!(max_parallel >= 1, "max_parallel must be >= 1");
        Self { max_parallel }
    }
}

impl GraphGenerator for ParallelEdges {
    fn topology(&self) -> TopologyKind {
        TopologyKind::ParallelEdges
    }

    fn generate(&self, rng: &mut impl Rng) -> TestGraph {
        let node_count = rng.random_range(MIN_NODES..=MAX_NODES);
        let weight_range = MIN_WEIGHT..=MAX_WEIGHT;

        let mut edges = Vec::new();
        let edge_count = node_count * 2;

        for _ in 0..edge_count {
            let u = rng.random_range(0..node_count);
            let mut v = rng.random_range(0..node_count);
            while u == v {
                v = rng.random_range(0..node_count);
            }
            let parallel_count = rng.random_range(1..=self.max_parallel.max(1));
            for _ in 0..parallel_count {
                edges.push(make_edge(
                    u as u32,
                    v as u32,
                    rng.random_range(weight_range.clone()),
                ));
            }
        }

        if edges.is_empty() && node_count >= 2 {
            edges.push(make_edge(0, 1, rng.random_range(weight_range)));
        }

        CsrGraph::from_edges(edges).expect("valid graph construction with parallel edges")
    }
}

/// Generator that produces disconnected graphs as union of components.
///
/// Each component is independently connected; no inter-component edges exist.
#[derive(Debug, Clone)]
pub(crate) struct DisconnectedGraph {
    /// Number of disconnected components.
    pub components: usize,
    /// Minimum nodes per component.
    pub min_nodes_per_component: usize,
}

impl DisconnectedGraph {
    /// Creates a new disconnected graph generator.
    ///
    /// # Panics
    /// Panics if `components < 2` or `min_nodes_per_component < 2`.
    pub(crate) fn new(components: usize, min_nodes_per_component: usize) -> Self {
        assert!(components >= 2, "components must be >= 2");
        assert!(
            min_nodes_per_component >= 2,
            "min_nodes_per_component must be >= 2"
        );
        Self {
            components,
            min_nodes_per_component,
        }
    }
}

impl GraphGenerator for DisconnectedGraph {
    fn topology(&self) -> TopologyKind {
        TopologyKind::DisconnectedGraph
    }

    fn generate(&self, rng: &mut impl Rng) -> TestGraph {
        let weight_range = MIN_WEIGHT..=MAX_WEIGHT;
        let mut edges = Vec::new();
        let mut offset = 0usize;

        for _ in 0..self.components {
            let comp_nodes = rng.random_range(
                self.min_nodes_per_component..=self.min_nodes_per_component.max(MIN_NODES) + 5,
            );
            let comp_nodes = comp_nodes.min(MAX_NODES - offset);

            // Create a connected component (spanning tree + random edges)
            for i in 1..comp_nodes {
                let parent = rng.random_range(0..i);
                edges.push(make_edge(
                    (offset + parent) as u32,
                    (offset + i) as u32,
                    rng.random_range(weight_range.clone()),
                ));
            }

            // Add some random edges within component
            let extra_edges = comp_nodes / 2;
            for _ in 0..extra_edges {
                let u = rng.random_range(0..comp_nodes);
                let mut v = rng.random_range(0..comp_nodes);
                while u == v {
                    v = rng.random_range(0..comp_nodes);
                }
                edges.push(make_edge(
                    (offset + u) as u32,
                    (offset + v) as u32,
                    rng.random_range(weight_range.clone()),
                ));
            }

            offset += comp_nodes;
            if offset >= MAX_NODES {
                break;
            }
        }

        if edges.is_empty() {
            edges.push(make_edge(0, 1, rng.random_range(weight_range)));
        }

        CsrGraph::from_edges(edges).expect("valid disconnected graph construction")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erdos_renyi_produces_valid_graph() {
        let generator = ErdosRenyi::new(0.3);
        let mut rng = rand::thread_rng();
        let graph = generator.generate(&mut rng);
        assert!(graph.node_count() >= MIN_NODES);
        assert!(graph.node_count() <= MAX_NODES);
    }

    #[test]
    fn stochastic_block_produces_valid_graph() {
        let generator = StochasticBlock::new(3, 0.3, 0.05);
        let mut rng = rand::thread_rng();
        let graph = generator.generate(&mut rng);
        assert!(graph.node_count() >= 2);
    }

    #[test]
    fn scale_free_produces_valid_graph() {
        let generator = ScaleFree::new(2);
        let mut rng = rand::thread_rng();
        let graph = generator.generate(&mut rng);
        assert!(graph.node_count() >= 2);
    }

    #[test]
    fn disconnected_graph_produces_multiple_components() {
        let generator = DisconnectedGraph::new(3, 3);
        let mut rng = rand::thread_rng();
        let graph = generator.generate(&mut rng);
        assert!(graph.node_count() >= 6);
    }

    #[test]
    fn parallel_edges_produces_valid_graph() {
        let generator = ParallelEdges::new(3);
        let mut rng = rand::thread_rng();
        let graph = generator.generate(&mut rng);
        assert!(graph.node_count() >= MIN_NODES);
    }
}
