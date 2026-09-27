# AI Agent v3 — Documentation Index

> ## ⚠️ 本專案已於 2026-09-28 停止開發並封存
>
> 不再接受以原始產品方向為目的的新功能開發。若你是新接手的 AI 或開發者：
> **先讀 [`00_PRODUCT_REDEFINITION_RESEARCH.md`](00_PRODUCT_REDEFINITION_RESEARCH.md)**，
> 它說明了為什麼停止、停止前完成了什麼、以及現存的架構概念在什麼條件下仍有價值。
>
> 本文件以下的導航規則保留原樣，作為這個專案文件治理方式的紀錄。

本文件是 `docs/` 的唯一導航入口。

它不定義新的 Agent Core / Runtime 架構，也不定義 implementation task。它只回答兩個問題：

1. 新 AI 必須先讀哪些文件？
2. 針對目前這個 task，接下來哪些文件才需要讀？

## 1. 固定規則：必讀文件 vs. 目前需要讀的文件

### A. 每次新 AI Session 固定必讀

以下 5 份文件是唯一的 repository startup reading set，也是目前治理基線的固定入口集合：

```text
AGENTS.md
DEVELOPMENT_WORKFLOW.md
IMPLEMENTATION_STATUS.md
README.md
docs/README.md
```

新 AI 不需要自行判斷這 5 份是否重要。

完成這 5 份後，才進入 task-specific reading。

### B. 目前需要讀

由本文件依 task 選出的最小必要 context：

```text
task-specific stable docs
        +
direct interface / requirement / acceptance
        +
direct source
        +
direct tests
```

以下內容不是 startup 必讀：

```text
docs/ 全部文件
src/ 全部 source
tests/ 全部 tests
history/ 全部歷史
```

固定入口文件是治理基線。未來新增 stable document 不需要為了更新 stable-document inventory 而修改 `AGENTS.md`、`README.md` 或 `DEVELOPMENT_WORKFLOW.md`；只有 startup governance、文件責任或工作流程本身發生真正的長期變更時，才需要重新對齊固定入口文件。

## 2. 文件層級與唯一責任

```text
Stable Design / Specification
    ↓
Current Implementation State
    ↓
Historical Record
    ↓
Temporary Development Context
```

對應責任：

```text
AGENTS.md
→ AI 必須遵守的規則與架構約束

DEVELOPMENT_WORKFLOW.md
→ 如何開始、實作、驗證、中斷、恢復

IMPLEMENTATION_STATUS.md
→ Repository 現在實際做到什麼

README.md
→ 人類閱讀的專案介紹與目標架構

docs/README.md
→ stable docs 的唯一導航入口

docs/01~11
→ 目前治理基線中的 stable requirements / design / interface / acceptance / test / roadmap / approved implementation increment design

history/
→ 歷史資料，不是目前工作指令

active conversation
→ 一次性 implementation prompt、debugging、暫時決策、未批准想法
```

禁止讓兩份文件同時成為同一規則的 source of truth。

`docs/README.md` 維護 stable-document inventory、唯一職責與 task-specific reading dependency；固定入口文件只描述治理基線與各自責任，不因 stable document 數量增加而逐次同步清單。

## 3. Stable Documents 現況

目前 repository 中已存在的 stable documents 是：

| 文件 | 唯一主要職責 | 何時讀 |
|---|---|---|
| `00_PRODUCT_REDEFINITION_RESEARCH.md` | **專案封存原因、2026 Agent 生態研究、既有架構的 keep / replace / delete 評估** | **任何新讀者的第一份文件** |
| `01_REQUIREMENTS.md` | 系統需求與目標行為 | 需求、目標或驗收範圍需要確認時 |
| `02_CONSTRUCTION.md` | 建構與實作原則 | 架構 / implementation 原則需要確認時 |
| `03_INTERFACES.md` | 穩定語意介面與資料契約 | 修改 Core / Runtime / Provider 邊界時 |
| `04_ACCEPTANCE_TESTS.md` | 行為驗收標準 | 實作或驗證需求時 |
| `05_TEST_PLAN.md` | 測試與驗證方法 | 規劃或執行驗證時 |
| `06_IMPLEMENTATION_GAP_ANALYSIS.md` | Target Design 與 Current Implementation 的差距 —— **⚠️ SUPERSEDED，內容過期** | 僅供理解當時的缺口判斷依據 |
| `07_AGENT_CORE_OBSERVATION_INCREMENT_DESIGN.md` | Observation Store + Context Compiler 的穩定設計 | 涉及 Observation / Context 時 |
| `08_AGENT_CORE_ROADMAP.md` | Agent Core 長期能力與依賴方向 —— **⚠️ NOT TAKEN，未被實作** | 僅供理解依賴關係分析 |
| `09_AGENT_CORE_EVIDENCE_KNOWLEDGE_SEMANTICS_DESIGN.md` | Evidence / Knowledge / Unknown semantic foundation | 涉及 Evidence / Knowledge / Unknown 時 |
| `10_B2_KNOWLEDGE_CONTEXT_INTEGRATION_INCREMENT_DESIGN.md` | B2 Knowledge Context Integration 的 implementation boundary、sub-increments 與 acceptance criteria | 實作 B2-A/B/C 或驗證 B2 data path 時 |
| `11_B2-C_SEMANTIC_UPDATE_CLOSED_LOOP_INCREMENT_DESIGN.md` | B2-C Semantic Update / Closed-Loop Integration 的 implementation boundary、ownership、closed-loop tests 與 release gate | 實作或驗證 B2-C 時 |

