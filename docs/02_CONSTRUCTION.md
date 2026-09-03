# Rust AI Agent v3

## System Requirements & Implementation Specification

Phase 0 — Deterministic Agent Core

建立：

- Core Rust data structures
- AgentDecision
- Action
- Observation
- ActionResult
- Goal
- VerificationStatus
- AgentState
- Mock LLM
- Fake Runtime
- Structured Event Log

首先讓以下閉環可以在完全假的環境中運作：

Goal
→ Observe
→ Decision
→ Action
→ Observation
→ Evaluation
→ Revised Decision
→ Action
→ Verification
→ Finish


### 1. 文件目的

本文件定義 Rust AI Agent v3 的系統需求、架構邊界、核心資料模型、Runtime 能力、Agent 決策循環、Observation 系統、LLM 介面、Windows 整合方式、錯誤處理、狀態持久化與驗證機制。

本文件的目的不是描述一個固定 Workflow，而是建立一個可以讓 Local LLM 自主操作 Windows 電腦、觀察結果、根據結果修正策略並完成使用者目標的通用 Agent Runtime。

Codex 或其他實作者應將本文件視為施工規格。

若本文件明確定義「必須達成的行為」，實作者不得任意降低該能力。

若本文件只定義目標、責任邊界或介面語意，而未指定具體實作方式，實作者可以自行選擇合理的 Rust crate、Windows API、資料結構、非同步模型、儲存方式與內部演算法，但不得破壞本文件定義的外部行為。

本文件中的資料結構與 Rust code 主要用於描述語意。除非明確標示為 Interface Requirement，否則不要求完全按照範例實作。

---

# 2. 施工原則

## 2.1 不得以目前模型能力限制架構

實作時不得假設目前本地模型的能力就是 Agent v3 的最終能力。

不得因目前模型較弱，而將大量問題解決流程硬編碼成：

```text
固定 Workflow
固定 Planner Tree
固定錯誤處理流程
固定問題解法
```

Runtime 應提供通用、可組合的能力，讓 Agent Core 根據 Observation 自行決定如何使用。

未來若替換更強的本地模型，應能在不大幅修改 Runtime 核心架構的情況下獲得更強的自主問題解決能力。

這並不表示 Agent Core 與 LLM 必須完全沒有介面耦合。

系統可以定義穩定的：

```text
Decision Schema
Action Schema
Observation Schema
Context Schema
```

但不得將目前某一模型的能力限制硬編碼成整個 Runtime 的工作流程。

---

## 2.2 優先建立能力，而不是流程

Agent v3 應優先提供：

```text
Agent 可以做什麼
```

而不是：

```text
Agent 遇到某問題時必須怎麼做
```

固定流程只能用於：

```text
Runtime reliability
resource management
error normalization
safety boundary
timeout
cancellation
persistence
verification
```

問題理解、策略選擇、行動順序與策略變更，應盡可能由 Agent Core 根據 Observation 自主決定。

---

## 2.3 實作自由

除非本文件明確要求，否則 Codex 不應被要求使用特定：

```text
crate
module layout
async runtime
database
IPC mechanism
PTY implementation
browser library
GUI automation library
JSON repair library
compression algorithm
```

Codex 可以選擇合理實作方式。

如果實作過程發現規格之間存在衝突，應優先保持：

```text
Architecture Boundary
Behavioral Requirement
Interface Contract
```

並在必要時修改實作，而不是透過增加特殊 case 解決規格矛盾。

---

# 3. System Goal

Agent v3 的核心目標是：

> 使用者提供一個自然語言 Goal，Agent 能夠自主觀察目前 Windows 環境，決定下一個合理行動，執行該行動，觀察結果，根據結果修正自己的理解與策略，持續工作直到 Goal 完成、確定無法完成，或確實需要使用者提供外部資訊。

核心模型：

```text
User Goal
    ↓
Agent Core
    ↕
Observation / State
    ↕
Computer Runtime
    ↕
Windows / Applications / External Environment
```

Agent Core 負責：

```text
理解
推理
假設
決策
策略調整
進度判斷
驗證
```

Runtime 負責：

```text
執行
觀察
資源管理
錯誤標準化
結果回報
```

---

# 4. Agent Decision Model

Agent Core 與 Runtime 之間應明確區分：

```text
Agent Decision
```

與：

```text
Runtime Action
```

Agent Decision 是「Agent 決定下一步做什麼」。

Runtime Action 是「Runtime 實際執行什麼操作」。

概念上：

```rust
enum AgentDecision {
    Observe(...),
    Act(...),
    Wait(...),
    Finish(...),
}
```

其中：

```text
Observe
    要求取得更多資訊

Act
    執行一個 Runtime Action

Wait
    暫時等待環境、Job 或外部條件變化

Finish
    宣布目前任務結果
```

`Finish` 必須包含：

```text
Done
Blocked
Impossible
NeedUser
```

具體 Rust enum、欄位名稱與 serialization 格式由實作者依照介面文件決定。

---

# 5. Runtime Action Model

Runtime 對外提供的 Action 概念應保持低數量、高表達力。

概念上：

```rust
enum Action {
    Observe(...),
    Execute(...),
    Interact(...),
    Wait(...),
}
```

語意：

### Observe

取得環境資訊、檔案資訊、Process 狀態、Browser 狀態等。

### Execute

執行具有明確輸入與目標的計算或系統操作。

例如：

