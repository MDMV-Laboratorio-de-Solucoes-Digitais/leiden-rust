//! Integration tests: `ExplanationState` transitions across Leiden events (T021, US2).

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use leiden::events::Phase;
use leiden::{LeidenEvent, TerminationReason};
use leiden_tui::ExplanationState;
use leiden_tui::explanation::contains_jargon;

/// The five phase names permitted by the `ExplanationState` contract.
const CONTRACT_PHASES: [&str; 5] = [
    "Initial State",
    "Local Moving",
    "Refinement",
    "Aggregation",
    "Finished",
];

/// Build one representative event for every `LeidenEvent` variant, covering
/// all three `IterationStarted` phases (and both index 0 and index >= 1 for
/// local moving). All 10 enum variants are represented.
fn all_contract_events() -> Vec<(&'static str, LeidenEvent)> {
    vec![
        (
            "GraphLoaded",
            LeidenEvent::GraphLoaded {
                nodes: 34,
                edges: 78,
                total_weight: 156.0,
            },
        ),
        (
            "IterationStarted(index 0, LocalMoving)",
            LeidenEvent::IterationStarted {
                index: 0,
                phase: Phase::LocalMoving,
            },
        ),
        (
            "IterationStarted(index 1, LocalMoving)",
            LeidenEvent::IterationStarted {
                index: 1,
                phase: Phase::LocalMoving,
            },
        ),
        (
            "IterationStarted(Refinement)",
            LeidenEvent::IterationStarted {
                index: 1,
                phase: Phase::Refinement,
            },
        ),
        (
            "IterationStarted(Aggregation)",
            LeidenEvent::IterationStarted {
                index: 1,
                phase: Phase::Aggregation,
            },
        ),
        (
            "LocalMovingProgress",
            LeidenEvent::LocalMovingProgress {
                iteration: 0,
                moved_nodes: 25,
            },
        ),
        (
            "LocalMovingDelta",
            LeidenEvent::LocalMovingDelta {
                iteration: 0,
                delta_q: 0.0125,
            },
        ),
        (
            "RefinementMerged",
            LeidenEvent::RefinementMerged {
                iteration: 1,
                from: 2,
                to: 0,
            },
        ),
        (
            "Aggregation",
            LeidenEvent::Aggregation {
                iteration: 1,
                aggregate_nodes: 6,
            },
        ),
        (
            "QualityComputed",
            LeidenEvent::QualityComputed {
                iteration: 0,
                quality: 0.42,
            },
        ),
        (
            "IterationFinished",
            LeidenEvent::IterationFinished {
                index: 0,
                quality: 0.42,
                partition: None,
            },
        ),
        (
            "Terminated",
            LeidenEvent::Terminated {
                iterations: 3,
                reason: TerminationReason::Converged,
                quality: 0.42,
            },
        ),
        ("Throttled", LeidenEvent::Throttled { dropped: 7 }),
    ]
}

#[test]
fn graph_loaded_maps_to_initial_state() {
    let event = LeidenEvent::GraphLoaded {
        nodes: 34,
        edges: 78,
        total_weight: 156.0,
    };
    let state = ExplanationState::from_leiden_event(&event, 1);
    assert_eq!(state.phase_name, "Initial State");
    assert!(
        state.headline.contains("MESSY NETWORK"),
        "headline should mention the messy network, got: {}",
        state.headline
    );
}

#[test]
fn first_local_moving_iteration_headline() {
    let event = LeidenEvent::IterationStarted {
        index: 0,
        phase: Phase::LocalMoving,
    };
    let state = ExplanationState::from_leiden_event(&event, 1);
    assert_eq!(state.phase_name, "Local Moving");
    assert!(
        state.headline.contains("FINDING BEST FRIEND CIRCLES"),
        "index 0 headline should be about finding friend circles, got: {}",
        state.headline
    );
}

