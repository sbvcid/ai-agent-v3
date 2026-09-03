# AI Agent v3 — Core Interfaces Specification

## 1. Purpose

本文件定義 AI Agent v3 的核心施工介面與責任邊界。

本文件不規定 Rust 的具體：

* struct
* enum
* trait
* crate
* async runtime
* database
* serialization format
* module layout

而是定義系統必須存在的核心語意、資料流與行為。

實作者可以自行選擇合理的 Rust implementation，只要不破壞本文件與其他需求文件定義的行為。

核心目標是建立：

```text
User Goal
    ↓
Agent Core
    ↓
Decision
    ↓
Runtime Action
    ↓
Environment
    ↓
Observation
    ↓
Agent State
    ↓
Agent Core
```

這是一個閉環 Agent，而不是固定 Workflow Engine。

---

# 2. Core Architecture

Agent v3 的核心資料流：

```text
User Goal
    ↓
Agent Core
    ↓
AgentDecision
    ↓
Action
    ↓
Runtime
    ↓
ActionResult / Observation
    ↓
Observation Store
    ↓
Agent State
    ↓
Context Compiler
    ↓
LLM
    ↓
Next AgentDecision
```

完整工作循環：

```text
Goal
 ↓
Observe
 ↓
Understand
 ↓
Hypothesize
 ↓
Decide
 ↓
Act / Wait
 ↓
Observe Result
 ↓
Evaluate
 ↓
Revise
 ↓
Act Again / Observe More / Wait
 ↓
Verify
 ↓
Finish
```

Agent 不應被實作成：

```text
Goal
 ↓
固定 Workflow
 ↓
Tool A
 ↓
Tool B
 ↓
Tool C
 ↓
Done
```

Agent 必須能根據新的 Observation 改變後續決策。

---

# 3. Core Concepts

系統至少必須具有以下語意概念：

```text
Goal
AgentState
AgentDecision
Action
ActionResult
Observation
ObservationStore
Job
VerificationState
TaskStatus
```

Engineering Runtime 相關能力也屬於 Runtime Capability 的一部分：

```text
Repository
Code Search
Symbol / Reference Search
Code Editing
Diff
Build
Test
Static Analysis
Debugging
Git
Checkpoint
Rollback
```

這些概念不必一一對應到 Rust type，但其語意必須存在。

---

# 4. Agent State

AgentState 表示 Agent 對目前任務的工作狀態。

至少必須能表達：

```text
Goal
Current Understanding
Environment State
Active Problems
Unknowns
Hypotheses
Evidence
Recent Actions
Recent Observations
Running Jobs
Verification State
Remaining Work
Task Status
```

概念上：

```text
AgentState
├── Goal
├── Understanding
├── EnvironmentState
├── ActiveProblems
├── Unknowns
├── Hypotheses
├── Evidence
├── RecentActions
├── RecentObservations
├── RunningJobs
├── VerificationState
├── RemainingWork
└── Status
```

具體資料結構由實作者決定。

---

# 5. Goal

Goal 是使用者提供的原始目標。

Goal 必須在整個 Task Lifecycle 中持續存在。

Agent 可以建立：

```text
subtask
intermediate objective
working hypothesis
```

但不得因此失去對 Original Goal 的追蹤。

所有最終 Verification 都必須回到 Original Goal。

---

# 6. Understanding

Understanding 表示 Agent 目前對任務與環境的工作理解。

Understanding 是可變的。

新的 Observation 可以：

```text
confirm
contradict
refine
replace
```

目前 Understanding。

Agent 不得將早期推測視為永久事實。

---

# 7. Knowledge State

Agent Core 必須能區分不同程度的知識確定性。

概念上至少包括：

```text
Fact
Observed
Inferred
Hypothesis
Unknown
```

這些不是互斥的 Runtime Observation Type，而是 Agent 對資訊的認知狀態。

例如：