```text
run program
shell command
filesystem operation
network request
system operation
```

### Interact

與互動式環境交互。

例如：

```text
interactive terminal
GUI
browser
```

### Wait

等待 Process、Job、Browser、外部條件或指定時間。

---

# 6. Verification Boundary

Verification 不應成為獨立的 Runtime Action。

Verification 是 Agent Core 的任務。

Agent 可以透過：

```text
Observe
Execute
Interact
Wait
```

取得驗證所需證據。

例如：

```text
Agent:
    Verify whether bug is fixed

Runtime:
    run tests
    observe output
    inspect files
    observe process state

Agent:
    evaluate evidence
    decide Goal Success
```

因此：

```text
VERIFY
```

是 Agent Core 的能力，而不是第五種 Runtime Action。

---

# 7. Agent Execution Status

不得使用單一 enum 同時混合：

```text
目前是否正在執行
```

與：

```text
最終任務結果
```

概念上應區分：

```text
Execution State
```

以及：

```text
Final Task Status
```

Final Task Status 至少：

```text
Done
Blocked
Impossible
NeedUser
```

Agent 在尚未完成任務時屬於：

```text
Running / Waiting
```

之類的 execution state。

`Continue` 不應作為最終 Task Status。

「Continue」只代表 Agent 在決策循環中沒有結束任務。

---

# 8. Final Status Semantics

## Done

Agent 有足夠證據確認原始 User Goal 已完成。

## Blocked

Agent 目前無法繼續，但沒有足夠證據認定 Goal 本身不可能完成。

例如：

```text
external service unavailable
required resource temporarily unavailable
long-running external condition
environment temporarily unavailable
```

## Impossible

Agent 有足夠證據判定目前環境或條件下無法完成 Goal。

例如：

```text
required hardware does not exist
required capability is fundamentally unavailable
goal conflicts with immutable environment constraint
```

## NeedUser

Agent 已經確認需要特定的使用者輸入、授權、外部資訊或實際人類操作。

例如：

```text
需要密碼
需要使用者提供外部資訊
需要使用者在 Secure Desktop 操作
需要使用者確認只有人類才能確認的資訊
```

「不知道下一步怎麼做」不得直接轉成 `NeedUser`。

---

# 9. Agent State

Agent 必須維護持續性的工作狀態。

概念資料至少包含：

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

其中：

```text
Fact
Observed
Inferred
Hypothesis
Unknown
```

必須能被區分。

Agent 不得將：

```text
Hypothesis
```

直接當作：

```text
Fact
```

Unknown 是合法狀態。

Agent 不需要為未知問題強行產生答案。

---

# 10. Knowledge State

Observation 與 Knowledge State 必須區分。

Observation 是：

```text
Environment 提供的證據
```

Knowledge State 是：

```text
Agent 對這些證據的理解
```

例如：

```text
Observation:
cargo build exited with code 101.

Inference:
the project currently fails to build.

Hypothesis:
the failure may be caused by a dependency mismatch.

Unknown:
whether updating the dependency will solve the problem.
```

因此：

```text
Observation ≠ Hypothesis
```

而：

```text
Observation → Evidence → Inference / Hypothesis
```

是 Agent Core 的正常工作方式。

---

# 11. Observation System

Observation 是 Agent 感知 Computer Environment 的主要資料來源。

完整 Computer Environment 可以非常巨大，因此必須採用：

```text
Environment
    ↓
Observation
    ↓
Observation Store
    ↓
Relevant Observation Selection
    ↓
Context Compiler
    ↓
LLM Context
```

Agent 不應每一輪將完整電腦狀態塞入 LLM。

Observation Store 應能保存歷史 Observation，並支援至少概念上的：

```text
store
retrieve
filter
query
relate to task
relate to action
relate to job
relate to resource
```

具體儲存方式由實作者決定。

---

# 12. Observation Types

系統至少應能表達：

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

不要求每種 Observation 都必須在每一輪產生。

---

# 13. Environment Observation

Environment Observation 可包含：

```text
Windows version
CPU
RAM
GPU
VRAM
Disk
Free space
Installed runtimes
Python
Rust
Cargo
Node
Git
FFmpeg
CUDA
GPU driver
Environment variables
Network state
Installed software
Available commands
```

Environment information 應具有適當 Cache 機制。

不應在每次 Decision cycle 都重新掃描完整環境。

當：

```text
environment may have changed
information expired
Agent explicitly requests refresh
```

時再更新。

---

# 14. Filesystem Observation

Filesystem Runtime 必須支援按需觀察：

```text
list
tree
metadata
file existence
file size
timestamps
search
content
diff
directory statistics
```

不應預設遞迴掃描整個磁碟。

大量資料必須支援：

```text
depth limit
item limit
pagination
targeted inspection
summary
```

---

# 15. Process Observation