#[test]
fn second_local_moving_iteration_headline() {
    let event = LeidenEvent::IterationStarted {
        index: 1,
        phase: Phase::LocalMoving,
    };
    let state = ExplanationState::from_leiden_event(&event, 1);
    assert_eq!(state.phase_name, "Local Moving");
    assert!(
        state.headline.contains("SWAPPING AND SETTLING GROUPS"),
        "index >= 1 headline should be about swapping and settling, got: {}",
        state.headline
    );
}

#[test]
fn refinement_maps_to_splitting_headline() {
    let event = LeidenEvent::IterationStarted {
        index: 0,
        phase: Phase::Refinement,
    };
    let state = ExplanationState::from_leiden_event(&event, 1);
    assert_eq!(state.phase_name, "Refinement");
    assert!(
        state.headline.contains("SPLITTING UP BIG CROWDS"),
        "refinement headline should mention splitting crowds, got: {}",
        state.headline
    );
}

#[test]
fn aggregation_maps_to_zooming_headline() {
    let event = LeidenEvent::IterationStarted {
        index: 0,
        phase: Phase::Aggregation,
    };
    let state = ExplanationState::from_leiden_event(&event, 1);
    assert_eq!(state.phase_name, "Aggregation");
    assert!(
        state.headline.contains("ZOOMING OUT"),
        "aggregation headline should mention zooming out, got: {}",
        state.headline
    );
}

#[test]
fn terminated_maps_to_finished() {
    let event = LeidenEvent::Terminated {
        iterations: 4,
        reason: TerminationReason::IterationCap,
        quality: 0.42,
    };
    let state = ExplanationState::from_leiden_event(&event, 5);
    assert_eq!(state.phase_name, "Finished");
    assert!(
        state.headline.contains("COMMUNITIES DISCOVERED"),
        "terminated headline should announce discovered communities, got: {}",
        state.headline
    );
}

#[test]
fn phase_names_always_from_contract_enum() {
    for (label, event) in all_contract_events() {
        let state = ExplanationState::from_leiden_event(&event, 3);
        assert!(
            CONTRACT_PHASES.contains(&state.phase_name.as_str()),
            "{label}: phase_name `{}` is not one of the contract phases {CONTRACT_PHASES:?}",
            state.phase_name
        );
    }
}

#[test]
fn grade_level_never_exceeds_eight() {
    for (label, event) in all_contract_events() {
        let state = ExplanationState::from_leiden_event(&event, 3);
        assert!(
            state.reading_grade_level <= 8.0,
            "{label}: grade level {} exceeds 8.0 for analogy: {}",
            state.reading_grade_level,
            state.analogy_text
        );
    }
}

#[test]
fn no_jargon_in_any_narrative() {
    for (label, event) in all_contract_events() {
        let state = ExplanationState::from_leiden_event(&event, 3);
        assert_eq!(
            contains_jargon(&state.analogy_text),
            None,
            "{label}: analogy contains jargon: {}",
            state.analogy_text
        );
        assert_eq!(
            contains_jargon(&state.headline),
            None,
            "{label}: headline contains jargon: {}",
            state.headline
        );
    }
}

#[test]
fn headline_and_analogy_length_bounds() {
    for (label, event) in all_contract_events() {
        let state = ExplanationState::from_leiden_event(&event, 3);
        assert!(
            state.headline.len() <= 60,
            "{label}: headline is {} chars (> 60): {}",
            state.headline.len(),
            state.headline
        );
        assert!(
            state.analogy_text.len() <= 240,
            "{label}: analogy is {} chars (> 240): {}",
            state.analogy_text.len(),
            state.analogy_text
        );
    }
}

#[test]
fn community_count_floor_of_one() {
    for (label, event) in all_contract_events() {
        let state = ExplanationState::from_leiden_event(&event, 0);
        assert_eq!(
            state.community_count, 1,
            "{label}: community_count should be floored at 1, got {}",
            state.community_count
        );
    }
}

#[test]
fn progress_within_bounds() {
    for (label, event) in all_contract_events() {
        let state = ExplanationState::from_leiden_event(&event, 3);
        assert!(
            (0.0..=1.0).contains(&state.phase_progress),
            "{label}: phase_progress {} outside [0.0, 1.0]",
            state.phase_progress
        );
    }
}
