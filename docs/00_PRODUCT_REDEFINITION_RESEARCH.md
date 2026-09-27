# AI Agent v3 — Product Redefinition Research

**文件性質**：研究報告。非 stable design，非 implementation task，非 approved increment。
**研究基準 checkpoint**：`fa0a6a5726873479fbf048e6f85316e4e800453a`
**研究日期**：2026-09-28
**授權範圍**：只讀研究。未修改 production code、未建立 implementation increment、未 commit。

本文件不受 `docs/01~11` 的 stable design 治理約束，也不構成任何 increment 的批准。
本文件亦**未**修改 `README.md`、`AGENTS.md`、`docs/01~11`、`src/`、`tests/`。

---

## 0. 執行摘要

研究的核心結論：

> **`ai-agent-v3` 的原始問題敘述（「今天的 LLM 不擅長決策，所以需要一個 deterministic Agent Core 來補」）在 2026 年已經過期。**
> 但它用來驗證那個敘述的**方法論**——把語意主張與環境證據分離、讓衍生狀態的 mutation 具備原子性、讓「不知道」保持為「不知道」——在 2026 年的產業需求中意外地**變得更相關，而不是更不相關**。

三個最具決定性的外部證據：

1. **OpenAI Codex 自己實作了一個 deterministic rejection circuit breaker**：連續 3 次拒絕、或同一回合內 50 次滑動視窗內 10 次拒絕，即中止該回合。同時 OpenAI 官方文件明文寫道 auto-review 這個 LLM reviewer「**is not a deterministic security guarantee**」。這證明：即使是最有能力、最新的 agent 廠商，也不會把「防止 agent 失控」交給 LLM。

2. **Anthropic 官方文件給出了本次研究最清楚的準則**：
   > 「Put guardrails in hooks. An instruction like "never edit `.env`" in CLAUDE.md or a skill **is a request, not a guarantee**. A `PreToolUse` hook that blocks the edit **is enforcement**.」
   > Hook 的 Determinism：「Always fires on its event; the trigger is guaranteed」；Skill 的 Determinism：「Claude interprets the instructions; **outcome can vary**」

   這直接回答了本研究的核心問題：Anthropic 自己把 deterministic 邏輯**移出 context、移進 harness**。

3. **「Agent Work Ledger / Work Receipt」這個產品類別在 2026 年正在被商業化，且已經有賣家**。這代表該方向被驗證，同時也代表它正在被爭奪。

**建議**（詳見 §7）：不要放棄 `ai-agent-v3` 的架構基因，但**徹底放棄其產品身分**。它不該是一個「更好的 agent」，它該是 **any agent 的 deterministic evidence layer**。

---

## 第一階段：目前 Repository 的真實狀態

### 1.1 已實作且經測試驗證的能力

| 能力 | 位置 | 驗證 |
|---|---|---|
| Canonical domain model 與驗證不變量 | `src/core/types.rs` | 121 unit tests |
| `ObservationStore` 權威 Observation 歷史 | `observation_store.rs:41-70` | 7 unit + integration |
| `KnowledgeStore` claim/evidence/unknown；**clone-validate-commit 原子 mutation** | `knowledge_store.rs:248-347` | 6 unit + C4 7 integration |
| Core semantic update boundary | `semantic_updater.rs` | 4 unit + integration |
| **Observation-first ordering**（`record()` 成功後才 `apply_semantic_update()`） | `agent_loop.rs:339-345`, `357-361` | C2 + C3 Test14 |
| `ContextCompiler` 決定性 bounded 編譯；對 Knowledge 唯讀 | `context_compiler.rs:45-59` | C4 T6a |
| Provider decision path 隔離（**不持有** observation store） | `decision_source.rs:59-65` | unit + integration |
| 完整閉環語意整合 | `closed_loop_tests.rs` | C3 4 tests |
| Failure / atomicity 邊界 | `closed_loop_tests.rs` | C4 7 tests |
| Filesystem Runtime（sandbox、path safety） | `filesystem_runtime.rs` | 11 tests |
| Process Runtime（timeout、policy deny-list） | `process_runtime.rs` | 10 tests |
| Provider interpreter 嚴格驗證 | `interpreter.rs:69-98` | 15 unit |
| JSON checkpoint + crash/resume | `checkpoint.rs` | 6 schema + 1 integration |

**總計 199 個測試**（121 unit / 22 closed-loop / 11 fs / 10 process / 6 schema / 6 doubles / 2 provider / 3 ignored）。

B1 + B2 的資料路徑確實成立且有測試證據。ownership 邊界清晰，atomicity 經 C4 實際釘住。**這部分品質是好的。**

### 1.2 只是 abstraction / scaffold

| Scaffold | 證據 |
|---|---|
| `AgentState` 語意欄位全是空殼 | `hypotheses`/`unknowns`/`evidence`/`environment_state`/`running_jobs`/`remaining_work`/`active_problems` **無任何 Core 邏輯寫入**；只有 `evidence` 被 `verify_goal()` 寫過一次 |
| **無任何 production reasoning producer** | `semantic_updater.rs:46-51` 只有 `NoOpSemanticUpdateProducer`。所有 KnowledgeClaim 都來自 test producer |
| `KnowledgeState` enum 是 dead type | 定義於 `types.rs:41-47`，未出現在任何 struct field 或邏輯 |
| `ObservationKind::EnvironmentDelta` 從未被建構 | `types.rs:205` |
| **`Runtime` trait 沒有 observation 能力** | `runtime.rs:19` 只有 `execute()`。不執行 Action 就無法取得環境資訊 |
| **`AgentDecision::Observe` 偽造證據** | `agent_loop.rs:350-353`：`Observation::new(obs_id, Environment, intent_string)` — summary 是 LLM 自己的意圖字串，`source_action_id: None`，卻被寫進權威 store 並餵進 `apply_semantic_update()` |
| Job model | `running_jobs: Vec<String>`，無 Job type。`docs/03` §28-30 已定義 lifecycle，未實作 |
| Progress / loop detection | src 內完全不存在 |
| LLM bounded correction / retry | `interpret()` 失敗 → `LoopError::DecisionSource` → `run()` 直接中止 |
| **無可執行 agent** | 無 `src/main.rs`、Cargo.toml 無 `[[bin]]`、無 CI |
| Ollama adapter 不送 tool 定義 | `ollama.rs:37-41` `OllamaChatRequest` 只有 `model`/`messages`/`stream`，LLM 從未被告知有哪些 tool |

