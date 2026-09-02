//! Curated demo datasets for the TUI visual explanation (FR-006).
//!
//! Provides built-in demo graphs (Karate Club, Two Cliques, Random Mess)
//! and CLI file-path loading via [`PresetDataset::from_cli_path`].

use crate::error::TuiError;

/// Identifier for available demo presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetId {
    /// Zachary's Karate Club (34 nodes, 78 edges).
    KarateClub,
    /// Two interconnected cliques (16 nodes, 56 edges).
    TwoCliques,
    /// Random messy unclustered network (30 nodes, 60 edges).
    RandomMess,
    /// Custom dataset loaded from CLI file path.
    Custom,
}

/// A curated demo dataset.
#[derive(Debug, Clone)]
pub struct PresetDataset {
    /// Unique preset identifier.
    pub id: PresetId,
    /// Display title.
    pub title: &'static str,
    /// Plain-English description.
    pub description: &'static str,
    /// Node count.
    pub node_count: usize,
    /// Edge count.
    pub edge_count: usize,
    /// Graph edges as `(source, target)` pairs.
    pub edges: Vec<(String, String)>,
}

impl PresetId {
    /// Return the title string for this preset identifier.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::KarateClub => "Zachary's Karate Club",
            Self::TwoCliques => "Two Cliques",
            Self::RandomMess => "Random Mess",
            Self::Custom => "Custom Dataset",
        }
    }

    /// Return the plain-English description for this preset.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::KarateClub => "A classic 34-node social network of a university karate club.",
            Self::TwoCliques => "Two dense cliques of eight members each.",
            Self::RandomMess => "A messy unclustered network to demonstrate the starting state.",
            Self::Custom => "A user-provided graph file loaded via CLI.",
        }
    }
}

impl PresetDataset {
    /// Build a `PresetDataset` from a list of edges, computing node count.
    fn from_edges(id: PresetId, edges: Vec<(String, String)>) -> Self {
        let mut node_set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for (src, tgt) in &edges {
            let _ = node_set.insert(src.clone());
            let _ = node_set.insert(tgt.clone());
        }
        let node_count = node_set.len();
        let edge_count = edges.len();
        let title = id.title();
        let description = id.description();
        Self {
            id,
            title,
            description,
            node_count,
            edge_count,
            edges,
        }
    }

    /// Load preset by identifier.
    ///
    /// The `Custom` identifier has no distinct built-in topology and falls
    /// back to the Random Mess dataset.
    #[must_use]
    pub fn get(id: PresetId) -> Self {
        match id {
            PresetId::KarateClub => Self::karate_club(),
            PresetId::TwoCliques => Self::two_cliques(),
            PresetId::RandomMess | PresetId::Custom => Self::random_mess(),
        }
    }

    /// Load custom dataset from CLI path; returns domain error if unreadable.
    ///
    /// Parses the file using the shared `leiden_cli::parse_graph_input` parser,
    /// extracting edges and node IDs to construct a [`PresetDataset`].
    ///
    /// # Errors
    ///
    /// Returns [`TuiError::DatasetNotFound`] if `path` cannot be read, or
    /// [`TuiError::ParseError`] if the file contents fail graph parsing.
    pub fn from_cli_path(path: &std::path::Path) -> Result<Self, TuiError> {
        let content = std::fs::read_to_string(path).map_err(|_| TuiError::DatasetNotFound {
            path: path.to_string_lossy().to_string(),
        })?;

        let path_str = path.to_string_lossy().to_string();
        let graph = leiden_cli::parse_graph_input(&content, &path_str).map_err(|err| {
            TuiError::ParseError {
                message: err.to_string(),
            }
        })?;

        let mut edges = Vec::new();
        let n = graph.node_count();
        let mut seen = std::collections::HashSet::new();

        for i in 0..n {
            let Ok(u_idx) = u32::try_from(i) else {
                continue;
            };
            let Some(id_a) = graph.node_id(u_idx) else {
                continue;
            };
            let nbrs = graph.neighbours_of(u_idx);
            for &nbr in nbrs {
                let Some(id_b) = graph.node_id(nbr) else {
                    continue;
                };
                // Deduplicate undirected edges (store canonical ordering)
                let (min, max) = if id_a <= id_b {
                    (id_a.clone(), id_b.clone())
                } else {
                    (id_b.clone(), id_a.clone())
                };
                let key = (min.clone(), max.clone());
                if seen.insert(key) {
                    edges.push((min, max));
                }
            }
        }

        Ok(Self::from_edges(PresetId::Custom, edges))
    }

