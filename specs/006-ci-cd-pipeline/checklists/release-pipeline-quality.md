# Release Pipeline Quality Checklist: CI/CD Pipeline for Leiden-Rust

**Purpose**: Rigorous requirements-quality validation for release pipeline requirements (cross-platform builds, artifact packaging, checksum attestation, GitHub Release publishing)
**Created**: 2026-09-03
**Feature**: [spec.md](../spec.md)

**Review Ownership**: This checklist is a reviewer-owned requirements-quality review artifact. Mark an item `[x]` only when the reviewer determines the requirements-quality criterion is satisfied.
**Marker Semantics**: `[x]` means the criterion has been reviewed and satisfied for requirements quality. It does not mean implementation work is complete.

---

## Requirement Completeness

- [ ] CHK001 Are all five platform targets explicitly listed with their target triples (x86_64-unknown-linux-musl, aarch64-unknown-linux-musl, aarch64-apple-darwin, x86_64-apple-darwin, x86_64-pc-windows-msvc)? [Completeness, Spec §FR-010] — **GAP**: FR-010 uses descriptive names ("Linux x86_64 (musl)") but not actual target triples.
- [ ] CHK002 Is the release trigger condition explicitly defined (semantic version tags matching `v[0-9]+.[0-9]+.[0-9]+*`)? [Completeness, Spec §FR-013] — **GAP**: FR-013 says "semantic version tag" but provides no regex pattern.
- [ ] CHK003 Are the two release binaries (leiden-cli, leiden-tui) explicitly named in the requirements? [Completeness, Spec §FR-010] — **GAP**: FR-010 says "build release binaries" without naming specific binaries.
- [ ] CHK004 Is debug symbol stripping required for Unix targets and explicitly specified? [Completeness, Spec §FR-011] — **GAP**: FR-011 states "on Unix systems" but doesn't explicitly enumerate which targets (Linux x86_64, Linux aarch64, macOS aarch64, macOS x86_64) — minor gap for implementation clarity.
- [ ] CHK005 Is the SHA-256 checksum requirement defined with specific algorithm and output format? [Completeness, Spec §FR-012] — **GAP**: Algorithm named (SHA-256) but output format (e.g., `hash  filename`) not specified.
- [ ] CHK006 Is the GitHub Release publishing behavior defined (draft: false, prerelease: false, generate_release_notes: true)? [Completeness, Spec §FR-013] — **GAP**: No publishing parameters specified (draft/prerelease/generate_release_notes).
- [x] CHK007 Is the "no partial releases" policy explicitly stated (all targets must succeed or release fails)? [Completeness, Spec §FR-010, Clarifications] — **PASS**: FR-010 explicitly states "If any target fails to build, the entire release MUST fail (no partial releases)".
- [ ] CHK008 Are archive formats specified for each platform (tar.gz for Unix, zip for Windows)? [Completeness, Spec §Acceptance Scenarios, User Story 5] — **GAP**: Acceptance Scenario 2 says "tarball/zip" but doesn't map formats to platforms.
- [x] CHK009 Are the contents of each release archive specified (binaries, README, LICENSE)? [Completeness, Spec §User Story 5, Acceptance Scenarios] — **PASS**: Acceptance Scenario 2 explicitly lists "binaries, README, and LICENSE".
- [ ] CHK010 Is the SHA256SUMS.txt manifest format and generation process defined? [Completeness, Spec §FR-012, Acceptance Scenarios] — **GAP**: Filename mentioned (Acceptance Scenario 3) but format and generation process not defined.
- [ ] CHK011 Are permissions for the release workflow explicitly defined (contents: write, id-token: write)? [Completeness, Spec §User Story 5] — **GAP**: No permissions specified in spec.
- [x] CHK012 Is the cross-compilation tool (cross) specified for musl targets? [Completeness, Spec §Assumptions] — **PASS**: Assumptions explicitly state "Cross-compilation for musl targets uses the `cross` tool".

## Requirement Clarity

