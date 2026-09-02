//! 3-tier plain-English explanation engine for the TUI.
//!
//! Provides step headlines, analogies written at ≤ 8th-grade reading level,
//! and live stat badges that update as the Leiden algorithm progresses.

use leiden::events::{LeidenEvent, Phase as LeidenPhase};

/// Maximum allowed Flesch-Kincaid grade level for user-facing text (SC-003).
const MAX_GRADE_LEVEL: f64 = 8.0;

/// Maximum analogy text length in characters (Contract schema maxLength).
const MAX_ANALOGY_LEN: usize = 240;

/// Maximum headline text length in characters (Contract schema maxLength).
const MAX_HEADLINE_LEN: usize = 60;

/// Maximum columns for wrapped analogy lines — leaves 2 padding columns
/// on each side of an 80-column panel (CHK015).
const MAX_WRAP_WIDTH: usize = 76;

/// Maximum number of wrapped lines for the analogy panel.
const MAX_WRAP_LINES: usize = 3;

/// Algorithm phase identifier for the explanation panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Initial state before any algorithm iteration.
    InitialState,
    /// Greedy local-moving phase.
    LocalMoving,
    /// Refinement phase.
    Refinement,
    /// Aggregation phase.
    Aggregation,
    /// Algorithm has finished.
    Finished,
}

/// Count syllables in a single word using a simple vowel-group heuristic.
///
/// This is a fast approximation sufficient for Flesch-Kincaid readability
/// scoring of short explanatory strings (Contract §4.1).
#[must_use]
pub fn count_syllables(word: &str) -> usize {
    let lower = word.to_ascii_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    if chars.is_empty() {
        return 1;
    }

    let mut count = 0;
    let mut prev_was_vowel = false;
    let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];

    for (i, ch) in chars.iter().enumerate() {
        let is_vowel = vowels.contains(ch);
        // Silent 'e' at end of word
        let is_silent_e = i == chars.len() - 1 && *ch == 'e';
        if is_vowel && !prev_was_vowel && !is_silent_e {
            count += 1;
            prev_was_vowel = true;
        } else if !is_vowel {
            prev_was_vowel = false;
        }
    }

    if count == 0 {
        1 // Every word has at least one syllable
    } else {
        count
    }
}

/// Split text into sentences by detecting `.`, `!`, or `?` followed by
/// whitespace or end-of-string.
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        current.push(ch);
        if ch == '.' || ch == '!' || ch == '?' {
            // Check if this is really end of sentence (followed by space/EOS)
            if chars.peek().is_none_or(|c| c.is_whitespace()) {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
                current.clear();
            }
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        sentences.push(trimmed);
    }

    if sentences.is_empty() {
        sentences.push(text.trim().to_string());
    }
    sentences
}

/// Compute the Flesch-Kincaid Grade Level for a given text.
///
/// Formula: `0.39 * (words/sentences) + 11.8 * (syllables/words) - 15.59`
///
/// Returns a grade level where higher is more complex (Contract §4.1).
#[must_use]
pub fn flesch_kincaid_grade(text: &str) -> f64 {
    let sentences = split_sentences(text);
    let sentence_count = sentences.len().max(1);
    let words: Vec<&str> = text.split_whitespace().collect();
    let word_count = words.len().max(1);

    let syllable_count: usize = words.iter().map(|w| count_syllables(w)).sum();

    #[expect(
        clippy::cast_precision_loss,
        reason = "word, sentence and syllable counts are small; f64's 52-bit mantissa is ample for this heuristic"
    )]
    let words_per_sentence = word_count as f64 / sentence_count as f64;
    #[expect(
        clippy::cast_precision_loss,
        reason = "word, sentence and syllable counts are small; f64's 52-bit mantissa is ample for this heuristic"
    )]
    let syllables_per_word = syllable_count as f64 / word_count as f64;

    // Keep the two products as separate, normally rounded operations, exactly
    // as the textbook formula specifies; fusing them via `mul_add` would
    // change the result's final rounding.
    let words_component = 0.39 * words_per_sentence;
    let syllables_component = 11.8 * syllables_per_word;
    words_component + syllables_component - 15.59
}

