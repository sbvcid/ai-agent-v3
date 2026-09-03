# AI Agent v3 — Test Plan

## 1. Purpose

This document defines how the behavioral requirements in:

```
docs/04_ACCEPTANCE_TESTS.md
```

should be verified.

This document is a test strategy, not an implementation specification.

`04_ACCEPTANCE_TESTS.md` defines what behavior is required.

This document defines the preferred level of evidence for demonstrating that behavior.

The implementation may use any reasonable Rust architecture, crate, data structure, persistence mechanism, or testing framework unless another specification explicitly constrains it.

---

## 2. Verification Model

Use the following evidence hierarchy:

```
Unit Test
    ->
Integration Test
    ->
Runtime / System Test
    ->
End-to-End Scenario Test
    ->
Manual Acceptance Test when automation is impractical
```

Use the lowest level that can reliably prove the requirement.

Do not force an end-to-end test when a deterministic unit or integration test provides stronger evidence.

Conversely, do not claim that complete Agent behavior is proven solely because an internal helper function passes a unit test.

---

## 3. Test Categories

### 3.1 Unit Tests

Use unit tests for deterministic internal behavior such as:

* state transitions
* decision parsing
* schema validation
* normalization
* action validation
* status conversion
* loop similarity calculations
* context selection
* checkpoint serialization
* persistence serialization
* error classification

Unit tests should be fast and deterministic.

---

### 3.2 Integration Tests

Use integration tests when multiple Agent components must cooperate.

Examples:

* Agent Core + Observation Store
* Agent Core + Context Compiler
* LLM parser + decision validator
* Agent Core + Runtime interface
* Job manager + persistence
* checkpoint + recovery
* Engineering Runtime + repository abstraction

Integration tests should prefer fake or controlled Runtime implementations when the behavior under test does not require a real Windows side effect.

---

### 3.3 Runtime / System Tests

Use real Windows behavior when the requirement concerns the actual operating environment.

Examples:

* filesystem operations
* process execution
* shell behavior
* interactive shell / ConPTY
* long-running processes
* process cancellation
* process exit
* GUI availability
* browser interaction
* network behavior
* permission boundaries

These tests should be isolated from destructive user data whenever possible.

---

### 3.4 End-to-End Scenario Tests

Use end-to-end tests for behaviors that emerge only from the complete Agent loop.

Examples:

* observe -> act -> observe -> revise -> act again
* failure followed by strategy change
* unknown followed by investigation
* long-running task monitoring
* permission boundary handling
* goal verification after a successful action
* recovery after interruption
* multi-step engineering tasks

E2E tests should verify externally observable behavior rather than internal implementation details.

---

### 3.5 Manual Acceptance Tests

Manual testing is acceptable where deterministic automation is impractical, particularly for:

* interactive GUI
* UAC Secure Desktop behavior
* real browser sessions
* physical/user interaction
* environment-specific permission boundaries

Manual tests should record:

* scenario
* preconditions
* action
* observed evidence
* expected result
* actual result
* pass/fail
* limitations

---

## 4. Test Doubles

The project should support controlled Runtime implementations for testing Agent Core behavior.

A fake Runtime should be able to simulate, where relevant:

* success
* failure
* timeout
* cancellation
* blocked operation
* unknown result
* permission denial
* changing environment state
* unchanged environment state
* long-running Job
* completed Job
* missing capability

This allows Agent Core behavior to be tested without making real destructive system changes.

Do not make the production Runtime depend on test-specific behavior.

---

## 5. Determinism

Tests for Agent Core should be deterministic wherever possible.

When LLM behavior is not the subject of the test, do not require a live LLM.

Use:

* scripted model responses
* mock providers
* fixtures
* deterministic decision generators

When testing the LLM boundary itself, include cases such as:

* valid structured output
* malformed output
* missing fields
* invalid enum values
* invalid action arguments
* unsupported capabilities
* unexpected fields where relevant

The objective is to verify the Agent's handling of model output, not to assume a particular model is always correct.

---

## 6. Acceptance Requirement Mapping

Every requirement in `04_ACCEPTANCE_TESTS.md` should have an evidence mapping.

Recommended record:

```
Requirement ID:
Behavior:
Test Level:
Test Location:
Preconditions:
Stimulus:
Expected Observable Result:
Evidence:
Status:
```

Example:

```
Requirement ID: TEST-012

Behavior:
Agent must not blindly repeat a failed action indefinitely.

Test Level:
Integration / E2E

Test Location:
tests/loop_detection.rs

Preconditions:
Runtime repeatedly fails the same action.

Stimulus:
Agent attempts the task.

Expected Observable Result:
Agent eventually changes strategy, waits, requests user input,
or reaches an appropriate terminal outcome.

Evidence:
Action history + observations + final state

Status:
Pending
```

Do not require every requirement to have exactly one test.

A requirement may be supported by multiple tests.

Do not couple unrelated tests merely because they appear in the same section of `04_ACCEPTANCE_TESTS.md`.

---

## 7. Core Behavioral Test Areas

The following areas should be represented in automated or scenario testing as appropriate.

### 7.1 Observation

Verify that the Agent can:

* obtain environmental evidence
* preserve observations
* identify meaningful changes where required
* handle incomplete observations
* handle contradictory observations
* retain decision-critical evidence

---

### 7.2 Unknown and Hypothesis

Verify that the Agent:

* can represent uncertainty
* does not treat unsupported assumptions as facts
* can form a hypothesis
* can investigate a hypothesis
* can revise a hypothesis after contradictory evidence

A test should distinguish:

```
evidence
from
inference
from
hypothesis
```

Do not require the Agent to express private chain-of-thought.

The test should verify observable state or decision behavior instead.

---

### 7.3 Decision and Action

Verify the decision model:

```
AgentDecision
  Observe
  Act
  Wait
  Finish
```

Verify the Runtime action model:

```
Action
  Observe
  Execute
  Interact
  Wait
```

Verify that malformed or invalid decisions do not directly execute.

Verify that unsupported Runtime capabilities are rejected or handled appropriately.

---

### 7.4 Failure and Adaptation

Verify that Runtime failure becomes usable evidence.

Test that the Agent can, when appropriate:

* retry
* inspect the cause
* change parameters
* choose another capability
* change strategy
* wait
* request user intervention
* terminate when the task is genuinely blocked or impossible

Test specifically against blind repetition.

A test should not merely verify:

```
retry_count < N
```

It should preferably verify that the Agent reacts to evidence and changes behavior appropriately.

---

### 7.5 Loop Detection

Test combinations of:

* identical actions
* highly similar actions
* repeated failures
* successful actions with no meaningful progress
* meaningful environment changes
* partial progress
* conflicting observations

Loop detection should influence Agent Core decisions.

Loop detection should not automatically determine the final task status.

---

### 7.6 Goal Verification

Verify the distinction between:

```
Action Success
Subtask Success
Goal Success
```

Example:

```
command succeeds
but requested output file is missing
=>
Goal is not verified
```

Also test:

```
action reports failure
but environment already satisfies the goal
=>
Agent inspects actual state before concluding failure
```

Final behavior should follow the requirements in `04_ACCEPTANCE_TESTS.md`.

---

### 7.7 Terminal Outcomes

Verify that the terminal task outcomes are:

```
Done
Blocked
Impossible
NeedUser
```

Verify that:

```
Continue
```

is not used as a terminal task outcome.

Verify that a Runtime failure does not automatically become:

```
Blocked
Impossible
NeedUser
```

without Agent Core evaluation.

---

### 7.8 Waiting and Long-Running Jobs

Test:

* starting a Job
* receiving a JobId
* observing a running state
* waiting
* inspecting progress/output
* detecting completion
* handling timeout
* cancelling when appropriate
* recovering Job state where supported
* avoiding unnecessary duplicate Jobs

Timeout must be treated as evidence requiring a decision.

---

### 7.9 Permission and Security Boundaries

Test:

* normal permission success
* AccessDenied
* unavailable elevation
* unavailable interactive desktop
* user-required action
* security boundary that cannot be bypassed

Verify that the Agent:

* does not assume elevation
* does not retry indefinitely
* does not hide the permission failure
* chooses an appropriate next state

---

### 7.10 Persistence and Crash Recovery

Test:

* saving Agent state
* saving action history
* saving observations
* persisting Job state
* creating checkpoints
* simulating process interruption
* restarting
* recovering state
* resuming reasoning
* retaining critical evidence
* rollback where the scenario requires it

The test must verify recovery behavior, not merely that a database row or file exists.

---

### 7.11 LLM Boundary

Test the complete boundary:

```
model output
  ->
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

Test invalid model outputs and verify that they cannot become arbitrary Runtime operations.

Test bounded correction/retry for malformed structured output.

Verify that a model cannot directly bypass Agent Core validation.

---

### 7.12 Context Compilation

Verify that Context Compiler:

* selects relevant observations
* preserves decision-critical evidence
* handles large observation history
* compresses or summarizes when necessary
* does not silently discard information required for correct decisions

Do not make acceptance depend on an arbitrary hard-coded percentage of context tokens.

The implementation may choose its own context budgeting strategy.

---

## 8. Runtime Test Areas

Runtime acceptance testing should cover the actual capabilities exposed by the interfaces.

Where implemented, cover:

* filesystem
* process
* shell
* interactive shell / ConPTY
* GUI
* browser
* network
* long-running Jobs
* cancellation
* permission errors
* capability discovery
* safety / policy boundaries

Runtime tests should verify observable behavior and error semantics.

Do not test implementation internals such as private function names unless the internal behavior itself is the requirement.

---

## 9. Engineering Runtime Test Areas

Engineering Runtime tests should cover general development capabilities such as:

* repository inspection
* file discovery
* code search
* symbol/reference discovery
* dependency inspection
* editing
* diff inspection
* build
* test
* static analysis
* Git status
* Git diff
* Git operations where implemented
* checkpoint / rollback
* debugging evidence

Do not encode a mandatory engineering workflow.

For example, this is a valid test goal:

```
Agent can diagnose a failing build and repair the underlying problem.
```

This is not a required workflow:

```
search -> edit -> cargo check -> cargo test -> git diff
```

The Agent must be allowed to choose another sequence when evidence requires it.

---

## 10. Test Data and Fixtures

Use small, controlled fixtures for deterministic tests.

Recommended fixtures include:

* temporary repository
* repository with an intentional compile error
* repository with a failing test
* repository with dependency mismatch
* large text file
* nested directory tree
* safely reproducible read-only resource
* mock long-running process
* mock failing process
* mock permission-denied operation
* scripted LLM responses
* saved observation history
* saved Agent checkpoint

Do not use the user's real project or important personal files as the default test fixture.

---

## 11. Destructive Operations

Tests that can:

* delete files
* overwrite data
* terminate processes
* change system configuration
* modify Git history
* access external services

must use isolated test resources whenever possible.

Prefer:

* temporary directories
* disposable repositories
* mock processes
* explicit test fixtures

A test must not assume that the developer's machine contains no valuable data.

---

## 12. Regression Testing

When fixing a bug:

1. Reproduce the bug with a test.
2. Confirm that the test represents the intended behavior.
3. Implement the fix.
4. Confirm that the test passes.
5. Run the relevant regression suite.
6. Run the broader suite when the change crosses module boundaries.

A bug fix without a regression test should be justified when a regression test is genuinely impractical.

---

## 13. Test Naming

Test names should describe behavior.

Prefer:

```
failed_action_causes_strategy_revision
```

over:

```
test_agent_loop_3
```

Prefer:

```
goal_not_verified_when_output_file_missing
```

over:

```
test_verify
```

Acceptance IDs may be included in test names, comments, or metadata when useful.

Do not make test names unreadable merely to include an ID.

---

## 14. Evidence Requirements

For important Agent behavior, tests should make relevant evidence inspectable.

Useful evidence includes:

* AgentDecision
* Runtime Action
* ActionResult
* Observation
* environment delta
* JobId and Job state
* hypothesis state
* verification state
* final task status

Do not store chain-of-thought.

Tests should assert observable decisions and state transitions rather than private reasoning text.

---

## 15. Flaky Tests

Tests involving:

* real processes
* GUI
* browser
* network
* timing
* concurrency

may be inherently less deterministic.

When unavoidable:

* minimize timing assumptions
* use explicit synchronization
* isolate resources
* record diagnostic evidence
* distinguish infrastructure failure from product failure
* avoid arbitrary sleeps where possible

Do not weaken a behavioral requirement merely because the first implementation is difficult to test.

---

## 16. Test Gates

### 16.1 Minimum Core Release Gate

Before considering Agent Core usable:

* project builds
* deterministic unit tests pass
* core integration tests pass
* decision parsing/validation is covered
* observation/state handling is covered
* failure adaptation is covered
* loop avoidance is covered
* goal verification is covered
* terminal statuses are covered

---

### 16.2 Runtime Capability Gate

Before considering Runtime usable:

* required Runtime capability tests pass
* permission/error semantics are covered
* long-running Jobs are covered where implemented
* cancellation is covered where implemented
* interactive behavior is covered where implemented

---

### 16.3 Engineering Runtime Gate

Before considering Engineering Runtime usable:

* repository inspection works
* code search works
* editing works
* diff works
* build/test integration works
* Git integration works where implemented
* debugging evidence is usable where implemented
* at least one end-to-end engineering scenario succeeds

A feature that is not implemented must not be marked as passing merely because its interface exists.

---

## 17. Test Execution Policy

Do not require all acceptance tests to run for every code change.

Instead:

1. Determine which requirements are affected.
2. Run focused tests.
3. Run integration tests for affected module boundaries.
4. Run the full suite for substantial architectural changes.
5. Run formatting and compile checks.
6. Run E2E or manual tests when the changed behavior requires them.

This keeps development feedback reasonably fast without reducing final acceptance standards.

---

## 18. Completion Criteria

A task is not considered fully verified merely because:

```
cargo build
cargo test
```

succeed.

For a behavioral Agent change, completion requires:

* affected acceptance requirements identified
* relevant tests added or updated
* relevant tests executed
* results inspected
* implementation evidence available
* remaining uncertainty documented where applicable

Do not claim complete acceptance when only compilation has been verified.

---

## 19. Acceptance Matrix

Maintain a living mapping between:

```
04_ACCEPTANCE_TESTS.md
```

and actual tests.

Recommended format:

| Acceptance ID | Requirement | Test Level               | Test Location | Status                | Evidence |
| ------------- | ----------- | ------------------------ | ------------- | --------------------- | -------- |
| TEST-001      | ...         | Unit / Integration / E2E | ...           | Pending / Pass / Fail | ...      |
| TEST-002      | ...         | Unit / Integration / E2E | ...           | Pending / Pass / Fail | ...      |

The matrix may be maintained:

* in this document
* in a generated report
* in test metadata
* in the test suite

The authoritative behavioral wording remains in `04_ACCEPTANCE_TESTS.md`.

---

## 20. Relationship to Codex Verification

Codex may independently run:

* cargo fmt
* cargo check
* cargo test
* linting
* static analysis
* integration tests
* end-to-end tests
* repository-specific checks

Those checks verify implementation health.

This document additionally defines how to verify that the resulting system actually behaves according to `04_ACCEPTANCE_TESTS.md`.

Do not replace behavioral acceptance with generic build/test success.

---

## 21. What This Document Must Not Do

Do not use this document to:

* prescribe a fixed Agent workflow
* prescribe one specific crate
* prescribe one storage technology
* prescribe one LLM
* prescribe one prompt implementation
* require hidden chain-of-thought
* prescribe one internal architecture
* force arbitrary context-token percentages
* create tests that only pass through hard-coded special cases
* require an implementation detail when an observable behavior is sufficient

The purpose of testing is to prove behavior, not to freeze implementation.

---

## 22. Final Principle

The test suite exists to answer:

```
"Does the Agent actually behave as specified?"
```

It does not exist merely to answer:

```
"Does the Rust code compile?"
```

The strongest evidence is therefore behavioral:

```
Environment
    ->
Observation
    ->
Agent Decision
    ->
Runtime Action
    ->
Result / Observation
    ->
Adaptation
    ->
Goal Verification
```

Tests should prove that this system can operate through uncertainty, failure, changing environments, long-running operations, permission boundaries, and multi-step tasks without relying on a hard-coded workflow.

### Deterministic Agent Test Environment

所有 Agent Core 核心行為必須能在：

Mock LLM
+
Fake Runtime
+
固定 Observation

下重現。

測試不得依賴：

- 真實 Windows 狀態
- 真實檔案系統
- 真實網路
- 真實 Ollama 回應
- 非確定性的外部服務

### Mandatory Closed-Loop Scenarios

至少測試：

1. 成功完成
2. Action Failure → Strategy Change → Success
3. Unknown → Investigation → Known
4. Repeated Failure → Loop Detection
5. Timeout → Wait / Inspect / Alternative
6. AccessDenied
7. NeedUser
8. Goal Verification Failure
9. Crash → Resume
10. Invalid LLM Decision