//! Local moving phase for the Leiden algorithm.

// ref: Traag 2019 §3 — Local moving phase optimizing modularity (Algorithm A.2 lines 1–32).

use std::collections::{HashMap, VecDeque};

use crate::graph::{CsrGraph, NodeId};
use crate::partition::Partition;
use crate::quality::{Modularity, MoveComponents, QualityFunction};

/// Run the local-moving phase on a graph.
///
/// Iteratively moves nodes to neighbouring communities that maximize modularity ΔQ.
/// Returns the updated partition, whether any node was moved, and the total count of node moves.
#[expect(clippy::redundant_pub_crate, reason = "satisfies -D unreachable-pub")]
pub(crate) fn local_moving<Id: NodeId>(
    graph: &CsrGraph<Id>,
    mut partition: Partition,
    quality: &Modularity,
) -> (Partition, bool, u32) {
    let n = graph.node_count();
    if n == 0 {
        return (partition, false, 0);
    }

    let mut queue = VecDeque::with_capacity(n);
    let mut in_queue = vec![true; n];

    for i in 0..n {
        if let Ok(idx) = u32::try_from(i) {
            queue.push_back(idx);
        }
    }

    let mut moved_any = false;
    let mut total_moves = 0_u32;

    while let Some(u) = queue.pop_front() {
        let Ok(u_idx) = usize::try_from(u) else {
            continue;
        };
        if let Some(in_q) = in_queue.get_mut(u_idx) {
            *in_q = false;
        }

        let current_comm = partition.community_of(u);
        let k_u = graph.degree_of(u);

        let neighbours = graph.neighbours_of(u);
        let weights = graph.weights_of(u);

        let mut weight_to_comm: HashMap<u32, f64> = HashMap::new();
        for (&v, &w) in neighbours.iter().zip(weights.iter()) {
            let c_v = partition.community_of(v);
            let entry = weight_to_comm.entry(c_v).or_insert(0.0);
            *entry += w;
        }

        let mut candidate_communities: Vec<u32> = weight_to_comm.keys().copied().collect();
        if !candidate_communities.contains(&current_comm) {
            candidate_communities.push(current_comm);
        }

        let mut best_target = current_comm;
        let mut best_delta = 0.0_f64;

        for &target_comm in &candidate_communities {
            if target_comm == current_comm {
                continue;
            }

            let sigma_in_to_target = weight_to_comm.get(&target_comm).copied().unwrap_or(0.0);
            let sigma_in_from_current = weight_to_comm.get(&current_comm).copied().unwrap_or(0.0);
            let sigma_tot_target = match usize::try_from(target_comm) {
                Ok(idx) => partition.sigma_tot.get(idx).copied().unwrap_or(0.0),
                Err(_) => 0.0,
            };
            let sigma_tot_current = match usize::try_from(current_comm) {
                Ok(idx) => partition.sigma_tot.get(idx).copied().unwrap_or(0.0),
                Err(_) => 0.0,
            };

            let components = MoveComponents::new(
                k_u,
                sigma_in_to_target,
                sigma_tot_target,
                sigma_in_from_current,
                sigma_tot_current,
            );

            let delta = quality.delta_move(graph, &partition, u, target_comm, &components);

            if delta > best_delta + f64::EPSILON {
                best_delta = delta;
                best_target = target_comm;
            } else if (delta - best_delta).abs() <= f64::EPSILON
                && delta > 0.0
                && target_comm < best_target
            {
                // Deterministic lowest-id tie-breaking per spec.md Edge Cases
                best_target = target_comm;
            }
        }

        if best_target != current_comm && best_delta > 0.0 {
            let sigma_in_to = weight_to_comm.get(&best_target).copied().unwrap_or(0.0);
            let sigma_in_from = weight_to_comm.get(&current_comm).copied().unwrap_or(0.0);

            partition.move_node_with_stats(u, best_target, k_u, sigma_in_from, sigma_in_to);
            moved_any = true;
            total_moves = total_moves.saturating_add(1);

            for &v in neighbours {
                if partition.community_of(v) != best_target {
                    let Ok(v_idx) = usize::try_from(v) else {
                        continue;
                    };
                    if let Some(in_q) = in_queue.get_mut(v_idx).filter(|q| !**q) {
                        *in_q = true;
                        queue.push_back(v);
                    }
                }
            }
        }
    }

    partition.renumber();
    (partition, moved_any, total_moves)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Edge;

    #[test]
    fn lowest_id_tiebreak_local_moving() {
        // Path: 0 - 1 - 2
        // Node 1 is connected to 0 and 2 with equal weight 1.0.
        // Initial partition: singletons {0}, {1}, {2}.
        // When node 1 considers moving to community 0 or 2, ΔQ is identical.
        // Lowest community id (0 < 2) must be chosen.
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
        let partition = Partition::singletons_from_graph(&graph);
        let quality = Modularity::new(1.0);

        let (final_part, _, _) = local_moving(&graph, partition, &quality);
        assert_eq!(final_part.community_of(1), 0);
    }
}