Process Observation 至少應能表達：

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
```

實際並非所有 Process 都能取得所有欄位。

對不存在或無法取得的資料應使用：

```text
unknown / unavailable
```

而不是假造值。

Agent 自己啟動的 Process / Job 必須可以被持續追蹤。

---

# 16. Action Result Observation

每次 Runtime Action 完成或產生重要狀態變化後，都應產生可供 Agent 使用的結果 Observation。

概念上包含：

```text
success / failure
status
result
exit code
stdout
stderr
duration
affected resources
error information
```

大型 stdout / stderr 不得無限制進入 LLM Context。

完整資料可以保存在 Observation Store / Log Store，而 Context 中只提供：

```text
relevant excerpt
summary
head / tail
diff
reference
```

---

# 17. Visual Observation

GUI 操作需要：

```text
Screenshot
Window information
Visible UI information
UI Automation information
```

優先使用結構化 UI Automation / accessibility information。

當結構化資訊不足時，可以使用：

```text
Vision
coordinate-based interaction
```

作為 fallback。

GUI Action 後應重新 Observation。

---

# 18. Browser Observation

Browser Observation 至少可以描述：

```text
URL
Page title
Visible text
DOM/interactable elements
Loading state
Download state
Error state
```

Browser 與一般 GUI 不應完全綁死。

Browser-level interaction 應優先於：

```text
GUI coordinate interaction
```

必要時再 fallback 至 GUI / Vision。

---

# 19. Engineering Observation

Engineering Runtime 應能產生：

```text
Repository state
Source search result
Symbol/reference result
Dependency information
Build result
Test result
Diff
Git state
Debug output
Code-change result
```

這些都是 Observation，而不是 Agent Core 的固定 Workflow。

---

# 20. Observation Compression

Observation Store 可以保存完整資料，但送入 LLM Context 的資料必須受控。

系統至少支援：

```text
truncation
summarization
deduplication
relevance filtering
diff extraction
history compression
pagination
```

對大量 CLI output：

```text
不得無限制塞入 Context
```

對大型 filesystem listing：

```text
不得無限制塞入 Context
```

檔案修改後，優先提供：

```text
Diff
affected region
changed symbols
```

而不是無條件重新提供完整檔案。

---

# 21. Context Compiler

Context Compiler 負責將：

```text
Current Goal
Agent State
Relevant Observations
Recent Actions
Relevant History
Active Hypotheses
Current Problems
Verification State
Available Runtime Capabilities
```

轉換為 LLM 可以使用的 Context。

Context Compiler 不負責高層決策。

它負責：

```text
select
filter
compress
organize
prioritize
```

資訊。

不得因壓縮而無意間刪除對目前 Goal 至關重要的證據。

---

# 22. Context Budget

系統必須具有 Context Budget。

當 Context 接近模型限制時，不得繼續無限制加入歷史。

至少支援：

```text
recent history retention
observation summarization
old action compression
old log compression
irrelevant observation removal
emergency context compaction
```

長時間任務不能因歷史紀錄單純累積而必然失敗。

Context size 應由：

```text
model capability
provider capability
configuration
```

共同決定。

不得在架構上假設所有模型永遠具有固定 Context Window。

---

# 23. Computer Runtime

Computer Runtime 是 Agent Core 與 Windows / Applications / External Environment 之間的主要執行介面。

概念架構：

```text
User Goal
    ↓
Agent Core
    ↓
Runtime Action
    ↓
Computer Runtime
    ↓
Windows / Applications / External Environment
    ↓
Observation
    ↓
