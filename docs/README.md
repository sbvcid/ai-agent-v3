# AI Agent v3 — Documentation Index

本文件是 `docs/` 的導航入口，不是新的架構規格，也不是 implementation task。

目的：讓新的 AI coding agent 在沒有先前聊天紀錄的情況下，可以先判斷「哪些文件需要讀」，而不是把整個 repository 的文件一次全部塞進 context。

---

## 1. 新 AI 的閱讀原則

新 session 不應無差別閱讀所有文件。

推薦流程：

```text
AGENTS.md
    ↓
DEVELOPMENT_WORKFLOW.md
    ↓
IMPLEMENTATION_STATUS.md
    ↓
docs/README.md
    ↓
判斷目前任務
    ↓
只閱讀該任務需要的穩定設計 / 規格
    ↓
檢查相關 source code / tests
    ↓
實作一個已批准的 implementation boundary
```

其中：

- `AGENTS.md`：定義 Coding Agent 必須遵守的架構與工作規則。
- `DEVELOPMENT_WORKFLOW.md`：定義如何開始、驗證、中斷、恢復與完成一個 implementation increment。
- `IMPLEMENTATION_STATUS.md`：描述目前 repository 的實際實作狀態與驗證結果。
- 本文件：告訴 AI 哪些文件負責什麼，以及何時需要閱讀它們。

不要因為存在多份文件，就假設每次工作都必須全部閱讀。

---

## 2. 文件層級

本專案文件分成四個主要類別：

```text
Stable Design / Specification
    ↓
Current Implementation State
    ↓
Historical Record
    ↓
Temporary Development Context
```

### Stable Design / Specification

這些文件描述目標、架構、介面、驗收標準、測試方法或長期方向。

修改它們代表可能涉及穩定設計或架構決策，不應因為單次 implementation increment 完成而例行修改。

### Current Implementation State

`IMPLEMENTATION_STATUS.md` 描述目前實際做到哪裡、哪些測試通過、哪些工作尚未完成。

它不是 target architecture 的替代品。

### Historical Record

`history/` 保存過去的實作與決策紀錄。

除非目前問題需要追查歷史，否則新 AI 不應在 startup 時讀取 `history/`。

### Temporary Development Context

一次性的 implementation prompt、debugging 過程、暫時 workaround、未決定的想法與單次測試輸出，優先留在目前開發對話、`IMPLEMENTATION_STATUS.md` 或適當的 history record。

不要建立永久文件來保存一次性的 coding prompt。

---

## 3. Stable Documents

| 文件 | 職責 | 一般情況是否需要閱讀 |
|---|---|---|
| `docs/01_REQUIREMENTS.md` | 系統需求與目標行為 | 需要確認需求或驗收目標時 |
| `docs/02_CONSTRUCTION.md` | 建構與實作原則 | 設計/架構實作需要時 |
| `docs/03_INTERFACES.md` | 穩定語意介面與資料契約 | 修改 Core / Runtime / Provider 邊界時 |
| `docs/04_ACCEPTANCE_TESTS.md` | 行為驗收標準 | 實作或驗證需求時 |
| `docs/05_TEST_PLAN.md` | 測試與驗證方法 | 規劃或執行驗證時 |
| `docs/06_IMPLEMENTATION_GAP_ANALYSIS.md` | Target Design 與 Current Implementation 的差距分析 | 判斷目前缺口、依賴或下一個 increment 時 |
| `docs/07_AGENT_CORE_OBSERVATION_INCREMENT_DESIGN.md` | Observation Store + Context Compiler 的穩定設計 | 涉及 Observation / Context 時 |
| `docs/08_AGENT_CORE_ROADMAP.md` | Agent Core 長期能力與依賴方向 | 判斷長期依賴或設計方向時 |
| `docs/09_AGENT_CORE_EVIDENCE_KNOWLEDGE_SEMANTICS_DESIGN.md` | Evidence / Knowledge Semantics 的穩定設計 | 涉及 Evidence / Knowledge 時 |

文件編號是導航用途，不代表 implementation Stage 編號。

例如：

