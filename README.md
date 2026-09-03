# AI Agent v3

一個以 Rust 開發的本地 AI Agent。

這個專案的目標不是單純做一個「可以呼叫工具的聊天機器人」，而是建立一個能夠：

* 理解使用者目標
* 觀察電腦環境
* 建立與修正假設
* 自主決定下一步
* 執行電腦操作
* 讀取執行結果
* 面對失敗時改變策略
* 判斷原始目標是否真的完成
* 在長時間任務中持續工作
* 在中斷或崩潰後恢復工作

的通用 Agent。

核心思想是：

> Agent 負責思考與決策，Runtime 負責操作電腦，Observation 負責把環境結果帶回 Agent，Verification 負責確認真正的目標是否完成。

---

## 1. 專案目前的核心架構

```text
Agent v3
│
├── Agent Core
│   ├── Reasoning
│   ├── Hypothesis
│   ├── Planning
│   ├── Decision
│   ├── Adaptation
│   └── Verification
│
├── Observation System
│   ├── Observation Store
│   ├── Context Compiler
│   └── Observation / State Processing
│
├── Runtime
│   ├── Windows OS
│   ├── Process
│   ├── Shell
│   ├── Filesystem
│   ├── GUI
│   ├── Browser
│   └── Network
│
└── Engineering Runtime
    ├── Repository
    ├── Code Search
    ├── Editing
    ├── Diff
    ├── Git
    ├── Build
    └── Test
```

整體運作概念：

```text
User Goal
   ↓
Observe
   ↓
Understand
   ↓
Hypothesize
   ↓
Decide
   ↓
Act
   ↓
Observe Result
   ↓
Evaluate
   ↓
Revise
   ↓
Act Again
   ↓
Verify Goal
   ↓
Done
```

Agent 不應該預先假設所有任務都有固定流程。

---

## 2. 最重要的設計原則

### Agent Core 與 Runtime 分離

Agent Core 不應該直接依賴 Windows API、PowerShell、檔案系統或其他具體實作。

Agent 只知道：

> 我現在有哪些能力可以使用。

Runtime 負責：

> 如何在實際電腦上執行這個能力。

因此未來更換 Runtime、增加 GUI、Browser 或其他環境時，不需要重新設計 Agent Core。

---

### Observation 是證據

Runtime 執行後產生的結果應該回到 Agent 成為 Observation。

例如：

```text
Action:
執行 cargo test

Observation:
exit code = 101
stderr = compilation error
```

Agent 再根據 Observation 判斷發生了什麼。

不要把：

```text
Agent 推測
```

直接當成：

```text
環境事實
```

---

### Unknown 是合法狀態

Agent 不知道某件事情時，可以是 Unknown。

例如：

```text
Unknown:
尚未確定程式為什麼失敗。
```

正確做法是：

```text
Unknown
 ↓
調查
 ↓
取得 Observation
 ↓
形成 Hypothesis
 ↓
驗證 Hypothesis
```

而不是直接猜答案。

---

### Hypothesis 不等於 Fact

Agent 可以提出：

```text
Hypothesis:
可能是缺少某個 dependency。
```

但只有取得證據後，才能提高可信度。

因此系統應該保留：

```text
Observation
Evidence
Hypothesis
Decision
Result
Verification
```

之間的區別。

---

### Failure 不代表任務結束

一次 Action 失敗，只代表：

> 這個方法沒有成功。

不代表：

> 使用者的目標不可能完成。

例如：

```text
cargo build
    ↓
失敗
    ↓
Observation
    ↓
分析錯誤
    ↓
修改程式
    ↓
重新 build
```

Agent 應該能根據結果改變策略。

---

### 最終驗證的是 Goal，不是最後一個 Action

例如使用者要求：

> 修復程式並讓測試全部通過。

即使：

```text
write_file → 成功
```

也不能代表任務完成。

真正需要驗證：

```text
程式是否修復？
測試是否通過？
原始 Goal 是否達成？
```

所以：

```text
Action Success
≠
Goal Success
```

---

## 3. Agent Decision 與 Runtime Action

目前採用的概念分離：

### AgentDecision

```text
Observe
Act
Wait
Finish
```

`Finish` 再帶有最終結果：

```text
Done
Blocked
Impossible
NeedUser
```

---

### Runtime Action

```text
Observe
Execute
Interact
Wait
```

AgentDecision 的：

```text
Act
```