```text
Observation:
cargo exited with code 101

Observed:
cargo build failed

Inferred:
the project currently does not compile

Hypothesis:
a dependency version may be incompatible

Unknown:
whether changing the dependency will solve the problem
```

規則：

> Hypothesis 不得自動升級為 Fact。

只有新的可靠 Evidence 支持時，Agent 才能提高其可信程度。

---

# 8. Evidence

Evidence 是支持或反駁 Agent 判斷的資訊。

Evidence 應能追溯到：

```text
Observation
ActionResult
JobResult
Environment State
Verification Result
```

Agent 應盡可能保留 Evidence 的來源。

例如：

```text
Hypothesis:
FFmpeg is not installed.

Evidence:
command lookup returned "not recognized"
```

這比只保存：

```text
FFmpeg missing
```

更有價值。

---

# 9. Observation

Observation 是 Agent 從 Environment 或 Runtime 獲得的環境證據。

Observation 可以來自：

```text
Filesystem
Process
Shell
GUI
Browser
Network
System
Job
Engineering Runtime
Action Result
```

Observation 不等於：

```text
Agent interpretation
```

例如：

```text
Observed:
cargo returned exit code 101

Inferred:
build failed
```

後者是 Agent Core 的理解，而不是 Runtime 原始 Observation。

---

# 10. Observation Store

Observation Store 負責保存 Agent 可用的環境證據。

至少需要支援概念上的：

```text
append observation
retrieve observation
query observation
filter observation
associate observation with task
associate observation with action
associate observation with job
associate observation with resource
retrieve recent observations
retrieve relevant historical observations
```

具體 API 與資料庫方式由實作者決定。

Observation Store 應能保存比單次 LLM Context 更多的資訊。

---

# 11. Observation Types

系統至少應支援以下 Observation 類型：

```text
EnvironmentObservation
FilesystemObservation
ProcessObservation
ActionResultObservation
VisualObservation
BrowserObservation
EnvironmentDelta
JobObservation
EngineeringObservation
```

實際 Rust type 不必使用上述名稱。

---

# 12. Environment Observation

EnvironmentObservation 可描述：

```text
OS
architecture
current user
drives
CPU
RAM
GPU
VRAM
installed software
runtime versions
environment variables
PATH
network state
desktop session
available permissions
```

環境資訊應具有合理的 Cache / Refresh 機制。

不應在每一個 Decision Cycle 無條件重新掃描完整環境。

---

# 13. Filesystem Observation

FilesystemObservation 可描述：

```text
file existence
directory existence
metadata
directory contents
file size
timestamps
content
hash
permissions
filesystem errors
```

大型資料應支援：

```text
targeted inspection
pagination
depth limit
item limit
summary
diff
```

不得預設遞迴掃描整個磁碟。

---

# 14. Process Observation

ProcessObservation 可描述：

```text
PID
parent PID
command
arguments
status
start time
runtime
CPU usage
memory usage
stdout
stderr
exit code
window state
```

Agent 啟動的 Process / Job 應具有可追蹤識別。

完整 stdout / stderr 可以保存於持久化儲存，但送入 LLM Context 時應受 Context Budget 控制。

---

# 15. ActionResult Observation

ActionResult 是 Runtime 對 Agent Action 的客觀回報。

例如：

```text
Action:
execute cargo build

ActionResult:
exit code = 101
stderr = ...
duration = ...
```

ActionResult 可以進一步產生：

```text
Observation
EnvironmentDelta
JobObservation
```

ActionResult 不代表 Goal 是否完成。

---

# 16. Visual Observation

VisualObservation 可以描述：

```text
screenshot
window
visible UI
UI Automation information
dialog
application state
screen change
```

GUI Runtime 優先使用結構化 UI Automation。

Vision / coordinate interaction 可以作為 fallback。

---

# 17. Browser Observation

BrowserObservation 可以描述：

```text
URL
page title
visible text
DOM state
interactable elements
loading state
navigation state
download state
error state
```

Browser-level information 應優先於單純 screenshot。

GUI 可以作為 Browser Automation 的 fallback。

