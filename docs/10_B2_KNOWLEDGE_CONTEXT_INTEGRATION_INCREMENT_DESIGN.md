# AI Agent v3 — B2 Knowledge Context Integration Increment Design

Status: **APPROVED STABLE DESIGN**

本文件定義 B2 implementation increment 的唯一 stable boundary。它建立在 `docs/09_AGENT_CORE_EVIDENCE_KNOWLEDGE_SEMANTICS_DESIGN.md` 與已完成、已驗證的 B1 Knowledge Semantic Foundation 之上。

本文件是 implementation increment design，不是 implementation status，也不是 implementation diary。實作完成後不應因為完成而例行修改本文件；若長期架構、介面或驗收要求改變，才依 conflict procedure 修改。

## 1. Current State

B1 已完成並通過 Release Gate。Repository 目前已具有：

```text
ObservationStore
KnowledgeStore
KnowledgeClaim
EvidenceLink
Unknown
ContextCompiler knowledge-aware compilation API
```

但目前仍存在明確的 data-path gap：

```text
KnowledgeStore
    ↓
ContextCompiler::compile_with_knowledge()
```

已存在，但 `AgentLoop` 尚未 authoritative-ownership 地持有 KnowledgeStore，`ProviderDecisionSource` 尚未在正常 decision path 使用 KnowledgeStore，而 `ProviderRequest` 尚未把 Knowledge / Evidence / Knowledge Unknown semantic context 傳遞給 provider。

因此 B1 建立的是 semantic foundation；B2 的目的不是重新設計 Knowledge semantics，而是把該 foundation 接入 Agent Core 的正常 decision data path。

## 2. B2 Objective

B2 的核心問題：

```text
Agent Core semantic state
    ↓
ObservationStore + KnowledgeStore
    ↓
ContextCompiler
    ↓
CompiledContext
    ↓
ProviderRequest
    ↓
DecisionSource / Provider
```

B2 必須使 Knowledge semantic state 成為正常 Agent Core decision context 的一級資料來源，同時保持 Observation、Knowledge、Runtime、Provider 的既有 ownership boundary。

B2 不負責建立完整 Knowledge reasoning engine。

## 3. Scope

B2 分為三個 implementation sub-increments。三者屬於同一 B2 stable boundary，但仍應一次只實作一個，完成並驗證後再進入下一個。

### B2-A — Ownership / Data Path

範圍：

1. `AgentLoop` 成為 authoritative `KnowledgeStore` owner。
2. checkpoint / constructor compatibility 維持 deterministic 行為。
3. DecisionSource 的 semantic input boundary 能同時取得 authoritative `ObservationStore` 與 `KnowledgeStore`。
4. `ProviderDecisionSource` 使用 authoritative KnowledgeStore，而不是自行建立第二份 semantic state。
5. 保持既有 implementor compatibility；若需要新增 API，優先 additive evolution，不任意破壞現有 trait contract。
6. 建立 ownership / data-path tests。

B2-A 不要求 provider request 已經輸出所有 Knowledge fields；該責任屬於 B2-B。

### B2-B — Context / Provider Integration

範圍：

1. `ProviderDecisionSource` 的正常 decision path 使用 `ContextCompiler::compile_with_knowledge()`。
2. `CompiledContext` 中的 KnowledgeClaim、EvidenceLink、Knowledge Unknown 成為 provider request 的 semantic input。
3. `ProviderRequest::from_compiled_context` 擴充為 deterministic、provider-independent 的 Knowledge semantic representation。
4. Provider adapter 只負責既有 wire formatting，不取得 Knowledge reasoning ownership。
5. 測試確認 Knowledge context 確實從 authoritative store 流入 ProviderRequest。
6. 驗證 conflicting claims 可以原樣共存；provider request 不自行選真或刪除衝突。

B2-B 不新增 provider-specific Knowledge semantics。

### B2-C — Closed-loop Semantic Integration

範圍：

1. 定義並實作符合 `09` 第 14 節的最小 Core-level semantic update boundary。
2. semantic update 不成為 `AgentDecision::UpdateKnowledge`。
3. semantic update 不成為 Runtime Action。
4. semantic update 可以把已存在的 Observation identity 與 Knowledge semantic state 建立 deterministic provenance。
5. 建立 closed-loop deterministic test：