### 1.3 兩個結構性事實（本研究後續全部推理的基礎）

**事實 A：這個 agent 在結構上無法終止任務。**

- `interpreter.rs` 只有 **2 個** `AgentDecision` 產出點（`:219`、`:245`），**兩者都是 `AgentDecision::act(action)`**。沒有任何路徑對應到 `Observe`／`Wait`／`Finish`。
- `verify_goal()` **只被測試呼叫**（grep 確認 production 無呼叫點）。
- 結論：provider 驅動的 agent 只能永遠「做事」，`run()` 無法終止，`Finish(Done)` 不可達。

**事實 B：這個 repo 已經親手示範了本研究最重要的反證風險。**

`agent_loop.rs:350-353` 就是「LLM assertion 冒充 evidence」的具體實作。如果這個 repo 自己在「證據基礎設施」thesis 上作弊，那麼 thesis 就必須被外部強制，而不能靠自覺。

---

## 第二階段：2026 Agent Landscape

### 2.1 主流系統的分工

| 系統 | 交給 LLM 的 | 留給 deterministic infrastructure 的 |
|---|---|---|
| **OpenAI Codex**（Rust） | 規劃、tool selection、程式碼生成 | **OS-level sandbox**（Seatbelt / Landlock / Windows native / WSL2 bwrap）、sandbox mode、approval policy、**rejection circuit breaker**、**OpenTelemetry audit export**、managed policy、network allow/deny、execpolicy prefix rules、`~/.codex/sessions` 記錄 |
| **Claude Code**（Anthropic） | Skills 的解讀、subagent 委派、workflow 推理 | **30+ hook events**（`PreToolUse`／`PostToolUse`／`SessionStart`／`SubagentStart`／`PermissionDenied`…）、5 種 hook type（command/http/mcp_tool/prompt/agent）、workspace trust dialog、managed policy、`agent_id`/`agent_type` 傳遞、worktree isolation |
| **OpenCode**（TS，client/server） | plan/build 推理、subagent | **三層 permission 優先序**（agent default → user config → session runtime）、allow/ask/deny + wildcards、`doom_loop` → ask、`containers` 隔離、config 8 層合併（含 MDM 最高優先） |
| **Gemini CLI**（TS） | 推理、model routing、subagent | **Plan mode 預設**（read-only）、OS-level sandbox、GEMINI.md 階層記憶、`SessionContext` 內的 `fs`/`shell` helper「adhere to policies/validation」 |
| **mini-SWE-agent / Pi** | 幾乎全部 | 幾乎沒有（4 tools、<1000 token prompt、無 MCP、無 subagent） |

### 2.2 三個關鍵的生態結構性事實

**事實 C：Agent harness 正在被刻意「抽薄」。**

> 「The industry moved from "build the smartest scaffold" to "**the model is the agent; the harness must be thin, safe, and replaceable**".」

而且 **SWE-bench Verified 這個評測標準本身假設最小 scaffold**：

> 「we evaluate all LMs using mini-SWE-agent in a minimal bash environment. **No tools, no special scaffold structure; just a simple ReAct agent loop.**」

這代表：harness 的「聰明程度」不是競爭維度。

**事實 D：MCP 正在刪除 agent 端的 deterministic 行為，同時強化邊界的 deterministic 行為。**

MCP `2026-07-28` spec：

- **刪除**：`Roots`、`Sampling`、`Logging` 三個 feature 正式 deprecated（建議改為「integrate directly with LLM provider APIs instead of Sampling」、「log to stderr or use OpenTelemetry」）。Legacy HTTP+SSE transport 也 deprecated。
- **強化**：stateless core、header-based routing（`Mcp-Method`／`Mcp-Name`，讓 gateway 可以直接對 header 做 routing 與 authorization）、RFC 9207 `iss` 驗證、CIMD 取代 DCR、credential 綁定 issuer。
- **新增治理機制**：feature lifecycle（Active / Deprecated / Removed）、**至少 12 個月的 deprecation window**、**deprecated feature registry**。

MCP 已於 2025-12 由 Anthropic 捐給 Linux Foundation 下的 Agentic AI Foundation。公開 server 逾 10,000 個，41% 組織已在 production 使用。

**事實 E：OTel GenAI semantic conventions 已經是「agent 活動的標準事件詞彙」。**

已定義的 `gen_ai.operation.name` 值包含：`invoke_agent`、`chat`、`execute_tool`、`plan`、`retrieval`、`create_memory`、`update_memory`、`upsert_memory`、`embeddings`。
已在使用：**VS Code Copilot、OpenAI Codex、Claude Code** 都已匯出 OTel。
AG-UI 則定義了另一套 stream 事件：`RUN_STARTED`、`TOOL_CALL_START`／`RESULT`／`END`、`TEXT_MESSAGE_CONTENT`、`RUN_FINISHED`、`STATE_SNAPSHOT`／`STATE_DELTA`。

**結論：跨 vendor 的 agent 行為記錄格式，正在被 OTel + AG-UI 標準化。這是「work ledger」產品的基礎設施，但也意味著 ledger 本身會變成 commodity——差異化必須在別處。**

### 2.3 基礎設施層的成熟度（對 `ai-agent-v3` 最重要的衝擊）

| 層 | 現況 | 對 `ai-agent-v3` 的意義 |
|---|---|---|
| **Sandbox** | E2B（Firecracker microVM、pause/resume、fork、BYOC、Embed 自架）、Daytona（90ms cold start、container+VM+GPU）、Modal（gVisor、GPU-in-sandbox）、Vercel Sandbox（Firecracker、**credential brokering 讓 secret 留在 sandbox 之外**、deny 優先於 allow 的 runtime firewall）。Vercel Sandbox 預設 image **已經內建 `claude`、`codex`、`opencode`** | `ai-agent-v3` 的 ProcessRuntime 與 FilesystemRuntime 是在重做一個已商品化且更安全的東西 |
| **Durable execution** | Temporal（Event History + Replay、workflow code 必須 deterministic、model call = Activity 各自有 timeout/retry）；DBOS（in-process library，checkpoint 到 Postgres/SQLite，**對既有 Pydantic AI agent 加 durable execution 少於 10 行**） | `ai-agent-v3` 的 JSON checkpoint 是在重做一個免費且更成熟的東西 |
| **Observability** | Langfuse、Phoenix、Braintrust、OTel collector；語意慣例已定 | `EventTrace` 幾乎立刻會變成 OTel 的 adapter |
| **Evaluation** | SWE-bench Verified、Inspect AI（scanners、approval、agent bridge、control channel）、promptfoo | `ai-agent-v3` 沒有任何真實任務評測能力 |