代表：

> Agent 決定要採取行動。

真正執行時，再轉換成 Runtime 可以理解的 Action。

Verification 是 Agent Core 的行為，不是一種獨立 Runtime Action。

如果 Verification 需要操作電腦，仍然透過：

```text
Observe
Execute
Interact
Wait
```

完成。

---

## 4. Runtime 的定位

Runtime 是 Agent 操作外部世界的能力層。

例如：

```text
Filesystem
Process
Shell
GUI
Browser
Network
System
Jobs
```

Runtime 不應該替 Agent 決定：

```text
下一步該做什麼
```

它只負責：

```text
收到 Action
→ 執行
→ 回傳 Result / Observation
```

例如：

```text
Agent:
執行 cargo test

Runtime:
啟動程序

Environment:
cargo test 執行失敗

Runtime:
回傳 exit code / stdout / stderr

Agent:
分析失敗原因
```

---

## 5. Engineering Runtime

這個 Agent 特別需要具備軟體工程能力。

Engineering Runtime 不是一套固定 workflow，而是一組能力。

例如：

```text
Repository exploration
Code search
Symbol / reference discovery
Dependency inspection
File editing
Patch
Diff
Git
Build
Test
Static analysis
Debugging
Checkpoint / Rollback
```

Agent 可以自行決定如何組合這些能力。

例如修 Bug 時，不應該硬編碼成：

```text
先搜尋
→ 修改
→ build
→ test
```

而應該讓 Agent 根據 Observation 決定下一步。

---

## 6. 長時間任務

有些工作不是一次 Action 就能完成。

例如：

```text
編譯大型專案
執行測試
影片轉碼
模型下載
長時間 Python 程式
```

因此 Runtime 需要支援 Job 概念。

基本概念：

```text
Start Job
   ↓
JobId
   ↓
Running
   ↓
Observe / Wait / Inspect / Cancel
   ↓
Succeeded / Failed / TimedOut / Cancelled / Unknown
```

Timeout 本身只是 Observation。

Agent 可以決定：

```text
等待
檢查
停止
重新執行
改變方法
```

而不是 Runtime 一看到 Timeout 就直接宣判任務失敗。

---

## 7. Permission 與安全邊界

需要區分三個概念：

```text
Capability
= 系統技術上能做什麼

Policy
= Agent 現在是否應該 / 可以做

OS Security
= Windows 實際允不允許
```

例如：

```text
AccessDenied
```

是一個環境結果。

而：

```text
NeedUser
```

代表真正需要使用者介入，例如：

* 使用者提供資訊
* 使用者確認某個操作
* 使用者進行 GUI 操作
* Windows Secure Desktop / UAC 等 Agent 無法直接跨越的界線

不能把「我不知道」濫用成 `NeedUser`。

---

## 8. Observation 與 Context

所有重要 Observation 應該可以被保存。

但不能把所有歷史資料永遠完整塞進 LLM Context。

因此分成：

```text
Observation Store
        ↓
Context Compiler
        ↓
LLM Context
```

Observation Store：

> 保存完整歷史。

Context Compiler：

> 根據目前任務選擇真正需要的資訊，並適當壓縮舊歷史。

重要原則：

> 不應該因為壓縮歷史而遺失決策所需要的證據。

---

## 9. Loop Detection

Agent 必須能發現自己可能陷入循環。

例如：

```text
Action A
→ Failure
→ Action A
→ Failure
→ Action A
→ Failure
```

不能只是無限重試。

Loop Detection 應該考慮：

```text
Action similarity
Environment delta
Progress
Repeated failures
```

發現循環後，Agent 應該嘗試：

```text
改變策略
取得更多資訊
降低操作範圍
嘗試其他方法
```

而不是由 Loop Detection 自己決定：

```text
Done / Impossible
```

---

## 10. Persistence 與 Recovery

Agent 是長時間工作的系統，因此不能假設：

> 程式永遠不會崩潰。

需要保存足夠的狀態，使 Agent 能夠在中斷後恢復。

需要考慮：

```text
Checkpoint
Task State
Job State
Action Record
Observation
Progress
Recovery
Rollback
```

如果某個操作已經執行，Agent 恢復後不能因為不知道歷史而重複執行危險操作。

---

## 11. LLM 的定位

LLM 是 Agent 的 reasoning / decision 元件之一。

但 LLM 不是 Runtime。

基本資料流：

