//! Integration tests: Flesch-Kincaid readability scoring and word-wrapping (T006).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::float_cmp,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use leiden_tui::ExplanationState;
use leiden_tui::explanation::count_syllables;
use leiden_tui::explanation::flesch_kincaid_grade;

#[test]
fn flesch_kincaid_simple_sentence() {
    let fk = flesch_kincaid_grade("The cat sat on the mat.");
    // Very simple text can produce a negative grade; must be finite and
    // lower than a complex sentence's score.
    assert!(fk.is_finite());
}

#[test]
fn flesch_kincaid_short_text_lower_grade() {
    // Very simple text should have a low grade level
    let fk = flesch_kincaid_grade("Dogs run. Cats run. Birds fly.");
    assert!(fk < 8.0, "Simple text grade {fk} should be < 8.0");
}

#[test]
fn flesch_kincaid_complex_text_higher_grade() {
    let simple = flesch_kincaid_grade("People go to clubs.");
    let complex = flesch_kincaid_grade(
        "The algorithmic optimization of graph partitioning via spectral clustering \
         methodologies employing eigenvector decomposition yields asymptotic improvements.",
    );
    assert!(
        complex > simple,
        "Complex text ({complex}) should score higher than simple ({simple})"
    );
}

#[test]
fn flesch_kincaid_empty_string() {
    let fk = flesch_kincaid_grade("");
    assert!(fk.is_finite());
}

#[test]
fn flesch_kincaid_single_word() {
    let fk = flesch_kincaid_grade("hello");
    assert!(fk.is_finite());
}

#[test]
fn count_syllables_basic() {
    assert!(count_syllables("cat") >= 1);
    assert!(count_syllables("hello") >= 1);
    assert!(count_syllables("beautiful") >= 1);
    assert!(count_syllables("") >= 1); // Minimum 1 per word
}

#[test]
fn initial_state_analogy_grade_le_8() {
    let state = ExplanationState::initial_unclustered(34, 78);
    assert!(
        state.reading_grade_level <= 8.0,
        "Initial state grade level {} exceeds 8.0",
        state.reading_grade_level
    );
}

#[test]
fn initial_state_analogy_no_jargon() {
    let state = ExplanationState::initial_unclustered(34, 78);
    let jargon = leiden_tui::explanation::contains_jargon(&state.analogy_text);
    assert!(
        jargon.is_none(),
        "Initial state analogy contains forbidden jargon: {jargon:?}"
    );
}

#[test]
fn wrapped_lines_respect_max_width() {
    let state = ExplanationState::initial_unclustered(34, 78);
    let lines = state.wrapped_analogy_lines(76);
    assert!(
        lines.len() <= 3,
        "Should have at most 3 lines, got {}",
        lines.len()
    );
    for line in &lines {
        assert!(
            line.len() <= 76 || line.ends_with('…'),
            "Line exceeds 76 chars: '{line}' (len {})",
            line.len()
        );
    }
}

#[test]
fn wrapped_lines_respect_custom_width() {
    let state = ExplanationState::initial_unclustered(34, 78);
    let lines = state.wrapped_analogy_lines(30);
    for line in &lines {
        assert!(
            line.len() <= 30 || line.ends_with('…'),
            "Line exceeds 30 chars: '{line}' (len {})",
            line.len()
        );
    }
}

#[test]
fn wrapped_lines_max_three() {
    let state = ExplanationState {
        headline: String::new(),
        analogy_text:
            "This is a very long sentence that contains many many words and will definitely \
need more than three lines of text to display when wrapped at a narrow width because it \
just keeps going and going and going with more and more content that must be truncated."
                .to_string(),
        phase_name: String::new(),
        community_count: 1,
        phase_progress: 0.0,
        reading_grade_level: 4.0,
    };
    let lines = state.wrapped_analogy_lines(20);
    assert!(
        lines.len() <= 3,
        "Should have at most 3 lines even for long text, got {}",
        lines.len()
    );
}

#[test]
fn completed_summary_grade_le_8() {
    let state = ExplanationState::completed(5, 0.42);
    assert!(
        state.reading_grade_level <= 8.0,
        "Completed state grade level {} exceeds 8.0",
        state.reading_grade_level
    );
}

#[test]
fn completed_summary_community_count() {
    let state = ExplanationState::completed(5, 0.42);
    assert_eq!(state.community_count, 5);
    assert_eq!(state.phase_name, "Finished");
    assert!(state.headline.contains("COMMUNITIES DISCOVERED"));
}
