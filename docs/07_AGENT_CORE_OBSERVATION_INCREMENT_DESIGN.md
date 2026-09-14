# Agent Core Observation Increment Design

## 1. Purpose

This document defines the first post-cycle Agent Core design increment. It is a stable design baseline, not an implementation-status document and not a temporary coding task.

The first implementation cycle established a deterministic closed-loop foundation. The next coherent semantic increment is to make observations durable within the running Agent Core and make provider context an explicit, deterministic compilation step.

This document intentionally does not assign a numbered implementation Stage. Stage numbering belongs to the development workflow and is assigned only after the design boundary is approved for implementation.

## 2. Current Baseline

The current master branch provides:

- canonical `Goal`, `Action`, `ActionResult`, `Observation`, `AgentDecision`, `AgentState`, execution state and terminal status models;
- `Runtime` abstraction and filesystem/process runtime foundations;
- provider abstraction and strict provider-response interpretation;
- a deterministic in-memory closed loop;
- Phase 0 JSON checkpoint persistence;
- deterministic unit/integration tests for the existing foundation.

The current Agent Core still has important semantic gaps:

- observations are primarily retained through `AgentState.recent_observations` rather than an independent history boundary;
- provider context is assembled directly from `AgentState` rather than through an explicit semantic compiler;
- evidence has no structured lifecycle or provenance semantics;
- hypotheses are only represented as state data rather than a lifecycle;
- there is no explicit progress model;
- there is no actual loop-detection mechanism;
- there is no adaptive recovery mechanism;
- goal verification remains structurally present but semantically limited.

## 3. Design Goal

The increment establishes this semantic boundary:

```text
Runtime
  ↓
ActionResult / Observation
  ↓
Observation Store
  ↓
Context Compiler
  ↓
Provider Request
  ↓
Agent Decision
```

The objective is not to create a larger tool catalog. It is to make observations durable within the running Agent Core and make provider context an explicit, deterministic compilation step.

## 4. Exact Scope

### 4.1 Observation Store

Introduce an explicit `ObservationStore` abstraction responsible for retaining observations independently of the provider context representation.

Minimum responsibilities:

- record observations;
- retrieve observations by stable identifier;
- retrieve observations in deterministic order;
- retain source action identity when present;
- provide a bounded recent view for context construction and compatibility;
- provide deterministic behavior suitable for tests.

The first implementation may be in-memory. It may retain the session's observations without an independent retention policy. Historical retention and durable persistence are separate future concerns.

### 4.2 Context Compiler

Introduce a `ContextCompiler` abstraction responsible for transforming Agent Core state plus stored observations into a provider-facing semantic context.

Minimum responsibilities:

- preserve the Original Goal;
- include current understanding when present;
- include relevant recent observations;
- include recent actions where useful for decision continuity;
- preserve explicit unknowns/problems already represented by the Core;
- enforce deterministic ordering and bounded context construction;
- keep semantic context selection separate from provider wire formatting.

The first compiler does not require sophisticated semantic ranking. Deterministic recency-based selection is acceptable provided the abstraction permits later relevance/evidence-aware selection.

The Core must not require a provider-specific tokenizer or token-counting implementation merely to establish this boundary. The first bounded policy should use explicit deterministic semantic limits such as maximum observations/actions. Provider-specific serialization or token budgeting may be layered later without changing the ownership boundary.

### 4.3 AgentState Integration

Integrate the abstractions without unnecessarily breaking existing public semantic interfaces.

The Observation Store becomes the authoritative in-process observation history for the new path. `AgentState.recent_observations`, if retained, is a bounded compatibility/operational view and must not become a second independent history.

When a new observation is produced:

```text
ActionResult / environment event
        ↓
Observation Store.record
        ↓
update bounded AgentState.recent_observations view
```

The store is authoritative for the new observation path; the bounded state view exists for compatibility and checkpoint continuity.

Checkpoint compatibility must be explicit. Existing checkpoints must either remain loadable or fail with a clear, deterministic compatibility error. Silent reinterpretation is prohibited.

Because the current Phase 0 checkpoint stores the bounded `AgentState` representation rather than a complete Observation Store history, restoring an existing checkpoint can reconstruct only the observations retained in that checkpoint. The design must not claim that older observations have been recovered when they were never persisted.

### 4.4 DecisionSource Integration

`ProviderDecisionSource` should obtain semantic provider context through the Context Compiler rather than constructing context directly from `AgentState`.

The provider adapter remains responsible for wire/protocol details. The compiler remains responsible for semantic context selection and ordering. These responsibilities must not be merged.

## 5. Explicit Non-Goals

This increment does not implement:

- full Hypothesis lifecycle;
- formal Evidence objects or evidence scoring;
- progress scoring;
- loop detection;
- adaptive recovery policies;
- Environment Delta semantics;
- long-running Job lifecycle;
- browser automation;
- GUI/vision automation;
- unrestricted shell execution;
- Git/Engineering Runtime;
- vector databases or RAG;
- SQLite;
- multi-agent orchestration;
- a large planner/tree-search system;
- automatic workflow templates.

These remain separate future concerns because this increment is a semantic foundation rather than a complete adaptive-agent implementation.

## 6. Semantic Interfaces

The exact Rust names and storage representation may be adjusted during implementation when necessary for idiomatic integration, but the semantic boundary must remain unchanged.

Conceptually:

```rust
pub trait ObservationStore {
    fn record(&mut self, observation: Observation) -> Result<(), ObservationStoreError>;
    fn get(&self, id: &str) -> Option<&Observation>;
    fn recent(&self, limit: usize) -> Vec<&Observation>;
}
```

The interface should not unnecessarily expose the store's underlying `Vec`, map, or other storage representation. If full ordered traversal is required by the implementation, expose semantic retrieval rather than committing the architecture to a concrete container API.