```text
User Goal
    ↓
Agent State
    ↓
Context Compiler
    ↓
LLM Provider
    ↓
Parsed AgentDecision
    ↓
Agent Core Validation / Policy
    ↓
Runtime Action
    ↓
Action Result
    ↓
Observation
    ↓
Agent State
    ↓
下一輪
```

LLM 輸出必須經過：

```text
Parse
→ Validate
→ Normalize
→ Execute
```

不能直接把原始 LLM 輸出當成可信的 Runtime 指令。

---

## 12. 不保存 Chain of Thought

系統不需要保存模型的隱藏思考過程。

應保存可驗證的工作資訊：

```text
Decision
Action
Observation
Evidence
Hypothesis
Result
Verification
```

目標是：

> 可以理解 Agent 做了什麼、看到了什麼、得到什麼結果，以及最後如何確認任務完成。

而不是保存模型的內部 Chain of Thought。

---

## 13. 專案文件

目前文件的分工：

```text
README.md
→ 給人看的專案入口

AGENTS.md
→ 給 Codex / Coding Agent 的工作規則

01_REQUIREMENTS.md
→ 系統需求與設計目標

02_CONSTRUCTION.md
→ 實作與建置方向

03_INTERFACES.md
→ 模組與資料介面

04_ACCEPTANCE_TESTS.md
→ 驗收條件

05_TEST_PLAN.md
→ 測試策略與測試計畫
```

README 不應該取代這些文件。

詳細規格以 `docs/` 為準。

如果 README 與規格文件內容不一致，應優先修正 README，而不是讓 README 變成第二份規格。

---

## 14. 開發原則

修改 Agent v3 時，優先遵守：

1. 不要為了讓目前的 LLM 看起來能工作，就把架構硬編碼成目前模型的能力。

2. 不要讓 Runtime 決定 Agent 的高階工作流程。

3. 不要把一次 Action 失敗直接等同於任務失敗。

4. 不要把 Hypothesis 當成 Fact。

5. 不要因為不知道答案就直接要求使用者介入。

6. 不要用固定 workflow 限制 Agent 解決未來未知類型的任務。

7. 不要讓新增一個 Runtime capability 就迫使 Agent Core 大幅修改。

8. 優先建立清楚的 Interface，而不是過早加入複雜功能。

9. 優先證明行為正確，而不是只追求程式可以編譯。

10. 修改架構時，同時檢查 Requirements、Interfaces、Acceptance Tests 與 Test Plan 是否仍然一致。

---

## 15. 第一階段不追求的東西

v3 第一版不需要一次解決所有問題。

暫時不是主要目標：

```text
Multi-Agent
Vector Database
大型 RAG 系統
巨大 Planner Tree
固定 Workflow DSL
大量專用 AI Tools
完美 GUI Automation
完美 Browser Automation
無限制權限提升
完美 World Model
完美 Long-Term Memory
```

先建立可靠的：

```text
Observe
→ Reason
→ Decide
→ Act
→ Observe
→ Adapt
→ Verify
```

再逐步擴充。

---

## 16. 開發與驗證

基本 Rust 驗證：

```powershell
cargo fmt --check
cargo check
cargo test
```

修改完成後還需要根據變更範圍執行：

```text
相關 Unit Tests
Integration Tests
Runtime Tests
Engineering Runtime Tests
Acceptance Tests
```

最後檢查：

```powershell
git diff
```

確認沒有不必要的修改。

完整測試策略請看：

```text
docs/05_TEST_PLAN.md
```

---

## 17. 如果未來忘記這個專案在做什麼

最簡單的理解方式：

> 這不是「讓 AI 呼叫幾個工具」。

而是：

> 建立一個可以透過觀察電腦環境、採取行動、取得結果、修正策略，最後自行驗證使用者目標是否完成的通用 Agent。

最核心的抽象只有幾個：

```text
Goal
Observation
Hypothesis
Decision
Action
Result
Verification
State
```

其中：

```text
Agent Core
= 思考、判斷、決策、適應、驗證

Runtime
= 操作電腦

Observation
= 環境回傳的證據

Persistence
= 讓任務能持續存在

Verification
= 判斷真正的 Goal 是否完成
```

如果這幾個概念始終保持清楚，之後增加 Windows、Shell、GUI、Browser、Git、編譯、測試或其他能力，都只是擴充 Agent 的「手腳」，而不是重新設計 Agent 的「大腦」。
