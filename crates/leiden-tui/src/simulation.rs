//! 2D force-directed layout simulation engine.
//!
//! Implements spring-electrical force relaxation that visually clusters
//! nodes into communities on the Ratatui `Canvas` widget (FR-001, FR-003).

use std::collections::HashMap;

use ratatui::layout::Rect;

// --- Physical constants (fixed per Contract §3.2) ---

/// Velocity damping factor (α = 0.85) — prevents oscillation and bounds
/// convergence within ~25 relaxation ticks per phase step.
const DAMPING: f64 = 0.85;

/// Repulsion force coefficient (`k_rep`).
const REPULSION_CONSTANT: f64 = 0.005;

/// Attraction force coefficient toward community centroids (`k_attr`).
const ATTRACTION_CONSTANT: f64 = 0.25;

/// Scale factor for repulsion between nodes of the same community.
///
/// Full pairwise repulsion would overpower centroid attraction and prevent
/// clusters from contracting; same-community nodes only need enough residual
/// repulsion to respect the minimum separation distance.
const SAME_COMMUNITY_REPULSION_SCALE: f64 = 0.02;

/// Minimum separation distance to prevent overlap (`d_min` = 0.04).
const MIN_SEPARATION: f64 = 0.04;

/// Softening factor for repulsion division (ε = 0.03).
const SOFTENING_EPSILON: f64 = 0.03;

/// Lower bound for normalized virtual coordinates.
const VIRTUAL_MIN: f64 = 0.05;

/// Upper bound for normalized virtual coordinates.
const VIRTUAL_MAX: f64 = 0.95;

/// A 2D spatial coordinate or displacement vector.
///
/// Coordinates are normalized to `[0.0, 1.0]` virtual unit space and
/// clamped to `[0.05, 0.95]` during simulation to prevent clipping
/// panel borders (Contract §3.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    /// Normalized horizontal position in `[0.0, 1.0]`.
    pub x: f64,
    /// Normalized vertical position in `[0.0, 1.0]`.
    pub y: f64,
}

impl Point2D {
    /// Create a new 2D point.
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Calculate Euclidean distance to another point.
    #[must_use]
    pub fn distance_to(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx.hypot(dy)
    }

    /// Vector addition with scalar scaling: `self + vec * scalar`.
    #[must_use]
    pub fn add_scaled(self, vec: Self, scalar: f64) -> Self {
        Self {
            x: vec.x.mul_add(scalar, self.x),
            y: vec.y.mul_add(scalar, self.y),
        }
    }

    /// Clamp coordinates within a bounding box.
    #[must_use]
    pub const fn clamp(self, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self {
        Self {
            x: self.x.clamp(min_x, max_x),
            y: self.y.clamp(min_y, max_y),
        }
    }
}

/// Generate a deterministic pseudo-random float in `[0.0, 1.0)` from a seed.
///
/// Uses a simple LCG so that node seeding is fully reproducible without
/// pulling in a hashing crate (Contract §3.2 deterministic seeding).
fn deterministic_rng(seed: u64) -> impl FnMut() -> f64 {
    // LCG constants from Knuth's MMIX generator (Numerical Recipes, §7.1).
    const LCG_MULTIPLIER: u64 = 6_364_136_223_846_793_005;
    const LCG_INCREMENT: u64 = 1_442_695_040_888_963_407;

    let mut state = seed
        .wrapping_mul(LCG_MULTIPLIER)
        .wrapping_add(LCG_INCREMENT);
    move || {
        state = state
            .wrapping_mul(LCG_MULTIPLIER)
            .wrapping_add(LCG_INCREMENT);
        #[expect(
            clippy::cast_precision_loss,
            reason = "the top 31 bits of state are < 2^31, exactly representable in f64"
        )]
        let val = (state >> 33) as f64 / f64::from(u32::MAX);
        val.clamp(0.0, 0.999_999)
    }
}