產業對 sandbox 的一個關鍵告誡：

> 「**The mistake almost everyone makes is treating the sandbox as the whole security story and forgetting the capability-permissions layer on top.**」
> 「Default-deny egress is the **single highest-leverage control**」

以及長時工作的一個硬限制：所有 sandbox 都有 5 分鐘到 24 小時的 session 上限（Modal 5 分鐘預設、E2B Base 1 小時 / Pro 24 小時、Daytona 15 分鐘 auto-stop、Vercel 45 分鐘到 24 小時）。**長時 agent 工作必須圍繞 session 邊界設計。**

---

## 第三階段：「LLM 越來越聰明」對架構的影響

### 3.1 會逐漸失去必要性的 deterministic logic

| Logic | 原因 |
|---|---|
| **決策面收斂** | SWE-bench 用 bare ReAct loop 評測所有模型；mini-SWE-agent 100 行、Pi 4 個 tools。決策邏輯沒有競爭護城河 |
| **Planner / hypothesis engine / strategy engine** | 這些是為了補償弱推理而存在的。LLM 強了，它們就是多餘的抽象層 |
| **Loop detector（作為語意判斷）** | LLM 自己能看出「我剛剛做了同樣的事三次」 |
| **Context compiler 的選擇策略** | 什麼該進 context 是 prompt engineering，而 prompt engineering 有半衰期 |
| **Semantic vocabulary 契約** | 若要求 LLM 填入框架自訂的 `SemanticUpdate` 分類，等於**框架對 LLM 推理施加分類法**。模型越強，這個契約越會變成障礙 |
| **Agent 內部的 session/checkpoint** | 已被 durable execution commodity 取代 |

### 3.2 不會消失、只會增強的 deterministic infrastructure

| 職責 | 產業證據 |
|---|---|
| **Sandbox / capability layer** | 每一個主要 agent 都有；sandbox 廠商明確說 sandbox 不等於全部 |
| **Permission / approval** | Codex 4 種 approval policy + granular；Claude Code hooks；OpenCode 三層 permission；Gemini CLI plan-mode 預設 |
| **Circuit breaker** | **Codex：連續 3 次拒絕或滑動視窗 50 次中 10 次拒絕 → 中止回合**。這是純 deterministic、寫死在 harness 裡的 |
| **Audit / telemetry** | Codex 匯出 OTel；OTel GenAI 慣例已定；**OWASP MCP Top 10 第 8 項就是「Lack of Audit & Telemetry: No forensic trail of what agents did, with which tools, and why」** |
| **Identity / registry** | EACP 與 agentic control plane 的第一級構件 |
| **Policy enforcement at runtime** | 「runtime policies and guardrails block non-compliant actions **at the point of execution rather than through paperwork**」 |
| **Durable state** | Temporal / DBOS 已商品化，但「用哪個 store、記什麼、如何 replay」仍是產品決策 |
| **Evidence records** | 「**AI outputs and agent actions must produce evidence records that can be reconstructed after the fact**」（AGL-1 設計原則 4） |
| **Determinism of the durable core** | Temporal：workflow code 必須 deterministic，model call 必須放在 Activity 裡 |

### 3.3 Agent Core 是否會縮小？

**明確答案：會，但只在「決策」這個維度上縮；在「邊界」這個維度上會長大。**

```
2024 年的假設（ai-agent-v3 的原始架構）：

LLM（弱）
  ↓
大型 Agent Core
  ├─ planner
  ├─ hypothesis engine
  ├─ loop detector
  ├─ decision engine
  ├─ strategy engine
  └─ recovery engine      ← 全部用 deterministic code 補償弱 LLM
  ↓
Runtime

2026/2030 的實際演化：

LLM（強）
  ↓
Thin, replaceable agent runner
  ├─ session / context management
  └─ tool loop
  ↓
═══════════════════════════════════
  DETERMINISTIC INFRASTRUCTURE  ← 這一層不縮小，反而長大
  ├─ sandbox + capability layer
  ├─ permission / approval / circuit breaker
  ├─ durable execution + replay
  ├─ evidence ledger + provenance
  ├─ identity + registry + policy
  └─ audit export (OTel / AG-UI)
═══════════════════════════════════
  ↓
Environment
```

**關鍵洞察**：`ai-agent-v3` 目前的架構是**一個 deterministic control plane 在上、LLM 在下**。而產業正在走向**LLM 在上、deterministic control plane 在下、environment 在最下**。

這個翻轉意味著：`ai-agent-v3` 的 `AgentLoop` 所佔據的位置（決策迴圈），在未來會被 LLM 佔據；而未來會變重要的位置（sandbox、permission、evidence、audit），在 `ai-agent-v3` 裡幾乎不存在。

### 3.4 更大的圖景：變的是「責任邊界」，不是「誰比較聰明」

`ai-agent-v3` 的 `AGENTS.md` 有一條原則：

> 「Agent Core 決定什麼應該發生。Runtime 提供 capabilities for performing computer operations.」
> 「Runtime 必須提供 general capability contracts rather than hard-coded workflows.」
> 「Do not add dependencies without a concrete reason.」
> 「Prefer standard Rust patterns and well-maintained crates.」
> 「Do not create a persistence abstraction merely to satisfy a test if it cannot support real recovery semantics.」

這五條原則的**精神**是正確的，而且比 2024 年任何單一 agent 專案都更清晰。問題不在原則，在於**同一批原則在 2026 年導出了不同的結論**：

- 「Runtime = general capability contract」→ 2026 年的結論是 capability contract 應該是**sandbox + policy provider**，不是 hand-rolled Windows abstraction
- 「Prefer well-maintained crates」→ 2026 年的結論是 sandbox、durable execution、OTel 都已有成熟 crate 或服務，**不該重做**
- 「不要為了讓架構看起來完整而加抽象」→ 2026 年的結論是 `SemanticUpdate`、`KnowledgeClaim` 這類詞彙層**就是**這種抽象

---

## 第四階段：LLM 不應該取代的部分

### 4.1 為什麼這些不能交給 LLM：結構性理由

**根本理由：LLM 是一個 claim generator，不是一個 fact recorder。兩者的 epistemic class 不同。**