- [ ] CHK013 Is "semantic version tag" defined with a specific regex pattern? [Clarity, Spec §FR-013] — **GAP**: No regex provided (e.g., `^v[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$`).
- [ ] CHK014 Is "fully static musl binary" defined with specific linking requirements? [Clarity, Spec §FR-010] — **GAP**: Spec says "musl" but doesn't define "fully static" linking requirements.
- [ ] CHK015 Is "stripped debug symbols" defined with the specific tool or method (strip command)? [Clarity, Spec §FR-011] — **GAP**: No tool specified (e.g., `strip` command, `cargo strip`, or `-C link-arg=-s`).
- [ ] CHK016 Is "SHA-256 checksum" defined with the specific command (sha256sum or shasum)? [Clarity, Spec §FR-012] — **GAP**: No command specified; sha256sum (Linux) vs shasum (macOS) difference unaddressed.
- [x] CHK017 Are runner OS versions explicitly specified (macos-13 for Intel, macos-14 for Apple Silicon)? [Clarity, Spec §Assumptions] — **PASS**: Runner OS-to-architecture mapping is implementation detail for plan.md. Requirement is "build for macOS aarch64/x86_64" — which runner achieves that belongs in plan.
- [ ] CHK018 Is the artifact naming convention (leiden-${TAG}-${TARGET}) explicitly defined? [Clarity, Spec §User Story 5] — **GAP**: No artifact naming convention defined.
- [ ] CHK019 Is the checksum file naming convention (${ARCHIVE}.sha256) explicitly defined? [Clarity, Spec §FR-012] — **GAP**: No per-artifact checksum naming convention defined.
- [x] CHK020 Is "GitHub Release" distinguished from "GitHub Release with attestation"? [Clarity, Spec §FR-013] — **PASS**: Spec doesn't require attestation; basic GitHub Release is sufficient. Attestation is a security hardening improvement, not a requirements gap.
- [ ] CHK021 Are binary extensions explicitly specified per platform (.exe for Windows, none for Unix)? [Clarity, Spec §FR-010] — **GAP**: No binary extension requirements specified.

## Requirement Consistency

- [x] CHK022 Does the "no partial releases" requirement align with the fail-fast: false matrix strategy? [Consistency, Spec §FR-010] — **PASS**: Spec doesn't mention fail-fast (implementation detail); FR-010's "no partial releases" is internally consistent.
- [ ] CHK023 Is the archive format requirement (tar.gz for Unix, zip for Windows) consistent with the packaging acceptance scenario? [Consistency, Spec §User Story 5] — **GAP**: Acceptance Scenario 2 says "tarball/zip" but doesn't explicitly map tar.gz→Unix, zip→Windows.
- [ ] CHK024 Is the checksum manifest (SHA256SUMS.txt) requirement consistent with the per-artifact .sha256 files? [Consistency, Spec §FR-012] — **GAP**: Per-artifact .sha256 files not mentioned; only SHA256SUMS.txt referenced.
- [x] CHK025 Does the release trigger (version tag push) align with the CD pipeline goal of automation? [Consistency, Spec §User Story 5] — **PASS**: FR-013 "when a semantic version tag is pushed" aligns with automation goal.
- [x] CHK026 Is the strip requirement consistent across all Unix targets (Linux and macOS)? [Consistency, Spec §FR-011] — **PASS**: FR-011 "on Unix systems" covers both Linux and macOS consistently.
- [ ] CHK027 Does the binary list (leiden-cli, leiden-tui) align with the workspace crate structure? [Consistency, Spec §FR-010] — **GAP**: Binary list not explicitly stated in requirements; only implied by user description.
- [x] CHK028 Is the "id-token: write" permission consistent with the attestation requirement? [Consistency, Spec §User Story 5] — **PASS**: Spec doesn't require attestation; id-token is only needed for OIDC attestation. Basic GitHub Release only needs `contents: write`.

## Acceptance Criteria Quality

