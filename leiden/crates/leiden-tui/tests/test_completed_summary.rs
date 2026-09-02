//! Integration tests: `ExplanationState::completed` summary and reading
//! level score (T029, US3).

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use leiden_tui::ExplanationState;
use leiden_tui::explanation::contains_jargon;

/// Contract §2: the completed headline must announce the discovered
/// community count in plain English.
#[test]
fn completed_headline_announces_communities() {
    let state = ExplanationState::completed(5, 0.42);
    assert!(
        state.headline.contains("COMMUNITIES DISCOVERED"),
        "headline must contain 'COMMUNITIES DISCOVERED', got: {}",
        state.headline
    );
    assert!(
        state.headline.contains('5'),
        "headline must contain the community count '5', got: {}",
        state.headline
    );
}

/// Contract §2: phase name for the completed state is "Finished".
#[test]
fn completed_phase_is_finished() {
    let state = ExplanationState::completed(5, 0.42);
    assert_eq!(state.phase_name, "Finished");
}

/// Contract §2 / Data Model: the completed state is always fully
/// progressed (exactly 1.0), for any input.
#[test]
fn completed_progress_is_full() {
    for &(count, quality) in &[(0_usize, 0.0_f64), (1, 0.35), (5, 0.42), (34, 0.99)] {
        let state = ExplanationState::completed(count, quality);
        assert!(
            (state.phase_progress - 1.0).abs() < 1e-9,
            "phase_progress must be exactly 1.0 for completed(count={count})"
        );
    }
}

/// Contract §2: the verified Flesch-Kincaid grade level of the completed
/// analogy must never exceed the 8.0 ceiling, for any community count.
#[test]
fn completed_grade_level_within_ceiling() {
    for &count in &[1_usize, 2, 5, 12, 34] {
        let state = ExplanationState::completed(count, 0.42);
        assert!(
            state.reading_grade_level <= 8.0,
            "grade level {} exceeds ceiling 8.0 for count={count}",
            state.reading_grade_level
        );
    }
}

/// Contract §2 / CHK011: the completed analogy must contain none of the
/// prohibited jargon terms.
#[test]
fn completed_no_jargon() {
    let state = ExplanationState::completed(5, 0.42);
    assert_eq!(
        contains_jargon(&state.analogy_text),
        None,
        "completed analogy must be jargon-free: {}",
        state.analogy_text
    );
}

/// Data Model: `community_count` is floored at one — even for zero or one
/// discovered communities the summary reports a single community.
#[test]
fn completed_single_community_floor() {
    let zero = ExplanationState::completed(0, 0.0);
    assert_eq!(
        zero.community_count, 1,
        "community_count(0) must floor to 1"
    );

    let one = ExplanationState::completed(1, 0.5);
    assert_eq!(one.community_count, 1, "community_count(1) must stay 1");
}

/// Contract §2: the completed headline stays within 60 chars and the
/// analogy within 240 chars for every community count.
#[test]
fn completed_analogy_length_bounds() {
    for &count in &[1_usize, 5, 34, 200] {
        let state = ExplanationState::completed(count, 0.42);
        assert!(
            state.analogy_text.len() <= 240,
            "analogy_text length {} exceeds 240 for count={count}: {}",
            state.analogy_text.len(),
            state.analogy_text
        );
        assert!(
            state.headline.len() <= 60,
            "headline length {} exceeds 60 for count={count}: {}",
            state.headline.len(),
            state.headline
        );
    }
}

/// Contract §2: the quality parameter must not push the reading grade
/// level past the 8.0 ceiling — neither at the high nor the low end.
#[test]
fn quality_does_not_change_grade_compliance() {
    let high = ExplanationState::completed(5, 0.99);
    assert!(
        high.reading_grade_level <= 8.0,
        "grade level {} exceeds ceiling 8.0 for quality=0.99",
        high.reading_grade_level
    );

    let low = ExplanationState::completed(5, 0.0);
    assert!(
        low.reading_grade_level <= 8.0,
        "grade level {} exceeds ceiling 8.0 for quality=0.0",
        low.reading_grade_level
    );
}