以上是目前治理基線中的 stable-document baseline，不是永久且封閉的文件全集。

`09_AGENT_CORE_EVIDENCE_KNOWLEDGE_SEMANTICS_DESIGN.md` 現已正式建立並標記為 **APPROVED STABLE DESIGN**。它定義 Evidence / Knowledge / Unknown 的第一版 semantic boundary；它不是 implementation task，也不自動建立下一個 implementation Stage。

`10_B2_KNOWLEDGE_CONTEXT_INTEGRATION_INCREMENT_DESIGN.md` 現已正式建立並標記為 **APPROVED STABLE DESIGN**。它只定義 B2 implementation boundary；B2-A、B2-B、B2-C 仍須各自取得使用者明確批准後才能實作，不自動建立 Stage 10。

`11_B2-C_SEMANTIC_UPDATE_CLOSED_LOOP_INCREMENT_DESIGN.md` 現已正式建立並標記為 **APPROVED STABLE DESIGN**。它只定義 B2-C Semantic Update / Closed-Loop Integration 的 implementation boundary；B2-C1 至 B2-C4 均已完成並通過驗證，B2-C 已通過 release gate 並正式關閉，且不自動建立新的 Stage。

未來正式建立新的 stable document 時，必須同時：

1. 在本索引加入它；
2. 定義它的唯一長期職責；
3. 定義哪些 task 需要讀它；
4. 確認它不重複既有文件。

不需要同步修改 `AGENTS.md`、`README.md` 或 `DEVELOPMENT_WORKFLOW.md`，除非新增文件同時代表治理規則、文件責任或工作流程本身發生真正的長期變更。

文件編號只代表導航分類，不代表 implementation Stage。

## 4. Task → 最小閱讀集合

### 新 AI 剛進 repository

只讀固定 5 份：

```text
AGENTS.md
DEVELOPMENT_WORKFLOW.md
IMPLEMENTATION_STATUS.md
README.md
docs/README.md
```

然後依下面規則選 task-specific context。

### 修復明確的 Bug / Test Failure

先讀：

```text
固定 5 份入口文件
```

然後只讀：

```text
failing test
→ direct source
→ direct interface / design
```

除非發現 stable-design conflict，否則不要讀完整套 architecture docs。

### 修改 Agent Core

先讀：

```text
03_INTERFACES.md
06_IMPLEMENTATION_GAP_ANALYSIS.md
```

再讀與本次 increment 直接相關的 Agent Core design。

若需求或驗收有疑問，再讀：

```text
01_REQUIREMENTS.md
04_ACCEPTANCE_TESTS.md
```

若需要測試方法，再讀：

```text
05_TEST_PLAN.md
```

不要因為「修改 Agent Core」就自動讀全部 Runtime capability 文件。

### 修改 Observation / Context

讀：

```text
03_INTERFACES.md
07_AGENT_CORE_OBSERVATION_INCREMENT_DESIGN.md
```

再讀直接相關 source/tests。

### 修改 Evidence / Knowledge

讀：

```text
03_INTERFACES.md
06_IMPLEMENTATION_GAP_ANALYSIS.md
09_AGENT_CORE_EVIDENCE_KNOWLEDGE_SEMANTICS_DESIGN.md
```

再依實際 increment 讀直接相關 source/tests。

若需求或驗收有疑問，再讀：

```text
01_REQUIREMENTS.md
04_ACCEPTANCE_TESTS.md
```

若需要測試方法，再讀：

```text
05_TEST_PLAN.md
```

`09` 是 stable design，但不代表 implementation 自動開始；仍必須依 development workflow 定義並取得明確批准的 implementation boundary。

### 實作 B2 Knowledge Context Integration

讀：