Agent Core
```

Runtime 必須提供：

```text
execution
observation
result normalization
resource management
error reporting
job management
```

Runtime 不負責高層問題解決。

---

# 24. Filesystem Runtime

Filesystem Runtime 必須支援一般 Windows 使用者可以執行的主要檔案操作：

```text
create
read
write
modify
patch
delete
rename
copy
move
search
list
create directory
remove directory
metadata
compare
batch processing
```

必須合理處理：

```text
UTF-8
UTF-16
binary files
large files
locked files
missing files
path errors
long paths where supported
```

Filesystem Runtime 應盡可能提供受影響資源資訊，以便 Observation 與 Verification 使用。

---

# 25. Process Runtime

Process Runtime 必須支援：

```text
start
observe
wait
terminate
restart
background execution
stdout capture
stderr capture
exit code
timeout
resource monitoring
child process tracking
```

長時間 Process 應可被抽象成：

```text
Job
```

而不是要求 Agent 一直阻塞等待。

---

# 26. Interactive Shell

Windows Shell Runtime 必須考慮互動式 CLI。

不能假設所有 CLI 都遵循：

```text
stdin → stdout → exit
```

部分程式會等待：

```text
stdin
password
confirmation
terminal interaction
pager
credential input
```

Runtime 必須避免因 pipe capture 而產生永久 deadlock。

Windows interactive shell 應支援適當的 pseudo-terminal / ConPTY 機制。

具體 crate 或 Windows API 實作方式由實作者決定。

---

# 27. Non-Interactive CLI

對於工具本身提供的 non-interactive mode，Agent 可以優先使用：

```text
--yes
--non-interactive
--no-input
```

等方式。

Runtime 不得假設所有 CLI 都支援這些參數。

Agent 應根據實際工具確認可用參數。

如果 Process 長時間沒有進展，系統應能產生 Observation，例如：

```text
running
possibly_unresponsive
awaiting_input
timeout
```

Agent 再決定：

```text
wait
inspect
interact
terminate
change strategy
```

---

# 28. GUI Runtime

GUI Runtime 應能支援：

```text
observe screen
find windows
launch application
switch window
click
double click
right click
drag
type
press key
scroll
wait
observe again
```

優先使用：

```text
Windows UI Automation / accessibility information
```

Vision / coordinate interaction 可以作為 fallback。

GUI 操作後必須重新 Observation。

---

# 29. Interactive Desktop Requirement

若 Agent 需要操作 GUI，Runtime 必須運行於 Interactive User Desktop Session。

GUI Runtime 應能偵測：

```text
interactive desktop available
```

或：

```text
interactive desktop unavailable
```

如果不存在 Interactive Desktop：

```text
GUI capability unavailable
```

應成為 Runtime Observation。

不得讓 Agent 無限重試相同 GUI Action。

---

# 30. UAC / Secure Desktop

UAC Secure Desktop 不應被視為一般 GUI。

如果 Windows 將操作切換至 Agent 無法控制的 Secure Desktop，Runtime 必須回報足夠資訊，例如：

```text
permission_boundary
secure_desktop
user_interaction_required
```

Agent 應判斷是否：

```text
可以繼續其他方式
等待
要求使用者介入
```

不得無限重試相同 GUI Action。

---

# 31. Browser Runtime

Browser Runtime 應支援：

```text
navigate
observe
click
type
scroll
wait
read
download
upload
form interaction
```

Browser automation 應優先使用結構化 browser-level interaction。

必要時可以退回：

```text
GUI
Vision
```

---

# 32. Network Runtime

Network Runtime 應支援：

```text
HTTP
HTTPS
API
download
upload
web search integration
repository access
package repository access
documentation access
```

Agent 可以利用網路取得完成 Goal 所需要的資訊。

Network Failure 必須成為 Observation。

---

# 33. Engineering Runtime

Agent 必須能處理一般 Software Engineering 工作：

```text
inspect project
search source
read source
modify source
create files
delete files
refactor
run build
run tests
inspect logs
debug
manage dependencies
modify configuration
run Git
inspect diff
checkpoint
rollback where possible
```

這些能力應以通用 Runtime capability 提供，而不是針對每一個工具建立一個獨立 Agent abstraction。

例如：

```text
Rust
Python
Node
C++
FFmpeg
Git
Cargo
npm
pip
```

不應各自成為高層 Agent tool。

它們主要是：

```text
programs
repositories
commands
files
```

Agent 可以透過通用 Runtime 使用。

---

# 34. Repository / Code Search

Engineering Runtime 應提供有效率的：

```text
repository inspection
file discovery
text search
symbol search where available
reference search where available
dependency inspection
```

不要求所有語言都必須有完整 AST / LSP 支援。

如果結構化分析不可用，應允許退回：

```text
text search
filesystem inspection
compiler output
command-line tools
```

---

# 35. Diff / Code Change

程式碼修改後，系統應能產生：

```text
changed files
diff
affected regions
```

Agent 應能在修改後檢查實際變更，而不是只相信自己原本產生的修改內容。

---

# 36. Build / Test / Debug

Engineering Runtime 應能執行：

```text
build
test
lint / static analysis where available
debug-related commands
```

並將結果轉換為 Observation。

例如：

```text
build succeeded
build failed
test failed
test passed
process crashed
compiler diagnostic
runtime error
```

Runtime 不應自行判定：

```text
這個 bug 一定是什麼原因
```

原因分析屬於 Agent Core。

---

# 37. Environment Discovery

Agent 必須可以觀察：

```text
installed software
available commands
runtime versions
PATH
GPU
VRAM
CPU
RAM
disk
network
development toolchains
```

如果完成 Goal 所需工具不存在：

```text
Agent should investigate whether it can install or obtain it.
```

缺少工具不應直接視為：

```text
NeedUser
```

---

# 38. Software Installation

Agent 可以透過：

```text
Windows
package manager
installer
shell
repository
download
```

等方式安裝完成 Goal 所需軟體。

Runtime 不應把：

```text
pip
npm
cargo
winget
git
ffmpeg
```

視為固定高層 Agent 能力。

它們只是 Agent 可以執行或操作的程式與資源。

安裝後應重新 Observation 環境，以確認：

```text
installation succeeded
command available
version changed
PATH changed
dependency became available
```

---

# 39. Runtime Error Model

Runtime 必須將錯誤結構化回報。

至少能表達：

```text
success
failure
timeout
permission_denied
not_found
invalid_argument
process_crashed
network_error
environment_error
interactive_input_required
```

錯誤資訊應盡可能包含：

```text
operation
target
resource
OS error information
exit code
stderr
```

如果某欄位不可取得，應標記為 unavailable / unknown。

---

# 40. Failure Is Observation

Runtime Failure 不應自動等於 Agent Failure。

概念：

```text
Action
 ↓
Failure
 ↓
Observation
 ↓
Agent evaluates
 ↓
