# Implementation Gap Analysis

## Purpose

This document records the post-Implementation-Cycle-1 comparison between the target specifications and the actual implementation baseline.

It is an analysis document, not a replacement for `docs/01_REQUIREMENTS.md` through `docs/05_TEST_PLAN.md`.

It must not be interpreted as an implementation schedule. A future implementation increment is selected only after the relevant analysis and dedicated design are reviewed and the implementation boundary is approved.

Baseline:

```text
Repository: sbvcid/ai-agent-v3
Branch: master
Last implementation-code baseline: b565f2a
```

The repository may contain later documentation-only commits. `b565f2a` identifies the source-code baseline being analyzed and should not be interpreted as the current repository HEAD.

---

## 1. Executive Finding

The current implementation is a deterministic closed-loop Agent Runtime foundation.

The basic execution cycle already exists:

```text
Decision
  ↓
Action
  ↓
Runtime
  ↓
ActionResult
  ↓
Observation
  ↓
AgentState
  ↓
Next Decision
```

The major missing capability is not another Runtime tool. It is the semantic layer that allows the Agent Core to interpret evidence, maintain hypotheses, detect progress or non-progress, change strategy, and select relevant context for the next decision.

The highest-value current gaps are therefore:

1. Observation Store.
2. Context Compiler.
3. Evidence / knowledge / hypothesis lifecycle semantics.
4. Progress detection.
5. Loop detection and circuit-breaker semantics.
6. Adaptive recovery / strategy revision.
7. Stronger goal verification.

Runtime expansion should follow these foundations rather than precede them.

---

## 2. Capability Matrix

| Area | Current state | Classification |
|---|---|---|
| Goal | Canonical model and persistence exist | Complete foundation |
| Action / ActionResult | Canonical boundary exists | Complete foundation |
| AgentDecision | Observe / Act / Wait / Finish exist | Complete foundation |
| ExecutionState / FinalTaskStatus | Separate and validated | Complete foundation |
| AgentState | Required fields exist | Partial semantics |
| Observation | Model exists | Partial |
| Observation Store | No dedicated store/query layer | Missing |
| Context Compiler | State-to-prompt formatting only | Missing as semantic component |
| KnowledgeState | Enum exists | Partial semantics |
| Evidence lifecycle | Basic evidence strings | Partial |
| Hypothesis lifecycle | No real lifecycle | Missing |
| Progress detection | No semantic mechanism | Missing |
| Loop detection | Step limit only | Missing |
| Adaptive recovery | No explicit strategy-revision mechanism | Missing |
| Goal verification | Basic invariant / evidence gate | Partial |
| Environment observation | Limited | Partial |
| Environment Delta | No dedicated mechanism | Missing |
| Job model | `running_jobs` storage only | Missing |
| Process Runtime | Synchronous foundation | Partial |
| Filesystem Runtime | Initial safe foundation | Partial |
| Shell Runtime | Not implemented as general capability | Missing |
| LLM provider abstraction | Exists | Complete foundation |
| Decision interpreter | Strict current tool surface | Complete for current scope |
| LLM correction loop | Not fully implemented | Partial / missing |
| Engineering Runtime | Not implemented | Missing |
| Browser Runtime | Not implemented | Deferred |
| GUI Runtime | Not implemented | Deferred |
| Advanced Runtime | Not implemented | Deferred |

---

## 3. Agent Core Gap

### 3.1 AgentState

The state model contains fields for understanding, problems, unknowns, hypotheses, evidence, recent actions, recent observations, running jobs, verification, and remaining work.

The gap is semantic lifecycle behavior. These fields are currently closer to storage than to a complete state-transition model.

Required direction:

```text
Observation
  ↓
Evidence
  ↓
Knowledge / Problem update
  ↓
Hypothesis update
  ↓
Progress evaluation
  ↓
Strategy decision
```

No new field should be added merely to make the state look more complete. New state must have explicit lifecycle semantics and tests.

### 3.2 Hypothesis Lifecycle

The specifications require the Agent to distinguish observations from interpretations and hypotheses.

The system must eventually support:

```text
create hypothesis
    ↓
seek evidence
    ↓
support / contradict / remain uncertain
    ↓
revise / replace / discard
```

A hypothesis must not silently become a fact because an LLM stated it.

### 3.3 Adaptive Recovery

A Runtime failure currently becomes an `ActionResult` / Observation and can be passed back to the decision source. That establishes the correct boundary but does not itself guarantee adaptation.

The Core must be able to distinguish:

```text
same strategy repeated
vs.
different strategy after new evidence
```

The intended behavior is:

```text
Failure
  ↓
Observation
  ↓
Evaluate current hypothesis
  ↓
Revise strategy
  ↓
Different action / additional observation
```

### 3.4 Progress Detection

Progress must not be equated with Action Success.

Examples of progress include:

- a new dependency discovered;
- an unknown becoming known;
- a failure cause narrowed;
- a required resource becoming available;
- a goal criterion becoming verified;
- a blocking condition being removed.

The implementation currently has no explicit progress model.

### 3.5 Loop Detection

`max_steps` is a safety bound, not semantic loop detection.

The Core eventually needs to recognize patterns such as:

```text
same action
same environment
same result
no new evidence
no progress
```

and then require investigation, strategy change, or a terminal decision rather than blindly continuing.

