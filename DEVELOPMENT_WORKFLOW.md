# DEVELOPMENT_WORKFLOW.md

# 可恢復開發流程（Resumable Development Workflow）

本文件定義本專案的長期 AI Coding Agent 開發流程。

本文件不定義 Agent Runtime 的功能或架構；功能與架構應以 `AGENTS.md` 及 `docs/` 中的正式規格為準。

本文件的目的，是讓任何新的 Codex session 或其他 AI coding agent，在失去先前聊天紀錄後，仍能透過專案本身判斷目前狀態並繼續開發。

---

## 1. 核心原則

### 1.1 專案本身是 Source of Truth

不得依賴：

* 先前的聊天紀錄
* Codex 的記憶
* 上一個 session 的口頭描述

應主要依靠：

* 規格文件
* Source Code
* Tests
* 實際 Compile / Test / Execution Results
* `IMPLEMENTATION_STATUS.md`

判斷目前狀態。

### 1.2 狀態文件不是絕對真相

`IMPLEMENTATION_STATUS.md` 是導航與進度記錄，不是實際程式狀態的替代品。

如果：

```text
IMPLEMENTATION_STATUS.md
```

與：

```text
Source Code
Tests
實際執行結果
```

不一致，必須以實際可驗證結果為準。

### 1.3 不要無理由重寫

如果專案已經存在部分實作：

* 不要假設它全部正確。
* 也不要假設它全部錯誤。
* 先檢查、測試、診斷。
* 保留可以使用的部分。
* 只進行必要的修改。

---

# 2. 文件職責

本專案不同文件具有不同職責。

```text
AGENTS.md
    ↓
AI Agent 在本專案中必須遵守的規則

docs/01_REQUIREMENTS.md
    ↓
系統需求

docs/02_CONSTRUCTION.md
    ↓
系統建構與實作規範

docs/03_INTERFACES.md
    ↓
介面與資料結構規範

docs/04_ACCEPTANCE_TESTS.md
    ↓
驗收標準

docs/05_TEST_PLAN.md
    ↓
測試計畫

Agent Runtime 實作任務.txt
    ↓
實作 Stage 與開發順序

DEVELOPMENT_WORKFLOW.md
    ↓
如何開發、驗證、中斷與恢復

IMPLEMENTATION_STATUS.md
    ↓
目前實際開發進度
```

如果未來新增正式規格文件，應明確定義其職責，避免不同文件重複描述同一規則。

---

# 3. 開始新的 Codex Session

每次開始處理本專案時，不得直接假設目前進度。

首先檢查：

```text
AGENTS.md
DEVELOPMENT_WORKFLOW.md
IMPLEMENTATION_STATUS.md（如果存在）
Agent Runtime 實作任務.txt
docs/
Cargo.toml
src/
tests/
```

如果專案使用 Git，也應視情況檢查：

```text
git status
git log
```

然後根據目前工作的需要閱讀相關規格，而不是無差別假設所有文件內容都必須一次記住。

最後執行與目前狀態相關的驗證，例如：

```text
cargo check
cargo test
```

以及必要的 integration test 或 E2E test。

---

# 4. 判斷目前真正進度

開始新的 Stage 前，必須比較：

```text
規格
+
IMPLEMENTATION_STATUS.md
+
Source Code
+
Tests
+
實際測試結果
```

不要只依靠其中一項。

例如：

```text
IMPLEMENTATION_STATUS.md
Stage 4 = COMPLETED
```

但：

```text
cargo test
FAILED
```

則 Stage 4 不應被視為真正完成。

必須先診斷並處理問題。

---

# 5. IMPLEMENTATION_STATUS.md

專案根目錄應存在：

```text
IMPLEMENTATION_STATUS.md
```

如果不存在：

1. 先檢查現有 source code、tests、文件與 Git history。
2. 判斷目前實際狀態。
3. 再建立 `IMPLEMENTATION_STATUS.md`。

不得因為沒有狀態文件，就假設專案是全新專案。

狀態文件至少應記錄：

```text
Current Stage
Overall Status
Completed Stages
Current Work
Incomplete Work
Tests
Known Issues
Unresolved Questions
Next Step
Important Decisions
```

狀態建議使用：

```text
NOT_STARTED
IN_PROGRESS
BLOCKED
COMPLETED
```

---

# 6. COMPLETED 的判定

不得因為以下原因直接標記：

```text
COMPLETED
```

例如：

* 程式看起來完成。
* Function 已經存在。
* Codex 認為完成。
* 文件寫著完成。

必須有與該 Stage 目標相符的實際驗證證據。

例如：

```text
cargo check: PASS
cargo test: PASS
integration test: PASS
```

實際需要哪些驗證，依 Stage 的目標決定。

如果驗證不足：

```text
IN_PROGRESS
```

如果因為外部條件或規格問題無法繼續：

```text
BLOCKED
```

不得偽造測試結果。

---

# 7. Stage 開發流程

本專案採用 incremental development。

一次只處理一個 Stage。

標準流程：