---

# 18. Environment Delta

EnvironmentDelta 描述 Environment 在 Action 前後的重要變化。

例如：

```text
Before:
ffmpeg unavailable

Action:
install ffmpeg

After:
ffmpeg available
```

Delta 對以下判斷非常重要：

```text
progress
side effect
verification
recovery
loop detection
```

不要求每個 Action 都產生完整 Environment Snapshot。

---

# 19. Action

Action 表示 Agent 要求 Runtime 對 Environment 採取的操作。

Action 至少具有以下語意：

```text
Action ID
Action Type
Parameters
Intent
Expected Effect
```

概念：

```text
Action
├── id
├── type
├── parameters
├── intent
└── expected_effect
```

Agent Core 決定：

```text
What should happen?
```

Runtime 負責：

```text
How to execute it reliably?
```

---

# 20. Action Abstraction

Runtime-facing Action 應保持低數量、高表達力。

核心 Action 類型至少包括：

```text
Observe
Execute
Interact
Wait
```

其中：

### Observe

取得 Environment 或 Resource 的資訊。

例如：

```text
inspect file
inspect process
inspect environment
inspect repository
inspect browser
inspect GUI
```

### Execute

執行一般可程式化操作。

例如：

```text
run command
modify file
copy file
delete file
start program
run build
run test
```

### Interact

處理需要互動的環境。

例如：

```text
GUI click
keyboard input
interactive terminal input
browser interaction
```

### Wait

等待 Job、Process、Browser 或其他外部狀態發生變化。

---

# 21. AgentDecision

AgentDecision 是 Agent Core 對「下一步做什麼」的決策。

核心 Decision 類型統一為：

```text
Observe
Act
Wait
Finish
```

概念：

```text
AgentDecision
├── decision type
├── action / request
├── intent
└── expected progress
```

其中：

```text
Observe
```

代表需要取得更多資訊。

```text
Act
```

代表要執行一個 Action。

```text
Wait
```

代表目前最合理的下一步是等待環境或 Job 發生變化。

```text
Finish
```

代表 Agent 認為任務已達到最終狀態。

---

# 22. Finish Status

`Finish` 必須帶有最終 Task Status。

概念上：

```text
Done
Blocked
Impossible
NeedUser
```

這四種才是 Task 的最終結果。

`Running` 是執行中的 Task Status，不應與上述終止結果混淆。

`Continue` 可以作為一般語意描述：

```text
Agent should continue
```

但不應作為必要的 AgentDecision 類型。

---

# 23. Task Status

概念上：

```text
Running
Done
Blocked
Impossible
NeedUser
```

### Running

Task 尚未進入最終狀態。

### Done

有足夠 Evidence 證明 Original Goal 已完成。

### Blocked

目前存在阻礙，使 Task 無法繼續有效進展。

阻礙可能是：

```text
external service unavailable
hardware unavailable
required resource temporarily unavailable
environmental condition
```

### Impossible

根據目前 Environment、Permission、Available Capability 與可靠 Evidence，可以判定 Goal 無法完成。

### NeedUser

必須由使用者提供 Agent 無法自行取得的資訊、授權或互動。

例如：

```text
password
specific external information
physical interaction
Secure Desktop interaction
```

缺少一般軟體或 dependency 不應直接產生 NeedUser。

---

# 24. Verification Is Not an Action

Verification 不屬於 Runtime Action。

Verification 是 Agent Core 的工作。

如果 Verification 需要：

```text
read file
run test
inspect process
decode output
inspect GUI
query browser
```

Agent 可以透過：

```text
Observe
Execute
Interact
```

取得 Verification Evidence。

因此：

```text
Verify
```

不是獨立 Runtime Action。

---

# 25. Verification State

VerificationState 表示目前 Goal 的驗證進度。

至少能表示：

```text
NotVerified
Verifying
Verified
Failed
Uncertain
```

Verification 必須針對：

```text
Original Goal
```

而不是只針對最後一個 Action。

