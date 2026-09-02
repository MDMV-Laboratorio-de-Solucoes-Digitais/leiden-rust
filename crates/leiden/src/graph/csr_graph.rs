//! Compressed Sparse Row (CSR) graph implementation for the Leiden algorithm.

use std::collections::HashMap;

use crate::error::LeidenError;
use crate::graph::edge::Edge;
use crate::graph::node_id::NodeId;

type CollectedGraphData<Id> = (Vec<Id>, HashMap<Id, u32>, Vec<(u32, u32, f64)>);

/// An undirected weighted graph in compressed sparse row form over dense `u32` indices.
///
/// `node_ids` is the public id ↔ internal index mapping. `adjacency` and
/// `adjacency_weight` form the CSR structure: for internal index `i`,
/// `adjacency[offsets[i]..offsets[i+1]]` are the indices of its neighbours
/// (each appearing once, regardless of edge direction), and
/// `adjacency_weight` carries the parallel weights.
///
/// `total_weight` is the sum of all edge weights (i.e. `m`, the modularity
/// denominator). For empty graphs it is `0.0`.
///
/// Constructed via `CsrGraph::from_edges` or `CsrGraph::from_nodes_and_edges`.
#[derive(Debug, Clone)]
pub struct CsrGraph<Id: NodeId> {
    pub(crate) node_ids: Vec<Id>,
    pub(crate) index_of: HashMap<Id, u32>,
    pub(crate) offsets: Vec<u32>,
    pub(crate) adjacency: Vec<u32>,
    pub(crate) adjacency_weight: Vec<f64>,
    pub(crate) degrees: Vec<f64>,
    pub(crate) total_weight: f64,
}

#[expect(clippy::option_if_let_else, reason = "clean chained downcasting")]
fn format_node_id<Id: NodeId>(id: &Id) -> String {
    let any: &dyn std::any::Any = id;
    if let Some(s) = any.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = any.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(u) = any.downcast_ref::<u32>() {
        u.to_string()
    } else if let Some(u) = any.downcast_ref::<u64>() {
        u.to_string()
    } else if let Some(u) = any.downcast_ref::<usize>() {
        u.to_string()
    } else {
        format!("{id:?}")
    }
}