```text
確認目前狀態
    ↓
閱讀本 Stage 相關規格
    ↓
檢查現有實作
    ↓
實作
    ↓
Compile
    ↓
Tests
    ↓
修復問題
    ↓
再次驗證
    ↓
更新 IMPLEMENTATION_STATUS.md
    ↓
Checkpoint
    ↓
報告結果
    ↓
STOP
```

完成目前 Stage 後，不得自行開始下一個 Stage。

必須等待使用者明確要求繼續。

---

# 8. 中斷與恢復

如果上一個 Codex session 在某個 Stage 中途停止：

不得直接重新開始該 Stage。

先確認：

```text
已完成什麼？
尚未完成什麼？
目前程式是否可以編譯？
哪些測試通過？
哪些測試失敗？
目前錯誤是什麼？
```

然後從目前狀態繼續。

例如：

```text
Stage 5
已完成：
- Core loop
- Tool dispatch

尚未完成：
- Tool result feedback

測試：
- 8 PASS
- 1 FAIL
```

則應繼續完成尚未完成的部分，而不是刪除 Stage 5 重新實作。

---

# 9. 損壞或不一致狀態

如果發現：

* Compile failure
* Test failure
* Partial implementation
* Incomplete refactor
* Interface mismatch
* Duplicate implementation
* Spec / code mismatch

先診斷，再修改。

至少確認：

```text
What is broken?
Why is it broken?
What still works?
What is missing?
What is the smallest safe fix?
```

除非正式規格要求重新設計，否則避免全面重寫。

---

# 10. 修改範圍

每個 Stage 應盡量限制在該 Stage 所需範圍。

如果發現與目前 Stage 無關的問題：

```text
不要順便大規模重構。
```

可以記錄到：

```text
IMPLEMENTATION_STATUS.md
```

或適當的 TODO / issue。

只有當該問題會阻止目前 Stage 正確完成時，才應在目前 Stage 處理。

---

# 11. 規格衝突或不確定

如果：

```text
AGENTS.md
```

與：

```text
docs/
```

之間存在無法自行判斷的衝突，或者規格與目前實作存在重大矛盾：

不要自行猜測並進行大幅修改。

應：

1. 找出衝突。
2. 說明不同解釋。
3. 說明目前程式採用的方式。
4. 說明可能影響。
5. 將狀態標記為 `BLOCKED`。
6. 等待使用者決定。

如果存在明確的既有架構決策，則依既有決策處理。

---

# 12. Checkpoint

每個 Stage 完成後，必須留下可恢復的狀態。

至少包括：

```text
Source Code
Tests
IMPLEMENTATION_STATUS.md
相關規格文件
```

如果專案使用 Git，可以使用清楚的 commit 或其他 Git history 作為 checkpoint。

例如：

```text
stage-01-repository-review
stage-02-core-data-model
stage-03-schema-validation
stage-04-mock-llm
stage-05-agent-loop
```

除非使用者要求，不得自行改變專案既有 Git workflow。

---

# 13. Execution Evidence

重要功能應留下簡潔、可重現的驗證資訊。

例如：

```text
Command:
cargo test

Result:
PASS

Relevant test:
agent_loop_write_file_then_finish
```

或者：

```text
Command:
cargo run

Scenario:
Create test.txt

Result:
PASS
```

不要保存大量不必要的 log。

目標是讓下一個 session 能快速知道：

```text
這個功能是否真的被驗證過？
```

---

# 14. 中斷時更新狀態

如果 Codex 即將停止工作，且目前 Stage 尚未完成：

應更新：

```text
IMPLEMENTATION_STATUS.md
```

至少記錄：

```text
Current Stage
Current Work
Completed Work
Remaining Work
Tests Passed
Tests Failed
Known Errors
Next Recommended Action
```

使下一個 session 能直接接續。

---

# 15. 新專案 / 完整資料夾重新提供

當完整專案被提供給新的 Codex session 時：

不要視為全新專案。

也不要假設所有 Stage 已完成。

應視為：

```text
已有未知程度實作的專案
```

流程：

```text
Inspect
    ↓
Verify
    ↓
Determine Actual State
    ↓
Repair if necessary
    ↓
Continue
```

不要直接：

```text
Delete
    ↓
Rebuild
```

除非正式規格明確要求重新建立。

---

# 16. 最終原則

任何新的 Codex session 都應能透過專案本身回答：

```text
目前專案做到哪裡？

哪些功能真的完成？

哪些功能只是部分完成？

哪些測試真的通過？

目前有哪些問題？

下一步應該做什麼？
```

任何完成狀態都必須有實際證據。

任何中斷都應能恢復。

任何半成品都應能接續。

任何錯誤都應先診斷。

任何不確定的規格都不應自行猜測。

本專案的基本原則是：

```text
Specification
    ↓
Implementation
    ↓
Verification
    ↓
Status
    ↓
Checkpoint
    ↓
Recoverable State
```

`AGENTS.md` 定義「必須遵守什麼」。

`docs/` 定義「系統應該是什麼」。

`Agent Runtime 實作任務.txt` 定義「要做什麼」。

`DEVELOPMENT_WORKFLOW.md` 定義「如何開發與恢復」。

`IMPLEMENTATION_STATUS.md` 定義「目前做到哪裡」。

Source Code 與 Tests 提供「實際做出了什麼，以及哪些內容被驗證」的證據。