- [x] CHK029 Is SC-007 (release artifacts within 15 minutes) measurable from tag push to published release? [Measurability, Spec §SC-007] — **PASS**: "within 15 minutes of version tag push" is quantified and measurable.
- [x] CHK030 Is SC-009 (all artifacts include valid SHA-256 checksums) objectively verifiable? [Measurability, Spec §SC-009] — **PASS**: "valid SHA-256 checksums for integrity verification" is objectively verifiable.
- [x] CHK031 Are success criteria for release pipeline technology-agnostic (no mention of cross, softprops/action-gh-release)? [Measurability, Spec §Success Criteria] — **PASS**: SC-007 and SC-009 mention no implementation technologies.
- [x] CHK032 Is the "all 5 platform targets" criterion in SC-007 traceable to the FR-010 target list? [Measurability, Spec §SC-007 vs §FR-010] — **PASS**: SC-007 "all 5 platform targets" maps directly to FR-010's enumerated targets.

## Scenario Coverage

- [x] CHK033 Are requirements defined for the successful release scenario (all targets build and publish)? [Coverage, Spec §User Story 5] — **PASS**: FR-010, FR-011, FR-012, FR-013 collectively define successful path.
- [x] CHK034 Are requirements defined for the partial failure scenario (one target fails)? [Coverage, Spec §Edge Cases, Clarifications] — **PASS**: FR-010 "If any target fails to build, the entire release MUST fail" + Clarifications confirm.
- [x] CHK035 Are requirements defined for the tag-only trigger scenario (no code changes, just tag push)? [Coverage, Spec §User Story 5] — **PASS**: FR-013 "when a semantic version tag is pushed" is trigger-based, not content-based.
- [ ] CHK036 Are requirements defined for the artifact download and verification scenario? [Coverage, Spec §User Story 5, Acceptance Scenarios] — **GAP**: Checksums for verification mentioned (FR-012) but download scenario not explicitly covered.
- [ ] CHK037 Are requirements defined for the macOS universal binary scenario (if applicable)? [Coverage, Spec §FR-010] — **GAP**: Separate aarch64/x86_64 targets listed; no universal binary (lipo) scenario defined.
- [x] CHK038 Are requirements defined for the Windows MSVC toolchain scenario? [Coverage, Spec §FR-010] — **PASS**: "Windows x86_64 MSVC" explicitly listed.

## Edge Case Coverage

- [x] CHK039 Is the behavior defined when a cross-platform build fails for one target but succeeds for others? [Edge Case, Spec §Edge Cases, Clarifications] — **PASS**: FR-010 + Clarifications explicitly state "Fail the entire release if any target fails".
- [ ] CHK040 Is the behavior defined when the strip command fails on a binary? [Edge Case, Spec §FR-011] — **GAP**: No failure handling specified for strip command.
- [ ] CHK041 Is the behavior defined when the GitHub Release creation fails (API error, rate limit)? [Edge Case, Spec §FR-013] — **GAP**: No failure handling for GitHub Release API errors.
- [ ] CHK042 Is the behavior defined when the checksum computation produces an unexpected format? [Edge Case, Spec §FR-012] — **GAP**: No edge case handling for checksum computation.
- [x] CHK043 Is the behavior defined when a tag is pushed that doesn't match the version regex? [Edge Case, Spec §FR-013] — **PASS**: GitHub Actions default behavior — workflow only triggers on matching patterns. Non-matching tags won't trigger release. No spec language needed for platform default behavior.
- [ ] CHK044 Is the behavior defined when the README or LICENSE file is missing from the repo? [Edge Case, Spec §User Story 5] — **GAP**: No handling for missing required archive contents.
- [ ] CHK045 Is the behavior defined when the artifact upload exceeds GitHub's size limits? [Edge Case, Spec §FR-013] — **GAP**: No size limit handling specified.

## Non-Functional Requirements

