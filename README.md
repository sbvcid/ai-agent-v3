# AI Agent v3

> ## ⚠️ ARCHIVED — 開發已於 2026-09-28 終止
>
> **本專案不再維護，也不接受以原始產品方向為目的的新功能開發。**
> Issues 與 PR 不會被處理。程式碼以 MIT License 釋出，可自由取用。
>
> **停止原因不是技術不可行，而是原始產品假設已失效。**
> 完整研究與判斷依據見 [`docs/00_PRODUCT_REDEFINITION_RESEARCH.md`](docs/00_PRODUCT_REDEFINITION_RESEARCH.md)。

---

## 為什麼停止

本專案最初的目標是建立一個具備 **deterministic decision / planning / knowledge / execution loop** 的 Agent Core，用來補足當時 LLM 在自主決策與長期工作上的不足。

在重新研究 2026 年 Agent 生態（OpenAI Codex、Anthropic Claude Code、Gemini CLI、OpenCode、MCP、A2A、Agent Skills、sandbox 與 durable execution 生態）之後，確認這個問題正在被通用 Agent、模型與官方 runtime 快速解決。因此，繼續把本專案發展成「另一個通用 Agent」並沒有足夠理由。

三個最具決定性的外部觀察：

1. **Anthropic 官方文件**明確區分 deterministic 與 interpretive 邊界 —— 寫在 `CLAUDE.md` 或 skill 裡的禁令「is a request, not a guarantee」，而 `PreToolUse` hook 的阻擋「is enforcement」。產業的共識是 harness 應該 thin、safe、replaceable，決策交給模型。
2. **SWE-bench Verified** 這個業界評測基準，本身就是用「bare ReAct loop、無特殊 scaffold」來評估所有模型的。
3. **OpenAI Codex 自己實作了一個 deterministic rejection circuit breaker**（連續 3 次拒絕即中止回合），並在文件中寫明其 LLM reviewer「is not a deterministic security guarantee」。

也就是說：**Agent Core 只會在「決策」這個維度縮小，在「邊界」這個維度變厚。** 而本專案把架構建在會縮小的維度上。

## 這個 repo 對你有什么用

研究過程確認了幾個架構概念仍然有價值，它們被完整保留下來：

- deterministic execution boundary
- observation / evidence separation
- provenance（`EvidenceLink` 必須指向存在的 record，dangling 是硬錯誤）
- atomic state mutation（clone-validate-commit）
- checkpoint / recovery
- bounded runtime
- verification 與 evidence 的分離

但這些**不足以證明應該繼續維護本專案**。所以本 repo 定位為：

1. **架構研究紀錄** —— 一個有紀律的系統如何設計、實作、誠實驗證，然後在證據顯示前提失效時停止
2. **Prototype** —— 178 個 deterministic tests，其中 11 個專門驗證 failure / atomicity 邊界
3. **未來新專案的參考** —— MIT license，可自由取用
4. **研究與實驗結果的展示** —— 包含推翻自身原始設計的結論

### 建議閱讀順序

| 你想知道的 | 讀這份 |
|---|---|
| 為什麼停止、架構該怎麼重新評估 | [`docs/00_PRODUCT_REDEFINITION_RESEARCH.md`](docs/00_PRODUCT_REDEFINITION_RESEARCH.md) |
| 實際做到什麼程度（含未完成部分） | [`IMPLEMENTATION_STATUS.md`](IMPLEMENTATION_STATUS.md) |
| 原始目標與架構（歷史紀錄） | 本文下方 + [`docs/01_REQUIREMENTS.md`](docs/01_REQUIREMENTS.md) |
| 設計原則與約束 | [`AGENTS.md`](AGENTS.md) / [`DEVELOPMENT_WORKFLOW.md`](DEVELOPMENT_WORKFLOW.md) |
| 完整文件導航 | [`docs/README.md`](docs/README.md) |
| 設計如何被推翻（Roadmap 走到底會怎樣） | [`docs/08_AGENT_CORE_ROADMAP.md`](docs/08_AGENT_CORE_ROADMAP.md) |

> 註：`docs/06_IMPLEMENTATION_GAP_ANALYSIS.md` 與 `docs/08_AGENT_CORE_ROADMAP.md` 已被標記為 **superseded / not-taken**，內容保留作為歷史紀錄，不要當作現況。

## 實際完成並經測試驗證的部分

截至停止時的 checkpoint，實際通過驗證的是：

```text
178 tests passed (+ 3 ignored)
  121 unit
   22 closed-loop integration   (含 B2-C3 端到端 4 個、B2-C4 failure/atomicity 7 個)
   11 filesystem / process integration
   10 process execution
    6 schema checkpoint
    6 test doubles
    2 ollama provider
    3 ollama real integration (ignored — 需要執行中的 Ollama)
```

已實作並驗證的核心元件：