---

# 26. Verification Levels

系統必須區分：

```text
Action Success
Subtask Success
Goal Success
```

例如：

```text
ffmpeg command succeeded
        ↓
output file exists
        ↓
output file can be decoded
        ↓
output satisfies requested format
```

最後一層才可能是 Goal Success。

---

# 27. Verification Evidence

Agent 宣稱 `Done` 前，應具有與 Goal 相關的 Evidence。

Verification Evidence 可以來自：

```text
exit code
file state
file content
test result
build result
process state
expected output
browser state
GUI state
runtime observation
semantic evaluation
```

如果證據不足：

```text
Do not claim Done.
```

Agent 可以：

```text
Observe more
Execute verification
Change strategy
Ask User
```

---

# 28. Job

Job 表示可能持續較長時間的 Runtime 工作。

例如：

```text
video encoding
large file copy
download
installation
build
test suite
model conversion
long-running process
```

Job 至少需要能表達：

```text
Job ID
Associated Action
Status
Start Time
Current State
Progress if available
Result
```

可額外保存：

```text
PID
process identifier
logs
resource usage
completion time
error
checkpoint information
```

---

# 29. Job Lifecycle

Job 至少應支援：

```text
Start
Observe
Wait
Complete
Fail
Cancel
Timeout
Retrieve Result
```

概念流程：

```text
Agent
 ↓
Start Job
 ↓
Job Running
 ↓
Observe
 ↓
Wait / Continue Other Work
 ↓
Observe
 ↓
Completed / Failed / Cancelled
 ↓
Result
```

Agent 不應因為單一長時間 Job 尚未完成而阻塞整個 Agent Runtime。

---

# 30. Job Recovery

Agent Crash 後，是否可以重新取得 Job 狀態取決於該 Job 的可恢復性。

系統應盡可能保存：

```text
Job ID
PID / process identifier
command
arguments
start time
status
logs
result
```

如果 Process 在 Agent Restart 後仍存在，Runtime 應嘗試重新觀察。

如果 Job 已經消失，Runtime 應回報：

```text
job unavailable
```

而不是假設 Job 仍在執行。

Agent Core 再決定：

```text
retry
recover
inspect
resume through checkpoint
or change strategy
```

---

# 31. ActionResult

ActionResult 是 Runtime 對單次 Action 的執行回報。

至少需要：

```text
Action ID
Execution Status
Output
Error
Duration
Produced Observations
Environment Changes
```

概念：

```text
ActionResult
├── action_id
├── status
├── output
├── error
├── duration
├── observations
└── environment_delta
```

大型 Output 不應強迫完整進入 LLM Context。

---

# 32. Action Execution Status

ActionResult 至少能區分：

```text
Success
Failed
Timeout
Cancelled
Blocked
Started
Running
Unknown
```

`Success` 僅表示 Runtime 成功執行該 Action。

不代表：

```text
Subtask Success
```

更不代表：

```text
Goal Success
```

---

# 33. Runtime Capability

Runtime 提供 Agent 可以使用的 Computer Capability。

至少涵蓋：

```text
Filesystem
Process
Shell
Interactive Shell
GUI
Browser
Network
System / Environment
Long-running Jobs
Engineering Runtime
```

Capability 的數量與內部實作方式不固定。

Runtime 可以將多種 Windows API、Process、Library 或工具組合成一個 Capability。

---

# 34. Runtime Capability Discovery

Agent Core 應能知道目前 Runtime 提供哪些 Capability，以及 Capability 的基本限制。

例如：

```text
GUI available
Interactive Desktop unavailable
Browser available
Network available
Python available
Cargo unavailable
```

Capability Discovery 不應被硬編碼成固定工具清單。

Runtime 應描述：

```text
what is currently possible
```

Agent 再決定：

```text
what should be done
```

---

# 35. Runtime Responsibility

Runtime 負責：

```text
execute Action
observe Environment
normalize results
manage processes
manage jobs
manage filesystem
manage GUI
manage browser
handle OS interaction
handle timeout
report errors
```