```text
03_INTERFACES.md
06_IMPLEMENTATION_GAP_ANALYSIS.md
09_AGENT_CORE_EVIDENCE_KNOWLEDGE_SEMANTICS_DESIGN.md
10_B2_KNOWLEDGE_CONTEXT_INTEGRATION_INCREMENT_DESIGN.md
```

再讀 B2 當前 sub-increment 的 direct source/tests。

B2-A、B2-B、B2-C 必須依序單獨批准與驗證；不得把本文件視為一次性授權全部 B2 程式碼實作。

### B2-C Semantic Update / Closed-Loop Integration

讀：

```text
03_INTERFACES.md
06_IMPLEMENTATION_GAP_ANALYSIS.md
09_AGENT_CORE_EVIDENCE_KNOWLEDGE_SEMANTICS_DESIGN.md
10_B2_KNOWLEDGE_CONTEXT_INTEGRATION_INCREMENT_DESIGN.md
11_B2-C_SEMANTIC_UPDATE_CLOSED_LOOP_INCREMENT_DESIGN.md
```

再讀：

```text
src/core/agent_loop.rs
src/core/knowledge_store.rs
src/core/observation_store.rs
src/core/context_compiler.rs
src/core/decision_source.rs
src/core/types.rs
```

以及直接相關 tests。

`11` 已獲明確批准為 stable design，C1 至 C4 均已完成並通過驗證；B2-C 已關閉，但這不代表後續 C–G roadmap item 獲得授權。任何新 increment 仍須各自取得明確批准。

### 修改 Runtime Capability

先讀：

```text
01_REQUIREMENTS.md
02_CONSTRUCTION.md
03_INTERFACES.md
04_ACCEPTANCE_TESTS.md
```

再讀該 capability 的 direct source/tests。

只有涉及長期缺口或依賴時才讀：

```text
06_IMPLEMENTATION_GAP_ANALYSIS.md
08_AGENT_CORE_ROADMAP.md
```

### 修改 Provider / LLM Boundary

先讀：

```text
02_CONSTRUCTION.md
03_INTERFACES.md
```

再讀 direct provider / decision source / interpreter source 與 tests。

### 判斷下一個 implementation increment

讀：

```text
IMPLEMENTATION_STATUS.md
06_IMPLEMENTATION_GAP_ANALYSIS.md
08_AGENT_CORE_ROADMAP.md
```

然後讀候選 increment 的 stable design。

Roadmap 與 gap analysis 都不能單獨產生新的 Stage。真正的 implementation boundary 必須有：

```text
stable design
+ dependencies
+ scope
+ non-goals
+ acceptance criteria
+ user approval
```

## 5. Stable Design、Current State、Source Code 的關係

```text
Stable Design
= 系統應該是什麼

Source Code
= 系統現在實際是什麼

Tests / Execution Results
= 系統實際證明了什麼

IMPLEMENTATION_STATUS.md
= 對 current state 的集中導航與驗證紀錄
```

如果 implementation 尚未達到 stable design，不代表 stable design 應該被改成描述目前程式。

如果 stable design 本身被證明錯誤，依 `AGENTS.md` / `DEVELOPMENT_WORKFLOW.md` 的 conflict procedure 處理，不得偷偷修改。

## 6. 新增文件的門檻

未來文件可能越來越多，因此新增 stable document 的標準不是「有東西可以記錄」，而是「現有文件無法合理承擔一個長期、獨立的 source-of-truth 職責」。

新增前必須能回答：

```text
為什麼現有文件不能負責？
這份文件唯一負責什麼？
哪些 task 需要讀它？
它依賴哪些文件？
它會不會重複其他文件？
```

以下內容禁止建立新的 stable document：

```text
一次性 implementation prompt
一次 debugging 過程
單次 test output
temporary workaround
未批准的 architecture idea
單次 coding-agent 意見
```

## 7. Index 的責任

本文件只負責：

```text
startup reading rule
stable-document inventory
task-specific reading selection
stable-document dependency navigation
```

本文件不負責：

```text
Agent Core architecture
Runtime implementation
stable interface definitions
implementation status
implementation stages
acceptance criteria
```

只有以下情況才需要更新本文件：

```text
新增 / 刪除 stable document
stable document 長期職責改變
task-specific reading dependency 改變
startup reading rule 改變
```

單純 implementation detail、module refactor、test output 或 increment 完成，不需要更新本索引。

## 8. 最小 Context 原則

永遠優先：

```text
最小必要 context
```

而不是：

```text
最大可讀文件量
```

標準流程：

```text
固定 5 份入口文件
        ↓
docs/README.md
        ↓
task-specific stable docs
        ↓
direct source
        ↓
direct tests
        ↓
verification
```

只有發現真正 dependency、missing requirement 或 stable-design conflict 時，才擴大 context。