/// Initialize a position in the normalized `[0.05, 0.95]` virtual space
/// using a deterministic seed based on the node index.
fn seed_position(index: usize, total: usize) -> Point2D {
    let seed = (index as u64)
        .wrapping_mul(2_654_435_761)
        .wrapping_add((total as u64).wrapping_mul(40503));
    let mut rng = deterministic_rng(seed);
    let jitter = VIRTUAL_MAX - VIRTUAL_MIN;
    let center = 0.5_f64.mul_add(VIRTUAL_MAX - VIRTUAL_MIN, VIRTUAL_MIN);
    let x = (rng() - 0.5).mul_add(jitter, center);
    let y = (rng() - 0.5).mul_add(jitter, center);
    Point2D::new(
        x.clamp(VIRTUAL_MIN, VIRTUAL_MAX),
        y.clamp(VIRTUAL_MIN, VIRTUAL_MAX),
    )
}

/// State of the 2D force-directed layout simulation.
#[derive(Debug, Clone)]
pub struct ForceSimulation {
    /// Current 2D positions of all nodes in virtual `[0.05, 0.95]` unit space.
    pub node_positions: HashMap<String, Point2D>,
    /// Current velocity vectors for smooth damping.
    pub node_velocities: HashMap<String, Point2D>,
    /// Calculated target centroids for each active community.
    pub community_centroids: HashMap<u32, Point2D>,
    /// Velocity damping coefficient (fixed: 0.85).
    pub damping: f64,
    /// Repulsion force coefficient between disjoint nodes (fixed: 0.005).
    pub repulsion_constant: f64,
    /// Attraction force coefficient toward community centroids (fixed: 0.08).
    pub attraction_constant: f64,
    /// Minimum separation distance to prevent overlap (fixed: 0.04).
    pub min_separation: f64,
}

impl ForceSimulation {
    /// Initialize simulation with deterministic seed distribution.
    #[must_use]
    pub fn new(nodes: &[String]) -> Self {
        let mut positions = HashMap::with_capacity(nodes.len());
        let mut velocities = HashMap::with_capacity(nodes.len());
        let total = nodes.len();
        for (i, node) in nodes.iter().enumerate() {
            let pos = seed_position(i, total);
            let _ = positions.insert(node.clone(), pos);
            let _ = velocities.insert(node.clone(), Point2D::new(0.0, 0.0));
        }
        Self {
            node_positions: positions,
            node_velocities: velocities,
            community_centroids: HashMap::new(),
            damping: DAMPING,
            repulsion_constant: REPULSION_CONSTANT,
            attraction_constant: ATTRACTION_CONSTANT,
            min_separation: MIN_SEPARATION,
        }
    }

