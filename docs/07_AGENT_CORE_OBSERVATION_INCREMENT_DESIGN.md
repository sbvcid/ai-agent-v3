# Agent Core Observation Increment Design

## 1. Purpose

This document defines the next implementation increment after the first implementation cycle. It is a design artifact, not an implementation-status document and not a request to weaken the target specifications.

The first cycle established a deterministic closed-loop foundation. The next increment must add the minimum semantic machinery required for the Agent Core to reason from accumulated observations rather than merely pass the latest in-memory state back to the provider.

This document intentionally does not name the increment as a numbered Stage. The implementation stage number remains reserved until this design is reviewed and accepted.

## 2. Current Baseline

The current master branch provides:

- canonical `Goal`, `Action`, `ActionResult`, `Observation`, `AgentDecision`, `AgentState`, execution state and terminal status models;
- `Runtime` abstraction and filesystem/process runtime capabilities;
- provider abstraction and strict provider-response interpretation;
- a deterministic in-memory closed loop;
- Phase 0 JSON checkpoint persistence;
- deterministic unit/integration tests for the existing foundation.

The current Agent Core still has important semantic gaps:

- observations are stored only in `AgentState.recent_observations`;
- there is no independent Observation Store;
- provider context is assembled directly inside `ProviderRequest::from_agent_state`;
- there is no explicit context selection/compilation boundary;
- evidence has no structured lifecycle or provenance semantics;
- hypotheses are only strings in `AgentState`;
- there is no progress model;
- there is no actual loop-detection mechanism;
- there is no adaptive recovery mechanism;
- goal verification remains structurally present but semantically weak.

## 3. Design Goal

The increment must establish this boundary:

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

The core objective is not to create a larger tool catalog. It is to make observations durable within the running Agent Core and make provider context an explicit, deterministic compilation step.

## 4. Exact Scope

### 4.1 Observation Store

Introduce an explicit `ObservationStore` abstraction responsible for retaining observations independently of the LLM context representation.

Minimum responsibilities:

- append observations;
- retrieve observations by deterministic order;
- retrieve observations by stable identifier;
- retain source action identity when present;
- expose a bounded recent view for compatibility with the current loop;
- provide deterministic behavior suitable for tests.

The first implementation may be in-memory. Persistence beyond the existing checkpoint mechanism is not required in this increment.

### 4.2 Context Compiler

Introduce a `ContextCompiler` abstraction responsible for transforming Agent Core state plus stored observations into a provider-facing request context.

Minimum responsibilities:

- preserve the Original Goal;
- include the current understanding when present;
- include relevant recent observations;
- include recent actions where useful for decision continuity;
- preserve explicit unknowns/problems when they are already represented;
- enforce deterministic ordering and bounded context construction;
- keep provider formatting separate from observation storage.

The first compiler does not need sophisticated semantic ranking. Deterministic recency-based selection is acceptable as the initial implementation, provided the interface allows a later compiler to select by relevance/evidence.

### 4.3 AgentState Integration

Integrate the new abstractions without breaking the existing public semantic interfaces unnecessarily.

The design should avoid duplicating the complete Observation Store inside `AgentState`. `AgentState` may retain a bounded compatibility view such as `recent_observations`, while the store becomes the authoritative in-process observation history for the new path.

Checkpoint compatibility must be considered explicitly. Existing checkpoints must either remain loadable or fail with a clear, deterministic compatibility error; silent reinterpretation is prohibited.

### 4.4 DecisionSource Integration

`ProviderDecisionSource` should obtain provider context through the Context Compiler rather than constructing the prompt directly from `AgentState`.

The provider adapter remains responsible for wire/protocol details. The compiler remains responsible for semantic context selection. These responsibilities must not be merged.

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

These are deliberately deferred because the Observation Store and Context Compiler are dependency foundations for later Agent Core behavior.

## 6. Proposed Semantic Interfaces

The exact Rust names may be adjusted during implementation only when the semantic boundary remains unchanged.

Conceptually:

```rust
pub trait ObservationStore {
    fn record(&mut self, observation: Observation) -> Result<(), ObservationStoreError>;
    fn get(&self, id: &str) -> Option<&Observation>;
    fn all(&self) -> &[Observation];
    fn recent(&self, limit: usize) -> Vec<&Observation>;
}
```

And:

```rust
pub trait ContextCompiler {
    fn compile(&self, state: &AgentState, observations: &dyn ObservationStore)
        -> Result<CompiledContext, ContextCompileError>;
}
```

`CompiledContext` should be a semantic intermediate representation rather than an opaque prompt string where practical. Provider-specific serialization can occur after compilation.

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

The implementation must validate bounds and preserve deterministic ordering.

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
- reject invalid observation data if the existing domain invariant requires rejection;
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
- older observations are retained by the store even when excluded from a bounded context.

### Integration tests

- action result becomes an observation and is stored;
- next provider request is produced through the Context Compiler;
- the provider-facing context contains observations from previous actions;
- Observation Store and compiled context remain separate representations;
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

## 14. Implementation Boundary for the Coding Agent

The coding agent must:

1. inspect the current repository and target documents before editing;
2. implement only the approved Observation Store + Context Compiler increment;
3. preserve existing semantic interfaces unless a documented compatibility issue requires change;
4. add deterministic tests before declaring completion;
5. run `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets --all-features -- -D warnings` where supported, and `cargo test`;
6. report changed files, test results, and any deviations from this design;
7. stop after the increment rather than proceeding into hypothesis, loop detection, recovery, or new runtime capabilities.

## 15. Release Gate

The increment is accepted only if:

- the repository builds and tests pass;
- Observation Store and Context Compiler are explicit boundaries;
- provider context no longer depends on ad-hoc prompt construction inside `ProviderRequest::from_agent_state`;
- historical observations can exist independently of the bounded provider context;
- deterministic behavior is demonstrated by tests;
- existing runtime semantics remain intact;
- no unapproved feature expansion is present;
- a reviewer can trace an action result into stored observation data and then into the next compiled provider context.

A passing compilation alone is insufficient.

## 16. Next Dependency After This Increment

After this increment is accepted, the next design question is evidence/knowledge semantics and hypothesis lifecycle. Only after those semantics exist should progress detection, loop detection, and adaptive recovery be implemented as explicit Agent Core behavior.