```text
ActionResult
    ↓
Observation
    ↓
ObservationStore
    ↓
Core semantic update
    ↓
KnowledgeStore
    ↓
ContextCompiler
    ↓
ProviderRequest
    ↓
next decision context
```

6. 驗證 `Observed / Inferred` provenance、Unknown 與 conflict coexistence 在下一次 decision context 中仍可追溯。

B2-C 不定義哪一個未來 reasoning component 必須產生所有 Knowledge；只建立目前必要且可驗證的 Core semantic update boundary。

## 4. Dependencies

B2 依賴：

```text
docs/03_INTERFACES.md

docs/06_IMPLEMENTATION_GAP_ANALYSIS.md

docs/09_AGENT_CORE_EVIDENCE_KNOWLEDGE_SEMANTICS_DESIGN.md

B1 Knowledge Semantic Foundation
Observation Store + Context Compiler foundation
```

直接相關 source 預期包括：

```text
src/core/agent_loop.rs
src/core/decision_source.rs
src/core/context_compiler.rs
src/core/knowledge_store.rs
src/core/mod.rs
src/lib.rs
```

以及對應的 Core / closed-loop / checkpoint / provider tests。實作前仍必須以當時 repository source 為準，不得假設本文件列出的檔案內容永遠不變。

## 5. Ownership Boundary

B2 完成後的高層 ownership 應為：

```text
AgentLoop / Agent Core
    owns AgentState
    owns ObservationStore
    owns KnowledgeStore

ObservationStore
    owns authoritative Observation history

KnowledgeStore
    owns derived Knowledge / Evidence / Unknown state

ContextCompiler
    selects / orders / bounds semantic context

ProviderRequest
    carries compiled semantic context

Provider Adapter
    formats provider-specific wire representation

Runtime
    executes Actions and returns ActionResult
```

禁止：

```text
KnowledgeStore 複製 Observation history
Provider 建立或修改 authoritative Knowledge
Runtime 成為 Knowledge engine
ContextCompiler 修改 KnowledgeStore
ProviderRequest 自行推理 / resolve conflict
```

## 6. Decision Data Path

B2 的正常 decision path 應收斂為：

```text
AgentState
    +
ObservationStore
    +
KnowledgeStore
        ↓
ContextCompiler
        ↓
CompiledContext
        ↓
ProviderRequest
        ↓
ProviderDecisionSource
        ↓
AgentDecision
```

其中：

- `ObservationStore` 是 authoritative observation history。
- `KnowledgeStore` 是 authoritative in-process derived semantic state。
- `ContextCompiler` 不產生 semantic truth，只負責 deterministic context construction。
- `ProviderRequest` 是 compiled semantic context 的 transport representation，不是 reasoning engine。
- Provider 的輸入不能成為 Core semantic state 的 source of truth。

## 7. Semantic Update Boundary

B2-C 必須落實 `09` 所要求的 Core-level semantic update boundary，但不得提前決定完整未來 reasoning architecture。

允許的語意形式必須能表達至少：

```text
Add / record KnowledgeClaim
Add EvidenceLink
Add Unknown
```

具體 Rust type / trait / method name 可在 B2-C implementation design 中確定，但必須滿足：

1. operation 屬於 Agent Core semantic layer。
2. 不屬於 `AgentDecision`。
3. 不屬於 Runtime `Action`。
4. 不要求 provider 直接操作 store。
5. 驗證失敗是 deterministic Core-level error。
6. provenance 必須引用既有 Observation identity。
7. 不新增 numeric confidence / probability / truth score。
8. 不自動 resolve conflicting claims。

B2-C 不批准完整 Hypothesis lifecycle；`Hypothesis` 仍只是 B1 定義的 status。

## 8. Checkpoint Compatibility

B2 不引入新的外部 persistence system。

對既有 checkpoint：

```text
load successfully
```

或：

```text
deterministically fail with a clear compatibility error
```

皆可接受，但不得靜默產生錯誤 Knowledge state。

如果 B2 第一版尚未將完整 KnowledgeStore persistence 納入 checkpoint，必須明確保持「只恢復已保存 subset」的語意；不得聲稱完整 Knowledge history 已恢復。

B2-A 對 `AgentLoop::from_checkpoint` 的處理不得破壞既有 Observation retained-subset compatibility。

