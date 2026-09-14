# AI Agent v3 — Coding Agent Instructions

## 1. Purpose

This repository implements AI Agent v3, a Rust-based local AI Agent.

The project is intended to provide a general Agent Runtime and Agent Core rather than a fixed workflow designed around the limitations of today's LLMs.

The core architectural principles are:

```text
Agent Core = reasoning, understanding, hypothesis, planning, decision, adaptation, verification

Runtime    = computer operation

Observation = evidence connecting the environment to the Agent

Persistence = state required for continuity and recovery
```

The Agent must be able to observe the environment, reason about the current situation, act on the environment, inspect the result, revise its approach when necessary, and verify whether the original user goal was actually achieved.

Do not turn the project into a collection of hard-coded workflows.

The objective is to build a general Agent architecture whose Runtime capabilities remain useful as LLM capabilities improve.

---

## 2. Mandatory Project Orientation

Before making substantial changes, establish the current project state using a two-phase orientation process.

### Phase 1 — Repository Orientation

First inspect only the project-level orientation documents:

```text
AGENTS.md
DEVELOPMENT_WORKFLOW.md
IMPLEMENTATION_STATUS.md
README.md
docs/README.md
```

Also inspect:

```text
git status
git log --oneline --decorate -5
```

Do NOT automatically read every file under `docs/`.

`docs/README.md` is the documentation navigation index. After Phase 1, use it to determine which stable design documents are actually relevant to the current task.

### Phase 2 — Task-Specific Context

Determine the task before loading additional project context.

Use:

```text
docs/README.md
```

to select the minimum necessary set of:

```text
requirements
architecture / construction principles
interfaces
acceptance criteria
test plan
implementation gap analysis
specific Agent Core design
roadmap
source code
tests
```

Do not load all stable design documents merely because they exist.

The minimum-context principle is mandatory:

```text
First understand how to find the required knowledge.
Then load only the knowledge required for the current task.
```

For implementation work, inspect the relevant source and tests after identifying the applicable stable design boundary.

Run relevant verification commands before modifying substantial code when practical.

Do not assume that the current source tree perfectly matches the target architecture described by the specifications.

Always distinguish:

```text
Target Design
    =
what the architecture is intended to become

Current Implementation
    =
what the repository actually implements now
```

These are related but are not interchangeable.

---

## 3. Source of Truth

The project documents have different responsibilities.

Do not treat them as interchangeable.

### 3.1 Stable Design Documents

The following documents describe relatively stable project requirements, architecture, interfaces, acceptance criteria, testing methodology, design analysis, and long-term direction:

```text
docs/01_REQUIREMENTS.md
docs/02_CONSTRUCTION.md
docs/03_INTERFACES.md
docs/04_ACCEPTANCE_TESTS.md
docs/05_TEST_PLAN.md
docs/06_IMPLEMENTATION_GAP_ANALYSIS.md
docs/07_AGENT_CORE_OBSERVATION_INCREMENT_DESIGN.md
docs/08_AGENT_CORE_ROADMAP.md
```

Their purposes are different:

```text
01 Requirements
    What the system is required to achieve.

02 Construction
    Architectural and implementation principles that construction must preserve.

03 Interfaces
    Stable semantic boundaries between components.

04 Acceptance Tests
    Observable behaviors that determine whether requirements are satisfied.

05 Test Plan
    Testing and verification methodology.

06 Implementation Gap Analysis
    Formal assessment of the relationship between the target architecture and current implementation.

07 Agent Core Observation Increment Design
    Stable design baseline for the Observation Store and Context Compiler increment.

08 Agent Core Roadmap
    Long-term Agent Core capability/dependency map and architectural direction.
```

These documents are not implementation diaries.

Do not routinely rewrite them after every coding increment.

### 3.2 Current Implementation State

```text
IMPLEMENTATION_STATUS.md
```

is the primary document for current implementation reality.

It may contain:

```text
current repository state
implemented capabilities
known gaps
current checkpoint
verification evidence
test results
active implementation status
```