new hypothesis / additional observation / alternative action
```

Agent 可以：

```text
retry
inspect
modify parameters
change method
change tool
change hypothesis
ask user
finish
```

具體選擇由 Agent Core 決定。

---

# 41. Adaptive Retry

Agent 不得在相同條件下無限重複相同 Action。

系統應保存足夠資訊以判斷：

```text
same action
same target
same relevant environment
same result
```

是否反覆出現。

可以使用：

```text
Action Fingerprint
```

或其他等價機制。

如果：

```text
same action
+
same environment
+
same result
```

反覆出現而沒有進展，Agent 應：

```text
inspect additional information
change hypothesis
change action
change strategy
```

而不是永遠重試。

---

# 42. Loop Detection / Circuit Breaker

系統必須具有 Loop Detection / Circuit Breaker。

判斷不應只依賴：

```text
maximum retries = 3
```

而應至少考慮：

```text
Action similarity
Observation similarity
Environment change
Progress
```

當偵測到可能的無效循環時，應產生明確 Observation，例如：

```text
Repeated action detected.
No meaningful environment change observed.
A different hypothesis or additional inspection may be required.
```

Circuit Breaker 的目的不是替 Agent 做決策，而是阻止：

```text
無限重複
```

並將問題交回 Agent Core。

---

# 43. Progress Detection

Agent 必須能區分：

```text
Action happened
```

與：

```text
Task progressed
```

Progress 可以表現為：

```text
environment changed
understanding improved
hypothesis eliminated
verification status improved
subtask completed
new evidence obtained
```

如果多次 Action 沒有產生有意義變化，Agent 應重新評估策略。

不要求建立固定數學 Progress Score。

---

# 44. Action Record

每次 Action 至少應記錄：

```text
action_id
timestamp
action type
parameters
target
start time
end time
result
affected resources
observation after action
```

如適用，可以記錄：

```text
stdout
stderr
exit code
precondition
environment snapshot/reference
undo information
job_id
```

大型輸出不應要求全部直接存進單一 Agent Context。

---

# 45. Reversibility

Agent 應能表示 Action 的可逆程度。

概念上：

```text
reversible
partially_reversible
irreversible
unknown
```

Filesystem / project modification 應盡可能提供 rollback 能力。

外部服務 mutation、網路 side effect 等通常具有：

```text
irreversible
```

特性，除非存在明確 rollback API。

Reversibility 是 Decision 的重要資訊，但不得要求 Agent 使用固定決策公式。

---

# 46. Checkpoint / Snapshot

對重要 Project / Workspace 操作，系統應支援合理的 checkpoint 機制。

可以使用：

```text
Git
shadow backup
filesystem snapshot
temporary copy
```

或其他方式。

不要求每個 Action 都建立完整 Workspace snapshot。

核心行為需求：

> Agent 進行實驗性修改後，如果新的 Hypothesis 被證明錯誤，系統應具有合理的回復能力。

---

# 47. Long-Running Jobs

長時間工作必須與 Agent Decision Loop 分離。

概念上：

```text
Agent
 ↓
Start Job
 ↓
JobId
 ↓
Agent continues
 ↓
Observe Job
 ↓
Wait / Act / Observe
 ↓
Job Complete
 ↓
Retrieve Result
```

Job 至少應能表達：

```text
job_id
process identifier where applicable
status
start time
runtime
result
output reference
```

如果有：

```text
progress
```

則可以提供給 Agent。

Agent 應能：

```text
observe
wait
continue other work
retrieve result
terminate
```

---

# 48. Job Persistence

重要長時間 Job 的狀態應能在 Agent Process 重啟後重新取得。

至少需要保留足夠資訊：

```text
job_id
process identifier where applicable
command / operation identity
start time
status
result reference
```

實作者應考慮 Process 是否仍存在，以及重新連接 / 重新觀察的方法。

不得假設所有 Job 在 Agent Process Crash 後都可以完整恢復。

如果某 Job 無法恢復，必須產生明確 Observation。

---

# 49. Persistence

Agent State 必須可以持久化。

至少保存：

```text
Goal
Task Status
Agent State
Action Records
Observations
Running Jobs
Verification State
Checkpoint information
```

長時間任務不得只依賴 RAM 狀態。

Agent Crash 後應能：

```text
reload state
inspect environment
reconcile running jobs
resume when possible
```

Resume 不應直接假設：

```text
Crash 前的世界狀態 = Crash 後的世界狀態
```

重新啟動後應重新 Observation / Reconciliation。

具體儲存格式可以是：

```text
JSON
JSONL
SQLite
```

或其他合理方式。

---

# 50. Crash Recovery

Agent Process 重新啟動後，Recovery 應遵循概念：

```text
load persisted state
        ↓
inspect current environment
        ↓
reconcile jobs / resources
        ↓
detect stale assumptions
        ↓
update observations
        ↓
resume Agent loop
```

不得直接從 Crash 前最後一個 Action 繼續執行，而完全不確認目前環境。

---

# 51. Verification

Verification 必須針對：

```text
Original User Goal
```

而不是只驗證最後一個 Action。

至少區分：

```text
Action Success
Subtask Success
Goal Success
```

例如：

```text
cargo build succeeded
```

不代表：

```text
user's requested bug is fixed
```

---

# 52. Verification Methods

系統至少應支援：

```text
Runtime Verification
LLM Evaluation
User Confirmation
```

Runtime Verification 可以使用：

```text
exit code
file existence
file content
test result
build result
process state
expected output
```

LLM Evaluation 用於需要語意判斷的 Goal。

User Confirmation 僅在系統無法可靠自行驗證，或確實需要使用者確認時使用。

---

# 53. Verification Hierarchy

Verification 可以分成：

```text
Level 1:
Action Success

Level 2:
Subtask Success

Level 3:
Goal Success
```

Agent 不得因 Level 1 成功而自動宣稱 Level 3 成功。

Goal Completion 必須有足夠證據支持。

---

# 54. LLM Integration

LLM Integration 必須透過穩定介面與 Agent Core 解耦。

至少支援：

```text
Ollama
```

未來可以增加：

```text
其他 Local Provider
Remote Provider
```

Agent Core 不應依賴 Ollama 特定 API。

LLM Provider 的責任主要是：

```text
send context
receive model output
report provider errors
```

Context Compiler 與 Agent Core 負責決定提供哪些資訊。

---

# 55. Structured Decision Output

LLM 應以結構化格式輸出 Agent Decision。

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

具體 schema 由 Interface 文件定義。

LLM output 必須經過：

```text
Parse
 ↓
