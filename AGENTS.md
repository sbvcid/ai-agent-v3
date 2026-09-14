# AI Agent v3 — Codex Instructions

## 1. Purpose

This repository implements AI Agent v3, a Rust-based local AI Agent.

The project is intended to provide a general Agent Runtime and Agent Core rather than a fixed workflow designed around the limitations of today's LLMs.

The core architectural principles are:

```
Agent Core = reasoning, understanding, hypothesis, planning, decision, adaptation, verification
Runtime    = computer operation
Observation = evidence connecting the environment to the Agent
Persistence = state required for continuity and recovery
```

The Agent must be able to observe the environment, reason about the current situation, act on the environment, inspect the result, revise its approach when necessary, and verify whether the original user goal was actually achieved.

Do not turn the project into a collection of hard-coded workflows.

---

## 2. Source of Truth

Before making substantial changes, read the current formal specifications:

1. `docs/01_REQUIREMENTS.md`
2. `docs/02_CONSTRUCTION.md`
3. `docs/03_INTERFACES.md`
4. `docs/04_ACCEPTANCE_TESTS.md`
5. `docs/05_TEST_PLAN.md`

These documents have different purposes:

```text
01 Requirements
   What the system is required to achieve.

02 Construction / implementation constraints
   Architectural and construction principles that implementation must preserve.

03 Interface contracts
   Stable semantic boundaries between components.

04 Behavioral acceptance criteria
   Observable behaviors that determine whether requirements are satisfied.

05 Test strategy
   How those behaviors should be tested and evidenced.

IMPLEMENTATION_STATUS.md describes the current implementation state.

It is the only project document whose primary purpose is to track implementation progress.

README.md is the human-facing project introduction. It describes the project purpose, principles, and target architecture, but is not an implementation progress tracker.

history/ contains historical implementation records. It is archival only and must not be treated as current project instructions or source of truth unless historical context is explicitly required.

Do not treat these documents as interchangeable.

If the documents conflict, do not silently choose one interpretation.

First identify the conflict, determine the interpretation that best preserves the overall architecture, and update the affected specification when necessary.

Do not knowingly implement an unresolved contradiction.

---

## 3. Architectural Principles

Preserve these principles unless the specifications are deliberately changed:

* Agent Core decides what should happen.
* Runtime provides capabilities for performing computer operations.
* Observation provides environmental evidence.
* Verification determines whether the intended goal was actually achieved.
* Failure is an Observation, not automatically a terminal task state.
* Unknown is a valid state.
* Unsupported assumptions must not be silently converted into facts.
* Hypotheses are revisable and must not be treated as facts.
* The Agent may retry, investigate, wait, change strategy, request user input, or terminate depending on evidence.
* Repeated failure should cause strategy evaluation rather than blind repetition.
* Runtime must not impose a fixed high-level workflow on Agent Core.
* Runtime capabilities should remain useful as LLM capabilities improve.
* Do not encode temporary weaknesses of a particular local model into the architecture.
* Do not expose or persist private chain-of-thought.
* Persist decisions, actions, observations, evidence, hypotheses, results, and verification state instead.

The architecture should remain general enough that a substantially stronger LLM can use the same Runtime without requiring a fundamental redesign.

---

## 4. Decision and Action Semantics

The logical Agent decision model is:

```
AgentDecision
  Observe
  Act
  Wait
  Finish
```

`Act` causes a Runtime action to be performed.

The Runtime action model is:

```
Action
  Observe
  Execute
  Interact
  Wait
