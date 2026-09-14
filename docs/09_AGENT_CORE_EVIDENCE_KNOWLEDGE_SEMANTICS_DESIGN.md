# AI Agent v3 — Agent Core Evidence / Knowledge Semantics Design

Status: **APPROVED STABLE DESIGN**

本文件定義 Agent Core 在 Observation Store + Context Compiler 之後，下一個語意基礎層：如何把 Observation 轉化為可追溯的 Evidence / Knowledge / Unknown 語意，而不把推理、真值判定、信心評分或未批准的未來能力提前塞入本層。

本文件是 stable design，不是 implementation task，也不是 implementation diary。

## 1. Design Boundary

本 increment 的核心問題是：

```text
Observation
    ↓
Evidence relationship
    ↓
Knowledge / Unknown semantic state
    ↓
Context Compiler
    ↓
Agent reasoning / decision
```

其中：

```text
Observation
= 已發生、可追溯的環境 / 執行證據

Evidence
= Observation 與 KnowledgeClaim 之間的 provenance / relationship

KnowledgeClaim
= Agent 目前採用的語意理解，不等於客觀真理

Unknown
= 明確存在且與目前 reasoning relevant 的資訊缺口
```

Observation 與 Knowledge 不得混成同一個 store。

`ObservationStore` 仍然是 authoritative historical observation source。

Knowledge 是 derived semantic state，不能反過來取代 Observation history。

## 2. Scope

本設計第一版只定義：

- KnowledgeClaim
- KnowledgeClaim status
- EvidenceLink
- Unknown
- KnowledgeStore 的 ownership / validation / deterministic retrieval
- Observation → Evidence → Knowledge 的 provenance invariant
- Knowledge / Evidence / Unknown 如何被 Context Compiler 消費
- checkpoint / compatibility boundary
- deterministic semantics

本設計不定義：

- Hypothesis lifecycle
- Progress detection
- Loop detection
- Adaptive recovery
- Stronger Goal Verification
- Planner / tree search
- RAG / vector database
- SQLite / external persistence
- Multi-agent knowledge sharing
- Browser / GUI / Vision
- unrestricted shell
- Engineering Runtime
- provider-specific knowledge semantics
- numeric confidence / probability / truth score

## 3. KnowledgeClaim

第一版 KnowledgeClaim 應具有明確、可識別的 semantic identity：

```text
KnowledgeClaim
    id
    subject
    predicate
    value
    status
    scope
    evidence_refs
```

欄位語意：

- `id`：穩定且 deterministic 的 claim identity。
- `subject`：claim 所描述的對象。
- `predicate`：描述該對象的語意關係。
- `value`：predicate 所對應的值或語意內容。
- `status`：目前 epistemic status。
- `scope`：claim 成立所適用的 context / boundary。
- `evidence_refs`：可追溯至 EvidenceLink / Observation 的 provenance references。

第一版 status 僅允許：

```text
Observed
Inferred
Hypothesis
```

`Unknown` 不作為 KnowledgeClaim status。

Unknown 是獨立的 semantic object，因為「不知道某件 relevant information」與「已有一個 claim，但它處於某種 epistemic status」是不同概念。

### 3.1 Status Semantics

`Observed`：claim 直接由 observation 支持，且其語意轉換不需要額外推理才能成立。

`Inferred`：claim 由一個或多個 observations / claims 經 reasoning 得出，但仍必須保留 provenance。

`Hypothesis`：目前為可被驗證或反駁的候選理解；其存在不表示已被證實。

第一版不引入：

```text
Fact
Confirmed
Rejected
Stale
Superseded
Confidence
Probability
TruthScore
```

`Observed` 也不等於絕對客觀真理；它只描述 claim 與 observation 的 epistemic relationship。

## 4. Claim Identity

Claim identity 必須明確、可重現、可 deterministic 判定。

不得使用自然語言相似度、LLM semantic deduplication 或模糊 matching 決定兩個 claim 是否相同。

規則：

```text
same stable claim ID
    = same semantic claim identity

different claim ID
    = potentially different claim
```

第一版不得因為兩個 claim「看起來意思很像」而自動合併。

## 5. EvidenceLink

Evidence 是 provenance relationship，不是另一個自由文字結論。

第一版至少定義：

```text
EvidenceLink
    observation_id
    claim_id
    relation
```

relation 至少包含：

```text
Supports
Contradicts
Qualifies
```

語意：

- `Supports`：Observation 提供支持 claim 的 evidence。
- `Contradicts`：Observation 與 claim 的內容衝突。
- `Qualifies`：Observation 限定、縮小或補充 claim 的適用範圍，而不直接等同支持或反駁。