Validate
 ↓
Normalize
 ↓
Execute
```

不能：

```text
LLM output
 ↓
直接執行
```

---

# 56. LLM Output Robustness

Local LLM 可能產生：

```text
invalid JSON
markdown wrapper
missing fields
unknown fields
truncated output
incorrect enum
```

Parser 必須具有合理容錯能力。

至少支援：

```text
markdown codeblock removal
schema validation
clear parse errors
bounded correction retry
```

是否進一步支援：

```text
trailing comma
JSON repair
field normalization
```

由實作者決定。

但輕微格式錯誤不得直接導致整個 Agent Task 終止。

---

# 57. LLM Correction Loop

如果 LLM output 無法 parse：

```text
Parse Failure
 ↓
Generate structured error
 ↓
LLM correction request
 ↓
Parse again
```

必須設定合理上限。

如果 correction repeatedly fails，應產生明確 Observation / Error，交由 Agent Core 判斷。

Parser Retry 不得形成無限循環。

---

# 58. Agent Core Decision Context

每次 Agent Decision 應提供 LLM 足以做出下一步判斷的資訊。

至少包括：

```text
Original Goal
Current State
Relevant Environment
Relevant Observations
Recent Actions
Current Problems
Active Hypotheses
Verification State
Available Runtime Capabilities
```

LLM 可以決定：

```text
observe
act
wait
verify
replan
finish
request user
```

但 `verify` 不需要成為獨立 Runtime Action。

Agent 可以透過：

```text
Observe
Act
Wait
```

取得驗證所需證據。

---

# 59. Observation Before Action

Agent 必須能在 Action 前主動要求更多 Observation。

例如：

```text
Goal:
fix build

Agent does not know why build fails.

→ observe project
→ inspect build environment
→ run build
→ observe stderr
→ inspect source
→ form hypothesis
→ modify code
→ build again
→ verify
```

以上只是行為示例，不是固定 Workflow。

Agent 在不同環境下可以採用不同順序。

---

# 60. Information-Seeking Actions

Agent 可以選擇低成本 Information Gathering Action。

Action 選擇可以考慮：

```text
probability of solving
information value
cost
risk
reversibility
expected progress
```

不要求建立固定數學決策模型。

這些因素是 Agent Decision 的設計原則，而不是固定演算法。

---

# 61. Runtime Trust Boundary

所有 LLM 產生的 Action 都必須經過 Runtime 驗證。

Runtime 至少應檢查：

```text
schema validity
required fields
parameter validity
resource validity
path normalization where applicable
process parameters
policy constraints
OS permission boundary
```

Runtime 不應直接相信：

```text
LLM output
```

但 Runtime 也不應自行取代 Agent Core 做高層決策。

---

# 62. Capability / Policy / Security Separation

三者必須區分：

```text
Capability
```

代表 Runtime 技術上可以執行什麼。

```text
Policy
```

代表目前 Agent / Configuration 是否允許執行。

```text
Security / OS Permission
```

代表 Windows 實際允許什麼。

例如：

```text
Runtime 能執行刪除檔案
≠
目前 Task 一定應刪除該檔案
≠
Windows 一定允許刪除該檔案
```

Access Denied 是環境 / OS 邊界的 Observation，不代表 Agent 可以自動取得更高權限。

---

# 63. Security Baseline

低限制不代表忽略 Windows 本身的權限邊界。

Agent 應使用目前：

```text
Windows User
Process Token
```

所具有的權限。

Runtime 不應自行假設：

```text
Administrator
SYSTEM
kernel-level access
```

如果 Windows 返回：

```text
Access Denied
```

Agent 可以嘗試合理的方法，例如：

```text
alternative path
alternative operation
non-privileged method
user request
```

但不得假設自己可以突破 OS 權限邊界。

---

# 64. NeedUser Boundary

`NeedUser` 不應成為：

```text
Agent 不知道答案
```

的通用錯誤出口。

只有在存在明確外部需求時才使用，例如：

```text
password
private credential
user-only confirmation
Secure Desktop interaction
physical device interaction
external information unavailable to Agent
```

如果只是：

```text
unknown cause
unclear next step
insufficient evidence
```

Agent 應優先：

```text
observe
search
inspect
experiment
change hypothesis
```

---

# 65. Dry Run / Debug Mode

系統應提供 Debug / Dry Run。

Dry Run：

```text
LLM
 ↓
Decision
 ↓
Validation
 ↓
show intended Action
 ↓