```text
09_AGENT_CORE_EVIDENCE_KNOWLEDGE_SEMANTICS_DESIGN.md
```

不是 `Stage 9` 或 `Stage 10`。

---

## 4. 文件之間的閱讀關係

不要把所有文件視為同一層級。

可以用以下模型理解：

```text
                    AGENTS.md
                        │
                        ▼
              DEVELOPMENT_WORKFLOW.md
                        │
                        ▼
             IMPLEMENTATION_STATUS.md
                        │
                        ▼
                 docs/README.md
                        │
          ┌─────────────┼─────────────┐
          │             │             │
          ▼             ▼             ▼
     Requirements   Interfaces    Design / Roadmap
          │             │             │
          └─────────────┼─────────────┘
                        ▼
                  Source Code
                        │
                        ▼
                      Tests
                        │
                        ▼
                 Actual Results
```

重要區別：

```text
Stable Documents
    = 系統應該是什麼

Source Code
    = 系統現在實際是什麼

Tests / Execution Results
    = 系統實際證明了什麼

IMPLEMENTATION_STATUS.md
    = 對目前狀態的導航與紀錄
```

如果它們互相矛盾，不應自動選一份文件修改其他文件；應依 `AGENTS.md` 與 `DEVELOPMENT_WORKFLOW.md` 的衝突處理流程診斷。

---

## 5. 依工作類型選擇文件

### 新 AI 剛接手專案

先讀：

```text
AGENTS.md
DEVELOPMENT_WORKFLOW.md
IMPLEMENTATION_STATUS.md
docs/README.md
```

然後才根據目前任務選擇其他文件。

### 修改 Agent Core

通常需要：

```text
01_REQUIREMENTS.md
02_CONSTRUCTION.md
03_INTERFACES.md
04_ACCEPTANCE_TESTS.md
06_IMPLEMENTATION_GAP_ANALYSIS.md
```

再加入與目前 increment 直接相關的 Agent Core design document，例如：

```text
07_AGENT_CORE_OBSERVATION_INCREMENT_DESIGN.md
09_AGENT_CORE_EVIDENCE_KNOWLEDGE_SEMANTICS_DESIGN.md
```

不需要因為修改 Core 就自動閱讀所有 Runtime capability 文件。

### 修改 Observation / Context

優先：

```text
03_INTERFACES.md
07_AGENT_CORE_OBSERVATION_INCREMENT_DESIGN.md
```

如果工作涉及 Evidence / Knowledge，再讀：

```text
09_AGENT_CORE_EVIDENCE_KNOWLEDGE_SEMANTICS_DESIGN.md
```

### 修改 Evidence / Knowledge

優先：

```text
03_INTERFACES.md
07_AGENT_CORE_OBSERVATION_INCREMENT_DESIGN.md
08_AGENT_CORE_ROADMAP.md
09_AGENT_CORE_EVIDENCE_KNOWLEDGE_SEMANTICS_DESIGN.md
```

必要時再回查 `01_REQUIREMENTS.md`、`04_ACCEPTANCE_TESTS.md` 與 `05_TEST_PLAN.md`。

### 修改 Runtime capability

優先閱讀：

```text
01_REQUIREMENTS.md
02_CONSTRUCTION.md
03_INTERFACES.md
04_ACCEPTANCE_TESTS.md
```

再閱讀與該 capability 直接相關的 source code、tests，以及必要的 gap analysis / roadmap。

### 修改 Provider / LLM boundary

優先閱讀：

```text
03_INTERFACES.md
02_CONSTRUCTION.md
AGENTS.md
```

並檢查相關 provider adapter、decision source、interpreter 與 integration tests。

### 只需要修 Bug / 測試失敗

不要先讀完整套設計文件。

先：

```text
AGENTS.md
DEVELOPMENT_WORKFLOW.md
IMPLEMENTATION_STATUS.md
```

然後：

```text
失敗測試
→ 相關 source code
→ 直接相關 interface / design
```

只有當問題可能涉及架構或 stable design conflict 時，才擴大閱讀範圍。

### 追查歷史決策

只有在目前 source、status 與 stable design 無法解釋問題時，才進入：

```text
history/
```

