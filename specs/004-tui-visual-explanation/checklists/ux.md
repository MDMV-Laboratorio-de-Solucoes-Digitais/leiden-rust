# UX & Visual Interaction Quality Checklist: TUI Visual Explanation

**Purpose**: Validate specification completeness, clarity, visual hierarchy, animation dynamics, and interactive usability for the TUI Visual Explanation feature.
**Created**: 2026-09-01
**Feature**: [spec.md](../spec.md) | [plan.md](../plan.md) | [contracts/tui-visual-explanation.md](../contracts/tui-visual-explanation.md)

**Review Ownership**: This checklist is a reviewer-owned requirements-quality review artifact. Mark an item `[x]` only when the reviewer determines the requirements-quality criterion is satisfied.
**Marker Semantics**: `[x]` means the criterion has been reviewed and satisfied for requirements quality. It does not mean implementation work is complete.

---

## 1. 2D Canvas & Graph Visualization Requirements

- [x] CHK001 Are visual rendering primitives explicitly specified for nodes (Unicode disc `●` `U+25CF`), intra-community edges (colored lines), and inter-community edges (dimmed `FG_3` lines)? [Completeness, Contract §3.1, Spec §FR-001]
  > **REVIEW NOTE**: Satisfied. Contract §3.1 specifies Unicode disc `●` (`U+25CF`), continuous `Line` widgets, and dimmed `FG_3` inter-cluster lines.
- [x] CHK002 Is the color transition from unassigned monochromatic `FG_2` to categorical `COMMUNITY_COLORS` specified as an instant per-step assignment update? [Clarity, Spec §FR-002, Spec §FR-003, Contract §3.1]
  > **REVIEW NOTE**: Satisfied. Contract §3.1 explicitly defines the instant color update upon community assignment event.
- [x] CHK003 Is the node ID label display rule quantified as a strict cutoff threshold (displayed if and only if total nodes N ≤ 40)? [Clarity, Contract §3.1]
  > **REVIEW NOTE**: Satisfied. Contract §3.1 establishes a strict N ≤ 40 threshold for rendering adjacent node labels.
- [x] CHK004 Is the canvas footer metadata format (`Dataset: [Title] (Active) · N nodes · E edges`) specified with exact color token bindings (`FG_2` & `ACCENT_PRIMARY`)? [Completeness, Contract §1.1, Contract §2.2]
  > **REVIEW NOTE**: Satisfied. Layout mockup in Contract §1.1 and §2.2 specifies exact footer format and token styling.
- [x] CHK005 Is the node collision mitigation approach defined using soft electrostatic repulsion ($F_{rep} = k_{rep}/\max(d^2, \epsilon^2)$ with $\epsilon=0.03$) and minimum separation distance ($d_{min}=0.04$)? [Clarity, Contract §3.2]
  > **REVIEW NOTE**: Satisfied. Contract §3.2 mathematically defines soft repulsion and separation clamping.

## 2. Force Simulation Animation & Motion Dynamics

- [x] CHK006 Are spatial coordinate boundaries defined in normalized $[0.05, 0.95]$ virtual unit space to strictly prevent clipping panel borders? [Clarity, Contract §3.2, Data Model §1.1]
  > **REVIEW NOTE**: Satisfied. Normalized coordinates bounded in $[0.05, 0.95]$ are enforced in Contract §3.2 and Data Model §2.1.
- [x] CHK007 Are the velocity damping constant ($\alpha = 0.85$) and maximum tick convergence budget (25 ticks per phase step) documented as fixed engine constants? [Completeness, Contract §3.2, Research §1]
  > **REVIEW NOTE**: Satisfied. Damping (0.85) and max 25 ticks budget are ratified in Contract §3.2.
- [x] CHK008 Is deterministic initial node placement mandated using a fixed pseudorandom seed / node-index hashing for 100% animation reproducibility? [Consistency, Contract §3.2, Constitution Determinism]
  > **REVIEW NOTE**: Satisfied. Deterministic CRC32/node-order seeding is mandated in Contract §3.2.
- [x] CHK009 Is the visual movement trajectory specified as a smooth acceleration vector pulling nodes toward their target community centroids? [Clarity, Spec §FR-003, Contract §3.2]
  > **REVIEW NOTE**: Satisfied. Smooth centroid acceleration vector is formalized in Contract §3.2.

## 3. Playback Controls & Dual Granularity Stepping UX

- [x] CHK010 Are keybinding interactions and state transitions documented for all standard controls (`Space`, `n`, `Right Arrow`, `t`, `1`–`3`, `r`, `?`, `q`)? [Completeness, Spec §FR-005, Contract §2.1]
  > **REVIEW NOTE**: Satisfied. Keybinding matrix in Contract §2.1 defines actions and preconditions for all 9 key combinations.
- [x] CHK011 Are the status bar visual badges for `Mode: Phase` (`FG_1`) versus `Mode: Micro` (`ACCENT_INFO`) explicitly specified? [Clarity, Contract §1.1, Contract §2.1]
  > **REVIEW NOTE**: Satisfied. Contract §1.1 and §2.1 define exact visual representations for both granularity modes.
- [x] CHK012 Is manual stepping (`n`) defined to auto-pause active auto-play before executing exactly one step forward? [Clarity, Contract §2.1]
  > **REVIEW NOTE**: Satisfied. Contract §2.1 keybinding table defines auto-pause upon pressing `n` during playback.