do not execute
```

Debug Mode 應能觀察：

```text
Goal
Observation
Decision
Action
Action Result
Verification
State transition
```

不得要求輸出或保存 LLM Chain-of-Thought。

可以保存：

```text
decision
action
observation
evidence
hypothesis summary
verification result
```

---

# 66. Logging

系統必須提供結構化 Log。

至少區分：

```text
Agent decision
Runtime action
Observation
LLM request
LLM response
Parser
Error
Job
Verification
Persistence
Recovery
```

Log 的目的包括：

```text
debugging
failure analysis
performance analysis
agent behavior analysis
```

敏感資訊應避免無必要寫入 Log。

---

# 67. Cancellation

Agent 必須支援 Task Cancellation。

Cancellation 應能：

```text
stop Agent decision loop
cancel cancellable jobs
release Runtime resources
persist final state
```

對無法立即終止的 Process，應產生明確狀態，而不是假裝已經停止。

---

# 68. Timeout

Runtime 必須支援合理的 Timeout。

Timeout 是：

```text
Observation / Runtime Result
```

而不是自動等於：

```text
Task Failure
```

Agent 可以在 Timeout 後決定：

```text
wait longer
observe
terminate
retry differently
change strategy
finish
```

---

# 69. Resource Awareness

Runtime 應能提供合理的資源資訊，例如：

```text
CPU
RAM
GPU
VRAM
Disk
Disk free space
Process resource usage
```

Agent 可以根據資源狀態決定：

```text
wait
reduce concurrency
change method
free resources
continue
```

Runtime 不應自行決定高層策略。

---

# 70. Concurrency

系統應支援必要程度的並行工作，例如：

```text
background jobs
downloads
long-running processes
independent observations
```

但不要求 v3 第一階段建立複雜的：

```text
multi-agent scheduler
distributed execution
workflow DAG
```

如果並行操作可能互相影響，系統應保留足夠資訊讓 Agent 判斷資源衝突。

---

# 71. Persistence / Observation Relationship

Persistence 與 Observation Store 可以是：

```text
same storage system
```

也可以是：

```text
separate storage systems
```

但語意必須區分：

```text
Observation Store
    保存環境證據

Agent State
    保存目前理解與工作狀態

Job State
    保存長時間工作的狀態

Action History
    保存 Agent 做過什麼
```

不得將這四種概念全部混成單一巨大狀態物件。

---

# 72. Architecture Responsibility Boundary

概念上：

```text
Agent Core
    ↓
Decision

Context System
    ↓
Relevant information

Observation System
    ↓
Environmental evidence

Runtime
    ↓
Execution / environment access

Persistence
    ↓
Long-lived state

LLM Provider
    ↓
Model inference
```

每層應保持責任清楚。

具體 module placement 可以由實作者決定。

---

# 73. Rust Architecture

可以採用類似：

```text
src/
├── agent/
├── runtime/
├── observation/
├── llm/
├── context/
├── persistence/
├── config/
└── main.rs
```

的模組分離方式。

更細的模組例如：

```text
agent/core
agent/state
agent/decision
agent/verification

runtime/filesystem
runtime/process
runtime/shell
runtime/gui
runtime/browser
runtime/network
runtime/jobs

observation/store
observation/types
observation/compression

llm/provider
llm/parser

context/compiler

persistence
```

僅為建議。

實作者可以自由調整檔案結構，只要責任分離與介面行為不被破壞。

---

# 74. Runtime Responsibility Boundary

Runtime 負責：

```text
execute
observe
normalize result
manage process
manage filesystem
manage GUI
manage browser
manage jobs
report errors
```

Runtime 不負責：

```text
deciding user's intention
creating long-term problem-solving strategy
diagnosing programming bugs
deciding whether a hypothesis is true
deciding whether the original Goal is complete
```

---

# 75. Agent Core Responsibility Boundary

Agent Core 負責：

```text
understand Goal
interpret Observation
maintain state
form hypotheses
select next action
adapt strategy
detect progress
request additional observations
decide verification
decide completion
```

Agent Core 不應依賴 Windows API。

---

# 76. Development Phases

以下 Phase 是：

```text
施工順序
```

不是：

```text
永久能力限制
```

也不是要求所有後續能力在前一 Phase 完成前完全不可使用。

如果實作者發現更合理的施工順序，可以調整，但不得降低 Behavioral Requirements。

---

## Phase 1 — Foundation

建立：

```text
Rust project
configuration
structured logging
error model
LLM provider abstraction
Ollama integration
basic persistence
basic data models
```

目標是建立可以運行的 Agent 基礎。

---

## Phase 2 — Computer Runtime Foundation

建立：

```text
Filesystem Runtime
Process Runtime
Shell Runtime
basic Job management
timeout
cancellation
Runtime result normalization
```

目標：

```text
Agent 可以可靠地觀察並操作基本 Windows 環境。
```

---

## Phase 3 — Observation & Context

建立：

```text
Observation model
Observation Store
Environment Observation
Filesystem Observation
Process Observation
Action Result Observation
Environment Delta
Context Compiler
Context Budget
history compression
```

目標：

```text
Agent 可以持續取得環境證據，而不是只依賴一次性 command output。
```

---

## Phase 4 — Agent Core Loop

建立：

```text
Goal handling
Agent State
Agent Decision
adaptive loop
hypothesis handling
progress detection
loop detection
circuit breaker
verification
recovery
```

目標：

```text
Goal
 ↓
Observe
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
Verify
```

形成完整閉環。

---

## Phase 5 — Engineering Runtime

加入：

```text
repository inspection
code search
symbol/reference discovery where available
editing
diff
build
test
dependency inspection
Git
checkpoint
rollback
debug-related operations
```

目標是讓 Agent 能處理真實 Software Engineering 任務。

---

## Phase 6 — Browser Runtime

加入：

```text
browser automation
navigation
structured page observation
forms
downloads
uploads
browser-level interaction
```

---

## Phase 7 — GUI Runtime

加入：

```text
Windows UI Automation
interactive desktop detection
window management
keyboard / mouse interaction
screenshot
Vision fallback
UAC boundary detection
```

---

## Phase 8 — Advanced Runtime

加入：

```text
advanced environment observation
external devices
advanced Windows integration
more robust job recovery
additional runtime capabilities
```

---

# 77. Initial Functional Milestone

第一個真正可運作的 Agent 應完成：

```text
User Goal
    ↓