And:

```rust
pub trait ContextCompiler {
    fn compile(&self, state: &AgentState, observations: &dyn ObservationStore)
        -> Result<CompiledContext, ContextCompileError>;
}
```

`CompiledContext` is a semantic intermediate representation, not an opaque prompt string.

A minimal conceptual representation is:

```rust
pub struct CompiledContext {
    pub goal: Goal,
    pub understanding: String,
    pub observations: Vec<Observation>,
    pub recent_actions: Vec<Action>,
    pub unknowns: Vec<String>,
    pub active_problems: Vec<String>,
}
```

The implementation may use equivalent domain types where the existing repository already has stronger invariants. It must validate bounds and preserve deterministic ordering.

## 7. Ownership Rules

Observation Store owns observation retention and retrieval.

Context Compiler owns context selection, ordering, and bounded compilation.

Agent Core owns reasoning, decision making, verification, and adaptation.

Runtime owns capability execution only.

Provider Adapter owns wire-format conversion only.

No layer may silently take over another layer's semantic responsibility.

## 8. Observation Semantics

For this increment, an observation remains evidence produced by the environment or by execution. It must not be silently converted into a fact, hypothesis, or verified goal.

In particular:

- `ActionResult.success == true` does not imply goal completion;
- a failed action becomes an observation available to later reasoning;
- missing information remains unknown;
- an observation may support a later hypothesis without itself becoming a hypothesis;
- an observation must retain its `source_action_id` when the observation originated from an action.

No new confidence score is introduced merely for storage convenience.

Duplicate observation identifiers must have deterministic behavior. The implementation must either reject duplicates or define an explicit replacement/update semantic; silent ambiguous duplication is not acceptable.

Invalid observations must be handled according to existing domain invariants. An empty store must have defined, deterministic behavior.

## 9. Determinism Requirements

For identical state and identical stored observations, the Context Compiler must produce identical compiled context.

Required deterministic properties:

- stable observation ordering;
- stable action ordering;
- stable bounded selection;
- no wall-clock-dependent selection;
- no random sampling;
- no provider-dependent context selection;
- no hidden global state.

## 10. Compatibility Requirements

The existing `DecisionSource` abstraction remains valid.

The existing `Runtime` trait remains unchanged.

The canonical `Action` and `ActionResult` models remain unchanged unless an actual incompatibility is demonstrated.

The provider interpreter remains responsible for validating provider output and must not be moved into the Context Compiler.

Existing tests must continue to pass unless a test encodes behavior explicitly superseded by this design. Any such change requires an explicit rationale.

## 11. Test Requirements

The increment is not complete merely because the new types compile.

### Observation Store tests

- record one observation and retrieve it;
- record multiple observations and preserve deterministic order;
- retrieve by identifier;
- bounded recent retrieval;
- preserve `source_action_id`;
- define duplicate-ID behavior;
- reject invalid observation data when existing domain invariants require rejection;
- verify empty-store behavior.

### Context Compiler tests

- compile Original Goal;
- compile current understanding;
- include recent observations;
- include recent actions;
- include unknowns and active problems when present;
- deterministic ordering;
- deterministic bounded selection;
- empty optional fields do not create misleading placeholders;
- identical inputs produce identical output;
- older observations remain retained by the store even when excluded from a bounded context;
- context construction does not mutate or delete historical observations.

### Integration tests

- action result becomes an observation and is stored;
- next provider request is produced through the Context Compiler;
- provider-facing context contains observations from previous actions;
- Observation Store and compiled context remain separate representations;
- checkpoint restoration preserves the explicitly retained observation subset without claiming unavailable history;
- existing closed-loop behavior remains intact.

## 12. Acceptance Mapping

This increment directly supports, but does not by itself complete, the following target behaviors:

- failure becomes an observation available for later reasoning;
- observation history is distinct from LLM context;
- context can be bounded without deleting historical observations;
- action/result history remains traceable;
- future adaptive recovery has a stable evidence source.

It does not claim to satisfy the full acceptance tests for hypothesis revision, loop detection, strategy change, long-running jobs, or full goal verification. Those remain future increments.

## 13. Migration Impact

Expected source impact:

- add an Observation Store module and tests;
- add a Context Compiler module and tests;
- modify `AgentLoop` to record observations through the store;
- modify `ProviderDecisionSource` to use the compiler;
- add the minimum state wiring required for construction and checkpoint compatibility;
- preserve existing Runtime and interpreter behavior.

Avoid broad refactoring of unrelated runtime code.

## 14. Release Gate

The increment is accepted only if:

- the repository builds and tests pass;
- Observation Store and Context Compiler are explicit boundaries;
- provider context no longer depends on ad-hoc semantic prompt construction inside `ProviderRequest::from_agent_state`;
- historical observations can exist independently of the bounded provider context;
- deterministic behavior is demonstrated by tests;
- existing runtime semantics remain intact;
- no unapproved feature expansion is present;
- a reviewer can trace an action result into stored observation data and then into the next compiled provider context.

A passing compilation alone is insufficient.

## 15. Relationship to Later Work

This increment establishes a stable evidence/context boundary for later Agent Core semantics. It does not prescribe that every later capability must be implemented serially.

Evidence/knowledge semantics and hypothesis lifecycle depend strongly on a reliable observation history. Progress detection and loop detection also benefit from that history but may have partially independent design work. Job lifecycle, environment discovery, and runtime capability expansion have their own semantic dependencies and must not be artificially coupled to this increment when they can be designed independently.

Later implementation work must be specified by a dedicated design decision before a coding task is issued. Temporary implementation instructions belong to the active development conversation or status record, not to this permanent design baseline.
