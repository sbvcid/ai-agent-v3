# AI Agent v3

一個以 Rust 開發的本地 Autonomous AI Agent。

本專案的目標不是建立固定 workflow，而是建立一個通用 Agent Core + Runtime，使 Agent 能夠：

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

## 目前狀態

第一輪 implementation cycle 已完成至 Stage 9.4。

第一個 post-cycle Agent Core increment：

```text
Observation Store + Context Compiler
= COMPLETED
```

這代表第一版的 Observation Store 與 Context Compiler 基礎已經實作並通過驗證；它們的「完整長期能力」仍不是已完成項目。未來更完整的 observation、evidence、knowledge、hypothesis、progress、loop detection、adaptive recovery 等能力仍需依 stable design 逐步實作。

目前已具備的主要能力：

- Canonical Agent Core domain model
- Deterministic Agent Loop
- Mock LLM / Fake Runtime
- JSON Checkpoint / Crash Resume foundation
- Filesystem Runtime foundation
- Process Runtime foundation
- Ollama Provider
- Provider Decision Source
- LLM Decision Interpretation / Validation
- Real Ollama Closed Loop
- Observation Store foundation
- Context Compiler foundation
- Release Gate

目前仍未完整實作的長期能力包括：

- 更完整的 Observation / Context semantics
- Evidence / Knowledge semantics
- Hypothesis lifecycle
- Progress detection
- Loop detection
- Adaptive recovery
- Stronger goal verification
- 完整 Environment Observation
- Shell / Interactive Shell
- 完整 Job Runtime
- Engineering Runtime
- Browser Runtime
- GUI Runtime
- 更完整 Network / System Runtime

**Stage 10 尚未定義。** 不得從本 README、文件編號或 roadmap 自行推導下一個 implementation Stage。

目前 repository 的實際完成狀態，以 `IMPLEMENTATION_STATUS.md`、source code、tests 與實際驗證結果共同判定。

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

## 新 AI 進入 Repository 時

新 AI **固定必讀以下 5 份入口文件，而且只先讀這 5 份**：

```text
AGENTS.md
DEVELOPMENT_WORKFLOW.md
IMPLEMENTATION_STATUS.md
README.md
docs/README.md
```

完成這 5 份後，不得自動讀完整個 `docs/`、`src/`、`tests/` 或 `history/`。

接下來由 `docs/README.md` 判斷本次任務的「目前需要讀」文件：

```text
task-specific stable design
        +
direct interface / requirement / acceptance
        +
direct source
        +
direct tests
```

因此：

```text
必讀文件
= 固定 5 份入口文件

目前需要讀的文件
= 由 docs/README.md 依本次 task 選出的最小必要 context
```

這個區分是本專案的固定規則，不需要新 AI 自行推理。

詳細導航與 task-specific reading set 見：

```text
docs/README.md
```

## 文件治理

```text
AGENTS.md
→ AI 必須遵守的規則

DEVELOPMENT_WORKFLOW.md
→ 如何工作、驗證、恢復

IMPLEMENTATION_STATUS.md
→ 目前實作現況

README.md
→ 人類閱讀的專案入口與高層架構

docs/README.md
→ stable docs 的唯一導航入口

docs/01~08
→ stable requirements / design / interface / acceptance / test / roadmap

history/
→ 歷史資料
```

不存在永久的 implementation-task 文件。一次性的 coding-agent prompt、debugging、暫時 workaround 與未批准想法留在 active conversation、status 或 history 的適當位置。

新增 stable document 前，必須先確認它具有獨立且長期存在的職責，且不能由既有 stable document 合理承擔。文件編號是導航標籤，不代表 implementation Stage。