Agent Core
    ↓
Observation
    ↓
Filesystem / Process / Shell Runtime
    ↓
Windows
```

至少能處理：

```text
inspect project
inspect environment
create / modify files
execute commands
observe results
recover from failure
change strategy
verify goal
```

例如：

```text
User:

「建立一個 Rust Hello World 專案並編譯通過。」
```

Agent 可以自行選擇類似：

```text
observe environment
↓
determine whether cargo is available
↓
create project
↓
observe result
↓
build
↓
observe result
↓
verify
↓
Done
```

但上述步驟只是示例。

Agent 不必被硬編碼成這個 Workflow。

第一個 milestone 不要求：

```text
GUI
Browser
Vision
advanced Windows automation
```

但架構不得因此阻止後續加入這些能力。

---

# 78. Capability Validation

v3 的驗證應以：

```text
Can the Agent complete real tasks?
```

為核心，而不是：

```text
How many tools exist?
```

建議使用以下類型的真實任務驗證：

### Filesystem

```text
建立
讀取
修改
搜尋
搬移
複製
刪除
```

### Development

```text
檢查 project
修改 Rust / Python project
build
test
修復失敗
```

### Environment

```text
發現缺少 dependency
調查可用安裝方式
安裝或尋找替代方案
重新確認環境
```

### Process / Job

```text
啟動長時間 Process
取得 JobId
監控
等待
取得結果
必要時終止
```

### Failure Recovery

第一次方法失敗後：

```text
observe
understand
change hypothesis
change strategy
```

而不是無限重試。

### Loop Recovery

相同 Action 在相同環境中持續失敗時：

```text
detect loop
stop repetition
produce observation
change strategy
```

### Verification

不能只因：

```text
last command succeeded
```

就宣稱：

```text
Goal completed
```

### Persistence

Agent 中途退出後：

```text
reload state
re-observe environment
reconcile jobs
resume when possible
```

### GUI / Browser

在相關 Runtime 完成後，再驗證：

```text
GUI interaction
Browser interaction
download
upload
visual fallback
```

以上是能力驗證方向，而不是固定 Release Gate。

實作者與使用者可以根據實際 Agent 能力增加新的測試任務。

---

# 79. Non-Goals

v3 第一階段不需要：

```text
multi-agent architecture
complex planner trees
vector database
RAG system
fixed workflow DSL
dozens of specialized AI tools
fully autonomous privilege escalation
perfect world-model simulation
perfect long-term memory
```

這些不是目前核心問題。

核心問題是：

> 建立可靠的 Observe → Decide → Act → Observe → Verify 閉環。

---

# 80. Implementation Freedom

本文件刻意不固定所有 implementation details。

例如以下均由實作者自行選擇：

```text
Tokio vs alternative async runtime
SQLite vs JSONL vs other persistence
ConPTY crate vs direct Windows API
browser automation library
UIA wrapper
snapshot implementation
JSON repair implementation
context compression algorithm
internal data structures
job supervision mechanism
```

但實作必須滿足本文件定義的：

```text
Behavioral Requirements
Responsibility Boundaries
Interface Semantics
Recovery Requirements
Verification Requirements
```

如果某項實作選擇與上述要求衝突，應修改實作，而不是降低需求。

---

# 81. Final Architecture Principle

最終系統應保持以下關係：

```text
                 ┌─────────────────────┐
                 │      User Goal      │
                 └──────────┬──────────┘
                            ↓
                 ┌─────────────────────┐
                 │     Agent Core      │
                 │                     │
                 │ Understand          │
                 │ Hypothesize         │
                 │ Decide              │
                 │ Adapt               │
                 │ Verify              │
                 └─────────┬───────────┘
                           ↕
                 ┌─────────────────────┐
                 │ Observation / State │
                 │ Context / Memory    │
                 └─────────┬───────────┘
                           ↕
                 ┌─────────────────────┐
                 │  Computer Runtime   │
                 │                     │
                 │ Filesystem          │
                 │ Process             │
                 │ Shell               │
                 │ GUI                 │
                 │ Browser             │
                 │ Network             │
                 │ System              │
                 │ Jobs                │
                 │ Engineering         │
                 └─────────┬───────────┘
                           ↓
                 ┌─────────────────────┐
                 │ Windows / External  │
                 │ Environment         │
                 └─────────────────────┘
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
ACT
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
DONE
```

v3 的成功標準不是：

```text
Runtime 有多少 API
```

而是：

> Agent 能否利用有限而強大的 Runtime primitives，自主操作真實 Windows 環境，在不確定、失敗、環境變化與長時間工作的情況下持續修正策略，最終完成使用者 Goal。

最重要的責任分界為：

```text
LLM
    負責提供推理能力

Agent Core
    負責狀態、假設、決策、適應與驗證

Observation System
    負責保存與提供環境證據

Runtime
    負責實際操作電腦

Persistence
    負責讓長時間任務跨越 Process lifetime

Windows
    負責實際 OS 權限與環境邊界
```

Agent v3 不應被設計成：

```text
一個會呼叫很多工具的 LLM
```

而應被設計成：

```text
一個具有感知、行動、記憶、恢復與驗證能力的 Agent Runtime，
其中 LLM 是負責理解與決策的核心推理元件。
```
