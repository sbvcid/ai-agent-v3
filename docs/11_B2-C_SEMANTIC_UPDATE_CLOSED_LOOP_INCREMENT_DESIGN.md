# B2-C Semantic Update / Closed-Loop Integration Increment Design

Status: **APPROVED STABLE DESIGN**

Current Implementation Progress:
- B2-C1 (Core Semantic Update Boundary): **COMPLETED**
- B2-C2 (AgentLoop Integration Point): **COMPLETED**
- B2-C3 (Deterministic Closed-Loop Integration): **NOT IMPLEMENTED / PENDING**
- B2-C4 (Failure and Atomicity Boundary Verification): **NOT IMPLEMENTED / PENDING**

## 1. Purpose

B2-C closes the semantic data path established by B1, B2-A, and B2-B.

B1 established the Knowledge semantic model and `KnowledgeStore`.
B2-A established authoritative `KnowledgeStore` ownership in Agent Core and the multi-store DecisionSource boundary.
B2-B established deterministic propagation from `KnowledgeStore` through `ContextCompiler` into `ProviderRequest`.

The remaining gap is the mutation boundary between newly produced `Observation` records and derived semantic state in `KnowledgeStore`.

B2-C therefore defines and implements the smallest Core-level semantic update boundary required to demonstrate the following closed loop:

```text
ActionResult
    ↓
Observation
    ↓
ObservationStore
    ↓
Core Semantic Update Boundary
    ↓
KnowledgeStore
    ↓
ContextCompiler
    ↓
CompiledContext
    ↓
ProviderRequest
    ↓
Provider / DecisionSource
```

This document defines the implementation increment only. It does not define a complete reasoning architecture, Knowledge inference engine, hypothesis lifecycle, or recovery system.

B2-C is not a new Stage number.

## 2. Current Baseline

The following are already implemented and are prerequisites for B2-C:

- `ObservationStore` is authoritative in-process Observation history.
- `KnowledgeStore` is authoritative derived semantic state.
- `AgentLoop` owns `AgentState`, `ObservationStore`, and `KnowledgeStore`.
- `DecisionSource` can receive authoritative Observation and Knowledge stores.
- `ContextCompiler::compile_with_knowledge()` validates and consumes Knowledge semantic state.
- `CompiledContext` carries `KnowledgeClaim`, `EvidenceLink`, and first-class Knowledge `Unknown` objects.
- `ProviderRequest` carries provider-neutral Knowledge semantic context.
- Provider adapters remain responsible for provider-specific wire formatting.
- Existing Runtime Action / ActionResult / Observation behavior remains unchanged.
- Existing legacy `AgentState` fields for `unknowns`, `evidence`, and `hypotheses` remain for compatibility.

B2-C must build on these boundaries rather than replacing them.

## 3. Problem / Gap

The current implementation has two connected but incomplete halves:

```text
ObservationStore → [missing semantic mutation boundary] → KnowledgeStore

KnowledgeStore → ContextCompiler → ProviderRequest → Provider
```

`KnowledgeStore` already enforces Knowledge, Evidence, Unknown, and provenance invariants, but no Core-level component currently owns the responsibility of receiving a semantic update and applying it to the authoritative store.

The missing boundary must be explicit so that:

1. AgentLoop does not become a reasoning engine.
2. Runtime does not become a Knowledge engine.
3. Provider adapters do not mutate Knowledge state.
4. Knowledge mutation is deterministic and testable.
5. The future reasoning producer can evolve independently from storage and context compilation.

## 4. Scope

B2-C consists of four implementation areas.

### 4.1 B2-C1 — Core Semantic Update Boundary

Introduce a provider-independent Core-level abstraction representing a semantic update operation against the authoritative `KnowledgeStore`.

The boundary must support, at minimum, the semantic mutation categories already defined by B1:

- add a Knowledge Claim;
- add the Evidence Links required by an Observed/Inferred Claim;
- add a first-class Unknown;
- return deterministic validation / storage errors.

The boundary must not define how the semantic conclusion was reasoned about.

The boundary is a mutation interface, not a reasoning engine.

### 4.2 B2-C2 — AgentLoop Integration Point

Integrate the semantic update boundary into the Agent Core execution flow at the point after an Observation has been successfully recorded in `ObservationStore`.

The required ordering is:

```text
ActionResult
    ↓
Observation construction
    ↓
ObservationStore.record()
    ↓
Semantic Update Boundary
    ↓
KnowledgeStore
```