When an implementation increment is completed, update `IMPLEMENTATION_STATUS.md` when necessary so that it reflects the actual repository state.

Do not use stable design documents as progress trackers.

### 3.3 Historical Records

```text
history/
```

contains historical implementation records.

Historical records are archival.

Do not treat them as current instructions unless historical context is explicitly required.

### 3.4 README

```text
README.md
```

is the human-facing introduction to the project.

It may describe:

```text
project purpose
target architecture
major principles
high-level capabilities
```

It should not become an implementation progress tracker.

### 3.5 Active Conversation

The active development conversation may contain:

```text
one-time implementation prompts
temporary debugging information
implementation-specific instructions
unresolved design discussions
temporary hypotheses
test output
one-off migration instructions
```

Do not create permanent documentation merely to store a one-time coding prompt.

---

## 4. Documentation Governance

This rule is mandatory.

The project has a strict distinction between:

```text
Stable Design
Current State
History
Temporary Implementation Context
```

Do not collapse these categories.

### 4.1 Stable Design Does Not Mean Immutable

The `docs/` files are stable architectural documents, not immutable files.

They may be changed when:

```text
requirements materially change
architecture materially changes
stable semantic interfaces change
acceptance criteria change
long-term dependency relationships change
the existing design is proven to be incorrect or incomplete
```

They should NOT be changed merely because:

```text
an implementation detail changed
a module was refactored
a test was reorganized
a coding agent prefers another implementation
a temporary workaround was used
a local data structure changed
a file was moved
an implementation increment was completed
the current implementation happens to differ from the target design
```

### 4.2 Implementation Must Not Redefine Design Automatically

Never use the following reasoning:

```text
"The code currently works this way,
therefore the architecture document should be changed to describe the code."
```

Instead determine:

```text
Is the implementation incomplete?

or

Is the stable design actually wrong?
```

If the implementation is incomplete, preserve the design and record the gap in:

```text
IMPLEMENTATION_STATUS.md
```

If the stable design is genuinely wrong or incomplete, stop the affected implementation work and propose a specification/design change.

Do not silently redefine the architecture.

### 4.3 Documentation Synchronization Rule

After completing an implementation increment:

1. Verify that the implementation matches the applicable stable design.
2. Update `IMPLEMENTATION_STATUS.md` if current-state information changed.
3. Add historical information to `history/` when appropriate.
4. Do NOT automatically rewrite `docs/01~08`.
5. Only modify a stable design document when the design itself has materially changed or was proven incorrect.

A coding agent must not perform a broad documentation rewrite merely because implementation has finished.

---

## 5. Stable Design Conflict Procedure

If implementation reveals a genuine conflict with a stable design document:

```text
implementation
    ↓
design conflict discovered
    ↓
STOP affected implementation
    ↓
describe the conflict
    ↓
explain alternatives and consequences
    ↓
obtain approval
    ↓
update stable design if approved
    ↓
re-establish implementation boundary
    ↓
continue implementation
```

Do not:

```text
silently modify the design
silently ignore the design
silently implement a third interpretation
rewrite the architecture to make the current code appear correct
```

When a design change is necessary, identify:

```text
what conflicts
why it conflicts
what the current design says
what the proposed design says
why the change is necessary
which documents are affected
what implementation consequences follow
```

---

## 6. Implementation Prompt Policy

Implementation prompts are temporary execution instructions.

Do not create a permanent:

```text
09_IMPLEMENTATION_TASK.md
```

or equivalent document merely to store the next coding task.

A specific implementation prompt should normally be generated in the active development conversation after:

```text
design boundary is understood
dependencies are understood
scope is defined
non-goals are defined
acceptance criteria are defined
the implementation increment is explicitly approved
```

The coding agent must implement the approved boundary and must not silently expand it.

Temporary implementation instructions belong to the active conversation unless they become a genuine stable architectural rule.

---

## 7. Architectural Principles

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

## 8. Decision and Action Semantics

The logical Agent decision model is:

```text
AgentDecision

    Observe
    Act
    Wait
    Finish
```

`Act` causes a Runtime action to be performed.

The Runtime action model is:

```text
Action

    Observe
    Execute
    Interact
    Wait
```

Verification is Agent Core behavior.

Verification is not a special Runtime Action.

Terminal task outcomes are:

```text
Done
Blocked
Impossible
NeedUser
```

Do not introduce `Continue` as a terminal task outcome.

`Verify` may be used as the name of an internal function or conceptual Agent behavior when appropriate, but do not add it to the Runtime Action enum merely to represent reasoning.

---

## 9. Runtime Design

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

```text
search -> edit -> build -> test
```

The Agent must be able to choose a different sequence when evidence requires it.

Do not create specialized tools merely because one current test happens to use them.

Prefer general capabilities that can support multiple tasks.

---

## 10. Environment and Permission Boundaries

Do not assume elevated privileges.

`AccessDenied`, UAC restrictions, unavailable GUI sessions, missing permissions, and similar conditions are environmental observations.

The Agent may attempt reasonable alternatives when appropriate.

If actual user action, authorization, credentials, physical interaction, or an unavailable interactive environment is required, the task may end as:

```text
NeedUser
```

Do not retry an impossible permission boundary indefinitely.

Do not implement unrestricted privilege escalation.

Do not hide permission failures from Agent Core.

---

## 11. Long-Running Jobs

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

```text
Failed
Blocked
Impossible
NeedUser
```

The Agent must evaluate the evidence and decide what to do next.

A Job model should remain a semantic concept rather than being permanently represented as an ad-hoc collection of strings or process IDs if richer semantics are required.

---

## 12. Observation and State

Observations are evidence from the environment.

The Observation Store is the authoritative in-process observation history when the architecture requires persistent observation semantics.

`AgentState.recent_observations`, when present, is a bounded operational or compatibility view and must not silently become a second independent observation history.

When recording an action result that produces an observation:

```text
Action Result
    ↓
Observation Store
    ↓
bounded AgentState view
```

Preserve decision-critical evidence.

The Context Compiler may select, summarize, compress, or prioritize observations for LLM context, but the compiled context must not silently become the authoritative history.

Do not confuse:

```text
Observation History
```

with:

```text
LLM Context
```

They have different purposes.

Do not hard-code a fragile provider-specific token ratio as an architectural requirement.

Prefer deterministic semantic bounds such as:

```text
maximum observations
maximum actions
maximum evidence items
```

when a bounded context is required.

`Fact`, `Observed`, `Inferred`, `Hypothesis`, and `Unknown` are epistemic concepts used by Agent Core.

Do not force raw Runtime observations into these categories unless the interface specification explicitly requires it.

---

## 13. Context Compiler

The Context Compiler is a semantic component.

It should transform Agent state and relevant evidence into an appropriate context representation for the decision model.

It should not merely be an opaque string-building helper.

The architecture should preserve the ability to reason about:

```text
goal
current state
recent actions
observations
evidence
hypotheses
unknowns
verification state
remaining work
```

Do not prematurely expose provider-specific token accounting as a Core semantic interface.

Do not make the Core dependent on one LLM provider's context format.

---

## 14. LLM Boundary

Treat LLM output as untrusted structured input.

The conceptual processing pipeline is:

```text
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

Do not persist private chain-of-thought.

Persist structured decision-relevant information instead.

---

## 15. Loop Detection and Adaptation

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

A generic safety bound such as maximum steps may exist as a resource protection mechanism, but it must not be mistaken for genuine loop detection.

---

## 16. Verification

Never equate:

```text
Action Success == Goal Success
```

Verification should operate at useful levels such as:

```text
Action Success

Subtask Success

Goal Success
```

Final verification must relate to the original user goal.

For example:

```text
command succeeded

but requested output is missing

=>

Goal is not verified
```

Likewise:

```text
command reported failure

but environment already satisfies the requested goal

=>