    /// List all available built-in demo presets.
    #[must_use]
    pub fn all_presets() -> Vec<Self> {
        vec![
            Self::karate_club(),
            Self::two_cliques(),
            Self::random_mess(),
        ]
    }

    /// Zachary's Karate Club — 34 nodes, 78 edges.
    ///
    /// Edge list follows the canonical 78-edge Zachary (1977) dataset
    /// (0-indexed) as documented in the standard network data set.
    fn karate_club() -> Self {
        let edges = vec![
            ("0", "1"), ("0", "2"), ("0", "3"), ("0", "4"),
            ("0", "5"), ("0", "6"), ("0", "7"), ("0", "8"),
            ("0", "10"), ("0", "11"), ("0", "12"), ("0", "13"),
            ("0", "17"), ("0", "19"), ("0", "21"), ("0", "31"),
            ("1", "2"), ("1", "3"), ("1", "7"), ("1", "13"),
            ("1", "17"), ("1", "19"), ("1", "21"), ("1", "30"),
            ("2", "3"), ("2", "7"), ("2", "8"), ("2", "9"),
            ("2", "13"), ("2", "27"), ("2", "28"), ("2", "32"),
            ("3", "7"), ("3", "12"), ("3", "13"), ("4", "6"),
            ("4", "10"), ("5", "6"), ("5", "10"), ("5", "16"),
            ("6", "16"), ("8", "30"), ("8", "32"), ("8", "33"),
            ("9", "33"), ("13", "33"), ("14", "32"), ("14", "33"),
            ("15", "32"), ("15", "33"), ("18", "32"), ("18", "33"),
            ("19", "33"), ("20", "32"), ("20", "33"), ("22", "32"),
            ("22", "33"), ("23", "25"), ("23", "27"), ("23", "29"),
            ("23", "32"), ("23", "33"), ("24", "25"), ("24", "27"),
            ("24", "31"), ("25", "31"), ("26", "29"), ("26", "33"),
            ("27", "33"), ("28", "31"), ("28", "33"), ("29", "32"),
            ("29", "33"), ("30", "32"), ("30", "33"), ("31", "32"),
            ("31", "33"), ("32", "33"),
        ];
        let edge_vec: Vec<(String, String)> = edges
            .iter()
            .map(|(s, t)| (s.to_string(), t.to_string()))
            .collect();
        Self::from_edges(PresetId::KarateClub, edge_vec)
    }

    /// Two Cliques — 16 nodes (two K8 cliques), 56 edges.
    ///
    /// Two complete graphs of 8 members each (nodes 0–7 and 8–15).
    /// Each K8 contributes 28 edges, giving exactly 56 edges per CHK021.
    /// Disconnected components relax into separated cluster centroids
    /// seamlessly per Contract §3.2 (CHK023).
    fn two_cliques() -> Self {
        let mut edge_vec = Vec::new();
        // Clique A: nodes 0-7 (complete graph K8 = 28 edges)
        for i in 0..8 {
            for j in (i + 1)..8 {
                edge_vec.push((i.to_string(), j.to_string()));
            }
        }
        // Clique B: nodes 8-15 (complete graph K8 = 28 edges)
        for i in 8..16 {
            for j in (i + 1)..16 {
                edge_vec.push((i.to_string(), j.to_string()));
            }
        }
        Self::from_edges(PresetId::TwoCliques, edge_vec)
    }

    /// Random Mess — 30 nodes, 60 edges, loosely clustered.
    ///
    /// A synthetic "messy" graph built from a ring plus long-distance
    /// chords, designed to look unclustered at first glance (CHK021).
    fn random_mess() -> Self {
        const NODES: u32 = 30;
        let mut edge_vec = Vec::new();
        let mut seen = std::collections::HashSet::new();

        let push_edge = |a: u32, b: u32, out: &mut Vec<(String, String)>, seen: &mut std::collections::HashSet<(u32, u32)>| {
            let key = (a.min(b), a.max(b));
            if a != b && seen.insert(key) {
                out.push((a.to_string(), b.to_string()));
            }
        };

        // Ring edges: (i, i+1) — 30 edges
        for i in 0..NODES {
            push_edge(i, (i + 1) % NODES, &mut edge_vec, &mut seen);
        }
        // Chord edges: (i, i+7) — 30 edges, giving a messy cross-linked mesh
        for i in 0..NODES {
            push_edge(i, (i + 7) % NODES, &mut edge_vec, &mut seen);
        }

        Self::from_edges(PresetId::RandomMess, edge_vec)
    }
}