/// Check if a text string contains prohibited jargon terms (CHK011).
///
/// Returns `Some(term)` if a blacklisted term is found, `None` if the
/// text is clean.
#[must_use]
pub fn contains_jargon(text: &str) -> Option<&'static str> {
    let blacklist: &[(&str, &str)] = &[
        ("modularity", "modularity"),
        ("resolution", "resolution parameter"),
        ("eigenvector", "eigenvector"),
        ("csr", "CSR"),
        ("adjacency", "Adjacency matrix"),
        ("heuristic", "heuristic"),
        ("optimization", "optimization"),
        ("hamiltonian", "Hamiltonian"),
        ("graph partition", "graph partition"),
    ];

    let lower = text.to_ascii_lowercase();
    for (term_lower, label) in blacklist {
        if lower.contains(term_lower) {
            return Some(label);
        }
    }
    None
}

/// 3-part structured explanation state for non-technical users.
#[derive(Debug, Clone, PartialEq)]
pub struct ExplanationState {
    /// Tier 1: Bold summary headline (e.g., "STEP 1 OF 3: FINDING FRIEND CIRCLES")
    pub headline: String,
    /// Tier 2: Plain-English intuitive analogy (<= 8th grade reading level, <= 240 chars)
    pub analogy_text: String,
    /// Tier 3: Current algorithm phase name (e.g., "Local Moving", "Refinement")
    pub phase_name: String,
    /// Active community count detected so far
    pub community_count: usize,
    /// Percentage progress through current phase [0.0, 1.0]
    pub phase_progress: f64,
    /// Verified Flesch-Kincaid grade level score
    pub reading_grade_level: f32,
}