`history/` 不是一般 coding session 的 startup reading set。

---

## 6. 新增 Stable Document 的規則

新增永久文件前，必須先確認它有獨立且長期存在的職責。

新增文件應能回答：

```text
為什麼現有文件不能負責這件事？
這份文件的長期 source-of-truth 範圍是什麼？
哪些 AI / implementation work 需要讀它？
它依賴哪些既有文件？
它是否會與其他文件重複定義規則？
```

不要為以下內容建立新的 stable document：

```text
一次性 implementation prompt
一次 debugging 過程
單次測試輸出
暫時 workaround
尚未批准的架構想法
單次 coding agent 意見
```

這些內容應留在適當的 temporary/current/history 層級。

---

## 7. 文件索引本身的責任

本文件只負責導航。

它不重新定義：

- Agent Core architecture；
- Runtime architecture；
- stable interfaces；
- implementation stages；
- acceptance criteria；
- implementation status。

如果其他文件的內容改變，本索引只需要在以下情況更新：

```text
新增或刪除 stable document
stable document 的長期職責改變
文件之間的閱讀依賴關係改變
```

單純 implementation detail 改變，不應更新本文件。

---

## 8. 最小 Context 原則

新的 AI coding agent 應採用：

```text
最小必要 context
```

而不是：

```text
最大可讀文件量
```

目標不是讓 AI 記住整個 repository，而是讓 AI 能夠：

1. 找到目前狀態；
2. 找到這次工作的 stable design；
3. 找到直接相關的 interface；
4. 找到相關 source code；
5. 找到相關 tests；
6. 完成驗證；
7. 在需要時再擴大閱讀範圍。

如果在閱讀過程中發現目前工作涉及另一個 semantic boundary，才沿 dependency graph 擴大 context。

---

## 9. Implementation Session 的推薦讀取順序

標準新 session：

```text
1. AGENTS.md
2. DEVELOPMENT_WORKFLOW.md
3. IMPLEMENTATION_STATUS.md
4. docs/README.md
5. 本次 increment 的 stable design
6. 該 design 的直接依賴文件
7. 相關 source code
8. 相關 tests
9. Git status / recent history
10. 實作與驗證
```

不要在第 1–4 步就把整個 `docs/`、`history/`、`src/` 全部載入 context。

如果目前只是修復明確的測試失敗，可以縮短為：

```text
AGENTS.md
DEVELOPMENT_WORKFLOW.md
IMPLEMENTATION_STATUS.md
directly relevant design / interface
failing test
relevant source
```

---

## 10. 與 Implementation Stage 的關係

文件索引不產生 implementation stage。

尤其：

```text
docs/07_...
docs/08_...
docs/09_...
```

都只是文件編號與穩定設計分類。

是否開始某個 implementation increment，必須依：

```text
stable design
+ dependency understanding
+ scope
+ non-goals
+ acceptance criteria
+ user approval
```

決定。

不得因為新增了一份 design document，就自行推導出新的 Stage。

---

## 11. Current Documentation Map

目前主要文件可以簡化理解為：

```text
Project Entry
└── README.md

Agent Rules
└── AGENTS.md

Development Process
└── DEVELOPMENT_WORKFLOW.md

Current Reality
└── IMPLEMENTATION_STATUS.md

Stable Specification / Design
├── 01_REQUIREMENTS.md
├── 02_CONSTRUCTION.md
├── 03_INTERFACES.md
├── 04_ACCEPTANCE_TESTS.md
├── 05_TEST_PLAN.md
├── 06_IMPLEMENTATION_GAP_ANALYSIS.md
├── 07_AGENT_CORE_OBSERVATION_INCREMENT_DESIGN.md
├── 08_AGENT_CORE_ROADMAP.md
└── 09_AGENT_CORE_EVIDENCE_KNOWLEDGE_SEMANTICS_DESIGN.md

Historical
└── history/
```

本索引本身不代表所有列出的 design document 都已經批准進入 implementation。

Implementation readiness 仍以 `IMPLEMENTATION_STATUS.md`、相關 stable design、實際 source/tests，以及使用者明確批准的 implementation boundary 為準.
