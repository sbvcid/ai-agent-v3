# DEVELOPMENT_WORKFLOW.md

# 可恢復開發流程（Resumable Development Workflow）

本文件定義本專案的長期 AI Coding Agent 開發流程。

本文件不定義 Agent Runtime 的功能或架構；功能與架構應以 `AGENTS.md`、正式規格與已批准的穩定設計文件為準。

本文件的目的，是讓任何新的 Codex session 或其他 AI coding agent，在失去先前聊天紀錄後，仍能透過專案本身判斷目前狀態並繼續開發。

---

## 1. 核心原則

### 1.1 專案本身是 Source of Truth

不得依賴：

- 先前的聊天紀錄；
- Codex 的記憶；
- 上一個 session 的口頭描述。

應主要依靠：

- 穩定規格與設計文件；
- Source Code；
- Tests；
- 實際 Compile / Test / Execution Results；
- `IMPLEMENTATION_STATUS.md`；
- Git history。

判斷目前狀態。

### 1.2 文件有不同穩定層級

本專案刻意區分「永久設計基準」與「目前工作狀態」。

```text
正式規格 / 穩定設計
        ↓
實作
        ↓
驗證
        ↓
IMPLEMENTATION_STATUS.md
        ↓
Git Checkpoint / history
```

永久文件不是聊天紀錄，也不是每次實作的工作日誌。只有需求、架構原則、介面契約、驗收標準或長期依賴關係真正改變時，才修改永久文件。

臨時性的內容，例如：

- 尚未決定的設計討論；
- 一次性的 coding-agent prompt；
- debugging 過程；
- 某次實作中的暫時 workaround；
- 尚未確認的架構想法；
- 單次測試輸出；

不應為了「留下紀錄」而塞進永久設計文件。這些內容應留在目前開發對話、`IMPLEMENTATION_STATUS.md` 或 `history/`，視其性質處理。

### 1.3 狀態文件不是絕對真相

`IMPLEMENTATION_STATUS.md` 是導航、進度與驗證記錄，不是實際程式狀態的替代品。

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

### 1.4 不要無理由重寫

如果專案已經存在部分實作：

- 不要假設它全部正確；
- 也不要假設它全部錯誤；
- 先檢查、測試、診斷；
- 保留可以使用的部分；
- 只進行必要的修改。

---

# 2. 文件職責與穩定層級

本專案不同文件具有不同職責。

```text
AGENTS.md
    ↓
AI Agent 在本專案中必須遵守的規則與架構約束

docs/01_REQUIREMENTS.md
    ↓
系統需求與目標行為

docs/02_CONSTRUCTION.md
    ↓
系統建構與實作原則

docs/03_INTERFACES.md
    ↓
語意介面與資料契約

docs/04_ACCEPTANCE_TESTS.md
    ↓
行為驗收標準

docs/05_TEST_PLAN.md
    ↓
測試與驗證方法

docs/06_IMPLEMENTATION_GAP_ANALYSIS.md
    ↓
階段性實作現況與規格差距分析

docs/07_AGENT_CORE_OBSERVATION_INCREMENT_DESIGN.md
    ↓
穩定的第一個 post-cycle Agent Core 設計基準

docs/08_AGENT_CORE_ROADMAP.md
    ↓
穩定的長期依賴關係與能力方向

DEVELOPMENT_WORKFLOW.md
    ↓
如何開發、驗證、中斷與恢復

IMPLEMENTATION_STATUS.md
    ↓
目前實際開發狀態、驗證證據與暫存工作資訊

history/
    ↓
歷史紀錄
```

不存在一份永久的「Implementation Task」文件。

具體 coding-agent 指令是在某個設計已經批准後，於當次開發對話中產生的一次性工作指令。完成後不需要把該 prompt 永久保存成新的規格文件。

如果新增永久文件，必須先明確定義它的長期職責，避免與現有文件重複描述同一規則。

---

# 3. 開始新的 Coding Session

每次開始處理本專案時，不得直接假設目前進度。

首先檢查：

```text
AGENTS.md
DEVELOPMENT_WORKFLOW.md
IMPLEMENTATION_STATUS.md
docs/
Cargo.toml
src/
tests/
```

若目前工作涉及特定設計，優先閱讀該設計文件及其直接依賴，而不是無差別把所有文件當成一次性的 prompt 全部塞入 context。

如果專案使用 Git，也應視情況檢查：

```text
git status
git log --oneline --decorate -10
```

然後執行與目前狀態相關的驗證，例如：

```text
cargo check
cargo test
```

以及必要的 integration test 或 E2E test。

---

# 4. 判斷目前真正進度

開始新的 Stage 或 implementation increment 前，必須比較：

```text
穩定規格 / 設計
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
某 Stage = COMPLETED
```

但：

```text
cargo test
FAILED
```

則該 Stage 不應被視為真正完成。

必須先診斷並處理問題。

---

# 5. IMPLEMENTATION_STATUS.md

專案根目錄應存在：

```text
IMPLEMENTATION_STATUS.md
```

如果不存在：

1. 先檢查現有 source code、tests、文件與 Git history；
2. 判斷目前實際狀態；
3. 再建立 `IMPLEMENTATION_STATUS.md`。

不得因為沒有狀態文件，就假設專案是全新專案。

狀態文件至少應能回答：