Runtime 不負責：

```text
理解 User Intent
制定完整問題解法
建立固定 Workflow
診斷所有問題
決定 Hypothesis 是否正確
判斷 Original Goal 是否完成
```

---

# 36. Agent Core Responsibility

Agent Core 負責：

```text
Goal understanding
Observation interpretation
State maintenance
Hypothesis generation
Action selection
Information gathering
Strategy adaptation
Progress evaluation
Failure recovery
Verification
Termination decision
```

Agent Core 不應依賴某一個特定 Runtime implementation。

---

# 37. Capability ≠ Policy

Runtime Capability 與 Agent Policy 必須分離。

概念：

```text
Capability:
What can technically be done?

Policy:
What should be done now?
```

另外存在第三層：

```text
OS / Security Boundary:
What does Windows actually permit?
```

例如：

```text
Runtime Capability:
delete file

Policy:
Agent may decide whether deletion is appropriate

Windows:
Access Denied
```

`Access Denied` 是環境／權限 Observation，不代表 Agent 可以自行突破 OS 權限。

---

# 38. Context Boundary

Observation Store 與 LLM Context 必須分離。

```text
Environment
    ↓
Observation Store
    ↓
Context Compiler
    ↓
Relevant Context
    ↓
LLM
```

Context Compiler 負責：

```text
relevance filtering
recency
task relation
hypothesis relation
recent failures
environment changes
verification state
running jobs
historical evidence
compression
deduplication
```

Context Compiler 不負責：

```text
high-level strategy
problem solving
Goal interpretation
```

---

# 39. Context Compiler Requirements

Context Compiler 應能：

```text
select relevant observations
compress old history
truncate large output
preserve critical evidence
extract diffs
deduplicate repeated information
retain active hypotheses
retain important failures
retain verification evidence
```

完整 Observation 應保存在 Observation Store。

Context Compiler 只是決定：

```text
what should be shown now
```

不得因 Context 壓縮而任意丟失對目前 Goal 有決定性影響的 Evidence。

---

# 40. Action History

系統必須保存 Action History。

至少包含：

```text
Action
Action Result
Related Observation
Environment Change
Timestamp
```

可額外包含：

```text
Job ID
Precondition
Undo Information
Environment Snapshot
```

Action History 用於：

```text
debugging
recovery
loop detection
progress detection
reasoning
verification
persistence
```

---

# 41. Loop Detection

系統必須能偵測可能的無效循環。

重要條件包括：

```text
same / similar Action
+
same / similar Environment
+
same / similar Result
+
no meaningful progress
```

例如：

```text
Action A
 ↓
Failure
 ↓
Action A
 ↓
Failure
 ↓
Action A
```

不應無限重複。

---

# 42. Loop Recovery

偵測到無效循環後，Agent 應：

```text
Observe more
or
change Hypothesis
or
change Action
or
reduce uncertainty
or
Finish as Blocked / Impossible
```

Loop Detection 不應只依賴：

```text
retry count
```

可以使用：

```text
Action similarity
Observation similarity
Environment Delta
Progress
time
resource state
```

等資訊。

具體演算法由實作者決定。

---

# 43. Persistence

以下資訊應具有持久化能力：

```text
Goal
Agent State
Action History
Important Observations
Running Jobs
Verification State
Checkpoint Information
```

核心行為：

```text
Agent Process Crash
        ↓
Restart
        ↓
Load Task State
        ↓
Recover Observable Jobs
        ↓
Resume
```

具體儲存方式由實作者決定。

例如：

```text
SQLite
JSON
JSONL
binary storage
```

長時間 Task 不得只依賴 RAM 中的暫時狀態。

---

# 44. Persistence Semantics

Persistence 不要求每一個瞬間都必須同步寫入磁碟。

但在合理的 Task Lifecycle 邊界上，系統必須保存足夠資訊以避免：

```text
Agent Crash
→ complete loss of task state
```

