# Agent Core Roadmap

## 1. Purpose

This document records the intended dependency relationships and broad direction for Agent Core development after the first implementation cycle. It is a stable roadmap, not a task list and not a temporary implementation plan.

The central rule is to establish semantic capability before expanding the runtime tool surface. Each implementation increment should be independently testable, reviewable, and releasable.

The roadmap is a dependency map, not a mandatory linear sequence. Items may be designed or implemented in parallel when their semantic dependencies allow it. No roadmap item authorizes implementation by itself.

## 2. Current Position

The first implementation cycle established the deterministic foundation through the 9.4 release gate.

The current architectural position is:

```text
Goal
  ↓
AgentState
  ↓
DecisionSource
  ↓
Action
  ↓
Runtime
  ↓
ActionResult
  ↓
Observation
```

The major missing semantic capabilities include:

```text
Observation History
        ↓
Context Compilation
        ↓
Evidence / Knowledge
        ↓
Hypothesis Lifecycle
        ↓
Progress / Loop Awareness
        ↓
Adaptive Recovery
        ↓
Stronger Goal Verification
```

These relationships describe useful dependencies, not a promise that every box must become a separate implementation stage.

## 3. Core Semantic Direction

### A — Observation Store + Context Compiler

Design baseline: `docs/07_AGENT_CORE_OBSERVATION_INCREMENT_DESIGN.md`

Purpose:

- establish authoritative in-process observation history;
- separate historical observations from bounded provider context;
- create a deterministic context compilation boundary;
- preserve existing Runtime and provider semantic boundaries.

This is the current smallest coherent Agent Core foundation identified after Cycle 1.

### B — Evidence and Knowledge Semantics

Purpose:

Turn raw observations into explicit, traceable knowledge without pretending that every observation is a fact.

Expected concepts:

- evidence reference/provenance;
- FACT / OBSERVED / INFERRED / HYPOTHESIS / UNKNOWN semantics;
- support/refute relationships where justified;
- traceability back to observation IDs;
- deterministic updates;
- explicit handling of stale or contradicted knowledge.

Constraint:

Do not introduce a numerical confidence system merely because it is convenient. The initial design should favor explicit provenance and state transitions over opaque scores.

Dependency:

A reliable Observation Store is a strong prerequisite because knowledge must be traceable to environmental evidence.

### C — Hypothesis Lifecycle

Purpose:

Give the Agent Core an explicit mechanism for maintaining and revising explanations of the current problem.

Expected lifecycle:

```text
Created
  ↓
Testable
  ↓
Supported / Refuted / Uncertain
  ↓
Revised or Retired
```

The implementation should permit more than one hypothesis when evidence does not uniquely determine a cause.

Dependencies:

Evidence/knowledge semantics and observation provenance should be sufficiently defined before the lifecycle is made authoritative.

### D — Progress Detection

Purpose:

Detect whether actions are producing meaningful progress toward the Original Goal.

Progress must not be defined merely as:

- action returned success;
- process exited zero;
- a file was modified;
- another loop iteration occurred.

Progress should be tied to observable changes relevant to the goal.

Initial implementation should prefer explicit deterministic evidence over a generalized reward model.

Dependencies:

Observation and goal-verification semantics are important. Full hypothesis lifecycle is useful but not necessarily a strict prerequisite for all progress detection.

### E — Loop Detection

Purpose:

Detect repeated or near-repeated strategies that produce no useful progress.

Minimum conceptual behavior:

- detect exact repeated actions or equivalent action signatures where practical;
- consider repeated observations/results;
- distinguish legitimate repeated polling/waiting from pathological repetition;
- provide a structured loop observation/reason for the Agent Core.

The existing `max_steps` limit remains only a safety bound. It must not be rebranded as loop detection.

Dependencies:

Observation history and progress signals are strong prerequisites. Loop detection may be designed alongside progress detection rather than strictly after it.

### F — Adaptive Recovery

Purpose:

Make strategy change an explicit Agent Core behavior.

Recovery may include:

- gather additional observations;
- revise hypothesis;
- choose an alternative action;
- change execution parameters;
- wait and inspect a running operation;
- terminate with `Blocked`, `Impossible`, or `NeedUser` when justified.

The recovery mechanism must not become a hard-coded workflow such as `if error then run command X`.

Dependencies:

Adaptive recovery depends on enough observation, hypothesis, progress, and loop semantics to make strategy change evidence-driven. It is therefore a later integration capability rather than merely another error handler.

### G — Stronger Goal Verification

Purpose:

Make `Done` dependent on explicit, goal-tied evidence rather than merely allowing a caller to mark the goal verified.

Expected behavior:

- verification criteria are tied to the Original Goal;
- evidence used for verification is traceable;
- action success is insufficient by itself;
- contradictory evidence can produce uncertain or failed verification;
- terminal `Done` requires verified goal state.

Dependencies:

Observation/evidence semantics are foundational. Stronger verification can be designed independently of full adaptive recovery, although recovery ultimately uses verification outcomes.

## 4. Runtime and Capability Expansion

Broader execution capability is a separate branch of the architecture and should not be treated as a single linear sequence after every Agent Core increment.

Likely capability areas include:

- richer shell/process execution;
- environment discovery;
- long-running Job model;
- cancellation and recovery;
- engineering/build/test runtime;
- Git workflows;
- Python tooling;
- browser automation;
- GUI/vision capabilities;
- broader network and advanced Windows capabilities.

