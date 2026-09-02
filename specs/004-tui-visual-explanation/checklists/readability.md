# Pedagogy & Readability Quality Checklist: TUI Visual Explanation

**Purpose**: Validate the clarity, pedagogical effectiveness, 8th-grade reading level compliance, cognitive load management, and jargon-free narrative quality of the TUI Visual Explanation feature.
**Created**: 2026-09-01
**Feature**: [spec.md](../spec.md) | [plan.md](../plan.md) | [contracts/explanation-content.md](../contracts/explanation-content.md)

**Review Ownership**: This checklist is a reviewer-owned requirements-quality review artifact. Mark an item `[x]` only when the reviewer determines the requirements-quality criterion is satisfied.
**Marker Semantics**: `[x]` means the criterion has been reviewed and satisfied for requirements quality. It does not mean implementation work is complete.

---

## 1. Flesch-Kincaid Readability & Complexity Constraints

- [x] CHK001 Is the mathematical formula for computing Flesch-Kincaid Grade Level explicitly specified with syllable-counting and sentence-splitting rules? [Measurability, Spec §SC-003, Contract §4.1]
  > **REVIEW NOTE**: Satisfied. Formula $0.39 \left(\frac{\text{words}}{\text{sentences}}\right) + 11.8 \left(\frac{\text{syllables}}{\text{words}}\right) - 15.59$ with standard syllable rules specified in Contract §4.1 and research §3.
- [x] CHK002 Is the ceiling threshold of Grade 8.0 strictly enforced across all 5 algorithm phases in the contract schema? [Completeness, Contract §1, Contract §4.1]
  > **REVIEW NOTE**: Satisfied. JSON schema enforces `"maximum": 8.0` in Contract §1, and all registered phrases score between Grade 4.8 and 7.1 (Contract §2).
- [x] CHK003 Are character length constraints defined for headline text (≤ 60 characters) and analogy text (≤ 240 characters) in the schema? [Clarity, Contract §1]
  > **REVIEW NOTE**: Satisfied. Schema defines `maxLength: 60` for headline and `maxLength: 240` for analogy_text (Contract §1).
- [x] CHK004 Is the fallback behavior specified when dynamically generated text exceeds the grade 8.0 ceiling? [Edge Case, Exception Flow, Contract §4.1]
  > **REVIEW NOTE**: Satisfied. Fallback registry strings are statically verified (Contract §2) and dynamic generators fallback to registered defaults if reading level > 8.0.
- [x] CHK005 Can the reading level calculation be verified independently in automated unit tests? [Measurability, Plan §Technical Context]
  > **REVIEW NOTE**: Satisfied. Plan §Technical Context and test suite `cargo test -p leiden-tui` test the Flesch-Kincaid calculator independently.

## 2. Pedagogical Clarity & Everyday Metaphors

- [x] CHK006 Are everyday social analogies (friend groups, lunch tables, clubs) explicitly documented for every algorithm transition? [Completeness, Contract §2, Spec §FR-004]
  > **REVIEW NOTE**: Satisfied. Complete table in Contract §2 maps Initial, Local Moving, Refinement, Aggregation, and Completed states to social analogies.
- [x] CHK007 Is the explanation for the "Refinement" phase clear in explaining why communities split without introducing graph theory terminology? [Clarity, Contract §2]
  > **REVIEW NOTE**: Satisfied. Refinement analogy explains "splitting up into smaller well-knit teams if two separate cliques exist at a table" (Contract §2).
- [x] CHK008 Is the concept of "Aggregation" (super-nodes/zoom out) explained using consistent, intuitive metaphors? [Clarity, Contract §2, Contract §3.1]
  > **REVIEW NOTE**: Satisfied. Aggregation analogy explains "treating each team as a single super-member and zooming out to see wider patterns" (Contract §2).
- [x] CHK009 Are headline summaries distinct, actionable, and non-redundant across iterative Local Moving loops? [Consistency, Contract §2]
  > **REVIEW NOTE**: Satisfied. Contract §2 provides distinct headlines for Iteration 1 vs. Iteration 2+ ("Finding Best Friend Circles" vs. "Swapping and Settling Groups").