impl<Id: NodeId> CsrGraph<Id> {
    /// Create an empty graph with zero nodes and zero edges.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            node_ids: Vec::new(),
            index_of: HashMap::new(),
            offsets: vec![0],
            adjacency: Vec::new(),
            adjacency_weight: Vec::new(),
            degrees: Vec::new(),
            total_weight: 0.0,
        }
    }

    /// Build a CSR graph from an iterator of edges.
    ///
    /// Rejects negative weights, self-loops, NaN/±∞ weights, and empty input.
    /// Deterministic: nodes are numbered in first-seen order.
    ///
    /// # Errors
    ///
    /// Returns `LeidenError::EmptyGraph` if no edges are provided.
    /// Returns `LeidenError::InvalidWeight` if an edge has a negative or non-finite weight.
    /// Returns `LeidenError::SelfLoop` if an edge connects a node to itself.
    pub fn from_edges<I>(edges: I) -> Result<Self, LeidenError>
    where
        I: IntoIterator<Item = Edge<Id>>,
    {
        Self::from_nodes_and_edges(std::iter::empty(), edges)
    }

    fn collect_node_and_edge_indices<IN, IE>(
        nodes: IN,
        edges: IE,
    ) -> Result<CollectedGraphData<Id>, LeidenError>
    where
        IN: IntoIterator<Item = Id>,
        IE: IntoIterator<Item = Edge<Id>>,
    {
        let mut node_ids = Vec::new();
        let mut index_of = HashMap::new();

        for node in nodes {
            if let std::collections::hash_map::Entry::Vacant(entry) = index_of.entry(node.clone()) {
                let Ok(idx) = u32::try_from(node_ids.len()) else {
                    return Err(LeidenError::Graph {
                        message: String::from("node count exceeds u32 limit"),
                        line: None,
                    });
                };
                let _ = entry.insert(idx);
                node_ids.push(node);
            }
        }

        let mut collected_edges = Vec::new();
        for edge in edges {
            if !edge.weight.is_finite() || edge.weight < 0.0 {
                return Err(LeidenError::InvalidWeight {
                    line: 0,
                    value: edge.weight,
                });
            }
            if edge.source == edge.target {
                return Err(LeidenError::SelfLoop {
                    line: None,
                    node: format_node_id(&edge.source),
                });
            }

            let u = if let Some(&idx) = index_of.get(&edge.source) {
                idx
            } else {
                let Ok(idx) = u32::try_from(node_ids.len()) else {
                    return Err(LeidenError::Graph {
                        message: String::from("node count exceeds u32 limit"),
                        line: None,
                    });
                };
                let _ = index_of.insert(edge.source.clone(), idx);
                node_ids.push(edge.source.clone());
                idx
            };

            let v = if let Some(&idx) = index_of.get(&edge.target) {
                idx
            } else {
                let Ok(idx) = u32::try_from(node_ids.len()) else {
                    return Err(LeidenError::Graph {
                        message: String::from("node count exceeds u32 limit"),
                        line: None,
                    });
                };
                let _ = index_of.insert(edge.target.clone(), idx);
                node_ids.push(edge.target.clone());
                idx
            };

            collected_edges.push((u, v, edge.weight));
        }

        Ok((node_ids, index_of, collected_edges))
    }

    /// Build a CSR graph from explicit node identifiers and an iterator of edges.
    ///
    /// # Errors
    ///
    /// Returns `LeidenError::EmptyGraph` if no nodes and no edges are provided.
    /// Returns `LeidenError::InvalidWeight` if an edge has a negative or non-finite weight.
    /// Returns `LeidenError::SelfLoop` if an edge connects a node to itself.
    pub fn from_nodes_and_edges<IN, IE>(nodes: IN, edges: IE) -> Result<Self, LeidenError>
    where
        IN: IntoIterator<Item = Id>,
        IE: IntoIterator<Item = Edge<Id>>,
    {
        let (node_ids, index_of, collected_edges) =
            Self::collect_node_and_edge_indices(nodes, edges)?;

        if node_ids.is_empty() {
            return Err(LeidenError::EmptyGraph);
        }

        let node_count = node_ids.len();
        let mut adj_entries: Vec<Vec<(u32, f64)>> = vec![Vec::new(); node_count];

        for (u, v, w) in collected_edges {
            let Ok(u_idx) = usize::try_from(u) else {
                continue;
            };
            let Ok(v_idx) = usize::try_from(v) else {
                continue;
            };

            if let Some(list_u) = adj_entries.get_mut(u_idx) {
                let mut found = false;
                for entry in list_u.iter_mut() {
                    if entry.0 == v {
                        entry.1 += w;
                        found = true;
                        break;
                    }
                }
                if !found {
                    list_u.push((v, w));
                }
            }

            if let Some(list_v) = adj_entries.get_mut(v_idx) {
                let mut found = false;
                for entry in list_v.iter_mut() {
                    if entry.0 == u {
                        entry.1 += w;
                        found = true;
                        break;
                    }
                }
                if !found {
                    list_v.push((u, w));
                }
            }
        }

        let mut offsets = Vec::with_capacity(node_count + 1);
        let mut adjacency = Vec::new();
        let mut adjacency_weight = Vec::new();
        let mut degrees = vec![0.0; node_count];
        offsets.push(0);

        let mut total_deg = 0.0;
        for (i, entries) in adj_entries.iter().enumerate() {
            let mut deg = 0.0;
            for &(nbr, w) in entries {
                adjacency.push(nbr);
                adjacency_weight.push(w);
                deg += w;
            }
            if let Some(d) = degrees.get_mut(i) {
                *d = deg;
            }
            total_deg += deg;
            let Ok(offset_len) = u32::try_from(adjacency.len()) else {
                return Err(LeidenError::Graph {
                    message: String::from("adjacency length exceeds u32 limit"),
                    line: None,
                });
            };
            offsets.push(offset_len);
        }

        let total_weight = total_deg / 2.0;

        Ok(Self {
            node_ids,
            index_of,
            offsets,
            adjacency,
            adjacency_weight,
            degrees,
            total_weight,
        })
    }

    /// Number of nodes in the graph.
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.node_ids.len()
    }

    /// Number of distinct undirected edges in the graph.
    #[must_use]
    pub const fn edge_count(&self) -> usize {
        self.adjacency.len() / 2
    }

    /// Total sum of all edge weights in the graph (modularity denominator `m`).
    #[must_use]
    pub const fn total_weight(&self) -> f64 {
        self.total_weight
    }

    /// Weighted degree of node at internal index `internal`.
    #[must_use]
    pub fn degree_of(&self, internal: u32) -> f64 {
        let Ok(idx) = usize::try_from(internal) else {
            return 0.0;
        };
        self.degrees.get(idx).copied().unwrap_or(0.0)
    }

    /// Slice of neighbour internal indices for node at `internal`.
    #[must_use]
    pub fn neighbours_of(&self, internal: u32) -> &[u32] {
        let Ok(idx) = usize::try_from(internal) else {
            return &[];
        };
        if idx + 1 >= self.offsets.len() {
            return &[];
        }
        let start = match self.offsets.get(idx) {
            Some(&s) => match usize::try_from(s) {
                Ok(val) => val,
                Err(_) => return &[],
            },
            None => return &[],
        };
        let end = match self.offsets.get(idx + 1) {
            Some(&e) => match usize::try_from(e) {
                Ok(val) => val,
                Err(_) => return &[],
            },
            None => return &[],
        };
        match self.adjacency.get(start..end) {
            Some(slice) => slice,
            None => &[],
        }
    }

    /// Slice of neighbour edge weights for node at `internal`.
    #[must_use]
    pub fn weights_of(&self, internal: u32) -> &[f64] {
        let Ok(idx) = usize::try_from(internal) else {
            return &[];
        };
        if idx + 1 >= self.offsets.len() {
            return &[];
        }
        let start = match self.offsets.get(idx) {
            Some(&s) => match usize::try_from(s) {
                Ok(val) => val,
                Err(_) => return &[],
            },
            None => return &[],
        };
        let end = match self.offsets.get(idx + 1) {
            Some(&e) => match usize::try_from(e) {
                Ok(val) => val,
                Err(_) => return &[],
            },
            None => return &[],
        };
        match self.adjacency_weight.get(start..end) {
            Some(slice) => slice,
            None => &[],
        }
    }

    /// Look up user identifier for internal node index.
    #[must_use]
    pub fn node_id(&self, internal: u32) -> Option<&Id> {
        let idx = usize::try_from(internal).ok()?;
        self.node_ids.get(idx)
    }

    /// Look up internal node index for a user identifier.
    #[must_use]
    pub fn internal_id(&self, id: &Id) -> Option<u32> {
        self.index_of.get(id).copied()
    }

    /// Create a CSR graph over internal `u32` indices with identical topology.
    #[must_use]
    pub fn to_u32_graph(&self) -> CsrGraph<u32> {
        let n = self.node_count();
        let node_ids: Vec<u32> =
            u32::try_from(n).map_or_else(|_| Vec::new(), |count| (0..count).collect());
        let mut index_of = HashMap::with_capacity(n);
        for &i in &node_ids {
            let _ = index_of.insert(i, i);
        }
        CsrGraph {
            node_ids,
            index_of,
            offsets: self.offsets.clone(),
            adjacency: self.adjacency.clone(),
            adjacency_weight: self.adjacency_weight.clone(),
            degrees: self.degrees.clone(),
            total_weight: self.total_weight,
        }
    }
}