EvidenceLink 必須能追溯到實際 ObservationStore 中存在的 observation。

## 6. Provenance Invariant

任何 `Observed` 或 `Inferred` KnowledgeClaim 都必須能透過 EvidenceLink 追溯到至少一個 Observation。

概念上：

```text
KnowledgeClaim
    ↓
EvidenceLink(s)
    ↓
ObservationStore
```

Knowledge 可以是錯的。

但 provenance 不可以被丟失或偽造。

若某 claim 沒有可追溯 observation，第一版不能宣稱它是 `Observed` 或 `Inferred`。

`Hypothesis` 仍然必須保留其目前可用的 provenance；如果 hypothesis 尚無 observation support，必須由語意模型明確允許，而不是假裝成 Observed / Inferred。

## 7. Conflict Semantics

Evidence 與 claims 可以互相衝突。

例如：

```text
Observation A → Supports Claim X
Observation B → Contradicts Claim X
```

第一版 KnowledgeStore 不得自動：

- 刪除其中一個 claim
- 覆寫其中一個 claim
- 選出「真正正確」的 claim
- 以數值 confidence 排名
- 自動產生 resolved truth

衝突本身是 Agent Core 可以消費的 semantic state。

後續由 reasoning / verification / hypothesis lifecycle 決定如何處理衝突；本設計不提前承擔該責任。

## 8. EvidenceLink Deduplication

相同：

```text
observation_id + claim_id + relation
```

的 EvidenceLink 視為同一 semantic relationship。

第一版若重複加入，必須 deterministic reject，或使用明確定義的 update semantics；不得靜默產生無限重複 provenance。

推薦第一版採 deterministic duplicate rejection。

## 9. Unknown

Unknown 是第一級 semantic object。

它表示：

```text
目前 reasoning 明確需要某項資訊，
但目前 evidence 不足以建立該資訊的有效 knowledge claim。
```

因此：

```text
Unknown != absence of evidence
```

沒有 observation 不代表系統已經知道「不知道什麼」。

Unknown 必須能描述至少：

```text
id
subject / scope
question 或 required information
```

第一版不要求建立完整 ontology，也不要求自動從所有 missing data 推導 Unknown。

Unknown 的產生屬於 Agent Core semantic reasoning；KnowledgeStore 負責保存、驗證與 deterministic retrieval。

## 10. Scope

KnowledgeClaim 必須具有 scope 概念，以避免將局部 observation 不當泛化成 universal truth。

第一版 scope 只需要能表達 claim 的適用 context / boundary，不建立完整 ontology、temporal logic 或複雜 domain-specific type system。

Scope 是 semantic boundary，不是 confidence。

## 11. KnowledgeStore Ownership

KnowledgeStore 負責：

- 儲存 KnowledgeClaim
- 儲存 EvidenceLink
- 儲存 Unknown
- 驗證 claim identity
- 驗證 EvidenceLink reference integrity
- deterministic duplicate handling
- deterministic retrieval / ordering
- provenance integrity

KnowledgeStore 不負責：

- reasoning
- truth evaluation
- confidence scoring
- hypothesis lifecycle
- progress detection
- loop detection
- adaptive recovery
- planning
- provider request formatting

因此：

```text
ObservationStore
= authoritative observation history

KnowledgeStore
= derived semantic knowledge state

Agent Core reasoning
= decides how evidence changes understanding

ContextCompiler
= selects / orders / bounds information for provider context
```

## 12. ObservationStore Relationship

KnowledgeStore 不得複製 Observation history。

正確 ownership：

```text
ObservationStore
    owns Observation history

KnowledgeStore
    owns Knowledge / Evidence / Unknown

EvidenceLink
    references ObservationStore observations
```

如果 KnowledgeStore 需要查 provenance，應透過 observation identity 取得 Observation，而不是建立第二份 authoritative observation copy。

## 13. Context Compiler Integration

ContextCompiler 可以消費：

```text
goal
current understanding
relevant observations
recent actions
relevant KnowledgeClaims
evidence relationships
Unknowns
active problems
```

Compiler 負責：

- selection
- ordering
- semantic bounds
- deterministic construction

Compiler 不負責：

- 建立 KnowledgeClaim
- 修改 KnowledgeStore
- 判斷哪個 claim 才是真的
- 自動解決 conflict
- 產生 confidence
- 執行 reasoning

因此語意邊界保持：

```text
Core semantic state
    ↓
Context Compiler
    ↓
Provider Request
    ↓
Provider Adapter
```

Provider 不得成為 Knowledge semantics 的 source of truth。

## 14. Semantic Update Boundary

第一版不新增：

```text
AgentDecision::UpdateKnowledge
```