- [x] CHK010 Does the initial state explanation clearly establish the baseline concept of an unorganized crowd? [Completeness, Spec §User Story 1, Contract §2]
  > **REVIEW NOTE**: Satisfied. Headline "A Messy Network Starting Point" and crowd analogy explicitly establish the baseline (Contract §2).

## 3. Jargon Blacklist & Terminology Enforcement

- [x] CHK011 Is the blacklist of prohibited technical terms (modularity, resolution parameter, eigenvectors, CSR, heuristic, graph partition) comprehensive and testable? [Coverage, Contract §4.2]
  > **REVIEW NOTE**: Satisfied. Blacklist is enumerated in Contract §4.2 and tested via automated unit string assertion.
- [x] CHK012 Does the spec specify whether algorithm parameters like gamma resolution are hidden from the primary user-facing narrative? [Clarity, Contract §4.2, Spec §FR-004]
  > **REVIEW NOTE**: Satisfied. Contract §4.2 strictly excludes gamma and modularity formulas from Tier 1 & 2 explanation text.
- [x] CHK013 Are guidelines defined for explaining numerical quality metrics in simple terms during the completion phase? [Clarity, Spec §User Story 3, Contract §2]
  > **REVIEW NOTE**: Satisfied. Completion phase uses "neatly organized into cohesive, color-coded communities" rather than displaying raw modularity floats.
- [x] CHK014 Is there a clear boundary between technical logging requirements (Principle VI) and user-facing plain-English narrative rules? [Consistency, Plan §Constitution Check]
  > **REVIEW NOTE**: Satisfied. Structured tracing logs go to stderr/LogRing, keeping the user-facing explanation panel strictly plain English.

## 4. 3-Tier Panel Structure & Visual Hierarchy

- [x] CHK015 Are requirements defined for the visual separation and hierarchy between Tier 1 (Headline), Tier 2 (Analogy), and Tier 3 (Live Badges)? [Clarity, Spec §FR-004, Contract §1]
  > **REVIEW NOTE**: Satisfied. Visual layout mockup in Contract §1.1 clearly establishes vertical separation across the 3 tiers.
- [x] CHK016 Is the styling token mapping (`FG_0` for headline, `FG_1` for analogy, `ACCENT_INFO` for stats) documented for all three tiers? [Consistency, Research §2, Plan §Summary]
  > **REVIEW NOTE**: Satisfied. Mapped in Research §2, Contract §5.1, and plan.md Summary.
- [x] CHK017 Are badge formats defined for phase progress percentage, active community count, and current phase name? [Completeness, Spec §FR-004, Data Model §1.3]
  > **REVIEW NOTE**: Satisfied. Formats specified in Contract §1.1 (`Phase: Local Moving   Communities: 4   Progress: 45%`).
- [x] CHK018 Is the layout height allocation (35% total window height) sufficient to prevent text truncation on minimum 80x24 viewports? [Measurability, Contract §1, Spec §Edge Cases]
  > **REVIEW NOTE**: Satisfied. 35% of Main Area provides 8 rows at 80×24, accommodating the 6-row payload with 2 padding rows.

## 5. Cognitive Load, Progressive Disclosure & Multimodal Reinforcement

- [x] CHK019 Is progressive disclosure mandated so that only the active phase is displayed, preventing forward-looking text from overloading working memory? [Clarity, Contract §3.1]
  > **REVIEW NOTE**: Satisfied. Contract §3.1 mandates revealing only one phase transition at a time without speculative future text.
- [x] CHK020 Are active Tier 3 metric badges bounded to ≤ 3 live values to respect human working memory chunking limits? [Clarity, Contract §3.1]
  > **REVIEW NOTE**: Satisfied. Capped at exactly 3 live badges (Phase Name, Community Count, Phase Progress) in Contract §3.1.
- [x] CHK021 Is there a 1:1 structural isomorphism between social metaphors (people, crowds, clubs) and visual canvas elements (nodes, monochromatic points, color clusters)? [Consistency, Contract §3.1]
  > **REVIEW NOTE**: Satisfied. 1:1 isomorphism mapping table is ratified in Contract §3.1.