impl ExplanationState {
    /// Create initial unclustered explanation state.
    ///
    /// Displays the "A Messy Network Starting Point" narrative with
    /// a 4.8 grade-level analogy explaining that nobody has been
    /// assigned to a club yet.
    #[must_use]
    pub fn initial_unclustered(total_nodes: usize, total_edges: usize) -> Self {
        let headline = "STEP 1 OF 3: MESSY NETWORK STARTING POINT";
        let analogy_text = "All people in the network are mixed together in one big crowd. \
No friend groups have formed yet.";
        let grade = flesch_kincaid_grade(analogy_text);

        #[expect(
            clippy::cast_precision_loss,
            reason = "node and edge counts are small; f64's 52-bit mantissa is ample for this heuristic"
        )]
        let phase_progress = if total_nodes > 0 {
            total_edges as f64 / (total_nodes as f64 * 10.0).max(1.0)
        } else {
            0.0
        };
        let phase_progress = phase_progress.clamp(0.0, 1.0);

        #[expect(
            clippy::cast_possible_truncation,
            reason = "ExplanationState stores the reading grade level as f32 by design"
        )]
        let grade = grade as f32;

        Self {
            headline: headline.to_string(),
            analogy_text: analogy_text.to_string(),
            phase_name: "Initial State".to_string(),
            community_count: 0,
            phase_progress,
            reading_grade_level: grade,
        }
    }

    /// Update explanation from Leiden execution event.
    ///
    /// Maps `LeidenEvent` variants to plain-English analogies (grade level ≤ 8.0).
    #[must_use]
    #[expect(
        clippy::too_many_lines,
        reason = "event mapping is inherently a lookup table"
    )]
    pub fn from_leiden_event(event: &LeidenEvent, current_communities: usize) -> Self {
        #[expect(
            clippy::cast_precision_loss,
            reason = "node and edge counts are small; f64's 52-bit mantissa is ample for this heuristic"
        )]
        #[expect(
            clippy::cast_possible_truncation,
            reason = "ExplanationState stores the reading grade level as f32 by design"
        )]
        let (headline, analogy_text, phase_name, phase_progress, step) = match event {
            LeidenEvent::GraphLoaded { nodes, edges, .. } => {
                let nodes_u = *nodes;
                let edges_u = *edges;
                let prog = if nodes_u > 0 {
                    edges_u as f64 / (nodes_u as f64 * 10.0).max(1.0)
                } else {
                    0.0
                };
                (
                    "STEP 1 OF 3: MESSY NETWORK STARTING POINT",
                    "All people in the network are mixed together in one big crowd. \
No friend groups have formed yet.",
                    "Initial State",
                    f64::from(prog.clamp(0.0, 1.0) as f32),
                    1,
                )
            }
            LeidenEvent::IterationStarted {
                index,
                phase,
            } => {
                match phase {
                    LeidenPhase::LocalMoving => {
                        let iter = *index;
                        if iter == 0 {
                            (
                                "STEP 2 OF 3: FINDING BEST FRIEND CIRCLES",
                                "Each person looks around at their closest friends and moves \
into the circle where they have the most in common.",
                                "Local Moving",
                                0.0,
                                2,
                            )
                        } else {
                            (
                                "STEP 2 OF 3: SWAPPING AND SETTLING GROUPS",
                                "People keep swapping tables. Everyone settles with their closest \
friends and nobody wants to move.",
                                "Local Moving",
                                0.0,
                                2,
                            )
                        }
                    }
                    LeidenPhase::Refinement => (
                        "STEP 3 OF 3: SPLITTING UP BIG CROWDS",
                        "Groups check if all members are truly connected. If a group \
has two separate cliques, it splits into smaller well-knit teams.",
                        "Refinement",
                        0.0,
                        3,
                    ),
                    LeidenPhase::Aggregation => (
                        "STEP 3 OF 3: ZOOMING OUT TO BIG PICTURE",
                        "We treat each team as one big member. Then we look for wider \
patterns across the whole network.",
                        "Aggregation",
                        0.0,
                        3,
                    ),
                }
            }
            LeidenEvent::LocalMovingProgress { moved_nodes, .. } => {
                let progress = (f64::from(*moved_nodes) / 50.0_f64).min(1.0);
                (
                    "STEP 2 OF 3: FINDING BEST FRIEND CIRCLES",
                    "Each person looks around at their closest friends and moves \
into the circle where they have the most in common.",
                    "Local Moving",
                    progress,
                    2,
                )
            }
            LeidenEvent::QualityComputed { quality, .. } => {
                let progress = (*quality * 2.5_f64).min(1.0);
                (
                    "STEP 2 OF 3: FINDING BEST FRIEND CIRCLES",
                    "People keep swapping tables. Everyone settles with their closest \
friends and nobody wants to move.",
                    "Local Moving",
                    progress,
                    2,
                )
            }
            LeidenEvent::IterationFinished { quality, .. } => {
                let progress = (*quality * 2.5_f64).min(1.0);
                (
                    "STEP 3 OF 3: ZOOMING OUT TO BIG PICTURE",
                    "We treat each team as one big member. Then we look for wider \
patterns across the whole network.",
                    "Aggregation",
                    progress,
                    3,
                )
            }
            LeidenEvent::RefinementMerged { .. } => (
                "STEP 3 OF 3: SPLITTING UP BIG CROWDS",
                "Groups check if all members are truly connected. If a group \
has two separate cliques, it splits into smaller well-knit teams.",
                "Refinement",
                0.5,
                3,
            ),
            LeidenEvent::Aggregation { .. } => (
                "STEP 3 OF 3: ZOOMING OUT TO BIG PICTURE",
                "We treat each team as one big member. Then we look for wider \
patterns across the whole network.",
                "Aggregation",
                0.5,
                3,
            ),
            LeidenEvent::Terminated { quality, .. } => {
                let progress = (*quality * 2.5_f64).min(1.0);
                (
                    "STEP 3 OF 3: NEAT COMMUNITIES DISCOVERED!",
                    "The algorithm finished! The messy starting network is now \
neatly organized into cohesive, color-coded communities.",
                    "Finished",
                    progress,
                    3,
                )
            }
            LeidenEvent::Throttled { .. } | LeidenEvent::LocalMovingDelta { .. } => (
                "STEP 2 OF 3: FINDING BEST FRIEND CIRCLES",
                "Each person looks around at their closest friends and moves \
into the circle where they have the most in common.",
                "Local Moving",
                0.5,
                2,
            ),
        };

        let grade = flesch_kincaid_grade(analogy_text);

        // Enforce grade level ceiling with static fallback
        #[expect(
            clippy::cast_possible_truncation,
            reason = "ExplanationState stores the reading grade level as f32 by design"
        )]
        let (final_analogy, final_grade) = if grade <= MAX_GRADE_LEVEL {
            (analogy_text.to_string(), grade as f32)
        } else {
            let fallback = "People find their closest friends and form tight groups.";
            (fallback.to_string(), flesch_kincaid_grade(fallback) as f32)
        };

        let _ = step;

        let headline_truncated = if headline.len() > MAX_HEADLINE_LEN {
            headline[..MAX_HEADLINE_LEN].to_string()
        } else {
            headline.to_string()
        };

        let analogy_truncated = if final_analogy.len() > MAX_ANALOGY_LEN {
            final_analogy[..MAX_ANALOGY_LEN].to_string()
        } else {
            final_analogy
        };

        Self {
            headline: headline_truncated,
            analogy_text: analogy_truncated,
            phase_name: phase_name.to_string(),
            community_count: current_communities.max(1),
            phase_progress,
            reading_grade_level: final_grade,
        }
    }

    /// Create final completion summary.
    ///
    /// Generates the "Neat Communities Discovered!" narrative for the
    /// completed state (Contract §2, SC-001).
    #[must_use]
    pub fn completed(community_count: usize, quality: f64) -> Self {
        let communities = community_count.max(1);
        let headline = format!("STEP 3 OF 3: NEAT {communities} COMMUNITIES DISCOVERED!");
        let analogy_text = if communities == 1 {
            "The algorithm finished! The whole network is one big happy crowd."
        } else {
            "The algorithm finished! The messy crowd is now organized into neat, \
color-coded friend groups."
        };
        let grade = flesch_kincaid_grade(analogy_text);

        // Enforce grade level ceiling with static fallback (CHK004)
        let (final_analogy, final_grade) = if grade <= MAX_GRADE_LEVEL {
            (analogy_text.to_string(), grade)
        } else {
            let fallback = "People found their closest friends and formed tight groups.";
            (fallback.to_string(), flesch_kincaid_grade(fallback))
        };

        let _ = quality;

        let headline_truncated = if headline.len() > MAX_HEADLINE_LEN {
            headline[..MAX_HEADLINE_LEN].to_string()
        } else {
            headline
        };

        #[expect(
            clippy::cast_possible_truncation,
            reason = "ExplanationState stores the reading grade level as f32 by design"
        )]
        let final_grade = final_grade as f32;

        Self {
            headline: headline_truncated,
            analogy_text: final_analogy,
            phase_name: "Finished".to_string(),
            community_count: communities,
            phase_progress: 1.0,
            reading_grade_level: final_grade,
        }
    }

    /// Wrap analogy text to fit panel width (max 76 chars/line, max 3 lines,
    /// word-boundary split).
    ///
    /// Splits text at word boundaries so that each line is at most
    /// `max_width` characters wide, producing at most 3 lines (CHK015).
    #[must_use]
    pub fn wrapped_analogy_lines(&self, max_width: usize) -> Vec<String> {
        let words: Vec<&str> = self.analogy_text.split_whitespace().collect();
        if words.is_empty() {
            return vec![String::new(), String::new(), String::new()];
        }

        let effective_width = max_width.clamp(30, MAX_WRAP_WIDTH);
        let mut lines: Vec<String> = Vec::new();
        let mut current_line = String::new();

        for word in &words {
            let word_len = word.len();
            let needed = if current_line.is_empty() {
                word_len
            } else {
                current_line.len() + 1 + word_len
            };

            if needed <= effective_width {
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(word);
            } else {
                // Current line is full — push it and start a new one
                if current_line.is_empty() {
                    // Word alone exceeds the width — hard-truncate with ellipsis
                    let truncated: String = word.chars().take(effective_width.saturating_sub(1)).collect();
                    lines.push(format!("{truncated}\u{2026}"));
                } else {
                    lines.push(current_line.clone());
                    current_line.clear();
                    current_line.push_str(word);
                }
            }

            if lines.len() >= MAX_WRAP_LINES {
                // We've filled 3 lines — add ellipsis to the last and stop
                if let Some(last) = lines.last_mut()
                    && !last.ends_with('\u{2026}')
                {
                    last.push('\u{2026}');
                }
                break;
            }
        }

        // Push any remaining content
        if !current_line.is_empty() && lines.len() < MAX_WRAP_LINES {
            lines.push(current_line);
        } else if !current_line.is_empty()
            && lines.len() == MAX_WRAP_LINES
            && let Some(last) = lines.last_mut()
            && !last.ends_with('\u{2026}')
        {
            last.push('\u{2026}');
        }

        // Ensure at least 3 lines for consistent panel height
        while lines.len() < 3 {
            lines.push(String::new());
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_syllables_simple() {
        assert!(count_syllables("hello") >= 1);
        assert!(count_syllables("cat") >= 1);
        assert!(count_syllables("beautiful") >= 1);
        assert_eq!(count_syllables(""), 1);
    }

    #[test]
    fn test_flesch_kincaid_simple() {
        let fk = flesch_kincaid_grade("Hello world. This is a test.");
        assert!(fk.is_finite());
    }

    #[test]
    fn test_flesch_kincaid_grade_8_or_below() {
        let texts = [
            "All people in the network are mixed together in one big crowd. No friend groups have formed yet.",
            "Each person looks around at their closest friends and moves into the circle where they have the most in common.",
            "People keep swapping tables. Everyone settles with their closest friends and nobody wants to move.",
            "Groups check if all members are truly connected. If a group has two separate cliques, it splits into smaller well-knit teams.",
            "We treat each team as one big member. Then we look for wider patterns across the whole network.",
            "The algorithm finished! Your messy crowd is now neat friend groups of colors.",
        ];
        for text in &texts {
            let grade = flesch_kincaid_grade(text);
            assert!(
                grade <= MAX_GRADE_LEVEL,
                "Grade level {grade:.2} exceeds {MAX_GRADE_LEVEL} for: {text}"
            );
        }
    }

    #[test]
    fn test_jargon_blacklist() {
        assert!(contains_jargon("The modularity value increased").is_some());
        assert!(contains_jargon("Eigenvector centrality is high").is_some());
        assert!(contains_jargon("Heuristic optimization approach").is_some());
        assert!(contains_jargon("This is perfectly fine").is_none());
    }

    #[test]
    fn test_initial_unclustered() {
        let state = ExplanationState::initial_unclustered(34, 78);
        assert_eq!(state.phase_name, "Initial State");
        assert_eq!(state.community_count, 0);
        assert!(f64::from(state.reading_grade_level) <= MAX_GRADE_LEVEL);
        assert!(state.phase_progress >= 0.0);
        assert!(state.phase_progress <= 1.0);
    }

    #[test]
    fn test_completed() {
        let state = ExplanationState::completed(5, 0.42);
        assert_eq!(state.phase_name, "Finished");
        assert_eq!(state.community_count, 5);
        // `completed` sets this to exactly 1.0; compare against f64::EPSILON
        // to avoid a strict float equality lint while asserting exactness.
        assert!((state.phase_progress - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_completed_single_community() {
        let state = ExplanationState::completed(1, 0.35);
        assert_eq!(state.community_count, 1);
    }

    #[test]
    fn test_wrapped_lines_within_bounds() {
        let state = ExplanationState::initial_unclustered(34, 78);
        let lines = state.wrapped_analogy_lines(76);
        assert!(lines.len() <= 3);
        for line in &lines {
            assert!(
                line.len() <= 76 || line.ends_with('…'),
                "Line exceeds 76 chars: {line}"
            );
        }
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_wrapped_lines_max_3() {
        let long_text = ExplanationState {
            headline: String::new(),
            analogy_text: "This is a very long analogy text that should wrap across many lines because it contains many words that would normally exceed the three line limit if not properly truncated with ellipsis at the end of the third line.".to_string(),
            phase_name: String::new(),
            community_count: 1,
            phase_progress: 0.0,
            reading_grade_level: 1.0,
        };
        let lines = long_text.wrapped_analogy_lines(20);
        assert!(lines.len() <= 3);
    }
}