An EvidenceLink referencing an Observation must never be accepted before the referenced Observation exists in the authoritative ObservationStore.

AgentLoop remains orchestration code. It must not itself infer arbitrary Knowledge from an Observation.

### 4.3 B2-C3 — Deterministic Closed-Loop Integration

Add deterministic tests proving that semantic state produced from an Observation becomes visible to the next provider decision context.

The integration test must demonstrate:

1. an Action is executed;
2. Runtime returns an ActionResult;
3. Agent Core creates an Observation;
4. the Observation is stored in ObservationStore;
5. a semantic update is applied through the new boundary;
6. KnowledgeStore contains the resulting semantic state;
7. the next decision compiles Knowledge through ContextCompiler;
8. ProviderRequest contains the resulting Knowledge / Evidence / Unknown context;
9. the provider-facing request remains provider-neutral;
10. no provider or Runtime component directly mutates KnowledgeStore.

The test may use a deterministic test semantic producer or test double. B2-C does not require a complete production reasoning producer.

### 4.4 B2-C4 — Failure and Atomicity Boundary

Semantic update failures must be deterministic and must not leave partially applied Knowledge state.

The Observation already recorded in ObservationStore remains historical evidence even if a subsequent semantic update fails.

KnowledgeStore mutation must obey the atomic validation behavior established by B1. A failed semantic update must not leave partially inserted claims, evidence links, or unknowns.

## 5. Semantic Update Model

B2-C introduces the concept of a Core semantic update without introducing a new AgentDecision variant.

Conceptually:

```text
SemanticUpdateProducer
        ↓
SemanticUpdate
        ↓
Semantic Update Boundary
        ↓
KnowledgeStore
```

The exact Rust type names may follow existing project conventions, but the implementation must preserve the following semantic separation:

- producer: determines what semantic update should be requested;
- update boundary: validates and applies the requested semantic mutation;
- KnowledgeStore: owns derived semantic state and its invariants;
- ContextCompiler: reads and selects semantic context;
- Provider: consumes context and produces a decision;
- Runtime: executes Actions and returns ActionResults.

The update boundary must not contain natural-language reasoning, provider-specific parsing, confidence calculation, or truth evaluation.

## 6. Required Semantic Operations

The first B2-C implementation must support these operations through the Core-level boundary:

### 6.1 Add Knowledge Claim

A Hypothesis claim may be recorded without Evidence.

An Observed or Inferred claim must be accompanied by valid EvidenceLinks through the atomic KnowledgeStore path already established in B1.

### 6.2 Add Evidence Link

An EvidenceLink must reference:

- an existing Observation ID;
- an existing KnowledgeClaim ID;
- a valid EvidenceRelation.

Supported relations remain:

- `Supports`
- `Contradicts`
- `Qualifies`

B2-C must not introduce automatic evidence ranking or truth resolution.

### 6.3 Add Unknown

Unknown remains an independent semantic object.

It must not be represented by changing a KnowledgeClaim status to `Unknown`.

### 6.4 Validation Failure

Invalid semantic updates must produce explicit deterministic errors.

Examples include:

- missing Observation referenced by EvidenceLink;
- missing KnowledgeClaim referenced by EvidenceLink;
- invalid empty semantic fields;
- duplicate semantic identifiers;
- Observed/Inferred claim without required provenance.

## 7. Ownership and Boundaries

The following ownership rules are mandatory.

| Component | Owns | Must not own |
|---|---|---|
| AgentLoop / Agent Core | orchestration and authoritative store ownership | semantic reasoning itself |
| ObservationStore | Observation history | Knowledge claims |
| KnowledgeStore | derived Knowledge / Evidence / Unknown state | Observation history |
| Semantic Update Boundary | controlled Knowledge mutation interface | reasoning, truth evaluation |
| ContextCompiler | selection, ordering, bounds, semantic context construction | store mutation |
| ProviderRequest | provider-neutral request representation | Knowledge ownership |
| Provider Adapter | provider wire formatting | Knowledge mutation |
| Runtime | Action execution and ActionResult production | Knowledge reasoning |

No component may create a second independent Observation history as part of B2-C.

## 8. Required Ordering and Invariants

The implementation must preserve these invariants:

1. ObservationStore remains the authoritative Observation history.
2. KnowledgeStore remains derived semantic state.
3. Every Observed/Inferred KnowledgeClaim remains traceable through EvidenceLink to at least one Observation.
4. EvidenceLink references must resolve against the authoritative ObservationStore and KnowledgeStore.
5. Conflicting claims may coexist.
6. The system must not automatically choose which conflicting claim is true.
7. Unknown remains first-class and independent from claim status.
8. Knowledge mutation must be deterministic.
9. Failed Knowledge mutation must not partially modify KnowledgeStore.
10. An Observation remains stored even if semantic processing after the Observation fails.
11. ContextCompiler remains read-only with respect to KnowledgeStore.
12. ProviderRequest remains provider-neutral.
13. Provider and Runtime do not become KnowledgeStore owners.

## 9. AgentDecision Boundary

B2-C explicitly does **not** add:

```text
AgentDecision::UpdateKnowledge
```

Knowledge mutation is a Core semantic operation, not a Runtime Action.

An AgentDecision continues to describe what the Agent should do through the existing decision model. Semantic state updates occur at the Core semantic boundary and are not represented as executable Runtime operations.

This preserves the existing separation:

```text
AgentDecision → Runtime Action
Semantic Update → KnowledgeStore
```

## 10. Provider Boundary

The provider path remains:

```text
KnowledgeStore
    ↓
ContextCompiler
    ↓
CompiledContext
    ↓
ProviderRequest
    ↓
Provider Adapter
```

B2-C must not make a provider responsible for:

- reading KnowledgeStore directly;
- validating KnowledgeStore invariants;
- creating EvidenceLinks directly;
- deciding truth between conflicting claims;
- persisting Knowledge;
- mutating Agent Core state.

A provider response may later become an input to a semantic producer, but that producer boundary is outside the provider adapter itself.

## 11. Closed-Loop Test Contract

At least one deterministic integration test must establish the complete semantic loop.

The test should use deterministic components and must not require a live external provider, network service, wall clock, random seed, or persistent database.

The test must verify at minimum:

```text
ActionResult
  → Observation
  → ObservationStore
  → SemanticUpdate
  → KnowledgeStore
  → ContextCompiler
  → ProviderRequest
```

The resulting ProviderRequest must contain the Knowledge claim and its EvidenceLink, and the EvidenceLink must point to the Observation generated from the ActionResult.

A failure result must also remain representable as an Observation and must not be silently discarded merely because the Action failed.

## 12. Compatibility

B2-C must preserve existing public behavior unless a change is explicitly required by the semantic boundary.

In particular:

- existing `DecisionSource` compatibility methods remain available unless an additive change is required;
- existing Runtime Action and ActionResult models remain unchanged;
- existing Observation construction remains valid;
- existing checkpoints remain loadable or fail deterministically;
- no new Knowledge persistence format is introduced;
- existing legacy AgentState Knowledge-related fields are not removed in B2-C;
- provider adapters remain compatible with the provider-neutral ProviderRequest boundary.

If a constructor or trait signature must evolve, prefer an additive compatibility-preserving API rather than replacing an existing public interface.

## 13. Checkpoint Behavior

B2-C does not introduce Knowledge persistence.

Existing checkpoints therefore do not become authoritative full Knowledge history.

The implementation must preserve deterministic behavior for existing checkpoints:

- an existing checkpoint must continue to load where currently supported;
- the absence of persisted KnowledgeStore data must not be represented as recovered full Knowledge history;
- the in-memory KnowledgeStore starts from the defined compatibility state when no Knowledge persistence exists;
- no undocumented migration format is introduced.

Checkpoint persistence for Knowledge remains a future independent design problem.

## 14. Determinism Requirements

For identical:

- AgentState;
- ObservationStore contents;
- KnowledgeStore contents;
- SemanticUpdate input;
- compiler configuration;

B2-C must produce identical semantic results and identical observable validation outcomes.

The semantic update boundary must not depend on:

- wall-clock time;
- random values;
- provider-specific behavior;
- global mutable state;
- iteration order that is not explicitly deterministic.

Stable identifiers supplied by the semantic producer remain authoritative. B2-C must not introduce semantic natural-language deduplication.

## 15. Explicit Non-Goals

B2-C does not implement or redesign:

- hypothesis lifecycle;
- hypothesis promotion / demotion policy;
- confidence or probability scores;
- truth scores;
- automatic conflict resolution;
- Knowledge ranking;
- stale / superseded Knowledge lifecycle;
- progress detection;
- loop detection;
- adaptive recovery;
- stronger goal verification;
- planner / tree search;
- RAG;
- vector databases;
- SQLite or external persistence;
- multi-agent Knowledge sharing;
- browser capability;
- GUI / vision capability;
- unrestricted shell capability;
- engineering runtime;
- Job lifecycle redesign;
- provider-specific Knowledge semantics;
- Runtime semantic reasoning;
- removal or migration of legacy AgentState Knowledge fields;
- `AgentDecision::UpdateKnowledge`.

## 16. Acceptance Criteria

B2-C is considered implementation-complete only when all of the following are true:

1. A provider-independent Core semantic update boundary exists.
2. The boundary can apply KnowledgeClaim, EvidenceLink, and Unknown mutations through KnowledgeStore invariants.
3. Observed/Inferred claims cannot bypass provenance requirements.
4. EvidenceLinks cannot reference missing Observations or Claims.
5. Observation is recorded before any semantic update can reference it.
6. AgentLoop remains orchestration code and does not contain semantic inference rules.
7. KnowledgeStore remains the authoritative derived semantic store.
8. ObservationStore remains the authoritative Observation history.
9. No duplicate Observation history is introduced.
10. No `AgentDecision::UpdateKnowledge` is introduced.
11. A deterministic closed-loop integration test proves Observation → Knowledge → next ProviderRequest propagation.
12. ProviderRequest receives semantic Knowledge context without taking ownership of KnowledgeStore.
13. Provider adapters remain wire-formatting boundaries.
14. Conflicting Knowledge claims can coexist without automatic truth resolution.
15. Unknown remains independent from KnowledgeClaim status.
16. Semantic update failures are deterministic and leave no partial Knowledge mutation.
17. Existing Action / ActionResult / Observation behavior remains compatible.
18. Existing checkpoints remain loadable or fail deterministically; no false Knowledge recovery claim is introduced.
19. No new persistence mechanism is introduced.
20. Legacy AgentState Knowledge fields remain untouched unless an explicitly approved compatibility change is required.
21. `cargo fmt --check` passes.
22. `cargo clippy --all-targets --all-features -- -D warnings` passes.
23. `cargo check` passes.
24. `cargo test` passes.
25. `git diff --check` passes.
26. No unapproved C–G scope is included.

## 17. Recommended Implementation Decomposition

Implementation should proceed incrementally and stop after each verified increment.

### B2-C1 — Semantic Update Boundary

Define the minimal Core-level update interface and deterministic error boundary.

No AgentLoop closed-loop wiring beyond what is required to expose the interface.

### B2-C2 — AgentLoop Integration

Connect the boundary after successful ObservationStore insertion while preserving Observation-first ordering and existing Runtime behavior.

### B2-C3 — Closed-Loop Verification

Add deterministic integration tests proving semantic state reaches the next provider context and that failure / atomicity behavior is preserved.

The implementation may combine B2-C1 through B2-C3 in one coding increment only if the resulting diff remains within this approved scope and all acceptance criteria are independently verifiable.

## 18. Release Gate

Before B2-C can be marked `VERIFIED / RELEASE GATE PASSED`, the implementation must demonstrate:

- explicit Core semantic update ownership;
- Observation-first ordering;
- KnowledgeStore invariant enforcement;
- deterministic mutation failure behavior;
- complete Observation → Knowledge → Context → ProviderRequest path;
- no provider or Runtime Knowledge ownership;
- no `AgentDecision::UpdateKnowledge`;
- no new persistence;
- no C–G scope expansion;
- deterministic closed-loop tests;
- existing test suite remains green;
- formatting, lint, build, test, and diff checks pass.

## 19. Approval Boundary

This document is an **approved stable design baseline**.

B2-C is subdivided into:
- B2-C1: Core Semantic Update Boundary (COMPLETED)
- B2-C2: AgentLoop Integration Point (COMPLETED)
- B2-C3: Deterministic Closed-Loop Integration (NOT IMPLEMENTED / PENDING)
- B2-C4: Failure and Atomicity Boundary Verification (NOT IMPLEMENTED / PENDING)

Currently, only B2-C1 and B2-C2 have been implemented and verified. B2-C3 and B2-C4 must not be claimed as complete until implementation and verification are performed.

Approval of this document does not automatically approve C, D, E, F, or G roadmap items, and does not create a new implementation Stage.

After completing each increment, the implementation process must stop, verify, and wait for explicit next instruction.