對重要修改，應能保存：

```text
checkpoint
undo information
diff
rollback information
```

具體方式由實作者決定。

---

# 45. Engineering Runtime

Engineering Runtime 是 Computer Runtime 的專業能力集合，用於處理 Software Engineering Goal。

它不是固定 Coding Workflow。

至少應提供以下能力：

```text
Repository Discovery
Repository Structure Observation
Code Search
Symbol Search
Reference Search
Dependency Discovery
Code Reading
Code Editing
Patch
Diff
Build
Test
Static Analysis
Debugging
Git State
Git Diff
Checkpoint
Rollback
```

Agent 可以自由決定使用順序。

例如：

```text
先搜尋
→ 讀取
→ Build
→ 修改
→ Test
```

或：

```text
先 Build
→ 分析錯誤
→ 搜尋 Symbol
→ 修改
→ Test
```

均屬合法策略。

---

# 46. Engineering Observations

Engineering Runtime 應能產生：

```text
RepositoryObservation
CodeSearchObservation
SymbolObservation
ReferenceObservation
DependencyObservation
BuildObservation
TestObservation
GitObservation
DiffObservation
CodeChangeObservation
DebugObservation
```

這些 Observation 應能被：

```text
AgentState
Evidence
Context Compiler
Verification
Loop Detection
```

使用。

---

# 47. Engineering State

Agent State 應能保存與目前工程問題相關的資訊，例如：

```text
Current Repository
Relevant Files
Relevant Symbols
Relevant Dependencies
Current Hypotheses
Changed Files
Previous Changes
Build Results
Test Results
Known Failures
Previous Failed Approaches
Current Verification State
```

不要求所有 Repository 資訊永久存在於 LLM Context。

Context Compiler 應依目前問題選擇相關內容。

---

# 48. Engineering Runtime Boundary

Engineering Runtime 可以提供：

```text
repository inspection
code search
code editing
build
test
diff
git
debugging
checkpoint
rollback
```

但不應替 Agent 決定：

```text
哪個 Bug 是真正原因
應該修改哪個檔案
哪個 Hypothesis 最正確
何時停止修改
Goal 是否已完成
```

Engineering Runtime 提供能力。

Agent Core 負責推理與決策。

---

# 49. LLM Interface

LLM Provider 必須與 Agent Core 解耦。

至少支援：

```text
Ollama
```

未來可以增加：

```text
其他 Local LLM
Remote LLM
不同 Model
不同 Provider
```

Agent Core 不應直接依賴 Ollama 特定 API。

概念介面：

```text
Agent Core
    ↓
LLM Provider Interface
    ↓
Ollama / Other Provider
```

---

# 50. Structured Decision Output

LLM 應優先以結構化格式輸出 AgentDecision。

概念上：

```json
{
  "type": "observe",
  "request": {}
}
```

或：

```json
{
  "type": "act",
  "action": {}
}
```

或：

```json
{
  "type": "wait",
  "request": {}
}
```

或：

```json
{
  "type": "finish",
  "status": "done"
}
```

實際 JSON schema 由實作者設計，但必須能表達：

```text
Observe
Act
Wait
Finish
```

以及必要的參數。

---

# 51. Decision Validation

LLM Output 不得直接執行。

必須經過：

```text
LLM Output
 ↓
Parse
 ↓
Schema Validation
 ↓
Semantic Validation
 ↓
Normalization
 ↓
Runtime
```

至少檢查：

```text
valid structure
known decision type
valid parameters
valid action type
required fields
resource references
basic safety / policy constraints
```

---

# 52. LLM Output Robustness

Local LLM 可能產生：

```text
invalid JSON
markdown wrapper
missing fields
unknown fields
truncated output
incorrect enum
malformed parameters
```

Parser 應提供合理容錯。

至少支援：

```text
markdown codeblock removal
schema validation
clear parse errors
correction request
bounded correction retry
```

可選擇支援：

```text
JSON repair
trailing comma correction
other lightweight normalization
```