## 9. Legacy AgentState Fields

既有：

```text
AgentState.unknowns
AgentState.evidence
AgentState.hypotheses
```

在 B2 不直接刪除。

B2 的目標是建立 authoritative KnowledgeStore data path，而不是一次完成 legacy field migration。若這些欄位仍作為 compatibility / operational views 存在，必須避免它們重新成為第二份 authoritative Knowledge history。

任何全面 migration / removal 必須另有明確 implementation boundary。

## 10. Determinism Requirements

B2 所有新增 data path 必須 deterministic：

- Knowledge retrieval order deterministic。
- Evidence retrieval order deterministic。
- Unknown retrieval order deterministic。
- ProviderRequest serialization order deterministic。
- Duplicate semantic operations deterministic reject 或明確定義的 update semantics。
- 相同 state + stores + compiler configuration 必須產生相同 compiled context。
- 不依賴 wall clock、random selection、global mutable state 或 provider-specific ordering。

## 11. Tests / Acceptance Criteria

B2 Release Gate 必須至少證明：

### Ownership

1. Agent Core / AgentLoop 持有唯一 authoritative KnowledgeStore。
2. DecisionSource 使用該 store，而非建立 temporary authoritative copy。
3. ObservationStore 與 KnowledgeStore 不互相複製 ownership。

### Context propagation

4. KnowledgeClaim、EvidenceLink、Knowledge Unknown 能由 KnowledgeStore 經 ContextCompiler 到 ProviderRequest。
5. ProviderRequest 不從 legacy AgentState string fields ad hoc 重新建立 Knowledge semantics。
6. Provider boundary 不產生、修改或 resolve Knowledge truth。

### Semantic integrity

7. Observed / Inferred provenance 可追溯至 ObservationStore。
8. Conflicting evidence 可以共存。
9. Unknown 與 absence of evidence 保持語意區別。
10. Duplicate handling deterministic。

### Closed loop

11. 至少一個 deterministic integration test 覆蓋：

```text
ActionResult → Observation → ObservationStore
→ semantic update → KnowledgeStore
→ ContextCompiler → ProviderRequest
```

12. 下一次 decision context 可以觀察到新增 Knowledge semantic state。

### Compatibility

13. 既有 Runtime execution model 保持有效。
14. 既有 `AgentDecision` model 不因 B2 新增 `UpdateKnowledge` variant。
15. 既有 checkpoint behavior 維持明確且 deterministic。
16. B1 tests 與既有 closed-loop tests 不退化。

### Verification

17. `cargo fmt --check` PASS。
18. `cargo clippy --all-targets --all-features -- -D warnings` PASS。
19. `cargo check` PASS。
20. `cargo test` PASS。
21. `git diff --check` PASS。

## 12. Explicit Non-Goals

B2 不批准：

```text
Hypothesis lifecycle
Progress detection
Loop detection
Adaptive recovery
Stronger Goal Verification redesign
Planner / tree search
confidence / probability / truth score
automatic truth resolution
RAG / vector DB
SQLite / external Knowledge persistence
multi-agent shared knowledge
Browser / GUI / Vision
Unrestricted Shell
Engineering Runtime
provider-specific Knowledge semantics
large-scale AgentState legacy migration
complete Knowledge checkpoint persistence redesign
```

任何上述能力若要進入 implementation，必須先有自己的 stable design / approved implementation boundary。

## 13. Increment Execution Rule

B2 是一個 stable design，下面三個 sub-increments 必須依序單獨批准與驗證：

```text
B2-A
Ownership / Data Path
    ↓
B2-B
Context / Provider Integration
    ↓
B2-C
Closed-loop Semantic Integration
```

每個 sub-increment 完成後：

```text
implementation
→ verification
→ IMPLEMENTATION_STATUS.md update
→ Git checkpoint
→ STOP
```

不得因 B2-A 完成而自動開始 B2-B；不得因 B2-B 完成而自動開始 B2-C。

## 14. Approval Boundary

本文件目前批准的是 **B2 implementation design**，不是立即批准 B2-A/B/C 的程式碼實作。

真正開始每一個 sub-increment 前，仍需依 `DEVELOPMENT_WORKFLOW.md` 取得使用者對該 sub-increment 的明確批准。

本文件也不建立 Stage 10。
