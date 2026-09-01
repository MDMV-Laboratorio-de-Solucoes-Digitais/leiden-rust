//! Partition data structure representing community assignments and statistics.

// ref: Traag 2019 §3 — Partition representation and modularity tracking.

use std::collections::HashMap;

use crate::graph::NodeId;

/// An assignment of every internal node index to a community id, plus the
/// derived per-community statistics used by every phase.
///
/// `assignment` is dense: `assignment[i] = community of node i`. Community
/// ids are dense `u32` starting at `0` and renumbered after each phase.
///
/// `sigma_tot[c]` is the sum of degrees of nodes in community `c`.
/// `sigma_in[c]` is the sum of internal edge weights of community `c`.
#[derive(Debug, Clone, PartialEq)]
pub struct Partition {
    pub(crate) assignment: Vec<u32>,
    pub(crate) sigma_in: Vec<f64>,
    pub(crate) sigma_tot: Vec<f64>,
    pub(crate) internal_to_community: Vec<Vec<u32>>,
    pub(crate) community_count: u32,
}

impl Partition {
    /// Build the singleton partition (every node in its own community).
    #[must_use]
    pub fn singletons(node_count: usize) -> Self {
        let count: u32 = u32::try_from(node_count).unwrap_or_default();
        let assignment: Vec<u32> = (0..count).collect();
        let internal_to_community: Vec<Vec<u32>> = (0..count).map(|i| vec![i]).collect();
        let sigma_in = vec![0.0; node_count];
        let sigma_tot = vec![0.0; node_count];

        Self {
            assignment,
            sigma_in,
            sigma_tot,
            internal_to_community,
            community_count: count,
        }
    }

    /// Build the singleton partition with initial node degrees.
    #[must_use]
    pub fn singletons_with_degrees(degrees: &[f64]) -> Self {
        let mut part = Self::singletons(degrees.len());
        part.sigma_tot.copy_from_slice(degrees);
        part
    }

    /// Build the singleton partition from a CSR graph.
    #[must_use]
    pub fn singletons_from_graph<Id: NodeId>(graph: &crate::graph::CsrGraph<Id>) -> Self {
        Self::singletons_with_degrees(&graph.degrees)
    }

    /// Number of distinct communities.
    #[must_use]
    pub const fn community_count(&self) -> u32 {
        self.community_count
    }

    /// Look up the community of an internal node.
    #[must_use]
    pub fn community_of(&self, node: u32) -> u32 {
        let Ok(idx) = usize::try_from(node) else {
            return 0;
        };
        self.assignment.get(idx).copied().unwrap_or(0)
    }

    /// Get all nodes belonging to a community.
    #[must_use]
    pub fn nodes_in_community(&self, community: u32) -> &[u32] {
        let Ok(idx) = usize::try_from(community) else {
            return &[];
        };
        match self.internal_to_community.get(idx) {
            Some(nodes) => nodes,
            None => &[],
        }
    }

    /// Move a node to a target community, expanding community list if needed.
    pub fn move_node(&mut self, node: u32, to: u32) {
        let Ok(node_idx) = usize::try_from(node) else {
            return;
        };
        if node_idx >= self.assignment.len() {
            return;
        }
        let from = self.assignment[node_idx];
        if from == to {
            return;
        }

        // Remove node from previous community
        if let Some(nodes) = usize::try_from(from)
            .ok()
            .and_then(|idx| self.internal_to_community.get_mut(idx))
        {
            nodes.retain(|&n| n != node);
        }

        // Ensure capacity for target community
        if to >= self.community_count {
            let new_count = to.saturating_add(1);
            let Ok(add_len) = usize::try_from(new_count.saturating_sub(self.community_count))
            else {
                return;
            };
            self.internal_to_community
                .resize_with(self.internal_to_community.len() + add_len, Vec::new);
            self.sigma_in.resize(self.sigma_in.len() + add_len, 0.0);
            self.sigma_tot.resize(self.sigma_tot.len() + add_len, 0.0);
            self.community_count = new_count;
        }

        if let Some(nodes) = usize::try_from(to)
            .ok()
            .and_then(|idx| self.internal_to_community.get_mut(idx))
        {
            nodes.push(node);
        }
        self.assignment[node_idx] = to;
    }