- [x] CHK013 Is the auto-play progress bar specified with a 10-block visual format (`[██████░░░░] %`) and live percentage readout? [Clarity, Contract §1.1]
  > **REVIEW NOTE**: Satisfied. 10-block progress bar format is explicitly specified in Contract §1.1.
- [x] CHK014 Are completion state indicators specified (`✔ Finished` in `ACCENT_SUCCESS`, headline update, and intra-cluster glow)? [Completeness, Contract §2.1, Contract §5.1]
  > **REVIEW NOTE**: Satisfied. Completion visuals are defined in Contract §2.1 and §5.1.

## 4. Preset Selection & Interactive Switching UX

- [x] CHK015 Are preset selection shortcuts (`1` for Karate Club, `2` for Two Cliques, `3` for Random Mess) documented with instant dataset reloading? [Completeness, Spec §FR-006, Contract §2.1]
  > **REVIEW NOTE**: Satisfied. Preset shortcuts and datasets are ratified in Contract §2.1 and Data Model §2.4.
- [x] CHK016 Is the active preset visual indicator specified as an `(Active)` badge in `ACCENT_PRIMARY` within the canvas footer? [Clarity, Contract §1.1, Contract §2.2]
  > **REVIEW NOTE**: Satisfied. Specified in Contract §1.1 and §2.2.
- [x] CHK017 Does the specification explicitly mandate that switching presets resets playback to Step 1 and auto-pauses auto-play? [Consistency, Contract §2.2]
  > **REVIEW NOTE**: Satisfied. Explicit reset-to-Step-1 invariant and auto-pause policy ratified in Contract §2.2.
- [x] CHK018 Is the visual presentation for custom CLI datasets defined using identical normalized coordinate auto-scaling? [Completeness, Spec §User Story 1, Contract §4.2]
  > **REVIEW NOTE**: Satisfied. Custom datasets auto-scale into identical $[0.05, 0.95]$ normalized canvas space per Contract §4.2.

## 5. Viewport Layout, Scaling & Terminal Guard UX

- [x] CHK019 Is the two-stage layout vertical split specified mathematically (35% explanation, 65% canvas of Main Area, plus 1 fixed status bar row)? [Consistency, Contract §1.1]
  > **REVIEW NOTE**: Satisfied. Two-stage splitting math is documented in Contract §1.1.
- [x] CHK020 Is coordinate re-normalization specified to map virtual $[0.0, 1.0]$ space to new terminal `Rect` dimensions instantly on resize? [Completeness, Spec §FR-007, Contract §4.2]
  > **REVIEW NOTE**: Satisfied. Instant projection on render frame is specified in Contract §4.2.
- [x] CHK021 Are warning modal dimensions ($46 \times 7$), centered placement, and blocking interaction rules defined for viewports $< 80 \times 24$? [Clarity, Contract §4.1, Spec §Edge Cases]
  > **REVIEW NOTE**: Satisfied. Centered 46×7 warning modal specifications ratified in Contract §4.1.
- [x] CHK022 Is the restoration policy defined with zero state loss, restoring prior play/pause state when expanding back to $\ge 80 \times 24$? [Completeness, Contract §4.2]
  > **REVIEW NOTE**: Satisfied. Restoration behavior with zero state loss ratified in Contract §4.2.

## 6. Visual Accessibility & Help Overlay UX

- [x] CHK023 Is the keyboard help modal specified as a centered $50 \times 14$ overlay dismissable via `?`, `Esc`, `Space`, or `Enter`? [Completeness, Contract §2.1]
  > **REVIEW NOTE**: Satisfied. 50×14 help modal and dismissal keys specified in Contract §2.1.
- [x] CHK024 Are all 11 design system color tokens (`BG_0`, `BG_1`, `FG_0`–`FG_3`, `ACCENT_PRIMARY`, `ACCENT_INFO`, `ACCENT_SUCCESS`, `ACCENT_WARNING`, `COMMUNITY_COLORS`) documented with hex, contrast ratios, and ANSI fallbacks? [Consistency, Contract §5.1, Research §2]
  > **REVIEW NOTE**: Satisfied. Complete token table with contrast ratios and ANSI fallbacks ratified in Contract §5.1.
- [x] CHK025 Is the prohibition of `Modifier::ITALIC` specified to prevent broken reverse-video or blinking text on legacy TTYs and tmux? [Consistency, Contract §5.2, Research §2]
  > **REVIEW NOTE**: Satisfied. Explicitly banned in Contract §5.2 and Research §2.
- [x] CHK026 Are rounded border styles (`BorderType::Rounded` `╭╮╰╯`) mandated across all panels and modal dialogs? [Consistency, Contract §5.2, Research §2]
  > **REVIEW NOTE**: Satisfied. `BorderType::Rounded` mandated in Contract §5.2 with ASCII fallback in non-UTF-8 terminals.

---

## Notes

- Mark items `[x]` only after review confirms the requirement-quality criterion is satisfied
- Leave items unchecked when they still require clarification, correction, or reviewer evaluation
- `/speckit-implement` reads checklist checkbox state as a gate and must not modify markers
- `checklists/requirements.md` has a separate built-in lifecycle maintained by `/speckit-specify` and `/speckit-clarify`
- Add comments or findings inline during PR review
- Items are numbered sequentially (CHK001–CHK026) for easy reference
