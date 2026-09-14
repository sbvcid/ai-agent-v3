# Agent Core Roadmap

## 1. Purpose

This document records the intended dependency order for future Agent Core development after the first implementation cycle. It is a roadmap, not a list of tasks that should all be implemented immediately.

The central rule is to add semantic capability before adding a large number of runtime tools. Each increment must be independently testable, reviewable, and releasable.

## 2. Current Position

The first implementation cycle established the deterministic foundation through the 9.4 release gate.

The current master branch also contains a formal implementation gap analysis and the design for the first post-cycle Agent Core increment.

Current architectural position:

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

The missing semantic path is:

```text
Observation
  ↓
Observation Store
  ↓
Context Compiler
  ↓
Evidence / Knowledge
  ↓
Hypothesis
  ↓
Progress
  ↓
Loop Detection
  ↓
Adaptive Recovery
  ↓
Goal Verification
```

## 3. Increment Order

### Increment A — Observation Store + Context Compiler

Design: `docs/07_AGENT_CORE_OBSERVATION_INCREMENT_DESIGN.md`

Purpose:

- establish a real observation history;
- separate stored knowledge from bounded provider context;
- create a deterministic context compilation boundary;
- preserve the current runtime and provider interfaces.

Must not include hypothesis management, loop detection, or broad runtime expansion.

Release evidence:

- store tests;
- compiler tests;
- action-result-to-observation integration;
- observation-to-provider-context integration;
- deterministic bounded context behavior.

### Increment B — Evidence and Knowledge Semantics

Purpose:

Turn raw observations into explicit, traceable knowledge without pretending that every observation is a fact.

Expected concepts:

- evidence reference/provenance;
- FACT / OBSERVED / INFERRED / HYPOTHESIS / UNKNOWN semantics;
- support/refute relationships where justified;
- traceability back to observation IDs;
- deterministic updates.

Important constraint:

Do not introduce a numerical confidence system merely because it is convenient. The first implementation should favor explicit provenance and state transitions over opaque scores.

Acceptance focus:

- unknown remains unknown;
- observations can support reasoning;
- inferred conclusions can be traced to evidence;
- stale or contradicted knowledge can be identified rather than silently overwritten.

### Increment C — Hypothesis Lifecycle

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

The implementation should permit more than one hypothesis when the evidence does not uniquely determine a cause.

Acceptance focus:

- a failed strategy can generate a revised hypothesis;
- new observations can refute an earlier hypothesis;
- the agent does not treat a hypothesis as a fact merely because an LLM proposed it.

### Increment D — Progress Detection

Purpose:

Detect whether actions are producing meaningful progress toward the Original Goal.

Progress must not be defined merely as:

- action returned success;
- process exited zero;
- a file was modified;
- another loop iteration occurred.

Progress should be tied to observable changes relevant to the goal.

Initial implementation should prefer explicit deterministic evidence over a generalized reward model.

Acceptance focus:

- distinguish action success from goal progress;
- detect successful actions that do not advance the goal;
- expose insufficient progress to the Agent Core.

### Increment E — Loop Detection

Purpose:

Detect repeated or near-repeated strategies that produce no useful progress.

Minimum behavior:

- detect exact repeated actions or equivalent action signatures where practical;
- consider repeated observations/results;
- distinguish legitimate repeated polling/waiting from pathological repetition;
- provide a structured loop observation/reason for the Agent Core.

The existing `max_steps` limit remains only a safety bound. It must not be rebranded as loop detection.

Acceptance focus:

- repeated failing strategy is detected;
- the agent can stop repeating the same action indefinitely;
- legitimate long-running polling is not automatically classified as a loop.

### Increment F — Adaptive Recovery

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

Acceptance focus:

- failure → observation → revised reasoning → alternative strategy → success;
- repeated failure → strategy change;
- missing dependency → investigation/recovery;
- timeout → inspect/wait/alternative;
- authorization boundary → NeedUser only when actually required.

### Increment G — Stronger Goal Verification

Purpose:

Make `Done` dependent on explicit, goal-tied evidence rather than merely allowing a caller to mark the goal verified.

Expected behavior:

- verification criteria are derived from the Original Goal;
- evidence used for verification is traceable;
- action success is insufficient by itself;
- contradictory evidence can produce `Uncertain` or `Failed` verification;
- terminal `Done` requires verified goal state.

Acceptance focus:

- goal vs action verification;
- output existence/content checks;
- contradictory evidence;
- no hallucinated completion.

## 4. Runtime Expansion Comes After Core Semantics

Only after the preceding Agent Core increments are stable should broader execution capabilities be expanded aggressively.

Likely later capabilities:

- richer shell/process execution;
- environment discovery;
- long-running Job model;
- cancellation and recovery;
- engineering/build/test runtime;
- Git workflows;
- Python tooling;
- browser automation;
- GUI/vision capabilities.

The ordering is intentional. More tools without adaptive semantics would produce a larger version of the existing `LLM + tools` pattern rather than the intended autonomous Agent Core.

## 5. Long-Running Jobs

The Job model is a separate semantic concern and should not be approximated by `running_jobs: Vec<String>` indefinitely.

A future Job increment should define:

- stable job identity;
- lifecycle state;
- start/progress/completion/failure observations;
- timeout semantics;
- cancellation;
- persistence/recovery;
- interaction with `ExecutionState`;
- safe resume after checkpoint restoration.

It should be introduced when the Agent Core has enough progress/observation semantics to reason about asynchronous work.

## 6. Environment and Capability Discovery

Environment discovery should be explicit and evidence-based.

The Agent Core should be able to determine facts such as:

- operating system/environment characteristics;
- available executables or capabilities;
- relevant versions;
- working-directory constraints;
- permission limitations.

The result should enter the Observation Store and knowledge path rather than being hidden in ad-hoc provider prompt text.

## 7. Engineering Runtime

Engineering tasks such as build/test/repair should be implemented as a capability layer, not as a fixed workflow.

The future Engineering Runtime should expose safe primitives for:

- inspect project;
- edit files;
- run build/check/test;
- inspect diagnostics;
- repeat after evidence-driven changes;
- preserve checkpoints and recover from failures.

The Agent Core, not the Engineering Runtime, decides whether another repair attempt is justified.

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

## 10. Testing Strategy for the Roadmap

Each increment must have three layers of evidence:

1. Unit tests for local invariants and deterministic transformations.
2. Integration tests showing the semantic path across component boundaries.
3. Acceptance scenarios tied to the formal acceptance-test document.

The minimum closed-loop scenarios that should eventually become regression tests include:

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

## 11. Rules for Coding-Agent Execution

The coding agent should never implement multiple roadmap increments in one pass unless a later increment is proven inseparable from the approved one.

Every implementation request should specify:

- exact design document;
- exact scope;
- explicit non-goals;
- expected files/modules;
- tests to add or preserve;
- required validation commands;
- release gate;
- stop condition.

If the coding agent discovers an architectural issue outside the approved scope, it should report it rather than silently expanding the implementation.

## 12. Rules for Independent AI Review

A second AI model is useful when an increment changes core semantics, persistence, security boundaries, or recovery behavior.

The reviewer should review:

- semantic correctness;
- invariant preservation;
- unintended coupling;
- missing failure cases;
- test quality;
- scope creep;
- compatibility with the formal requirements and acceptance tests.

The reviewer should not rewrite the architecture merely because it prefers a different design.

## 13. Stage Numbering Policy

No new Stage number is assigned merely because an item appears on this roadmap.

A Stage or implementation-cycle label should be assigned only after:

1. the relevant design document is complete;
2. dependencies are understood;
3. scope and non-goals are explicit;
4. acceptance criteria are defined;
5. the user approves the implementation boundary.

This prevents `Stage 10` from becoming an arbitrary bucket for every unfinished feature.

## 14. Immediate Next Action

The immediate next action is not coding.

First:

- review `docs/06_IMPLEMENTATION_GAP_ANALYSIS.md`;
- review `docs/07_AGENT_CORE_OBSERVATION_INCREMENT_DESIGN.md`;
- verify that Increment A is the correct smallest coherent boundary;
- only then assign an implementation-stage number and issue the coding-agent implementation prompt.

Until that review is complete, do not expand the runtime tool surface.