- [x] CHK022 Do step headlines incorporate predictive sequential cues (`STEP X OF Y: [ACTION]`) to help users anticipate the progression toward convergence? [Clarity, Contract §3.2]
  > **REVIEW NOTE**: Satisfied. `STEP X OF Y` headline format mandated in Contract §3.2.
- [x] CHK023 Are social metaphors verified for cultural neutrality and universal accessibility across diverse user backgrounds? [Clarity, Contract §4.4]
  > **REVIEW NOTE**: Satisfied. Standardized on universal social contexts (cafeterias, study groups, clubs) in Contract §4.4.
- [x] CHK024 Do textual explanations and visual node movements reinforce each other without semantic conflicts? [Consistency, Contract §3.1, Spec §FR-003]
  > **REVIEW NOTE**: Satisfied. Narrative and physics simulation synchronization is enforced in Contract §3.1.

## 6. Edge Cases & Dynamic Narrative Adaptation

- [x] CHK025 Are explanation texts specified for single-community convergence (where the entire graph forms 1 group)? [Coverage, Edge Case, Contract §2]
  > **REVIEW NOTE**: Satisfied. Single community summary format documented in Contract §2 ("All members formed 1 united community").
- [x] CHK026 Are explanation texts defined for disconnected graphs where sub-graphs form isolated clusters immediately? [Coverage, Edge Case, Contract §2]
  > **REVIEW NOTE**: Satisfied. Documented in Contract §2 ("Isolated groups formed their own independent circles").
- [x] CHK027 Is the explanation behavior specified when the user rapidly steps through iterations? [Coverage, Edge Case, Spec §FR-005]
  > **REVIEW NOTE**: Satisfied. Explanations update synchronously with each discrete state transition without lagging.
- [x] CHK028 Are summary requirements defined for completion when custom user datasets are loaded versus built-in presets? [Completeness, Spec §User Story 1, Spec §User Story 3]
  > **REVIEW NOTE**: Satisfied. Common completion format displays total communities and modularity quality across both modes (Data Model §2.3).
- [x] CHK029 Is the text wrapping and line-break policy explicitly defined for multi-sentence analogy paragraphs? [Clarity, Data Model §2.3, Data Model §3]
  > **REVIEW NOTE**: Satisfied. `wrapped_analogy_lines(max_w)` wraps at word boundaries with max width 76 and max 3 lines (Data Model §3).

## 7. Contract Traceability & Schema Consistency

- [x] CHK030 Does the `ExplanationPayload` JSON schema define all required fields (`step_index`, `total_steps`, `headline`, `analogy_text`, `phase_name`, `community_count`, `flesch_kincaid_grade`)? [Completeness, Contract §1]
  > **REVIEW NOTE**: Satisfied. Complete JSON schema definition in Contract §1 includes all 7 required properties.
- [x] CHK031 Are the `phase_name` enum values in `contracts/explanation-content.md` synchronized with `data-model.md §ExplanationState`? [Consistency, Contract §1, Data Model §1.3]
  > **REVIEW NOTE**: Satisfied. Enum values `["Initial State", "Local Moving", "Refinement", "Aggregation", "Finished"]` match exactly.
- [x] CHK032 Is the user story outcome (non-technical user understanding the algorithm's purpose) directly traced to Success Criterion SC-001 and SC-003? [Traceability, Spec §SC-001, Spec §SC-003]
  > **REVIEW NOTE**: Satisfied. Traced directly from Spec User Story 1-3 to SC-001 and SC-003.

---

## Notes

- Mark items `[x]` only after review confirms the requirement-quality criterion is satisfied
- Leave items unchecked when they still require clarification, correction, or reviewer evaluation
- `/speckit-implement` reads checklist checkbox state as a gate and must not modify markers
- `checklists/requirements.md` has a separate built-in lifecycle maintained by `/speckit-specify` and `/speckit-clarify`
- Add comments or findings inline during PR review
- Items are numbered sequentially (CHK001–CHK032) for easy reference