一個控制面**無法**驗證自己：如果你用 X 去驗證 X，得到的不是證據，是一致的錯誤。
更細緻地說：LLM 的輸出（包含 confidence、包含 reasoning、包含「我確認過了」）與它所聲稱的主張（環境實際狀態）**屬於同一個不可信來源**。LLM 可以說「測試通過了」，但「測試通過了」這個句子和「環境是這樣的」這個句子，來自同一個過程。**它無法自我認證。**

這與 LLM 聰不聰明無關。這是結構問題，不是能力問題。

### 4.2 逐項分析

| 職責 | 為什麼不能完全交給 LLM | 如果 LLM 可以判斷，deterministic infrastructure 還需要做什麼 |
|---|---|---|
| **Security / Permission** | 這是**授權**問題，不是**判斷**問題。LLM 判斷「這個指令看起來安全」不等於使用者授權。授權必須預先建立，且不可被對話推翻 | 1) 預先宣告的 capability set（不可被 LLM 擴張）2) 不可繞過的執行點強制執行 3) 每次越界的**記錄** 4) 使用者可稽核、可撤銷的授權清單 |
| **Sandbox** | Agent 產生的程式碼是 untrusted code。「在自己的 runtime 執行模型輸出是 remote code execution 加上幾個步驟」 | 1) microVM / namespace 隔離 2) **secret 不進 sandbox**（credential brokering 在 egress proxy 注入）3) egress default-deny + deny 優先於 allow 4) read-only mount 5) 明確的 lifetime |
| **Resource limits** | 成本與資源不是判斷問題，是**邊界**問題。「這個任務值得 3 美元嗎」不該由會為你燒錢的元件回答 | 1) 硬性上限 2) 超限即中止 3) 中止是**可觀察事實**而非錯誤 |
| **Persistence / Crash recovery** | 需要的是**確實執行過的步驟**，不是「我認為我執行過」 | 1) 已完成步驟的 durable record 2) replay 必須能還原 3) 未完成步驟的重跑語義（至少一次 vs 至多一次）4) 「我必須容忍 process-local state 消失」 |
| **Evidence recording** | 記錄必須在事件發生**當下**完成，LLM 是在事後敘述 | 1) append-only 事件流 2) 每個事件綁定 actor identity 3) 關聯 ID 串起因果鏈 4) 不可竄改的 chain of custody |
| **Provenance** | 「這個結論從哪來」是**查詢**問題，不是推理問題。LLM 可以猜，但猜的結果不是 provenance | 1) claim → evidence 鏈必須是**機械可檢查**的 2) 每條 link 必須指向存在的 record 3) dangling reference 必須是硬錯誤 |
| **Audit trail** | OWASP MCP08 把它列為 top-10 缺口。CSA 報告：**僅 38% 組織端到端監控 AI traffic，僅 17% 監控 agent-to-agent 互動**。SIEM/DLP 只能抓到最終 API 呼叫，**抓不到產生它的決策鏈** | 1) 與框架無關的記錄格式（OTel / AG-UI）2) 包含 tool call 而不只是 tool name 3) 包含 policy decision record 4) 可匯出成 audit bundle |
| **State consistency / Atomicity** | 這是資料庫問題。LLM 不參與 | 1) transaction 邊界 2) 失敗時完全不變（clone-validate-commit）3) 可測試的不變量 |
| **Artifact storage** | 大檔案不該塞進 event history。Temporal 的 state placement table 是通用答案 | 1) 內容-addressed store 2) event 中只放 reference + hash 3) 內容不可變 |
| **Environment access** | 「agent 能看到什麼」必須先決定，不能讓 agent 決定 | 1) 宣告式可見範圍 2) 記錄每次讀取 3) 跨邊界的傳輸需授權 |
| **Process isolation** | 同 sandbox | 同上 |
| **Checkpoint / Rollback** | rollback 是**破壞性操作**，需要誰授權、誰執行、如何驗證還原正確 | 1) checkpoint 必須包含足以還原的證據引用 2) rollback 本身是一個可稽核事件 3) 還原後的完整性驗證 |
| **Goal verification** | 「使用者要的東西真的拿到了嗎」需要**獨立於產生它的那個過程**的檢查 | 1) 目標條件的形式化 2) 每個條件綁定到 evidence record 而非 agent 陳述 3) 證據不足時結果是 **Uncertain**，不是 Verified 4) 矛盾證據必須被偵測為矛盾 |

### 4.3 一個必須誠實承認的弱點

`ai-agent-v3` 目前在這一整列中只真正做到了三件事：

1. **Atomicity**（clone-validate-commit）— 這是真的做好了
2. **Provenance 的資料模型**（`EvidenceLink` 必須指向存在的 observation，dangling 是硬錯誤）— 設計正確且有測試
3. **Unknown 是一等狀態**（`Unknown` 物件與「absence of evidence」語意區分）— 這個決定在 2026 年極為罕見且正確

其餘全部未實作。而最關鍵的「**evidence vs assertion**」分離，repo 自己在 `agent_loop.rs:350-353` 破了例。

---

## 第五階段：重新評估 ai-agent-v3