    /// Advance simulation physics by one relaxation step.
    ///
    /// Computes community centroids from the partition, then applies
    /// repulsive forces between all node pairs and attractive forces
    /// toward community centroids, damping and clamping positions to
    /// the virtual `[0.05, 0.95]` bounds.
    ///
    /// One call performs exactly one relaxation step so the render loop
    /// (20 FPS / 50 ms tick) produces smooth node motion; convergence is
    /// reached naturally within the 25-step budget per phase jump due to
    /// velocity damping (Contract §3.2.4).
    pub fn tick(&mut self, partition: &[(String, u32)], edges: &[(String, String)]) {
        let _ = edges; // Edges reserved for future edge-spring force model

        // --- Compute community membership map for repulsion scaling ---
        let mut community_of: HashMap<&str, u32> = HashMap::with_capacity(partition.len());
        for (node_id, comm_id) in partition {
            let _ = community_of.insert(node_id.as_str(), *comm_id);
        }

        // --- Compute community centroids ---
        self.community_centroids.clear();
        let mut comm_positions: HashMap<u32, Vec<Point2D>> = HashMap::new();
        for (node_id, comm_id) in partition {
            if let Some(pos) = self.node_positions.get(node_id) {
                comm_positions.entry(*comm_id).or_default().push(*pos);
            }
        }
        for (&comm_id, positions) in &comm_positions {
            if !positions.is_empty() {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "community sizes are node counts (< 10^4), far below f64's 2^53 exact-integer range"
                )]
                let count = positions.len() as f64;
                let sum_x: f64 = positions.iter().map(|p| p.x).sum();
                let sum_y: f64 = positions.iter().map(|p| p.y).sum();
                let _ = self
                    .community_centroids
                    .insert(comm_id, Point2D::new(sum_x / count, sum_y / count));
            }
        }

        // --- One relaxation step: pairwise repulsion + centroid attraction ---
        let node_list: Vec<String> = self.node_positions.keys().cloned().collect();
        let mut deltas: HashMap<String, Point2D> = HashMap::with_capacity(node_list.len());

        for (i, ni) in node_list.iter().enumerate() {
            let Some(&pi) = self.node_positions.get(ni) else {
                continue;
            };
            let mut fx = 0.0_f64;
            let mut fy = 0.0_f64;

            // Repulsion from all other nodes: F_rep = k_rep / max(d^2, eps^2)
            for (j, nj) in node_list.iter().enumerate() {
                if i == j {
                    continue;
                }
                let Some(&pj) = self.node_positions.get(nj) else {
                    continue;
                };
                let dx = pi.x - pj.x;
                let dy = pi.y - pj.y;
                let dist_sq = dy.mul_add(dy, dx * dx);
                let dist = dist_sq.sqrt();
                let softened = dist.max(SOFTENING_EPSILON);

                if dist < MIN_SEPARATION {
                    // Separation displacement: push apart (zero-division guarded)
                    let push = (MIN_SEPARATION - dist) * 0.5;
                    fx = ((dx / softened) * push * self.repulsion_constant).mul_add(50.0, fx);
                    fy = ((dy / softened) * push * self.repulsion_constant).mul_add(50.0, fy);
                } else {
                    let mut force = self.repulsion_constant
                        / (dist_sq.max(SOFTENING_EPSILON * SOFTENING_EPSILON));
                    // Same-community members are held by centroid attraction;
                    // only a weak residual repulsion keeps them spaced apart,
                    // while disjoint communities repel at full strength.
                    if community_of.get(ni.as_str()) == community_of.get(nj.as_str())
                        && community_of.contains_key(ni.as_str())
                    {
                        force *= SAME_COMMUNITY_REPULSION_SCALE;
                    }
                    fx = (dx / dist).mul_add(force, fx);
                    fy = (dy / dist).mul_add(force, fy);
                }
            }

            // Attraction toward community centroid if assigned
            if let Some((_node_id, comm_id)) = partition.iter().find(|(id, _)| id == ni)
                && let Some(centroid) = self.community_centroids.get(comm_id)
            {
                let dx = centroid.x - pi.x;
                let dy = centroid.y - pi.y;
                let dist = dx.hypot(dy).max(SOFTENING_EPSILON);
                let force = self.attraction_constant * dist;
                fx = (dx / dist).mul_add(force, fx);
                fy = (dy / dist).mul_add(force, fy);
            }

            // Apply damping to velocity then accumulate delta
            if let Some(vel) = self.node_velocities.get_mut(ni) {
                vel.x = (vel.x + fx) * self.damping;
                vel.y = (vel.y + fy) * self.damping;
                let delta = deltas
                    .entry(ni.clone())
                    .or_insert_with(|| Point2D::new(0.0, 0.0));
                delta.x = vel.x;
                delta.y = vel.y;
            }
        }

        // Apply velocity displacements
        for (node_id, delta) in &deltas {
            if let Some(pos) = self.node_positions.get_mut(node_id) {
                *pos = pos.add_scaled(*delta, 1.0).clamp(
                    VIRTUAL_MIN,
                    VIRTUAL_MAX,
                    VIRTUAL_MIN,
                    VIRTUAL_MAX,
                );
            }
        }
    }

    /// Project normalized node positions to terminal screen coordinates.
    ///
    /// Maps virtual `[0.05, 0.95]` space to the pixel bounds of `area`
    /// without modifying simulation state (CHK027).
    #[must_use]
    pub fn screen_coordinates(&self, area: Rect) -> HashMap<String, (f64, f64)> {
        let mut result = HashMap::with_capacity(self.node_positions.len());
        for (node_id, pos) in &self.node_positions {
            let screen_x = pos.x.mul_add(f64::from(area.width), f64::from(area.x));
            let screen_y = pos.y.mul_add(f64::from(area.height), f64::from(area.y));
            let _ = result.insert(node_id.clone(), (screen_x, screen_y));
        }
        result
    }

    /// Reset simulation back to initial unorganized mesh.
    pub fn reset(&mut self, nodes: &[String]) {
        let total = nodes.len();
        for (i, node) in nodes.iter().enumerate() {
            let pos = seed_position(i, total);
            let _ = self.node_positions.insert(node.clone(), pos);
            let _ = self
                .node_velocities
                .insert(node.clone(), Point2D::new(0.0, 0.0));
        }
        self.community_centroids.clear();
    }
}