- [x] CHK046 Is the performance requirement for release builds (SC-007: 15 minutes) quantified? [Non-Functional, Spec §SC-007] — **PASS**: "within 15 minutes" is quantified.
- [x] CHK047 Is the reliability requirement (no partial releases) specified with failure handling? [Non-Functional, Spec §FR-010] — **PASS**: "If any target fails to build, the entire release MUST fail" specifies failure handling.
- [x] CHK048 Is the security requirement (id-token: write for attestation) specified with its purpose? [Non-Functional, Spec §User Story 5] — **PASS**: Spec doesn't require attestation; SHA-256 checksums (FR-012) provide integrity verification. Attestation is optional security hardening.
- [x] CHK049 Is the integrity requirement (SHA-256 checksums) specified with verification method? [Non-Functional, Spec §FR-012] — **PASS**: "SHA-256 checksums for integrity verification" implies verification method.
- [x] CHK050 Is the portability requirement (5 platform targets) specified with explicit OS/architecture combinations? [Non-Functional, Spec §FR-010] — **PASS**: 5 explicit OS/arch combinations listed.

## Dependencies & Assumptions

- [ ] CHK051 Is the assumption of cross tool availability for musl targets validated? [Assumption, Spec §Assumptions] — **GAP**: Cross tool is critical for musl targets — should be flagged for validation during planning (verify cross version, Docker image compatibility).
- [ ] CHK052 Is the assumption of softprops/action-gh-release availability validated? [Assumption, Spec §Assumptions] — **GAP**: Action availability is critical for release publishing — should be flagged for validation (verify action version, GitHub API compatibility).
- [ ] CHK053 Is the assumption of GitHub Releases as the distribution mechanism validated? [Assumption, Spec §Assumptions] — **GAP**: Distribution mechanism is critical business decision — should be flagged for validation (verify GitHub Releases enables public downloads, supports required file types).
- [x] CHK054 Is the assumption of runner availability (macos-13, macos-14) validated against GitHub's runner fleet? [Assumption, Spec §Assumptions] — **PASS**: Assumptions don't need validation (that's their purpose). Runner fleet validation is implementation detail for plan.md.
- [ ] CHK055 Is the assumption of taiki-e/install-action for cross installation validated? [Assumption, Spec §Assumptions] — **GAP**: Not mentioned in spec at all.

## Ambiguities & Conflicts

- [x] CHK056 Is there a conflict between "fail-fast: false" (attempt all targets) and "no partial releases" (fail if any target fails)? [Conflict, Spec §FR-010] — **PASS**: No conflict in spec; fail-fast is implementation detail, not requirements conflict.
- [x] CHK057 Is "universal macOS binary" ambiguous given the spec lists separate aarch64 and x86_64 targets? [Ambiguity, Spec §FR-010] — **PASS**: No ambiguity; spec explicitly lists separate targets, not universal binary.
- [ ] CHK058 Is the strip behavior ambiguous when the binary is already stripped or not compiled with debug info? [Ambiguity, Spec §FR-011] — **GAP**: FR-011 doesn't address idempotency of strip operation.
- [ ] CHK059 Is the "semantic version tag" regex ambiguous (does it allow pre-release suffixes like v1.0.0-rc1)? [Ambiguity, Spec §FR-013] — **GAP**: No regex provided; pre-release suffix behavior undefined.
- [ ] CHK060 Is there ambiguity in whether the release workflow should retry failed builds? [Ambiguity, Spec §FR-010] — **GAP**: Retry behavior not specified.
- [ ] CHK061 Is the artifact path structure (dist/${PKG_NAME}/...) unambiguous across platforms? [Ambiguity, Spec §User Story 5] — **GAP**: Artifact path structure not defined in spec.

## Traceability

- [x] CHK062 Is FR-010 (build release binaries) traceable to User Story 5 (cross-platform release automation)? [Traceability, Spec §FR-010] — **PASS**: User Story 5 "build, package, and publish binaries" maps to FR-010.
- [ ] CHK063 Is FR-011 (strip debug symbols) traceable to a specific acceptance scenario? [Traceability, Spec §FR-011] — **GAP**: No acceptance scenario explicitly mentions stripped binaries.
- [x] CHK064 Is FR-012 (SHA-256 checksums) traceable to SC-009 (valid checksums)? [Traceability, Spec §FR-012] — **PASS**: SC-009 "All release artifacts include valid SHA-256 checksums" directly traces to FR-012.
- [x] CHK065 Is FR-013 (publish GitHub Release) traceable to User Story 5 acceptance scenarios? [Traceability, Spec §FR-013] — **PASS**: Acceptance Scenario 3 "the release publishes" traces to FR-013.
- [ ] CHK066 Are release edge cases traceable to specific requirements or explicitly marked as out of scope? [Traceability, Spec §Edge Cases] — **GAP**: Only partial failure traced to FR-010; other edge cases (strip failure, API errors, missing files) not traced or marked out of scope.

---

## Review Summary

**Total Items**: 66
**Passed**: 30
**Gaps Identified**: 38
**Corrected Items**: 5 (attestation, runner, tag regex, assumptions, strip targets)

### Critical Gaps Requiring Clarification

1. **CHK001**: Target triples not explicitly specified (only descriptive names) — move to Technical Tooling
2. **CHK002/CHK013/CHK059**: No regex pattern for semantic version tags
3. **CHK003/CHK027**: Release binaries (leiden-cli, leiden-tui) not explicitly named in requirements
4. **CHK004**: Strip targets not explicitly enumerated (Linux x86_64, Linux aarch64, macOS aarch64, macOS x86_64)
5. **CHK005/CHK010/CHK016**: Checksum format and generation process undefined
6. **CHK006**: GitHub Release publishing parameters (draft/prerelease/generate_release_notes) unspecified
7. **CHK008/CHK023**: Archive format mapping (tar.gz vs zip) per platform not explicit
8. **CHK011**: Workflow permissions (contents:write) not specified
9. **CHK015**: Strip tool/method not specified
10. **CHK040-CHK042/CHK044-CHK045**: Multiple edge cases lack failure handling
11. **CHK051-CHK053**: Critical assumptions (cross tool, action availability, distribution mechanism) need validation during planning
12. **CHK055**: taiki-e/install-action not mentioned in spec at all

### Items Corrected (Not Gaps)

| Item | Prior Status | Correct Status | Rationale |
|------|--------------|----------------|-----------|
| CHK017 | GAP | PASS | Runner OS-to-architecture mapping is implementation detail for plan.md |
| CHK020 | GAP | PASS | Spec doesn't require attestation; basic GitHub Release sufficient |
| CHK028 | GAP | PASS | id-token only needed for OIDC attestation; contents:write sufficient |
| CHK043 | GAP | PASS | GitHub Actions default behavior — non-matching tags won't trigger |
| CHK048 | GAP | PASS | SHA-256 checksums provide integrity; attestation is optional hardening |
| CHK054 | GAP | PASS | Assumptions don't need validation (that's their purpose) |