```

Verification is Agent Core behavior.

Verification is not a special Runtime Action.

Terminal task outcomes are:

```
Done
Blocked
Impossible
NeedUser
```

Do not introduce `Continue` as a terminal task outcome.

`Verify` may be used as the name of an internal function or conceptual Agent behavior when appropriate, but do not add it to the Runtime Action enum merely to represent reasoning.

---

## 5. Runtime Design

Runtime capabilities should be expressed as general capability contracts rather than hard-coded workflows.

Relevant capability domains include:

* Windows OS
* Process
* Shell
* Filesystem
* GUI
* Browser
* Network
* Jobs / long-running processes
* Security / policy boundaries
* Engineering Runtime

Engineering Runtime should provide general development capabilities such as:

* repository exploration
* code search
* symbol/reference discovery
* dependency inspection
* file editing
* diff inspection
* build
* test
* static analysis
* Git
* checkpoint / rollback
* debugging

Do not implement a mandatory workflow such as:

```
search -> edit -> build -> test
```

The Agent must be able to choose a different sequence when the evidence requires it.

Do not create specialized tools merely because one current test happens to use them.

Prefer general capabilities that can support multiple tasks.

---

## 6. Environment and Permission Boundaries

Do not assume elevated privileges.

`AccessDenied`, UAC restrictions, unavailable GUI sessions, missing permissions, and similar conditions are environmental observations.

The Agent may attempt reasonable alternatives when appropriate.

If actual user action, authorization, credentials, physical interaction, or an unavailable interactive environment is required, the task may end as:

```
NeedUser
```

Do not retry an impossible permission boundary indefinitely.

Do not implement unrestricted privilege escalation.

Do not hide permission failures from Agent Core.

---

## 7. Long-Running Jobs

Long-running operations must not be represented only by a blocking function call when the operation needs to remain observable or controllable.

Where appropriate, use persistent Job identity and state so the Agent can:

* start a job
* observe its state
* wait
* inspect progress or output
* cancel it
* recover after process interruption
* continue reasoning from the latest known state

Timeout is an Observation.

Timeout does not automatically mean:

```
Failed
Blocked
Impossible
NeedUser
```

The Agent must evaluate the evidence and decide what to do next.

---

## 8. Observation and State

Observations are evidence from the environment.

Keep raw observations available for later inspection.

The Context Compiler may select, summarize, compress, or prioritize observations for the LLM context, but must preserve decision-critical evidence.

Do not hard-code a fragile context-token ratio as an architectural requirement.

`Fact`, `Observed`, `Inferred`, `Hypothesis`, and `Unknown` are epistemic concepts used by Agent Core.

Do not force raw Runtime observations into these categories unless the interface specification explicitly requires it.

---

## 9. LLM Boundary

Treat LLM output as untrusted structured input.

The conceptual processing pipeline is:

```
parse
  ->
validate
  ->
normalize
  ->
policy / capability validation
  ->