| Existing Component | KEEP | REPLACE | DELETE | NEW ROLE | Reason |
|---|:--:|:--:|:--:|---|---|
| **ObservationStore** | ✅ | | | **Evidence sink**：所有外部可觀察事實的唯一權威去處。成為 ledger 的證據來源 | 核心不變量正確。差別是它今天只存「環境觀察」，未來要存**所有**可稽核事件（含 policy decision、approval、artifact） |
| **KnowledgeStore** | ✅ | | | **Derived semantic state**：事實上的 evidence index | `claim → evidence` 索引在治理需求中仍是第一級需求（provenance management） |
| **EvidenceLink** | ✅ | | | **Provenance edge**：work receipt 的核心 | dangling reference 是硬錯誤，這個決定是 repo 最有價值的部分之一 |
| **ContextCompiler** | ⚠️ 保留介面 | | | **Bounded projection**：只負責「給定 evidence 視窗，產生視圖」 | 選擇策略會商品化，但「有界投影 vs 權威歷史」的分離是正確的不變量 |
| **SemanticUpdate** | | | ✅ | — | **見下方詳述**。這是**框架對 LLM 推理施加的分類法**，而 prompt engineering 有半衰期。In-band decision+evidence 嚴格更一般 |
| **ProviderRequest** | ⚠️ 保留概念 | | | **Canonical, provider-neutral work-state representation** | 從「給 provider 的 prompt」變成「可持久化、可 handoff、可稽核的狀態描述」 |
| **AgentLoop** | | ✅ | | **Thin, replaceable agent runner** | 今天是控制面。應降級為：session 管理 + tool loop + durable checkpoint 觸發點。決策權歸 LLM |
| **Runtime** | | ✅ | | **Capability / sandbox provider** | 手寫 Windows abstraction 應讓位給 sandbox + policy provider。agent 產生的程式碼是 untrusted code |
| **Checkpoint** | ⚠️ 保留概念 | ✅ | | **Durable execution substrate** | JSON checkpoint 應讓位給 durable execution。設計原則（最簡單滿足需求）保留，實作替換 |
| **Verification** | ✅ 概念 / ❌ 實作 | | | **Evidence-based verification** | `docs/01` §35 的三層驗證概念是對的。但 `verify_goal()` 只被測試呼叫，provider-driven agent 結構上無法達成 Done。**repo 最 aspirational 的元件是最不 functional 的元件** |
| **AgentDecision** | ⚠️ 保留概念 | | | **Decision + in-band evidence** | variant 可以簡化，但必須能承載 evidence。決策與證據分開傳遞在未來是退步 |
| **Hypothesis** | | | ✅ | — | 生命週期管理是為了補償弱推理。LLM 強了，「保留多個假設並測試」是 prompt 層面的事，不是框架類別。`KnowledgeClaimStatus::Hypothesis` 保留即可 |
| **Loop Detection** | | | ✅ | — | LLM 自己能看出重複。Codex 用的是 3-denials 這種 deterministic circuit breaker，不是語意 loop detector。repo 應支援「circuit breaker」概念而非「loop detector」概念 |
| **Recovery** | | | ✅ | — | 強行留在 repo 會把 product thesis 拉回 2024。「failure 可以被 durable state 與 evidence 重建」才是正確的基礎設施答案 |
| **EventTrace** | | ✅ | | **OTel / AG-UI event export** | 幾乎必然變成 OTel adapter。自建 schema 沒有護城河 |
| **Interpreter** | | ✅ | | **Capability/sandbox boundary** | 從「解析模型輸出」變成「執行前的能力與權限檢查」 |
| **Testing discipline** | ✅ | | | — | 199 tests、deterministic、failure/atomicity 有實證。**這是 repo 最被低估的資產**：它示範了如何用測試證明架構不變量。新產品沿用同樣紀律 |
| **Docs governance** | ✅ | | | — | stable design / current state / history / temporary 四層分離，與 `AGENTS.md` 一致。治理能力可攜 |

### 5.1 為什麼 `SemanticUpdate` 建議刪除（本研究最艱難的判斷）

`SemanticUpdate` 存在的理由是：框架要求 LLM（或一個 producer）**另外**產出一個語意事件，好讓 KnowledgeStore 有一個確定性的寫入點。

這個設計的隱含假設是「LLM 的決策輸出」和「LLM 的語意產出」需要被分開處理。這個假設在 2026 年不成立：

- 模型足夠強時，它本來就在做推理。框架再要求它填一個框架自訂的三選一枚舉，只是在**已經有推理的地方**強迫它做一次分類。
- 分類法是耦合點。模型能力提升時，分類法不會自動升級，只會變成需要維護的過時契約。
- 有更一般的做法：**in-band**——決策與其證據一起回傳，框架不需要預先定義語意詞彙。
- 這並不牴觸 provenance。`ai-agent-v3` 真正有價值的是「evidence 必須指向存在的 record，dangling 是硬錯誤」這個**強制力**，而不是「evidence 必須是框架定義的三種形狀之一」這個**分類**。

換言之：**保留強制力，刪除分類法。**

---

## 第六階段：產品方向探索

### 候選 A — Agent Work Ledger（no-claim 端）

| 問題 | 答案 |
|---|---|
| **User 是誰？** | 同時管理多種 agent 的工程/IT 團隊與受監管產業。痛點是：agent 變多，但沒有人能回答「它昨晚到底改了什麼、為什麼」 |
| **痛點** | CSA 報告：僅 38% 端到端監控 AI traffic、17% 監控 agent-to-agent；64% 營收 >$1B 的組織在 2025 年因 AI 系統失效損失 >$1M；80% 記錄過風險行為。SIEM 抓得到最終 API 呼叫，抓不到決策鏈 |
| **為何現有 agent 沒解決** | 它們各自有私有 trace：Codex 匯出 OTel、Claude Code 有 hooks 與 session log、OpenCode 有自己的 session state。**每個都有，但沒有任何一個是跨 vendor 的完整 ledger** |
| **Moat** | 薄且可被商品化。OTel GenAI 慣例 + AG-UI 事件詞彙幾乎免費提供了格式。**如果只是「把 OTel 存起來」，幾乎確定會被吸收進既有平台** |
| **LLM 變強後** | 不變重要——反而更相關 |
| **2030 是否存在** | 作為獨立產品**存疑**，作為其他產品的元件**幾乎必然存在** |
| **可重用程式碼** | ~25%（`EventTrace`、`Action`、`Observation` 的資料模型）。**是 repo 中重用率最低的方向** |
| **MVP** | OTel collector + 事件存儲 + ledger view |
| **最容易失敗的地方** | 競爭者就是 OpenTelemetry 本身；已經有 `useHello.ai` 等商業玩家 |

### 候選 B — MCP Gateway / Capability Enforcement Point

| 問題 | 答案 |
|---|---|
| **User 是誰？** | 企業平台/安全團隊 |
| **痛點** | MCP server 權限過寬、無稽核記錄。Endor Labs 調查：82% 的 2,614 個 server 使用 traversal-prone file ops；36.7% 的 7,000+ server 可能 SSRF |
| **為何現有 agent 沒解決** | 每個 agent 都有自己的 permission 系統（Codex sandbox、Claude hooks、OpenCode ruleset、Gemini plan mode），彼此不互通 |
| **Moat** | 政策引擎、稽核整合、企業部署。**但 market 已明確，且是平台競賽** |
| **LLM 變強後** | 不變重要（能力邊界與模型無關） |
| **2030 是否存在** | 作為**類別**幾乎確定存在 |
| **可重用程式碼** | **~5%**。`ProcessPolicy` 是一個起點，但離可用產品極遠 |
| **MVP** | stdio/HTTP MCP proxy + policy engine + audit log + OAuth |
| **最容易失敗的地方** | 與 MCP gateway 廠商正面競爭；需要企業憑證與部署能力，這是純工程問題，沒有 research moat |