但不得因輕微格式問題直接使整個 Task 終止。

---

# 53. LLM Correction

若 Decision 無法解析：

```text
Parse Failure
 ↓
Structured Error
 ↓
Correction Request
 ↓
LLM
 ↓
Parse Again
```

Correction Retry 必須有明確上限。

如果持續無法產生有效 Decision：

```text
Agent Core
```

應獲得該錯誤作為 Observation，並決定後續策略。

不得形成無限 Parser Retry Loop。

---

# 54. Decision Context

每次 Agent Decision 應能讓 LLM 取得與目前問題相關的：

```text
Original Goal
Current State
Relevant Environment
Relevant Observations
Recent Actions
Recent Failures
Active Hypotheses
Unknowns
Running Jobs
Verification State
Available Runtime Capabilities
```

不要求所有資訊每輪全部提供。

Context Compiler 決定 Relevant Context。

---

# 55. Runtime Result Normalization

Runtime 可以使用不同：

```text
Windows API
CLI
PowerShell
Process
Library
Browser automation
GUI automation
```

但回到 Agent Core 時，結果應轉換成一致的語意。

例如：

```text
Windows API error
PowerShell error
CLI exit code
```

最終都應能轉化成 Agent 可以理解的：

```text
ActionResult
Observation
Error
EnvironmentDelta
```

Agent 不應需要理解每一個底層 API 的內部錯誤格式。

---

# 56. Error Boundary

Runtime Error 是 Environment / Action 的結果。

例如：

```text
not_found
permission_denied
timeout
invalid_argument
process_crashed
network_error
interactive_input_required
environment_error
```

這些應回報給 Agent Core。

不得直接等同於：

```text
Task Failure
```

Agent Core 再判斷：

```text
retry
inspect
change strategy
wait
recover
Blocked
Impossible
NeedUser
```

---

# 57. Interactive Runtime Boundary

Interactive Runtime 包括：

```text
ConPTY
interactive shell
GUI
Browser
```

如果操作需要：

```text
stdin
password
confirmation
GUI interaction
Secure Desktop
```

Runtime 必須回報可觀察狀態。

例如：

```text
awaiting_input
interactive_required
secure_desktop
desktop_unavailable
```

Agent 不得因為沒有取得輸入而無限等待。

---

# 58. Permission Boundary

Runtime 使用目前 Windows User / Process 擁有的權限。

Runtime 不應假設：

```text
Administrator
SYSTEM
kernel access
```

如果 Windows 回報：

```text
Access Denied
```

Agent 可以調查替代方法，但不得假設可以突破 OS Security Boundary。

如果需要使用者完成無法由 Agent 自動完成的授權或 Secure Desktop 操作：

```text
NeedUser
```

可以成為最終結果。

---

# 59. Interface Stability

核心語意介面應盡量穩定。

未來如果：

```text
LLM 更強
Runtime 更強
增加新的 Browser backend
增加新的 GUI backend
增加新的 Process backend
```

不應需要重新設計 Agent Core 的基本閉環。

新的能力應優先透過：

```text
Capability
Observation
Action
```

加入，而不是建立大量新的 Agent-level special cases。

---

# 60. Implementation Freedom

以下由實作者自行決定：

```text
Rust crate
async/sync architecture
thread model
Tokio
struct layout
enum layout
trait design
serialization
database
logging framework
process implementation
ConPTY implementation
browser implementation
GUI implementation
HTTP implementation
snapshot implementation
context compression algorithm
```

只要滿足：

```text
Semantic Contract
Behavioral Requirements
Responsibility Boundaries
```

即可。

---

# 61. No Hard-Coded Workflow

Interface 不得要求 Agent 按照固定順序：

```text
Search
→ Read
→ Edit
→ Build
→ Test
```

也不得要求：

```text
Observe
→ Action A
→ Action B
→ Action C
```

固定流程只能存在於 Runtime 的必要可靠性機制，例如：

```text
parse
validate
execute
observe result
```