---

## 4. Observation and Context Gap

### 4.1 Observation Store

The current implementation has an `Observation` model but not a dedicated Observation Store abstraction.

The required conceptual operations are:

```text
append
retrieve
query
filter
associate with task
associate with action
associate with job
associate with resource
```

The first implementation does not need a database. A deterministic in-memory implementation is sufficient for the first semantic increment if it satisfies the required interface and tests.

### 4.2 Context Compiler

The current provider request construction is primarily a formatter over a small subset of `AgentState`.

The required direction is:

```text
Observation Store
      ↓
Context Compiler
      ↓
Provider-facing context
```

The compiler should be a semantic boundary, not merely a string-concatenation helper. The first implementation can use deterministic recency and explicit bounds; sophisticated relevance ranking is a later concern.

### 4.3 Environment Delta

The specifications eventually require the Agent to understand what changed in the environment after an action.

This is not necessarily a separate runtime subsystem. It may emerge from comparing observations, but the semantic contract must eventually be explicit.

---

## 5. Runtime Gap

### 5.1 Process / Job

The current Process Runtime can execute bounded synchronous processes, including timeout handling and cleanup.

It does not yet provide a generalized asynchronous Job model with stable identity, persistent lifecycle, cancellation, reconciliation, and resume semantics.

### 5.2 Filesystem

The current Filesystem Runtime is intentionally a safe foundation. It is not the full long-term engineering filesystem capability set.

### 5.3 Shell

A generalized interactive shell capability is not yet implemented. It should not be introduced merely as a shortcut around missing Agent Core semantics.

### 5.4 Engineering Runtime

The eventual engineering capability layer should support repository inspection, file editing, build/test execution, diagnostics, Git operations, rollback/checkpointing, and engineering-specific verification.

It must remain a capability layer. It must not encode a fixed repair workflow that replaces Agent Core reasoning.

---

## 6. Provider Gap

The provider boundary is established and the current interpreter is intentionally strict.

Remaining gaps include:

- bounded correction/retry for malformed provider decisions;
- richer structured context;
- stronger validation against current Agent Core state;
- future provider-independent context compilation.

These should preserve the rule that provider output is untrusted input and never executes directly.

---

## 7. Persistence Gap

Phase 0 JSON checkpointing is sufficient for the current implementation baseline.

An Observation Store does not automatically require SQLite.

A stronger persistence mechanism should be introduced only when actual requirements include durable observation history, transactional updates, crash-safe append semantics, large retention, or other needs that JSON checkpointing cannot reasonably satisfy.

---

## 8. Dependency Assessment

The most useful current dependency relationship is:

```text
Observation History
        ↓
Context Compilation
        ↓
Evidence / Knowledge
        ↓
Hypothesis Semantics
        ↓
Progress / Loop Awareness
        ↓
Adaptive Recovery
        ↓
Stronger Goal Verification
```

This is a dependency assessment, not a mandatory linear implementation schedule.

Some branches have partially independent dependencies. In particular:

- Job lifecycle has its own execution/persistence dependencies and should not be artificially delayed until every Agent Core semantic is complete;
- environment discovery is primarily a capability branch, but its results must enter the observation/knowledge path;
- Engineering Runtime, Browser, GUI, and Vision are capability branches that must reuse the same Agent Core semantic boundaries;
- progress and loop detection may be designed together when their evidence requirements overlap.

The dedicated design documents determine the actual implementation boundary for each future increment.

---

## 9. Recommended Design Direction

The next coherent Agent Core design baseline is:

```text
Observation Store
        ↓
Context Compiler
```

The detailed stable design is maintained in:

`docs/07_AGENT_CORE_OBSERVATION_INCREMENT_DESIGN.md`

The broader dependency and capability direction is maintained in:

`docs/08_AGENT_CORE_ROADMAP.md`

No Stage number is assigned merely by this recommendation. Implementation begins only after the relevant design is reviewed and the user approves the implementation boundary.

---

## 10. Important Constraints

The following remain architectural constraints:

1. Original Goal persists throughout the task lifecycle.
2. Observation is evidence, not interpretation.
3. Hypothesis is not automatically Fact.
4. Action Success is not Goal Success.
5. Runtime Failure is an Observation, not automatically Task Failure.
6. New observations may change subsequent decisions.
7. LLM output never executes directly.
8. Goal Verification targets the Original Goal.
9. Observation history and provider context remain conceptually separate.
10. Runtime provides capabilities and does not replace Agent Core reasoning.
11. The system must not depend on a fixed high-level workflow.
12. Runtime capability expansion must not require redesigning Agent Core.
13. New state must have explicit semantics and tests.
14. Determinism is preferred for core transformations and testable state transitions.
15. Temporary implementation instructions do not become permanent architecture merely because they were useful for one coding session.

---

## 11. Conclusion

Implementation Cycle 1 established the execution foundation but not the full autonomous semantic loop.

The next architectural step is to establish a durable observation/history boundary and a deterministic context compilation boundary. This creates the evidence path required for later knowledge, hypothesis, progress, loop-detection, recovery, and stronger verification behavior.

The gap analysis remains an assessment document. It should be revised when the implementation baseline changes materially or when the comparison against the stable target specifications materially changes; it should not be used as a running implementation diary.
