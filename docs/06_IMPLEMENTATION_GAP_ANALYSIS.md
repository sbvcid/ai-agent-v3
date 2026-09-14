# Implementation Gap Analysis

## Purpose

This document records the post-Implementation-Cycle-1 comparison between the target specifications and the actual implementation on `master`.

It is an analysis document, not a replacement for `docs/01_REQUIREMENTS.md` through `docs/05_TEST_PLAN.md`.

It must not be interpreted as an implementation schedule. The next implementation increment is selected only after this analysis and the dedicated increment design are reviewed.

Baseline:

```text
Repository: sbvcid/ai-agent-v3
Branch: master
Baseline commit: b565f2a
```

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

A true Context Compiler should select and organize information based on decision relevance.

Minimum conceptual input:

```text
Original Goal
Current State
Relevant Environment
Relevant Observations
Recent Actions
Current Problems
Active Hypotheses
Verification State
Available Runtime Capabilities
```

The compiler should own selection and context-budget policy rather than the provider adapter.

### 4.3 Environment Delta

The specifications distinguish environmental evidence from the Agent's interpretation and require enough information to understand what changed after actions.

A dedicated Environment Delta mechanism is not yet present.

---

## 5. Verification Gap

The current implementation correctly prevents `Finish(Done)` without verification evidence.

That invariant should be preserved.

The remaining gap is that verification itself is still lightweight. The long-term design requires evidence tied to the Original Goal and the ability to distinguish:

```text
Verified
Failed
Uncertain
```

from merely observing that an individual Action succeeded.

The next semantic design must strengthen this without turning verification into a Runtime action.

---

## 6. Job / Runtime Gap

The Process Runtime already has a useful synchronous foundation including timeout and termination.

The missing abstraction is the generalized Job lifecycle:

```text
Created
  ↓
Running
  ↓
Waiting / Progressing
  ↓
Completed / Failed / TimedOut / Cancelled
```

The Agent must be able to observe a running job without treating it as an immediate task failure.

Job persistence, reconciliation, cancellation, and background execution remain later increments after the core semantic layer is established.

---

## 7. Engineering Runtime Gap

The long-term Engineering Runtime requires repository inspection, code search, editing, diff, build, test, dependency inspection, Git, rollback, and engineering-specific verification.

These are intentionally deferred from the immediate increment.

The reason is architectural rather than convenience: increasing the tool surface before adaptive Agent Core semantics are established would make it harder to distinguish a genuinely adaptive agent from a large collection of tools driven by implicit workflow assumptions.

---

## 8. Acceptance Coverage Gap

The most important uncovered behavioral classes are:

- failure → hypothesis/strategy revision → success;
- repeated or equivalent actions without progress;
- successful actions without goal progress;
- observation before environment-dependent action;
- unknown → investigation → known;
- timeout → wait/observe/alternative strategy;
- goal verification independent of action success;
- long-running Job lifecycle;
- conflicting or changing observations;
- context selection from historical evidence;
- bounded correction after invalid LLM output.

The existing deterministic test doubles provide the right foundation for these tests.

---

## 9. Dependency Graph

The main dependency relationship is:

```text
Observation Store
        ↓
Context Compiler
        ↓
Evidence / Knowledge Semantics
        ↓
Hypothesis Lifecycle
        ↓
Progress Detection
        ↓
Loop Detection
        ↓
Adaptive Recovery
        ↓
Stronger Goal Verification
        ↓
Broader Runtime / Engineering Capabilities
```

This graph is not a mandatory schedule. It identifies which semantics depend on which foundations.

---

## 10. Explicit Non-Goals for the Next Increment

The next increment should not simultaneously attempt to implement:

- Browser automation.
- GUI automation.
- Vision.
- Full Engineering Runtime.
- Multi-agent scheduling.
- Vector databases / RAG.
- A large planner tree.
- SQLite merely for storage convenience.
- A fixed workflow for software engineering tasks.
- A large new tool catalog without corresponding Agent Core semantics.

---

## 11. Required Next Design Artifact

Before implementation begins, create a dedicated design document for the first post-cycle increment.

That document should define:

- exact scope;
- semantic interfaces;
- ownership boundaries;
- data model changes;
- state-transition invariants;
- deterministic tests;
- acceptance tests;
- migration impact;
- explicit non-goals;
- review and release gate.

The design document must not weaken the existing formal requirements. It should describe the smallest implementation that closes a coherent subset of the identified semantic gaps.