```text
Current Stage / Increment
Overall Status
Completed Work
Current Work
Incomplete Work
Tests / Verification
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

`IMPLEMENTATION_STATUS.md` 可以隨實作更新。它是本專案主要的「目前狀態」容器，因此不必為每個小變化修改永久設計文件。

---

# 6. COMPLETED 的判定

不得因為以下原因直接標記：

```text
COMPLETED
```

例如：

- 程式看起來完成；
- Function 已經存在；
- Coding Agent 認為完成；
- 文件寫著完成。

必須有與該 Stage / increment 目標相符的實際驗證證據。

例如：

```text
cargo check: PASS
cargo test: PASS
integration test: PASS
```

實際需要哪些驗證，依工作目標決定。

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

# 7. Implementation Increment 流程

本專案採用 incremental development。

一次只處理一個已批准的 implementation boundary。

標準流程：

```text
確認目前狀態
    ↓
閱讀相關穩定規格 / 設計
    ↓
檢查現有實作
    ↓
確認 implementation boundary
    ↓
在開發對話中產生一次性 implementation prompt
    ↓
實作
    ↓
Compile
    ↓
Tests
    ↓
修復目前範圍內的問題
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

完成目前 increment 後，不得自行開始下一個 increment。

必須等待使用者明確要求繼續。

Implementation prompt 是工作指令，不是永久設計文件；除非設計本身改變，不應因此修改 `docs/`。

---

# 8. 中斷與恢復

如果上一個 Coding Agent session 在某個 increment 中途停止：

不得直接假設它完全失敗，也不得直接從頭重做。

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

如果中斷資訊只屬於一次性的工作細節，優先記錄於 `IMPLEMENTATION_STATUS.md`，而不是修改永久設計文件。

---

# 9. 損壞或不一致狀態

如果發現：

- Compile failure；
- Test failure；
- Partial implementation；
- Incomplete refactor；
- Interface mismatch；
- Duplicate implementation；
- Spec / code mismatch；
- Status / code mismatch；

先診斷，再修改。

至少確認：

```text
What is broken?
Why is it broken?
What still works?
What is missing?
What is the smallest safe fix?
```

除非穩定規格要求重新設計，否則避免全面重寫。

---

# 10. 修改範圍

每個 increment 應盡量限制在已批准的範圍。

如果發現與目前工作無關的問題：

```text
不要順便大規模重構。
```

可以記錄到：

```text
IMPLEMENTATION_STATUS.md
```

或適當的 issue / history。

只有當該問題會阻止目前 increment 正確完成時，才應在目前 increment 處理。

如果發現真正需要改變穩定設計基準的架構問題，應先停止實作、提出差異與影響，再決定是否修改設計文件。不得為了讓當前程式方便通過而偷偷改寫設計基準。

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

之間存在無法自行判斷的衝突，或者穩定規格與目前實作存在重大矛盾：

不要自行猜測並進行大幅修改。

應：

1. 找出衝突；
2. 說明不同解釋；
3. 說明目前程式採用的方式；
4. 說明可能影響；
5. 將狀態標記為 `BLOCKED`；
6. 等待使用者決定。

如果存在明確的既有架構決策，則依既有決策處理。

---

# 12. Checkpoint

每個 implementation increment 完成後，必須留下可恢復的狀態。

至少包括：

```text
Source Code
Tests
IMPLEMENTATION_STATUS.md
相關穩定設計 / 規格（若有變更）
```

如果專案使用 Git，可以使用清楚的 commit 或其他 Git history 作為 checkpoint。

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

如果 Coding Agent 即將停止工作，且目前 increment 尚未完成：

應更新：

```text
IMPLEMENTATION_STATUS.md
```

至少記錄：

```text
Current Increment
Current Work
Completed Work
Remaining Work
Tests Passed
Tests Failed
Known Errors
Next Recommended Action
```

不要為了一次中斷而修改永久設計文件。

---

# 15. 新專案 / 完整資料夾重新提供

當完整專案被提供給新的 Coding Agent session 時：

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

不要因為換了 session 就：

```text
Delete
    ↓
Rebuild
```

除非使用者或正式架構決策明確要求重新建立。

---

# 16. 永久文件修改原則

永久文件的目標是長期穩定，而不是完整記錄每一次討論。

應修改永久文件的情況：

- 正式需求改變；
- 架構責任邊界改變；
- 核心介面契約改變；
- 驗收標準改變；
- 長期依賴關係或 roadmap 方向發生實質改變；
- 發現永久文件本身存在明確矛盾或會持續誤導未來實作。

不應僅因以下原因修改永久文件：

- 某一次 coding prompt 改變；
- 某個 bug 修好了；
- 某次測試結果改變；
- 某個 AI model 提出暫時建議；
- 某次 debugging 發現細節；
- 某個 increment 的臨時檔案列表改變。

這些資訊應留在 `IMPLEMENTATION_STATUS.md`、history 或當次對話。

---

# 17. 最終原則

任何新的 Coding Agent session 都應能透過專案本身回答：

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
Stable Specification / Design
          ↓
Approved Implementation Boundary
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

文件角色如下：

`AGENTS.md` 定義「AI Agent 必須遵守什麼」。

`docs/` 定義「系統應該是什麼」以及已批准的穩定設計方向。

`DEVELOPMENT_WORKFLOW.md` 定義「如何開發、驗證與恢復」。

`IMPLEMENTATION_STATUS.md` 定義「目前做到哪裡以及目前驗證結果」。

`history/` 保存「過去發生過什麼」。

Source Code 與 Tests 才是「實際實作與行為證據」。