Knowledge update 是 Agent Core semantic operation，而不是 Runtime Action，也不是必須直接暴露給 provider 的 decision variant。

具體由哪個 reasoning component 產生 / 更新 Knowledge，保留給後續 Agent Core implementation 定義。

本設計只要求存在一個明確、可驗證的 Core-level semantic update boundary。

## 15. Determinism

在相同：

```text
KnowledgeStore state
ObservationStore state
Agent semantic state
compiler configuration
```

下，semantic result 必須 deterministic。

禁止依賴：

- wall clock
- random selection
- global mutable state
- provider-specific ordering
- unordered iteration whose order is externally observable
- LLM semantic deduplication

Retrieval ordering、duplicate handling、validation 結果與 compiler output 都必須可重現。

## 16. Persistence / Checkpoint Compatibility

第一版不得因 Knowledge semantics 而引入新的外部 persistence system。

Persistence 仍依既有 checkpoint architecture 處理。

既有 checkpoint 必須：

```text
load successfully
```

或：

```text
deterministically fail with a clear compatibility error
```

不得靜默產生錯誤 semantic state。

如果第一版尚未將完整 KnowledgeStore persistence 納入 checkpoint，則只能恢復已明確保存的 subset；不得宣稱恢復了完整 Knowledge history。

同樣地，ObservationStore 的既有 retained-observation compatibility 不得被重新描述成完整歷史恢復。

## 17. Failure and Invalid Data Semantics

Invalid KnowledgeClaim、EvidenceLink 或 Unknown 不得靜默進入 store。

至少需要 deterministic validation：

- required identity fields valid
- claim ID valid
- evidence reference valid
- relation valid
- status valid
- duplicate identity deterministic
- duplicate EvidenceLink deterministic

Validation failure 應成為可觀察的 Core-level error，不應被轉成假的 knowledge。

## 18. First-Version Acceptance Criteria

本 design 的第一版 implementation 必須能證明：

1. Observation history 與 Knowledge semantic state 有清楚 ownership boundary。
2. KnowledgeClaim 具有 deterministic identity。
3. `Observed / Inferred / Hypothesis` 語意清楚且不與 Unknown 混淆。
4. EvidenceLink 至少支援 `Supports / Contradicts / Qualifies`。
5. Observed / Inferred claim 能追溯到 Observation。
6. Duplicate EvidenceLink 的行為 deterministic。
7. Conflicting claims 可以共存，不被自動刪除或選真。
8. Unknown 是獨立 semantic object，而不是 claim status。
9. Scope 存在且不被當成 confidence。
10. KnowledgeStore 不取代 ObservationStore。
11. ContextCompiler 可以消費 relevant Knowledge / Evidence / Unknown。
12. Provider boundary 不承擔 Knowledge semantics。
13. 不新增 `AgentDecision::UpdateKnowledge` 作為第一版必要介面。
14. 不引入 numeric confidence / probability / truth score。
15. deterministic tests 可以重現 semantic state、duplicate handling、provenance 與 conflict coexistence。
16. Runtime execution model 不因本 increment 被改造成 Knowledge engine。
17. 既有 checkpoint compatibility 維持明確、可驗證的行為。
18. 沒有偷偷擴大到 C–G、RAG、SQLite、multi-agent 或其他 non-goals。

## 19. Non-Goals Reaffirmed

本文件不批准以下 implementation：

```text
Hypothesis lifecycle
Progress detection
Loop detection
Adaptive recovery
Stronger Goal Verification
Planner / tree search
RAG / vector DB
SQLite / external Knowledge DB
Multi-agent shared memory
Browser / GUI / Vision
Unrestricted Shell
Engineering Runtime
Provider-specific knowledge representation
Numeric confidence / truth scoring
```

這些能力必須有各自的 stable design / implementation boundary 後才能進入 implementation。

## 20. Implementation Boundary

本文件批准的是 **Evidence / Knowledge semantic foundation**，不是 Stage 10，也不是完整 Knowledge system。

Implementation 開始前仍必須依 `DEVELOPMENT_WORKFLOW.md` 建立單一明確 implementation increment，重新確認：

```text
current implementation state
relevant source / tests
dependencies
scope
non-goals
acceptance criteria
user-approved implementation boundary
```

本文件本身不得被解讀成「下一個 Stage 自動開始」。

## 21. Design Stability Rule

本文件一旦進入 stable baseline：

- 不因單次 implementation detail 而修改。
- 不因 test implementation 方便而降低 semantic requirements。
- 若實作與本設計衝突，依 stable-design conflict procedure 停止並處理。
- 若未來證明本設計本身需要改變，必須明確修改本文件並重新建立受影響的 implementation boundary。