    /// Move a node to a target community with incremental update to `sigma_in` and `sigma_tot`.
    pub fn move_node_with_stats(
        &mut self,
        node: u32,
        to: u32,
        k_i: f64,
        sigma_in_from: f64,
        sigma_in_to: f64,
    ) {
        let Ok(node_idx) = usize::try_from(node) else {
            return;
        };
        if node_idx >= self.assignment.len() {
            return;
        }
        let from = self.assignment[node_idx];
        if from == to {
            return;
        }

        self.move_node(node, to);

        let from_idx = usize::try_from(from).ok();
        let to_idx = usize::try_from(to).ok();

        if let Some(idx) = from_idx {
            if let Some(s_tot) = self.sigma_tot.get_mut(idx) {
                *s_tot -= k_i;
            }
            if let Some(s_in) = self.sigma_in.get_mut(idx) {
                *s_in = 2.0_f64.mul_add(-sigma_in_from, *s_in);
            }
        }

        if let Some(idx) = to_idx {
            if let Some(s_tot) = self.sigma_tot.get_mut(idx) {
                *s_tot += k_i;
            }
            if let Some(s_in) = self.sigma_in.get_mut(idx) {
                *s_in = 2.0_f64.mul_add(sigma_in_to, *s_in);
            }
        }
    }

    /// Renumber non-empty communities to dense `0..k` indices.
    pub fn renumber(&mut self) {
        let mut old_to_new = HashMap::new();
        let mut new_internal_to_comm = Vec::new();
        let mut new_sigma_in = Vec::new();
        let mut new_sigma_tot = Vec::new();

        let mut next_id = 0_u32;
        for (old_c, nodes) in self.internal_to_community.iter().enumerate() {
            if !nodes.is_empty() {
                if let Ok(old_c_u32) = u32::try_from(old_c) {
                    let _ = old_to_new.insert(old_c_u32, next_id);
                }
                new_internal_to_comm.push(nodes.clone());
                new_sigma_in.push(self.sigma_in.get(old_c).copied().unwrap_or(0.0));
                new_sigma_tot.push(self.sigma_tot.get(old_c).copied().unwrap_or(0.0));
                next_id = next_id.saturating_add(1);
            }
        }

        for c in &mut self.assignment {
            if let Some(&new_c) = old_to_new.get(c) {
                *c = new_c;
            }
        }

        self.internal_to_community = new_internal_to_comm;
        self.sigma_in = new_sigma_in;
        self.sigma_tot = new_sigma_tot;
        self.community_count = next_id;
    }

    /// True iff this partition is a refinement of `other` (every community
    /// of `self` is a subset of some community of `other`).
    #[must_use]
    pub fn is_refinement_of(&self, other: &Self) -> bool {
        for nodes in &self.internal_to_community {
            let Some(&first_node) = nodes.first() else {
                continue;
            };
            let parent_community = other.community_of(first_node);
            for &node in nodes {
                if other.community_of(node) != parent_community {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::float_cmp,
        clippy::uninlined_format_args,
        reason = "test assertions on exact float values"
    )]

    use super::Partition;

    #[test]
    fn singletons_produces_n_communities() {
        let part = Partition::singletons(5);
        assert_eq!(part.community_count(), 5);
        for i in 0..5_u32 {
            assert_eq!(part.community_of(i), i);
            assert_eq!(part.nodes_in_community(i), &[i]);
        }
    }

    #[test]
    fn move_node_updates_sigma_in_and_tot_incrementally() {
        let degrees = vec![2.0, 3.0, 2.0, 1.0];
        let mut part = Partition::singletons_with_degrees(&degrees);

        assert_eq!(part.sigma_tot[0], 2.0);
        assert_eq!(part.sigma_tot[1], 3.0);

        // Move node 0 to community 1 with edge weight 1.0 between node 0 and community 1
        part.move_node_with_stats(0, 1, 2.0, 0.0, 1.0);

        assert_eq!(part.community_of(0), 1);
        assert_eq!(part.sigma_tot[0], 0.0);
        assert_eq!(part.sigma_tot[1], 5.0);
        assert_eq!(part.sigma_in[1], 2.0);
    }

    #[test]
    fn is_refinement_of_detects_subsets() {
        // Parent partition: 4 nodes, {0, 1} in comm 0, {2, 3} in comm 1
        let mut parent = Partition::singletons(4);
        parent.move_node(1, 0);
        parent.move_node(3, 2);
        parent.renumber();

        // Singletons is a refinement of parent
        let child = Partition::singletons(4);
        assert!(child.is_refinement_of(&parent));

        // Same partition is a refinement of itself
        assert!(parent.is_refinement_of(&parent));

        // Cross-cutting partition: {0, 2} in comm 0, {1, 3} in comm 1
        let mut non_child = Partition::singletons(4);
        non_child.move_node(2, 0);
        non_child.move_node(3, 1);
        non_child.renumber();
        assert!(!non_child.is_refinement_of(&parent));
    }
}