### 候選 C — Agent Work Receipt（合併 A+B+C+E）

| 問題 | 答案 |
|---|---|
| **User 是誰？** | 受監管產業中，需要向稽核者證明 agent 行為的機構：金融、醫療、保險。**以及**需要跨 agent 交接一致性的小團隊 |
| **痛點** | 「Who authorized the work? What context did the agent use? What was it allowed to do? Which model acted? Where did human judgment enter? What changed? What proof remains?」——機構回答不了這些問題，就只有 activity，沒有 governed work |
| **為何現有 agent 沒解決** | 每個 agent 都產出**私有** trace。OTel 統一了遙測，但**不統一「什麼是 evidence」**。且沒有任何 agent 提供**簽章的、per-action 的證據封包** |
| **Moat** | 1) 跨 vendor protocol adapter（Claude hooks / Codex OTel / A2A artifacts / MCP tool calls / AG-UI events）2) `claim → evidence` 強制力與 dangling-detection 3) **政策版本與事件綁定**（「policy version locked to the event」）4) 留存與稽核匯出 |
| **LLM 變強後** | **更相關**。模型越強、動作越多、越快，機構越需要證明「做了什麼」而非「說做了什麼」 |
| **2030 是否存在** | 幾乎確定。監管壓力（EU AI Act、NIST Q4 2026 interoperability profile、42 CFR Part 2 已於 2026-02 開始執法）只增不減 |
| **可重用程式碼** | **~55%**（`KnowledgeStore`、`EvidenceLink`、`Atomicity`、`Unknown` 語意、`ValidationError` 模式、`EventTrace` 資料模型、測試紀律） |
| **MVP** | 1 個真實 protocol adapter（先做 Claude Code hooks，因為它事件最豐富且可程式化）→ canonical event envelope → immutable ledger → `action → claim → evidence` chain → 匯出簽章封包 |
| **最容易失敗的地方** | 定位錯。必須是「**any** agent 的證據層」，一旦變成「另一個 agent」就掉進最擁擠的市場。以及不要變成 billing 系統（那會捲入另一場戰爭） |

### 候選 D — Agent-independent Project State（純 E）

| 問題 | 答案 |
|---|---|
| **User 是誰？** | 在多個 agent 之間切換的開發者 |
| **痛點** | Codex 做了 40%，你開 Claude Code 做了 60%，agent 不記得前 40% |
| **為何現有 agent 沒解決** | 這是**廠商刻意不解決的**。A2A 的核心設計原則是「**Opaque Execution**」——agent 刻意不共享內部 state、memory、tools。所以「共享 work state」是 opacity 的反面 |
| **Moat** | **幾乎沒有**。Git + Markdown 已經解決大部分 |
| **2030 是否存在** | 作為產品**存疑**。AG-UI issue #2091 至今仍是一個開放問題，問的就是「共享 state 該如何在 agent 邊界傳遞」——產業自己也沒答案 |
| **可重用程式碼** | ~20% |
| **最容易失敗的地方** | 需求是隱性的、pain 是真的但不急、且完全被開源方案侵蝕 |

### 候選 E — Rust Local Agent（原始目標的現代版）

| 問題 | 答案 |
|---|---|
| **為何現有 agent 沒解決** | Claude Code / Codex / OpenCode / Gemini CLI 都**不是 Rust 寫的**；Vercel Sandbox 顯示 agent 已經是 sandbox 裡的 commodity |
| **Moat** | 「用 Rust 寫」不是護城河。Goose、Hermes、openhands 都是開源的 |
| **2030 是否存在** | 存在，但**沒有商業理由**讓你做第 15 個 |
| **可重用程式碼** | 100%（這是它唯一的優點） |
| **最容易失敗的地方** | 就是它自己 |

---

## 第七階段：反證研究

### 7.1 「為什麼三大廠自己做掉它？」

| 方向 | 檢驗 | 結果 |
|---|---|---|
| Work Ledger | 三大廠已有 OTel 匯出、hooks、session logs | **部分致命**。但它們只做自己平台的。做跨 vendor ledger 需要承擔「不站在任何一方的競爭者」角色——這是三大廠**結構上不會做**的事 |
| MCP Gateway | 三大廠都會做 | **致命**。這是他們的天然地盤 |
| Work Receipt | 三大廠目前**沒有**做（無跨 vendor 證據層） | **存活**。而且正因如此，現場已有商業玩家（useHello.ai、Mercury Nexus）搶佔 → 意味著**方向對，但空窗正在關閉** |
| Agent-independent State | 三大廠**刻意不解決** | **反證成立**。A2A 的 opacity 是設計選擇，不是遺漏 |
| Rust Local Agent | 三大廠可能做 Codex（已用 Rust） | **致命** |

### 7.2 「為什麼 GitHub / Microsoft 不會順手整合？」

- **Work Ledger**：GitHub 已有 Copilot 的 OTel；Microsoft 已有 Entra agent identity 與 Purview。**兩者都有能力做。** 但它們的激勵是賣 seat，不是賣 evidence layer。純 evidence vendor 仍有機會。
- **Work Receipt**：合規價值遠高於開發者價值，與 GitHub 的產品定位錯位。Zafin 的定位是「banking discipline applied to agentic work」——這恰恰說明賣家來自**受監管產業內部**，不是工具產業。

### 7.3 「為什麼使用者不直接用 Git + Markdown？」

這是 Work Ledger 與 Work State 的**致命反證**，但對 Work Receipt **不成立**：

- Git + Markdown 記錄的是**產出**（code、doc），不是**決策鏈**。它無法回答「這個改動是根據哪次工具輸出、哪次 approval、哪個 policy 版本做出的」。
- Git + Markdown 無法回答「誰授權了這次執行」。
- 在受監管情境，「能展示結果」不等於「能證明過程」。

### 7.4 「這不能只是 Agent Feature 嗎？」

對 Work Ledger：**幾乎確定會變成 feature**。這是它最大的弱點。
對 Work Receipt：**不能**，因為它必須站在 agent 之外。它是後設層（meta-layer），不是 agent 內部功能。這正是它的設計約束，也正是它的護城河——它必須做到「agent 不知道自己的存在」，這與 `ai-agent-v3` 目前把 KnowledgeStore 放在 AgentLoop 內部的做法相反。