- Canonical domain model 與驗證不變量
- `ObservationStore`（authoritative observation 歷史）
- `KnowledgeStore`（claim / evidence / unknown，含 **clone-validate-commit 原子 mutation**）
- `SemanticUpdate` boundary 與 **Observation-first ordering**
- `ContextCompiler`（bounded deterministic 編譯，對 Knowledge 唯讀）
- `FilesystemRuntime`（sandbox + path safety）
- `ProcessRuntime`（timeout + policy deny-list）
- JSON checkpoint + crash resume
- Provider interpreter 嚴格驗證
- B1 → B2-C4 的完整設計 / 實作 / 驗證軌跡（見 `IMPLEMENTATION_STATUS.md`）

## 必須誠實說明的限制

這個 repo **無法執行一個真正自主的 agent**。以下是結構性事實，不是待辦事項：

- `interpreter.rs` 只能產生 `AgentDecision::Act`。`Observe` / `Wait` / `Finish` 不可達。
- `verify_goal()` 只被測試呼叫。provider-driven agent 結構上**無法達成 Done**。
- 唯一的 production semantic producer 是 `NoOpSemanticUpdateProducer`。所有 KnowledgeClaim 都來自 test producer。
- `AgentState` 的大部分語意欄位（`hypotheses` / `unknowns` / `environment_state` / `running_jobs` / `remaining_work`）**沒有任何 production 寫入點**。
- `Runtime` trait 沒有 observation 能力 —— 不執行 Action 就無法取得環境資訊。
- `AgentDecision::Observe` 會把 **LLM 自己的意圖字串偽造成 Observation** 並寫入權威 store（`agent_loop.rs`）。這是本研究報告中特別指出的反面教材：evidence 基礎設施自己作弊。

這些限制在停止時被完整記錄在 `IMPLEMENTATION_STATUS.md`，**未被修補** —— 因為修補它們等於回到已被否定的產品方向。

---

# 以下為原始專案紀錄（2024–2026，已停止）

以下內容保留作為架構設計的歷史紀錄。

## 原始目標

一個以 Rust 開發的本地 Autonomous AI Agent。

原始目標不是建立固定 workflow，而是建立一個通用 Agent Core + Runtime，使 Agent 能夠：

- 理解使用者目標
- 觀察電腦環境
- 建立與修正理解 / 假設
- 自主決定下一步
- 執行電腦操作
- 取得 Action Result / Observation
- 根據結果改變策略
- 驗證原始 Goal 是否真正完成
- 在長時間工作中持續運作
- 在中斷後恢復

核心邊界：

```text
Agent Core
= understanding / reasoning / decision / adaptation / verification

Runtime
= computer-operation capability

Observation
= environmental evidence

Persistence
= continuity / recovery state
```

## 目標架構

```text
User Goal
    ↓
Agent Core
    ↕
State / Observation / Knowledge
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
Environment
    ↓
ActionResult / Observation
    ↓
Agent Core
```

重要原則：

- Agent Core 決定下一步；Runtime 不得變成固定 workflow engine。
- Action Success 不等於 Goal Success。
- Failure 是 Observation，不自動等於任務終止。
- Unknown 是合法語意狀態，不得用猜測填補未知。
- Hypothesis 不等於 Fact。
- Observation history 與 LLM context 是不同概念。
- LLM output 必須經 Parse / Validate / Normalize / Policy validation 後才能執行。
- 不保存 private chain-of-thought，只保存可驗證的 decision-relevant information。

## 文件治理

這個專案在文件治理上投入了相當的紀律，是 repo 中值得學習的部分：

```text
AGENTS.md                → AI 必須遵守的規則
DEVELOPMENT_WORKFLOW.md  → 如何工作、驗證、恢復
IMPLEMENTATION_STATUS.md → 實際實作現況（唯一現況來源）
README.md                → 人類閱讀的專案入口
docs/README.md           → stable docs 的唯一導航入口
docs/01~11               → requirements / design / interfaces / acceptance / test / analysis / roadmap
history/                 → 歷史資料
```

核心區分是**四層分離**：

```text
Stable Design        = 系統「打算」變成什麼
Current Implementation = repository「實際」是什麼
History              = 之前發生了什麼
Temporary Context    = 現在正在做什麼
```

這個區分讓「設計與實作不一致」變成可被看見與處理的問題，而不是被混淆掉的模糊地帶。

`AGENTS.md` 與 `DEVELOPMENT_WORKFLOW.md` 原本是給 coding agent 看的運作規範；它們同時也是這個專案工程文化的紀錄，因此一併保留。

## 執行

```bash
cargo test          # 178 tests
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo check
```

本專案是 **library crate**，沒有 `main.rs`、沒有 binary target、沒有 CI。測試是唯一的執行入口。

## 授權

MIT License，見 [`LICENSE`](LICENSE)。可自由取用、修改與再利用。