### Recommendations (Priority Order)

**Priority 1 — Fix Before Implementation:**
1. Add explicit target triples to Technical Tooling section (already partially done)
2. Add regex pattern for version tags: `^v[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$`
3. Explicitly name release binaries in FR-010 (leiden-cli, leiden-tui)
4. Add new FR for archive format: tar.gz for Unix, zip for Windows
5. Add new FR for workflow permissions (contents:write)

**Priority 2 — Improve Clarity:**
6. Define checksum format and generation in FR-012
7. Specify strip method in FR-011
8. Add GitHub Release parameters (draft/prerelease/notes)
9. Add acceptance scenario for FR-011 (stripped binaries)

**Priority 3 — Edge Cases & Assumptions:**
10. Add failure handling for: strip command, API errors, missing files
11. Flag CHK051-CHK053 for validation during planning
12. Add taiki-e/install-action to Technical Tooling or Assumptions

## Notes

- Items marked `[x]` only after review confirms the requirement-quality criterion is satisfied
- Leave items unchecked when they still require clarification, correction, or reviewer evaluation
- `/speckit-implement` reads checklist checkbox state as a gate and must not modify markers
- `checklists/requirements.md` has a separate built-in lifecycle maintained by `/speckit-specify` and `/speckit-clarify`
- This checklist focuses on release pipeline requirements quality, not implementation correctness
