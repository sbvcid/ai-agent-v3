# AI Agent v3 — Implementation Status

本文件是 **Current Implementation State** 的唯一集中入口。

它描述 Repository 現在實際做到什麼，不重新定義 target architecture，也不產生新的 implementation Stage。

## 1. Current Status

```text
Implementation Cycle 1: COMPLETED through Stage 9.4
Post-Cycle Increment: Observation Store + Context Compiler — COMPLETED
B1 — Knowledge Semantic Foundation: IMPLEMENTED, LOCAL VERIFICATION PENDING
Current overall state: deterministic closed-loop foundation + observation/context foundation + first Evidence/Knowledge semantic foundation
```

**Stage 10 尚未定義。** B1 是經批准的 Agent Core semantic implementation increment，不自動建立 Stage 10。

目前 repository 的真實狀態仍必須以 source code、tests 與實際驗證結果確認。

## 2. Repository Checkpoint

```text
Repository: sbvcid/ai-agent-v3
Branch: master
Latest documentation checkpoint: b8ada04
Latest B1 implementation commit: 6f86cdce
```

以上 commit 是狀態導航資訊，不是永久真相。新的 AI session 必須先執行：

```text
git status
git log --oneline --decorate -5
```

## 3. Completed Implementation

### Stage 1 — Foundation / Specification Review

**COMPLETED**

建立 Rust project foundation、規格對齊、architecture boundary 與 implementation status tracking。

### Stage 2 — Canonical Core Data Model

**COMPLETED**

建立主要 domain concepts：

```text
Goal
Action / ActionType
ActionResult
Observation / ObservationKind
ExecutionState
FinalTaskStatus
AgentDecision
KnowledgeState
VerificationState
AgentState
```

Canonical boundary：

```text
Provider
  ↓
ToolCall
  ↓
Action
  ↓
Runtime
  ↓
ActionResult
```

### Stage 3 — Serialization / Checkpoint

**COMPLETED**

Serde serialization、validation、StateCheckpoint、CheckpointStore、JSON persistence、crash/resume restoration。

Phase 0 使用 JSON；未因目前需求引入 SQLite 或其他外部資料庫。

### Stage 4 — Deterministic Test Doubles

**COMPLETED**

`MockLlm` 與 `FakeRuntime` 支援 deterministic Core testing。

### Stage 5 — Deterministic Closed Loop / Event Trace

**COMPLETED**

基本 closed loop：

```text
AgentState
 ↓
DecisionSource
 ↓
AgentDecision
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

同時完成 EventTrace、state transition validation、execution/final-status separation、checkpoint cursor restoration、goal verification gate、failure-recovery coverage 與 crash/resume coverage。

核心 invariant：

```text
Running => final_status == None
Final status present => ExecutionState == Waiting
Finish(Done) => Goal must be Verified
```

### Stage 6 — Filesystem Runtime

**COMPLETED for the implemented foundation**

包含 path safety、sandbox containment、existence/read/list/metadata observation、create/write/append/delete/create-directory、structured errors 與 Runtime integration。

這是 foundation，不代表完整長期 Filesystem capability 已完成。

### Stage 7 — Process Runtime

**COMPLETED for the implemented foundation**

包含 `ProcessSpec`、process execution、executable / working-directory policy、timeout polling、termination/reaping、stdout/stderr capture、exit status 與 structured results。

這是 foundation，不代表完整長期 Job / Shell / Process capability 已完成。

### Stage 8 — LLM Provider / Ollama

**COMPLETED**

完成 provider abstraction、Ollama adapter、configurable base URL/model、native `/api/chat` integration 與 provider-independent response representation。

### Stage 9 — Real LLM Integration / Release Gate

**COMPLETED through Stage 9.4**

- Stage 9.1：provider-response interpretation 與 strict tool-call validation。
- Stage 9.2：DecisionSource、provider-backed decision sourcing、ProviderDecisionSource、provider-independent error mapping。
- Stage 9.3：real Ollama closed-loop integration。
- Stage 9.4：release gate passed。

目前 interpreter 的 implemented tool surface 仍刻意狹窄，主要為 `execute_process` 與 `read_file`。擴充 tool surface 必須依後續已批准的 semantic boundary 進行。

## 4. Completed Post-Cycle Increment

### Observation Store + Context Compiler

**COMPLETED**

第一版 semantic foundation 已完成：

```text
Runtime ActionResult / Observation
        ↓
Observation Store
        ↓
Context Compiler
        ↓
Compiled Context
        ↓
Provider Request
        ↓
Agent Decision
```

已完成的主要能力：

- `ObservationStore` abstraction
- `InMemoryObservationStore`
- stable observation ID / validation / duplicate rejection
- deterministic retrieval / recent bounded view
- `ContextCompiler` abstraction
- `DefaultContextCompiler`
- deterministic bounded context construction
- `AgentLoop` authoritative observation storage integration
- checkpoint restoration of the retained observation subset
- `ProviderDecisionSource` 使用 `ContextCompiler`
- provider request 不再直接從 `AgentState` ad-hoc 建立 semantic context
- closed-loop tests 覆蓋 action result → observation → store → next provider context

重要語意：

```text
Observation Store
= authoritative in-process observation history

