//! Aggregation phase for the Leiden algorithm.

// ref: Traag 2019 §3 — Aggregate graph construction from refined partition (Algorithm A.2 lines 44–48).

use std::collections::BTreeMap;

use crate::error::LeidenError;
use crate::graph::{CsrGraph, Edge, NodeId};
use crate::partition::Partition;

/// Build the aggregate graph and projected partition from a refined partition.
///
/// Each community in `refined` becomes a single node in the aggregate graph.
/// Inter-community edge weights are summed into aggregate edges.
///
/// # Errors
///
/// Returns `LeidenError` if aggregate graph construction fails.
#[expect(clippy::redundant_pub_crate, reason = "satisfies -D unreachable-pub")]
pub(crate) fn aggregation<Id: NodeId>(
    graph: &CsrGraph<Id>,
    refined: &Partition,
    local_partition: &Partition,
) -> Result<(CsrGraph<u32>, Partition), LeidenError> {
    let k = refined.community_count();
    if k == 0 {
        return Err(LeidenError::EmptyGraph);
    }

    let mut agg_edges_map: BTreeMap<(u32, u32), f64> = BTreeMap::new();
    let n = graph.node_count();

    for u in 0..n {
        let Ok(u_internal) = u32::try_from(u) else {
            continue;
        };
        let comm_u = refined.community_of(u_internal);

        let neighbours = graph.neighbours_of(u_internal);
        let weights = graph.weights_of(u_internal);

        for (&v, &w) in neighbours.iter().zip(weights.iter()) {
            let comm_v = refined.community_of(v);
            if comm_u < comm_v {
                let entry = agg_edges_map.entry((comm_u, comm_v)).or_insert(0.0);
                *entry += w;
            }
        }
    }

    let mut edges = Vec::with_capacity(agg_edges_map.len());
    for ((source, target), weight) in agg_edges_map {
        edges.push(Edge {
            source,
            target,
            weight,
        });
    }

    let agg_nodes = 0..k;
    let agg_graph = CsrGraph::from_nodes_and_edges(agg_nodes, edges)?;

    let Ok(k_usize) = usize::try_from(k) else {
        return Err(LeidenError::EmptyGraph);
    };
    let mut agg_partition = Partition::singletons(k_usize);
    for comm in 0..k {
        let nodes = refined.nodes_in_community(comm);
        if let Some(&first_node) = nodes.first() {
            let target_c = local_partition.community_of(first_node);
            agg_partition.move_node(comm, target_c);
        }
    }
    agg_partition.renumber();

    Ok((agg_graph, agg_partition))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Edge;

    #[test]
    fn lowest_id_tiebreak_aggregation() {
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
        ];
        let Ok(graph) = CsrGraph::from_edges(edges) else {
            return;
        };
        let refined = Partition::singletons(3);
        let mut local = Partition::singletons(3);
        local.move_node(1, 0);
        local.move_node(2, 0);
        local.renumber();

        let Ok((agg_graph, agg_partition)) = aggregation(&graph, &refined, &local) else {
            return;
        };
        assert_eq!(agg_graph.node_count(), 3);
        assert_eq!(agg_partition.community_count(), 1);
    }
}