高層問題解決順序應由 Agent Core 根據 Observation 決定。

---

# 62. Interface Invariants

以下為不可破壞的核心不變量：

### Invariant 1

```text
Original Goal
```

必須在 Task Lifecycle 中持續存在。

### Invariant 2

```text
Observation
```

是 Environment Evidence。

### Invariant 3

```text
Hypothesis
```

不得自動視為 Fact。

### Invariant 4

```text
Action Success
```

不得自動視為 Goal Success。

### Invariant 5

```text
Runtime Failure
```

不得自動視為 Task Failure。

### Invariant 6

```text
AgentDecision
```

必須能根據新的 Observation 改變。

### Invariant 7

```text
LLM Output
```

不得直接執行。

### Invariant 8

```text
Verification
```

必須針對 Original Goal。

### Invariant 9

```text
Observation Store
```

與：

```text
LLM Context
```

必須分離。

### Invariant 10

```text
Runtime
```

不得取代 Agent Core 進行高層問題求解。

---

# 63. Final Interface Model

Agent v3 的核心關係：

```text
                    ┌──────────────────┐
                    │    User Goal     │
                    └────────┬─────────┘
                             ↓
                    ┌──────────────────┐
                    │    Agent Core    │
                    │                  │
                    │ Understand       │
                    │ Hypothesize      │
                    │ Decide           │
                    │ Adapt            │
                    │ Verify           │
                    └────────┬─────────┘
                             ↕
                    ┌──────────────────┐
                    │   Agent State    │
                    │                  │
                    │ Goal             │
                    │ Understanding    │
                    │ Hypotheses       │
                    │ Evidence         │
                    │ Jobs             │
                    │ Verification     │
                    └────────┬─────────┘
                             ↕
                    ┌──────────────────┐
                    │ Observation Store│
                    └────────┬─────────┘
                             ↕
                    ┌──────────────────┐
                    │     Runtime      │
                    │                  │
                    │ Filesystem       │
                    │ Process          │
                    │ Shell            │
                    │ GUI              │
                    │ Browser          │
                    │ Network          │
                    │ System           │
                    │ Engineering      │
                    │ Jobs             │
                    └────────┬─────────┘
                             ↓
                    ┌──────────────────┐
                    │     Windows      │
                    └──────────────────┘
```

LLM 位於 Agent Core 的推理實作之中：

```text
Agent Core
    ↓
Context Compiler
    ↓
LLM Provider
    ↓
AgentDecision
    ↓
Validation
    ↓
Runtime Action
```

核心閉環：

```text
GOAL
 ↓
OBSERVE
 ↓
UNDERSTAND
 ↓
HYPOTHESIZE
 ↓
DECIDE
 ↓
ACT / WAIT
 ↓
OBSERVE RESULT
 ↓
EVALUATE
 ↓
REVISE
 ↓
ACT AGAIN
 ↓
VERIFY
 ↓
FINISH
```

最終介面原則：

```text
Action
    = Agent 對 Environment 採取的操作

Observation
    = Agent 從 Environment 取得的 Evidence

AgentState
    = Agent 對 Task 的工作狀態

AgentDecision
    = Agent 下一步的決策

ActionResult
    = Runtime 對 Action 的客觀回報

Job
    = 持續執行中的 Runtime 工作

VerificationState
    = Agent 對 Original Goal 的驗證狀態

Runtime Capability
    = Computer 能夠提供的操作能力
```

Agent v3 的核心不是：

```text
LLM → Tool → Result
```

而是：

```text
              ┌───────────────┐
              │  Agent Core   │
              └───────┬───────┘
                      ↕
              ┌───────────────┐
              │ Observation   │
              │ + Agent State │
              └───────┬───────┘
                      ↕
              ┌───────────────┐
              │   Runtime     │
              └───────┬───────┘
                      ↕
              ┌───────────────┐
              │  Environment  │
              └───────────────┘
```

這些語意與資料流構成 Agent v3 的核心施工介面。
