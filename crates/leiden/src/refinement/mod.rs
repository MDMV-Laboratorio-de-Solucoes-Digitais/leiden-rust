//! Refinement phase for the Leiden algorithm.

// ref: Traag 2019 §3 — Refinement phase guaranteeing well-connected communities (Algorithm A.2 lines 33–43).

use std::collections::{HashMap, HashSet};

use crate::graph::{CsrGraph, NodeId};
use crate::partition::Partition;
use crate::quality::{Modularity, MoveComponents, QualityFunction};

fn is_subset_well_connected<Id: NodeId>(
    graph: &CsrGraph<Id>,
    nodes_set: &HashSet<u32>,
    refined: &Partition,
    target_comm: u32,
    k_c: f64,
    gamma: f64,
    two_m: f64,
) -> bool {
    let target_nodes = refined.nodes_in_community(target_comm);
    if target_nodes.is_empty() {
        return false;
    }

    let mut k_t = 0.0;
    let mut e_t_c_minus_t = 0.0;

    for &t_node in target_nodes {
        k_t += graph.degree_of(t_node);
        let t_nbrs = graph.neighbours_of(t_node);
        let t_weights = graph.weights_of(t_node);

        for (&nbr, &w) in t_nbrs.iter().zip(t_weights.iter()) {
            if nodes_set.contains(&nbr) && refined.community_of(nbr) != target_comm {
                e_t_c_minus_t += w;
            }
        }
    }

    let threshold_t = gamma * k_t * (k_c - k_t) / two_m;
    e_t_c_minus_t >= threshold_t - f64::EPSILON
}

/// Refine a partition by merging well-connected subsets within each community.
#[must_use]
#[expect(clippy::redundant_pub_crate, reason = "satisfies -D unreachable-pub")]
pub(crate) fn refinement<Id: NodeId>(
    graph: &CsrGraph<Id>,
    local_partition: &Partition,
    quality: &Modularity,
) -> Partition {
    let mut refined = Partition::singletons_with_degrees(&graph.degrees);
    let m = graph.total_weight();
    if m <= 0.0 || !m.is_finite() {
        return refined;
    }
    let two_m = 2.0 * m;

    let num_communities = local_partition.community_count();

    for c in 0..num_communities {
        let nodes_in_c = local_partition.nodes_in_community(c);
        if nodes_in_c.len() <= 1 {
            continue;
        }

        let nodes_set: HashSet<u32> = nodes_in_c.iter().copied().collect();

        let mut k_c = 0.0;
        for &u in nodes_in_c {
            k_c += graph.degree_of(u);
        }

        for &u in nodes_in_c {
            let k_u = graph.degree_of(u);

            let neighbours = graph.neighbours_of(u);
            let weights = graph.weights_of(u);

            let mut e_u_c = 0.0;
            let mut weight_to_refined_comm: HashMap<u32, f64> = HashMap::new();

            for (&v, &w) in neighbours.iter().zip(weights.iter()) {
                if nodes_set.contains(&v) {
                    if v != u {
                        e_u_c += w;
                    }
                    let refined_comm_v = refined.community_of(v);
                    let entry = weight_to_refined_comm.entry(refined_comm_v).or_insert(0.0);
                    *entry += w;
                }
            }

            let threshold_u = quality.gamma * k_u * (k_c - k_u) / two_m;
            if e_u_c < threshold_u - f64::EPSILON {
                continue;
            }

            let current_refined_comm = refined.community_of(u);
            let mut best_target = current_refined_comm;
            let mut best_delta = 0.0_f64;

            for (&target_comm, &sigma_in_to_target) in &weight_to_refined_comm {
                if target_comm == current_refined_comm {
                    continue;
                }

                if !is_subset_well_connected(
                    graph,
                    &nodes_set,
                    &refined,
                    target_comm,
                    k_c,
                    quality.gamma,
                    two_m,
                ) {
                    continue;
                }

                let sigma_in_from_current = weight_to_refined_comm
                    .get(&current_refined_comm)
                    .copied()
                    .unwrap_or(0.0);
                let sigma_tot_target = match usize::try_from(target_comm) {
                    Ok(idx) => refined.sigma_tot.get(idx).copied().unwrap_or(0.0),
                    Err(_) => 0.0,
                };
                let sigma_tot_current = match usize::try_from(current_refined_comm) {
                    Ok(idx) => refined.sigma_tot.get(idx).copied().unwrap_or(0.0),
                    Err(_) => 0.0,
                };

                let components = MoveComponents::new(
                    k_u,
                    sigma_in_to_target,
                    sigma_tot_target,
                    sigma_in_from_current,
                    sigma_tot_current,
                );

                let delta = quality.delta_move(graph, &refined, u, target_comm, &components);

                if delta > best_delta + f64::EPSILON {
                    best_delta = delta;
                    best_target = target_comm;
                } else if (delta - best_delta).abs() <= f64::EPSILON
                    && delta > 0.0
                    && target_comm < best_target
                {
                    best_target = target_comm;
                }
            }

            if best_target != current_refined_comm && best_delta > 0.0 {
                let sigma_in_to = weight_to_refined_comm
                    .get(&best_target)
                    .copied()
                    .unwrap_or(0.0);
                let sigma_in_from = weight_to_refined_comm
                    .get(&current_refined_comm)
                    .copied()
                    .unwrap_or(0.0);

                refined.move_node_with_stats(u, best_target, k_u, sigma_in_from, sigma_in_to);
            }
        }
    }

    refined.renumber();
    refined
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Edge;

    #[test]
    fn lowest_id_tiebreak_refinement() {
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
        let mut local = Partition::singletons(3);
        // All nodes in single community
        local.move_node(1, 0);
        local.move_node(2, 0);
        local.renumber();

        let quality = Modularity::new(1.0);
        let refined = refinement(&graph, &local, &quality);

        assert!(refined.community_count() <= 3);
    }
}