execute
```

Prefer structured or schema-constrained output where the selected LLM provider supports it.

Malformed output may receive a bounded repair or correction attempt.

Do not execute arbitrary model output merely because it is syntactically valid text.

Validate, as appropriate:

* decision type
* action type
* required fields
* capability availability
* policy constraints
* argument validity
* task state compatibility

Do not assume the model will always produce valid JSON or valid action arguments.

---

## 10. Loop Detection and Adaptation

The Agent must avoid blind repetition.

Relevant evidence may include:

* repeated or highly similar actions
* repeated failures
* lack of meaningful environment change
* lack of progress toward the goal
* conflicting observations
* elapsed time
* Job state

Loop detection should inform Agent Core decision making.

Loop detection must not itself become the final task-status authority.

Do not solve loop-detection acceptance tests through hard-coded special cases.

The Agent should be capable of changing strategy rather than merely counting retries.

---

## 11. Verification

Never equate:

```
Action Success == Goal Success
```

Verification should operate at useful levels such as:

```
Action Success
Subtask Success
Goal Success
```

Final verification must relate to the original user goal.

For example:

```
command succeeded
but requested output is missing
=>
Goal is not verified
```

Likewise:

```
command reported failure
but environment already satisfies the requested goal
=>
Agent should inspect actual state before concluding that the goal failed
```

When verification is uncertain, preserve the uncertainty instead of claiming success.

---

## 12. Persistence and Recovery

The Agent must support continuity for long-running or multi-step tasks.

The exact persistence technology is an implementation choice unless explicitly constrained elsewhere.

The implementation should support, where required:

* Agent state persistence
* action history
* observations
* Job state
* checkpoints
* crash recovery
* rollback where appropriate

Do not create a persistence abstraction merely to satisfy a test if it cannot support real recovery semantics.

---

## 13. Security and Safety

Keep these concepts separate:

```
Capability = what Runtime technically exposes
Policy     = what Agent is currently allowed to request
OS security = what Windows actually permits
```

A capability declaration is not proof that an operation will succeed.

Do not bypass Windows security boundaries.

Do not silently elevate privileges.

Do not hide security-related failures from Agent Core.

---

## 14. Implementation Strategy

Prefer the simplest architecture that satisfies the behavioral requirements.

Do not prematurely add:

* multi-agent systems
* vector databases
* RAG infrastructure
* giant planner trees
* fixed workflow DSLs
* dozens of specialized AI tools
* unrestricted GUI automation
* unrestricted privilege escalation
* elaborate world models
* complicated long-term memory systems

Use abstractions when they clarify stable boundaries.

Do not introduce abstractions merely because they appear theoretically elegant.

Prefer standard Rust patterns and well-maintained crates.

Do not add dependencies without a concrete reason.

Unless the specifications explicitly constrain implementation details, Codex should determine:

* crate selection
* internal data structures
* serialization format
* persistence implementation
* module organization
* async/runtime strategy
* process management implementation
* exact testing framework

The specifications define required behavior and stable contracts, not every implementation detail.

---

## 15. Change Procedure

For every non-trivial change:

1. Inspect the current implementation.
2. Read the relevant specification sections.
3. Identify affected interfaces.
4. Identify affected acceptance requirements.
5. Check for contradictions or missing assumptions.
6. Design the smallest coherent change.
7. Implement the change.
8. Add or update tests.
9. Run appropriate verification commands.
10. Inspect the resulting diff.
11. Re-check affected acceptance criteria.
12. Report remaining uncertainty or limitations honestly.

Do not modify unrelated code merely to make the current task appear complete.

Do not rewrite large parts of the architecture without first establishing why the existing design cannot satisfy the requirements.

---

## 16. Testing Requirements

Use:

```
docs/04_ACCEPTANCE_TESTS.md
```

as the behavioral acceptance specification.

Use:

```
docs/05_TEST_PLAN.md
```

as the testing strategy.

For deterministic behavior, prefer automated unit or integration tests.

For cross-module behavior, prefer integration tests.

For complex autonomous behavior, use end-to-end or scenario tests where practical.

For GUI, browser, UAC, or environment-specific behavior, manual acceptance testing may be appropriate when reliable automation is impractical.

Do not create tests that merely reproduce implementation details when the requirement is behavioral.

Do not make acceptance tests pass through hard-coded special cases.

A passing:

```
cargo test
```

is useful evidence, but it is not by itself proof that all Agent behavioral requirements are satisfied.

---

## 17. Required Verification Before Completion

When applicable, run:

```
cargo fmt --check
cargo check
cargo test
```

Run additional repository-specific checks when appropriate, including linting, static analysis, integration tests, or end-to-end tests.

If a required check cannot be run, state why.

For Agent behavior changes, add or update tests that provide evidence for the affected acceptance requirements.

For significant changes, inspect:

```
git diff
```

and ensure no accidental files, generated artifacts, secrets, or unrelated modifications were introduced.

---

## 18. Handling Specification Problems

If you discover:

* contradictory requirements
* an impossible interface
* an acceptance test that cannot be objectively evaluated
* an implementation requirement that unnecessarily over-constrains architecture
* a missing state transition
* a missing error case
* a security boundary that is not represented
* a behavior that cannot be implemented from the current interfaces

do not silently work around the problem.

Analyze the issue first.

Prefer revising the specification over introducing hidden assumptions.

When a specification change is necessary, explain:

* what is ambiguous or contradictory
* why the change is necessary
* what behavior the revised wording establishes
* which other documents are affected

Do not silently rewrite requirements merely to simplify implementation.

---

## 19. Completion Report

When reporting completed development work, summarize:

1. What changed.
2. Which specifications or interfaces were affected.
3. Which tests and verification checks were run.
4. The relevant results.
5. Any remaining limitations or uncertainty.

Do not claim a requirement is satisfied without corresponding evidence.

Do not claim an end-to-end behavior is verified merely because the code compiles.

---

## 20. General Rule

The objective is not to make the current test cases pass through special handling.

The objective is to build a general Agent that can:

```
observe
  ->
understand
  ->
hypothesize
  ->
decide
  ->
act
  ->
observe result
  ->
evaluate
  ->
revise
  ->
act again
  ->
verify
  ->
finish
```

The implementation may realize this loop differently.

The behavior must remain general.

## Documentation State Boundary

The formal specification documents describe the target system and its required behavior.

They should not be used as implementation progress trackers.

Do not add "current status", "implemented", "not implemented", "partial", "Stage X completed", or similar progress information to:

- `docs/01_REQUIREMENTS.md`
- `docs/02_CONSTRUCTION.md`
- `docs/03_INTERFACES.md`
- `docs/04_ACCEPTANCE_TESTS.md`
- `docs/05_TEST_PLAN.md`

Implementation progress belongs in:

- `IMPLEMENTATION_STATUS.md`

Historical implementation details belong in:

- `history/`

`README.md` may describe the project's target architecture and purpose, but should not become an implementation progress tracker.

When implementation reality differs from the specification, do not modify the specification merely to make it match the current implementation.

Record the implementation gap in `IMPLEMENTATION_STATUS.md`.

If the specification itself is found to be ambiguous, contradictory, incomplete, or architecturally incorrect, handle that as a specification change according to Section 18.