Each capability must have its own semantic boundary, safety model, tests, and integration contract. Adding a capability does not automatically require redesigning Agent Core.

The guiding rule remains: do not add large numbers of tools merely to compensate for missing Agent Core semantics.

## 5. Long-Running Jobs

The Job model is a separate semantic concern and should not be approximated by `running_jobs: Vec<String>` indefinitely.

A future Job design should define:

- stable job identity;
- lifecycle state;
- start/progress/completion/failure observations;
- timeout semantics;
- cancellation;
- persistence/recovery;
- interaction with `ExecutionState`;
- safe resume after checkpoint restoration.

Job semantics may be developed when their requirements become concrete. They should not be artificially delayed until every Agent Core item is complete, but they must integrate with the observation, progress, and execution-state boundaries rather than bypass them.

## 6. Environment and Capability Discovery

Environment discovery should be explicit and evidence-based.

The Agent Core should be able to determine facts such as:

- operating system/environment characteristics;
- available executables or capabilities;
- relevant versions;
- working-directory constraints;
- permission limitations.

The result should enter the Observation Store and knowledge path rather than being hidden in ad-hoc provider prompt text.

Environment discovery can be implemented as a runtime capability when needed. Its semantic result should remain ordinary observation/evidence, not a second state system.

## 7. Engineering Runtime

Engineering tasks such as build/test/repair should be implemented as a capability layer, not as a fixed workflow.

A future Engineering Runtime should expose safe primitives for:

- inspect project;
- search/read/edit files;
- run build/check/test;
- inspect diagnostics;
- repeat after evidence-driven changes;
- preserve checkpoints and recover from failures;
- interact with Git where explicitly required.

The Agent Core, not the Engineering Runtime, decides whether another repair attempt is justified.

The Engineering Runtime is a capability branch. It does not define a new autonomous-agent architecture.

## 8. Browser / GUI / Vision

These capabilities are later because they add high-dimensional observations, asynchronous behavior, permissions, and additional failure modes.

They should reuse the same semantic boundaries:

```text
Capability
  ↓
Observation
  ↓
Observation Store
  ↓
Knowledge / Hypothesis
  ↓
Decision
```

They should not create a second autonomous-agent architecture.

## 9. Persistence Strategy

Phase 0 JSON checkpointing remains the baseline until real persistence requirements justify a stronger mechanism.

Do not introduce SQLite merely to create an Observation Store. The first Observation Store can remain in-memory while AgentState checkpoints preserve the minimum required state.

When persistence semantics become complex enough to require transactional history, crash-safe append, or large observation retention, persistence can be redesigned explicitly.

Persistence changes are architectural changes and require their own design decision; they are not implied by the roadmap.

## 10. Testing Strategy for the Roadmap

Each substantive increment should have three layers of evidence:

1. Unit tests for local invariants and deterministic transformations.
2. Integration tests showing the semantic path across component boundaries.
3. Acceptance scenarios tied to the formal acceptance-test document.

The eventual regression set should include at least:

- success;
- failure → strategy change → success;
- unknown → investigation;
- repeated failure → loop detection;
- timeout → inspect/wait/alternative;
- permission boundary → NeedUser;
- goal verification failure;
- crash → resume;
- invalid provider decision → rejection/correction;
- long-running job lifecycle;
- environment discovery before environment-dependent action.

The roadmap does not require all of these tests to exist before a capability's own design is approved.

## 11. Implementation Governance

This roadmap does not itself issue coding instructions.

Before implementation of a substantive new capability:

- the relevant design boundary must be explicit;
- scope and non-goals must be clear;
- acceptance criteria must be defined;
- dependencies must be understood sufficiently to avoid accidental coupling;
- the user must approve the implementation boundary.

The actual coding instruction is generated in the active development conversation for that specific increment. It is intentionally not maintained as a permanent `09_IMPLEMENTATION_TASK` document.

A coding agent must not silently expand an approved scope because it discovers an adjacent roadmap item. Out-of-scope architectural concerns should be reported and recorded in the appropriate status/history location.

## 12. Independent AI Review

A second AI model is useful when an increment changes core semantics, persistence, security boundaries, or recovery behavior.

The reviewer should examine:

- semantic correctness;
- invariant preservation;
- unintended coupling;
- missing failure cases;
- test quality;
- scope creep;
- compatibility with the formal requirements and acceptance tests.

The reviewer should not rewrite the architecture merely because it prefers a different design. Any proposed architectural change must be evaluated against the stable design baseline.

## 13. Stage Numbering Policy

No new Stage number is assigned merely because an item appears on this roadmap.

A Stage or implementation-cycle label should be assigned only after:

1. the relevant design document is complete;
2. dependencies are understood;
3. scope and non-goals are explicit;
4. acceptance criteria are defined;
5. the user approves the implementation boundary.

This prevents an arbitrary future `Stage 10` from becoming a bucket for every unfinished feature.

## 14. Roadmap Stability Rule

This document should change only when the project's long-term architectural direction, dependency understanding, or capability priorities materially change.

Temporary implementation details, debugging notes, test output, coding-agent instructions, and unresolved discussion do not belong here.

Those belong in the active conversation, `IMPLEMENTATION_STATUS.md`, or `history/` as appropriate.