Agent should inspect actual state before concluding that the goal failed
```

When verification is uncertain, preserve the uncertainty instead of claiming success.

---

## 17. Persistence and Recovery

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

Checkpoint restoration must preserve the intended semantic state rather than merely restoring arbitrary implementation data.

Do not create a persistence abstraction merely to satisfy a test if it cannot support real recovery semantics.

Do not introduce a stronger persistence technology merely because it appears architecturally impressive.

Use the simplest persistence mechanism that satisfies the actual requirements.

---

## 18. Security and Safety

Keep these concepts separate:

```text
Capability = what Runtime technically exposes

Policy     = what Agent is currently allowed to request

OS security = what Windows actually permits
```

A capability declaration is not proof that an operation will succeed.

Do not bypass Windows security boundaries.

Do not silently elevate privileges.

Do not hide security-related failures from Agent Core.

---

## 19. Implementation Strategy

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

Unless the specifications explicitly constrain implementation details, the coding agent may determine:

* crate selection
* internal data structures
* serialization format
* persistence implementation
* module organization
* async/runtime strategy
* process management implementation
* exact testing framework

The specifications define required behavior and stable contracts, not every implementation detail.

Do not change stable semantic interfaces merely to simplify a local implementation.

---

## 20. Incremental Implementation Rules

Implementation must proceed through explicit coherent increments.

For each increment:

```text
Design boundary
    ↓
Dependencies
    ↓
Scope
    ↓
Non-goals
    ↓
Acceptance criteria
    ↓
Implementation
    ↓
Verification
    ↓
Checkpoint
```

Do not silently expand an approved increment.

If implementation exposes a dependency that was not included in the approved boundary:

1. Identify the dependency.
2. Determine whether it is required or merely convenient.
3. Do not automatically expand scope.
4. Report the issue.
5. Only expand the implementation boundary after explicit approval.

Do not implement future roadmap items merely because they appear related.

---

## 21. Change Procedure

For every non-trivial change:

1. Inspect the current implementation.
2. Read the relevant specification sections.
3. Inspect `IMPLEMENTATION_STATUS.md`.
4. Identify affected interfaces.
5. Identify affected acceptance requirements.
6. Check for contradictions or missing assumptions.
7. Identify whether the requested change is within the approved implementation boundary.
8. Design the smallest coherent change.
9. Implement the change.
10. Add or update tests.
11. Run appropriate verification commands.
12. Inspect the resulting diff.
13. Re-check affected acceptance criteria.
14. Update `IMPLEMENTATION_STATUS.md` if current state changed.
15. Report remaining uncertainty or limitations honestly.

Do not modify unrelated code merely to make the current task appear complete.

Do not rewrite large parts of the architecture without first establishing why the existing design cannot satisfy the requirements.

Do not modify stable design documents merely to reflect implementation details.

---

## 22. Testing Requirements

Use:

```text
docs/04_ACCEPTANCE_TESTS.md
```

as the behavioral acceptance specification.

Use:

```text
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

```text
cargo test
```

is useful evidence, but it is not by itself proof that all Agent behavioral requirements are satisfied.

Tests must provide evidence for the behavior they claim to validate.

---

## 23. Required Verification Before Completion

When applicable, run:

```text
cargo fmt --check
cargo check
cargo test
```

Run additional repository-specific checks when appropriate, including:

```text
linting
static analysis
integration tests
end-to-end tests
runtime-specific verification
```

If a required check cannot be run, state why.

For Agent behavior changes, add or update tests that provide evidence for the affected acceptance requirements.

For significant changes, inspect:

```text
git diff
git status
```

and ensure no accidental files, generated artifacts, secrets, or unrelated modifications were introduced.

Do not claim completion merely because compilation succeeds.

---

## 24. Checkpoint and Recovery

Every completed implementation increment should leave the repository in a recoverable state.

A completed increment should have:

```text
working tree understood
tests run
verification results recorded
current implementation state updated
commit/checkpoint identifiable
```

If implementation is interrupted:

* do not pretend the increment is complete
* record the current state in `IMPLEMENTATION_STATUS.md` when appropriate
* preserve useful debugging information
* do not rewrite stable design documents merely to describe an interrupted implementation
* leave enough information for another coding agent to resume safely

A new coding agent must be able to determine:

```text
what was intended
what was actually implemented
what passed
what failed
what remains
```

without relying on undocumented assumptions.

---

## 25. Handling Specification Problems

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
* what implementation consequences follow

Do not silently rewrite requirements merely to simplify implementation.

Do not treat the current implementation as proof that the specification should be changed.

---

## 26. Documentation State Boundary

The formal specification documents describe the target system and required behavior.

They should not be used as implementation progress trackers.

Do not add:

```text
current status
implemented
not implemented
partial
Stage X completed
latest test result
current commit
temporary workaround
```

to stable design documents merely for progress tracking.

Implementation progress belongs in:

```text
IMPLEMENTATION_STATUS.md
```

Historical implementation details belong in:

```text
history/
```

Temporary implementation instructions belong in:

```text
active development conversation
```

`README.md` may describe the project's target architecture and purpose, but should not become an implementation progress tracker.

When implementation reality differs from the specification, do not modify the specification merely to make it match the current implementation.

Record the implementation gap in:

```text
IMPLEMENTATION_STATUS.md
```

If the specification itself is found to be ambiguous, contradictory, incomplete, or architecturally incorrect, handle that as a specification change according to Section 25.

---

## 27. Roadmap Discipline

`docs/08_AGENT_CORE_ROADMAP.md` is a long-term dependency and capability map.

It is not a mandatory linear implementation schedule.

Do not interpret roadmap items as automatically approved coding tasks.

A roadmap item becomes an implementation increment only after:

```text
design is sufficiently defined
dependencies are understood
scope is bounded
non-goals are explicit
acceptance criteria are defined
implementation boundary is approved
```

Do not assign arbitrary Stage numbers merely because a roadmap item exists.

Do not implement future roadmap capabilities opportunistically during an unrelated increment.

Runtime capability branches and Agent Core semantic evolution may progress independently where the architecture permits it.

---

## 28. Current Agent Core Direction

The current stable post-cycle Agent Core design begins with:

```text
Observation Store
        +
Context Compiler
```

The Observation Store is the authoritative in-process observation history.

The Context Compiler is responsible for producing an appropriate semantic context representation for Agent decision making.

This increment is followed conceptually by broader Agent Core capabilities such as:

```text
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
```

These are architectural directions, not automatic implementation instructions.

The detailed current design is defined in:

```text
docs/07_AGENT_CORE_OBSERVATION_INCREMENT_DESIGN.md
```

The broader dependency map is defined in:

```text
docs/08_AGENT_CORE_ROADMAP.md
```

Do not independently redefine these boundaries during implementation.

---

## 29. General Agent Behavior

The objective is not to make the current test cases pass through special handling.

The objective is to build a general Agent that can:

```text
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

Do not encode a particular task sequence into Agent Core merely because it makes a demonstration easier.

---

## 30. Completion Report

When reporting completed development work, summarize:

1. What changed.
2. Which implementation boundary was completed.
3. Which specifications or interfaces were affected.
4. Which tests and verification checks were run.
5. The relevant results.
6. Whether the implementation matches the applicable stable design.
7. Whether `IMPLEMENTATION_STATUS.md` was updated.
8. Any remaining limitations or uncertainty.
9. Any discovered design conflict that requires separate approval.

Do not claim a requirement is satisfied without corresponding evidence.

Do not claim an end-to-end behavior is verified merely because the code compiles.

Do not claim a design document was updated unless the design itself actually changed.

---

## 31. Final Rule

When in doubt, preserve the distinction:

```text
Stable Design
    =
what the system is intended to be

Current Implementation
    =
what the repository currently implements

History
    =
what happened previously

Temporary Implementation Context
    =
what is being worked on now
```

The coding agent's job is not to make these four categories look identical.

The coding agent's job is to make the implementation converge toward the approved stable design while preserving a truthful record of current state and history.

Never silently change the target architecture to match an implementation shortcut.
Never silently expand an approved implementation boundary.
Never treat a temporary implementation detail as a permanent architectural rule.
Never treat successful compilation as proof of behavioral correctness.
Never hide uncertainty or design conflicts.