### 7.5 「如果 LLM 三年內強 10 倍，這個產品會消失嗎？」

| 方向 | 答案 | 理由 |
|---|---|---|
| Work Ledger | 會變弱 | agent 自己就有完美記憶與稽核能力 |
| Work Receipt | **不會消失，會變更相關** | 模型能力不是證明問題。證明問題是**跨信任域問題**：使用者不完全信任模型，也不可能完全驗證模型。CSI 顯示 10,000+ MCP server 中大量存在 command injection 與 SSRF——這不是模型能力問題 |
| Local Agent | 會消失 | 模型就是 agent |

### 7.6 淘汰結果

- **候選 E（Rust Local Agent）** — 淘汰
- **候選 A（純 Work Ledger）** — 淘汰為獨立產品，降級為 Work Receipt 的元件
- **候選 B（純 MCP Gateway）** — 淘汰為獨立產品，降級為 Work Receipt 的元件（policy 執行點）
- **候選 D（純 Agent-independent State）** — 淘汰
- **候選 C（Agent Work Receipt）** — **唯一存活**

---

## 第八階段：產品建議

### 建議方案：Agent Work Receipt

**一句話定義**：**任何 agent 的 deterministic evidence layer。一個位於 agent 之外、與 agent 框架無關、記錄並簽章每個動作之「做了什麼、依據什麼、被誰允許」的系統。**

不是「更好的 agent」。是 **no-claim 端**。

#### Target user
第一階段：受監管產業（金融、醫療、保險）中，必須向內部稽核、外部監管者、保險方證明 agent 行為的機構。
第二階段：需要跨 agent 交接一致性與可重現性的工程團隊。

#### Problem
機構回答不了「誰授權了這份工作？agent 用了什麼 context？它被允許做什麼？哪個模型執行？人類判斷在哪裡介入？什麼改變了？留下什麼證據？」
在這種狀態下，**它擁有的不是 governed work，只是 activity。**

#### Why now
1. **Protocol 已就位**：OTel GenAI 慣例、AG-UI 事件詞彙、A2A artifacts、MCP tool annotations、Claude Code hooks——2024 年時這些都不存在
2. **Protocol 已開放**：MCP 歸 Linux Foundation，跨 vendor 成為可能
3. **監管已逼近**：NIST AI Agent Standards Initiative（2026-02）、interoperability profile 預計 2026 Q4、42 CFR Part 2 已於 2026-02 執行
4. **傷害已發生**：2025 年 64% 大型組織報告 >$1M AI 相關損失
5. **生態系自己承認**：OWASP MCP Top 10 第 8 項就是 audit/telemetry 缺口

#### Why existing agents don't completely solve it
每個 agent 都有遙測，但：
- 遙測是**平台特定**的（Codex OTel ≠ Claude hooks ≠ OpenCode session）
- 遙測記錄**觀察到的事件**，不是**可主張的證據**
- 沒有任何 agent 產出**跨框架的、簽章的、per-action 證據封包**
- **關鍵缺口**：`ai-agent-v3` 讓我們看到「第一方 agent 可以靠 evidence 基礎設施作弊」。第三方 layer 不能。

#### Why LLM improvement helps rather than destroys it
- 模型越強 → 能做的動作越多、越快、越自主 → **需要的證明越多**，不是越少
- 模型越強 → 越難從 output 表面區分「真的做了」和「宣稱做了」→ **獨立 evidence layer 越關鍵**
- 模型的自我陳述永遠無法認證自己，這是結構性限制，不隨能力改變

#### Core architecture
```
┌──────────────────────────────────────────────────────────────┐
│  ANY AGENT  (Claude Code / Codex / OpenCode / Gemini / …)   │
│  不知道、也不應知道本層存在                                 │
└───────────────────────┬──────────────────────────────────────┘
                        │ protocol adapters
        ┌───────────────┼───────────────┬──────────────┐
        ▼               ▼               ▼              ▼
   Claude hooks    Codex OTel      A2A artifacts   MCP tool calls
        └───────────────┴───────────────┴──────────────┘
                        ▼
┌──────────────────────────────────────────────────────────────┐
│  CANONICAL EVENT ENVELOPE                                   │
│  action_id · actor identity · timestamp · correlation id      │
│  · policy version · decision · inputs · outputs · artifacts   │
└───────────────────────┬──────────────────────────────────────┘
                        ▼
┌──────────────────────────────────────────────────────────────┐
│  EVIDENCE LEDGER  (append-only, immutable chain of custody)  │
│    ├─ Policy engine  (rule packs; verdict attached BEFORE exit)│
│    ├─ Idempotency / duplicate suppression                     │
│    └─ System-of-record write confirmation                     │
└───────────────────────┬──────────────────────────────────────┘
                        ▼
┌──────────────────────────────────────────────────────────────┐
│  VERIFICATION LAYER  ←── ai-agent-v3 的核心天賦在這裡          │
│    claim → evidence links; dangling = hard error              │
│    atomic mutation; Unknown ≠ absence                        │
│    goal condition satisfied ⟺ evidence-backed                │
└───────────────────────┬──────────────────────────────────────┘
                        ▼
        AUDIT BUNDLE / REGULATORY EXPORT / PROOF OF WORK
```

#### MVP（建議順序，不授權實作）
1. **一個 protocol adapter**：Claude Code hooks（事件最豐富、可程式化、有 PreToolUse/PostToolUse/PermissionDenied/SubagentStart 等 30+ 事件）
2. **Canonical event envelope** + 不可竄改 ledger
3. **`action → claim → evidence` chain**，含 dangling detection
4. **稽核匯出**：回答「列出 2026-09-01 至今所有未經 approval 的寫入動作」

#### Reuse from ai-agent-v3
| 現有元件 | 新用途 | 重用量 |
|---|---|---|
| `KnowledgeStore` + `EvidenceLink` | Verification layer 的核心 | **高** |
| clone-validate-commit atomicity | Ledger 寫入語意 | **高** |
| `Unknown` / absence-of-evidence 語意 | 證據不足 → Uncertain，而非 Verified | **高** |
| `EventTrace` | Canonical envelope 的雛形 | **中** |
| `ValidationError` 模式 | 資料契約嚴格性 | **中** |
| 199-test 測試紀律 | 新產品的品質基準 | **高（方法論）** |
| Docs governance | 沿用 | **高（方法論）** |
| `AgentLoop` / `Runtime` / `SemanticUpdate` | **不需要** | 0% |

