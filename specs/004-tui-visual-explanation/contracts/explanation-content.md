# Contract: Explanation Content & Readability Schema

**Feature**: TUI Visual Explanation (`specs/004-tui-visual-explanation`)  
**Contract Version**: 1.1.0  
**Status**: Ratified  
**Aligned With**: `design-system.md`, `.specify/memory/constitution.md` (v1.1.0)  

---

## 1. Explanation Schema Definition

Each lifecycle step in the visual explanation generates an `ExplanationPayload` conforming to this schema:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ExplanationPayload",
  "type": "object",
  "required": [
    "step_index",
    "total_steps",
    "headline",
    "analogy_text",
    "phase_name",
    "community_count",
    "flesch_kincaid_grade"
  ],
  "properties": {
    "step_index": {
      "type": "integer",
      "minimum": 1
    },
    "total_steps": {
      "type": "integer",
      "minimum": 1
    },
    "headline": {
      "type": "string",
      "maxLength": 60
    },
    "analogy_text": {
      "type": "string",
      "maxLength": 240
    },
    "phase_name": {
      "type": "string",
      "enum": ["Initial State", "Local Moving", "Refinement", "Aggregation", "Finished"]
    },
    "community_count": {
      "type": "integer",
      "minimum": 1
    },
    "flesch_kincaid_grade": {
      "type": "number",
      "maximum": 8.0
    }
  }
}
```

---

## 2. Standard Explanation Content Registry

| Phase | Headline | Analogy Text | Max Grade Level |
|---|---|---|---|
| **Initial State** | *A Messy Network Starting Point* | *All people in the network are mixed together in one big crowd. No friend groups have formed yet.* | 4.8 |
| **Local Moving (Iter 1)** | *Finding Best Friend Circles* | *Each person looks around at their closest friends and moves into the circle where they have the most in common.* | 6.2 |
| **Local Moving (Iter 2+)** | *Swapping and Settling Groups* | *People keep swapping tables until everyone is sitting with their closest friends and nobody wants to move.* | 6.5 |
| **Refinement** | *Splitting Up Big Crowds* | *Groups check if all members are truly connected. If a table has two separate cliques, it splits into smaller well-knit teams.* | 6.8 |
| **Aggregation** | *Zooming Out to the Big Picture* | *We treat each tightly-knit team as a single super-member and look for wider patterns across the whole network.* | 7.1 |
| **Completed** | *Neat Communities Discovered!* | *The algorithm finished! The messy starting network is now neatly organized into cohesive, color-coded communities.* | 6.9 |

---

## 3. Pedagogical Design & Cognitive Load Principles

### 3.1 Cognitive Load & Working Memory Discipline
1. **Progressive Disclosure**: Explanations reveal only one algorithm phase transition at a time. The panel MUST NOT show forward-looking speculative text about unreached future phases.
2. **Chunking Bounds**: Tier 3 live metrics are strictly limited to $\le 3$ active badges (`Phase Name`, `Community Count`, `Phase Progress`) to prevent cognitive overload.
3. **Analogy-to-Visual Isomorphism**: Every social metaphor element maps 1:1 to a visual primitive on the canvas:
   - *Person* $\leftrightarrow$ Node disc (`●`)
   - *Friendship / Connection* $\leftrightarrow$ Canvas edge (`Line`)
   - *Crowd (Unassigned)* $\leftrightarrow$ Monochromatic nodes (`FG_2`)
   - *Club / Lunch Table (Community)* $\leftrightarrow$ Color-coded cluster (`COMMUNITY_COLORS`)
   - *Super-Member (Aggregation)* $\leftrightarrow$ Aggregated community centroid

### 3.2 Predictive Scaffolding & Active Processing
1. **Headline Progression**: Step headlines use structured numbering (`STEP X OF Y: [ACTION]`) allowing users to form predictive mental models of upcoming convergence.
2. **Phase Completion Cues**: Transitional steps explicitly explain *why* the algorithm changes phases (e.g., *"Nobody wants to move $\to$ Time to refine and split"*).

---

## 4. Readability & Tone Rules

1. **8th-Grade Reading Level Constraint (SC-003)**: All analogy strings MUST be validated to guarantee Flesch-Kincaid grade level $\le 8.0$.
2. **Jargon Blacklist**: The following technical terms are strictly prohibited in user-facing narrative text:
   - *Modularity*, *Resolution parameter*, *Eigenvector*, *CSR*, *Adjacency matrix*, *Heuristic*, *Optimization*, *Hamiltonian*, *Graph partition*.
3. **Everyday Metaphor Whitelist**: Preferred explanatory metaphors include:
   - *Friend groups*, *Lunch tables*, *Clubs*, *Neighborhoods*, *Teams*, *Cliques*, *Crowds*.
4. **Cultural Inclusivity**: Analogies must use universally accessible social situations (cafeterias, study groups, sports teams, clubs) avoiding regional or subculture-specific slang.