AgentState.recent_observations
= bounded compatibility / operational view

ContextCompiler output
= decision context
```

這三者不是三份獨立 history。

### 驗證

該 increment 已通過：

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo check
cargo test
git diff --check
```

並完成對應 closed-loop integration verification。實際 test count 應以目前 repository 執行結果為準，不應依賴舊 session 的數字。

## 5. B1 — Knowledge Semantic Foundation

**IMPLEMENTED — LOCAL VERIFICATION PENDING**

本 increment 依 `docs/09_AGENT_CORE_EVIDENCE_KNOWLEDGE_SEMANTICS_DESIGN.md` 實作，範圍固定為 Evidence / Knowledge semantic foundation。

已加入：

```text
KnowledgeClaim
    id / subject / predicate / value / status / scope / evidence_refs

KnowledgeClaimStatus
    Observed / Inferred / Hypothesis

EvidenceLink
    observation_id / claim_id / relation

EvidenceRelation
    Supports / Contradicts / Qualifies

Unknown
    id / subject / scope / question

KnowledgeStore
InMemoryKnowledgeStore
KnowledgeStoreError
```

已建立的 boundary：

```text
ObservationStore
    = authoritative observation history

KnowledgeStore
    = derived Knowledge / Evidence / Unknown state

EvidenceLink
    = references ObservationStore by stable observation ID

ContextCompiler
    = can consume KnowledgeClaim / EvidenceLink / Unknown deterministically
```

重要限制：

- 不新增 `AgentDecision::UpdateKnowledge`。
- 不加入 confidence / probability / truth score。
- 不自動解決 conflicting claims。
- 不複製 Observation history。
- 不引入 SQLite、RAG、vector DB 或外部 Knowledge persistence。
- 不修改 Runtime 成為 Knowledge engine。
- `Observed` / `Inferred` claim 必須透過 atomic `record_claim_with_evidence` 建立 provenance。
- `Hypothesis` 可以沒有 evidence support，但不得被表示為 Observed / Inferred。
- EvidenceLink duplicate deterministic reject。

### B1 驗證狀態

程式碼與 deterministic unit tests 已加入 repository，但本 session 尚未在使用者本機執行：

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo check
cargo test
git diff --check
```

因此在本機驗證完成前，不將 B1 標記為 VERIFIED / RELEASED。

## 6. Current Architecture Boundary

目前高層資料流：

```text
User Goal
    ↓
Agent Core
    ↕
Agent State
    ↕
Observation Store
    ↕
Knowledge Store
    ↓
Context Compiler
    ↓
Decision Source / LLM Provider
    ↓
Agent Decision
    ↓
Action
    ↓
Runtime
    ↓
Windows / Environment
    ↓
ActionResult / Observation
    ↓
Agent Core
```

責任邊界：

```text
Agent Core
= state / decision / adaptation / verification / semantic state

Runtime
= computer-operation capability

Observation Store
= authoritative historical evidence

Knowledge Store
= derived Knowledge / Evidence / Unknown semantic state

Context Compiler
= deterministic semantic context construction

Provider Adapter
= provider-specific wire formatting
```

Runtime 不得變成 hard-coded high-level workflow engine。

Agent Core 不得依賴 Windows-specific implementation details。

Provider 必須保持可替換。

## 7. Remaining Long-Term Work

以下代表 **尚未完整實作的長期能力**，不是說第一版 Observation Store / Context Compiler / B1 foundation 不存在：

### Agent Core semantic direction

- richer Observation semantics
- Hypothesis lifecycle
- Progress detection
- Loop detection
- Adaptive recovery
- stronger Goal Verification

### Runtime / capability direction

- richer Environment Observation
- Shell / Interactive Shell
- complete Job Runtime semantics
- Engineering Runtime
- Browser Runtime
- GUI Runtime
- richer Network / System Runtime

這些項目不能自行變成新的 implementation Stage。是否進入 implementation，必須先有適用的 stable design、scope、non-goals、acceptance criteria、dependency understanding 與使用者批准。

## 8. Known Documentation Rule

本 repository 對新 AI 的 startup reading 已固定為：

```text
AGENTS.md
DEVELOPMENT_WORKFLOW.md
IMPLEMENTATION_STATUS.md
README.md
docs/README.md
```

然後由 `docs/README.md` 選擇 task-specific stable documents。

不要因為 `docs/` 文件越來越多，就把所有文件加入 startup context。

Stable design inventory 與 task-specific reading rules 的唯一導航來源是：

```text
docs/README.md
```

## 9. Current Next Step

B1 完成後不自動進入下一個 increment。

目前唯一待辦是完成本機驗證並確認：

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo check
cargo test
git diff --check
```

驗證通過後 STOP，等待下一個明確指令。