估計重用率 **50–60%**——這是五個候選中最高的。

#### New code required
- Protocol adapters（全新）
- 具名 actor identity 與憑證管理（全新）
- Append-only storage 與 retention policy（全新）
- Policy engine 與 rule packs（全新）
- 稽核匯出與簽章（全新）
- 多租戶隔離（全新）

#### Major risks
1. **平台吸收風險**：Codex / Claude Code 若推出跨平台 evidence 層，此產品被併入
2. **商業空窗關閉**：useHello.ai、Mercury Nexus 等已在做；窗口可能只有 12–18 個月
3. **冷啟動困難**：使用者要換掉現有習慣才能獲得價值
4. **過早商品化風險**：核心可能退化成 OTel 轉發器，失去差異化
5. **Rust 生態邊緣化**：evidence layer 未來很可能需要在 server 端；repo 的 Rust 選擇變成成本而非資產
6. **合規要求變動**：42 CFR Part 2 類規範的實際要求可能與預期不同

#### Potential defensibility
- **跨 vendor protocol adapter 的累積深度**（每加一個 adapter，替換成本上升）
- **制度性嵌入**：一旦成為稽核記錄，轉換成本極高（等同更換記錄保存系統）
- **不站在任何 LLM 廠商一方**——這在 2030 年會比今天更有價值
- **與 policy engine 的耦合**：policy 版本與事件綁定，創造真實的資料資產

---

## 9. 兩個直接問題

### 9.1 「如果今天完全沒有這個 Repository，從零開始，你會建議做哪一種產品？」

**我會做 Agent Work Receipt，但起點會和 `ai-agent-v3` 不一樣：**

- 從**一個真實客戶的稽核需求**開始，不是從架構開始。找一個受監管機構，問他們「上次 agent 出事，你怎麼回答稽核員？」他們的答案就是規格。
- 第一個 adapter 會是**他們正在用的 agent**，不是「最通用的」。
- 會先做**匯出**（回答稽核問題的 read-only view），再做**攔截**。攔截會被 agent 開發者視為敵人；匯出不會。
- 技術上會用 TypeScript 或 Go 而非 Rust，因為 server 端生態更成熟。

**我不會做的事**：不會先建 Agent Core，不會先定義語意本體，不會先做 reasoning 抽象層。

**原因**：本研究的核心結論是——決策層正在商品化，邊界層沒有。把工程投入放在商品化中的部分，是最大的資源錯配。

### 9.2 「如果保留 ai-agent-v3，應該把它改造成什麼？」

**改造成 Agent Work Receipt 的 Verification Layer 與 evidence model。**

具體地，它應該從「一個 autonomous AI agent」重新定義為：

> **「一個 deterministic evidence substrate：給定一串不可信來源產生的事件，確定性地判斷哪些主張被哪些證據支持、哪些沒有。」**

這個重新定義意味著：

**保留（構成產品核心）**
- `KnowledgeStore` + `EvidenceLink`：provenance 資料模型
- clone-validate-commit 原子性
- `Unknown` 與 absence-of-evidence 的語意區別
- 嚴格 `ValidationError` 契約
- 199-test 測試紀律（證明架構不變量的能力）
- 文件治理方法論

**降級（變成 adapter 或範例）**
- `AgentLoop` → 一個參考 harness / 測試 fixture
- `EventTrace` → OTel export adapter
- `Checkpoint` → durable execution substrate adapter
- `FilesystemRuntime` / `ProcessRuntime` → sandbox provider 的一個實作
- `ProviderDecisionSource` / `OllamaProvider` → 一個 plugin producer

**刪除**
- `SemanticUpdate`（框架分類法，見 §5.1）
- `AgentState` 中無生產者的語意欄位（`hypotheses` / `unknowns` / `evidence` / `environment_state` / `running_jobs` / `remaining_work` / `active_problems`）
- `KnowledgeState` enum（dead type）
- `ObservationKind::EnvironmentDelta`（從未使用）
- roadmap C–G 的全部內容（hypothesis lifecycle / progress detection / loop detection / adaptive recovery / stronger verification 作為**框架職責**的定位）

**必須先修正（不是 scope，是正確性）**
- `agent_loop.rs:350-353` 的偽造 evidence 路徑。若繼續做 evidence 產品，這個 shortcut 是致命的，因為它證明 repo 會作弊。
- `verify_goal()` 從未被 production 呼叫、`Finish(Done)` 不可達。若 verification 升格為產品價值主張，這個結構性缺陷必須被明確處理（而不是靠外部呼叫）。
- `docs/06_IMPLEMENTATION_GAP_ANALYSIS.md` 已過期（仍宣稱 Observation Store / Context Compiler / Evidence lifecycle 為 Missing）。

**一句話總結**：
> `ai-agent-v3` 的**架構直覺**值得保留；`ai-agent-v3` 的**產品身分**應該被放棄。
> 它的問題從來不是架構錯了，而是它假設了一個「LLM 永遠不夠強」的世界。在那個世界裡它很出色。在這個世界裡，它的直覺剛好指向一個更值錢的問題。

---

## 10. 本研究明確不做的事

- 未建立任何 implementation increment
- 未建立新的 Stage
- 未定義 B2-D
- 未修改 production code
- 未修改 `docs/01~11`、`README.md`、`AGENTS.md`
- 未 commit、未 push
- 未因發現 gap 就開始補 code
- 未為延續舊專案而強行支持原始設計
- 未假設 roadmap 上不存在的 increment 為已批准

## 11. 研究誠信聲明

本報告的判斷基於：

- **Repository 實際程式碼與測試**（非文件敘述）
- 2026 年公開的官方文件、官方 repository 與技術資料
- 明確標示了推翻既有 repo 設計的結論（`SemanticUpdate` 應刪除、AgentLoop 應降級、roadmap C–G 作為框架職責不成立）

本報告承認的已知不確定性：

- §7 的反證是基於公開資料的推理，未做實際客戶訪談
- §6 候選 C 的商業可行性未經驗證；現場已有商業玩家，時間窗口未知
- 「三大廠不做跨 vendor evidence layer」是結構性論證，不是路線圖資訊
- 本報告**沒有**證明 Agent Work Receipt 一定會成功；它只證明了其他四個候選在現有證據下不成立
