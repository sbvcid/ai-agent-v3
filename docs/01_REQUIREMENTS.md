# AI Agent v3 — Rust Requirements Specification

## 1. Project Definition

### 1.1 Project Name

AI Agent v3

### 1.2 Target Platform

Windows 11 x64

### 1.3 Implementation Language

Rust

### 1.4 Core Objective

建立一個具備自主電腦操作能力的 Windows Autonomous Computer Agent。

使用者只需要提供一個自然語言目標，例如：

* 「幫我把這個專案修好並通過測試。」
* 「找出 C 槽為什麼快滿了，清理不必要的檔案。」
* 「把這批影片轉成 AV1。」
* 「幫我安裝缺少的環境並執行這個 Python 專案。」
* 「分析這個程式為什麼啟動失敗並修好。」
* 「到網站下載資料，整理後產生報告。」

Agent 必須能自行：

1. 觀察環境。
2. 理解目前狀態。
3. 建立對問題的判斷與假設。
4. 決定下一步。
5. 實際執行操作。
6. 觀察執行結果。
7. 根據結果修正判斷。
8. 必要時嘗試其他方法。
9. 驗證原始目標是否真正完成。
10. 在無法完成時明確說明阻塞原因。

Agent 不應被設計成：

```text
LLM + 一堆固定工具 + 固定 Workflow
```

而應設計成：

```text
User Goal
    ↓
Agent Core / LLM
    ↕
Observation System
    ↕
Computer Runtime
    ↕
Windows
```

### 長期架構目標：模型能力與 Agent Runtime 解耦

Agent v3 的長期目標不是複製目前特定雲端 Agent，例如 Codex 的內部實作。

目標是建立一個與模型能力高度解耦的本地自主 Agent Runtime。

系統應假設未來本地模型會持續：

* 變得更強
* 變得更小
* 變得更便宜
* 具備更好的推理能力
* 具備更好的規劃能力
* 具備更好的程式理解能力
* 具備更好的環境理解能力
* 具備更好的問題解決能力

因此，Agent v3 不應將目前模型的弱點硬編碼成大量固定 Workflow。

例如，不應因為目前模型較弱，就設計成：

```text
如果遇到 A → 固定做 B
如果遇到 B → 固定做 C
如果遇到 C → 固定做 D
```

這種邏輯可以在必要時作為 Runtime 層的可靠性保護或過渡性啟發式，但不能成為 Agent 的主要問題解決方式。

Runtime 的責任是提供 Agent 足以：

```text
觀察
操作
修改
執行
等待
監控
驗證
```

Windows 電腦環境的通用能力。

Agent Core 的責任則是：

```text
理解
推理
建立假設
決策
行動
觀察結果
評估
修正策略
重新規劃
驗證
```

未來替換成更強的本地模型時，應主要提升 Agent 的問題解決能力，而不需要重新設計 Runtime 的核心架構。

最終目標是讓本地 Agent 越來越接近：

> 人類專家使用電腦解決實際問題的方式。

而不是：

> 執行預先設計好的自動化流程。

因此：

```text
Model = Intelligence

Agent Core = Reasoning / Decision / Adaptation

Runtime = Computer Operation Capability

Observation = Connection Between Agent and Environment

State = Working Memory

Persistence = Long-running Task State

Verification = Determining Whether the Actual Goal Was Achieved
```

上述各層應保持清楚的責任邊界。

### 通用 Computer Runtime

Agent Runtime 不應只針對單一用途設計。

基本原則：

> 凡是目前 Windows 使用者在正常權限範圍內可以透過電腦完成的工作，原則上都應具有被 Agent 執行、觀察或透過 Runtime 取得相關資訊的能力。

包括但不限於：

```text
Filesystem
Process
Shell
Interactive Shell
Python
GUI
Browser
Network
Software Installation
System Information
Long-running Jobs

Repository
Code Search
Code Navigation
Code Editing
Build
Test
Debugging
Git
Diff
Checkpoint / Rollback

Media
Documents
Archives
External Devices
```

上述能力應以通用 Runtime 能力存在，而不是被設計成固定 Workflow。

Agent 應根據當前環境與問題自行決定需要使用哪些能力。

---

# 2. Design Principles

## 2.1 Agent Is Goal-Oriented

Agent 的核心輸入是：

```text
Goal
```

而不是：

```text
Tool → Tool → Tool → Tool
```

使用者不應需要告訴 Agent：

```text
先 read_file
再 run_python
再 run_command
```

例如使用者說：

> 把影片轉成 AV1。

Agent 應自行判斷：

```text
觀察環境
→ 找影片
→ 判斷影片格式
→ 檢查 FFmpeg
→ 判斷編碼能力
→ 必要時安裝缺少環境
→ 決定適當方法
→ 執行
→ 觀察結果
→ 修正問題
→ 驗證輸出
```

這些步驟不是固定 Workflow，而是 Agent 根據當時環境產生的行動序列。

---

## 2.2 Capability Must Be Broad

基本原則：

> 凡是一般 Windows 使用者在其權限範圍內可以透過電腦完成的工作，Agent 原則上都應具備完成它的能力。

Agent 不應因為預先定義的工具清單過小而無法完成工作。

例如不能只支援：

```text
read_file()
write_file()
run_python()
```

然後遇到：

```text
FFmpeg
Cargo
PowerShell
Git
Browser
GUI
Package Manager
```

就失去能力。

Agent 應具有通用的電腦操作能力。

---

## 2.3 Capability ≠ Policy

系統必須區分：

```text
Capability
Policy
```

Capability 表示：

> Agent 技術上能不能做到。

Policy 表示：

> Agent 在什麼情況下應該做。

v3 不應透過大量硬編碼限制削弱 Runtime。

在一般使用者權限範圍內，Runtime 原則上應提供足夠廣泛的操作能力。

如果 Windows 回傳：

```text
Access Denied
```

這應被視為環境狀態，而不是立即判定 Agent 沒有能力。

Agent 應先嘗試合理解決方法。

但 Agent 不得：

```text
自行取得不存在的 Administrator 權限
繞過 Windows 安全機制
假設自己具有 SYSTEM / Kernel 權限
```

如果確實需要：

* 使用者密碼
* 使用者確認
* 外部帳號授權
* Administrator 權限
* 不存在的硬體能力
* 無法取得的外部資訊

才應進入：

```text
NEED_USER
```

---

# 3. High-Level Architecture

```text
┌─────────────────────────────┐
│            User             │
│        Natural Language     │
└──────────────┬──────────────┘
               │ Goal
               ▼
┌─────────────────────────────┐
│        Agent Core           │
│                             │
│ Goal / State / Reasoning    │
│ Decision / Evaluation       │
│ Replanning / Verification   │
└──────────────┬──────────────┘
               │
       ┌───────┴────────┐
       ▼                ▼
┌──────────────┐ ┌────────────────┐
│ Observation  │ │    Runtime     │
│   System     │ │                │
│              │ │ Windows        │
│ Environment  │ │ Filesystem     │
│ Process      │ │ Process        │
│ Files        │ │ Shell          │
│ Browser      │ │ GUI            │
│ Visual       │ │ Browser        │
│ Results      │ │ Network        │
│ Deltas       │ │ Programs       │
│ History      │ │ Engineering    │
└──────┬───────┘ └───────┬────────┘
       │                 │
       └─────────────────┘
                 │
                 ▼
           ┌───────────┐
           │  Windows  │
           └───────────┘
```

Agent Core 不應直接依賴 Windows API 細節。

Runtime 不應負責高階問題解決。

Observation System 負責把真實環境轉換成 Agent 可以使用的資訊。

---

# 4. Core Agent Loop

Agent 不採用固定 Workflow。

核心循環：

```text
GOAL
 ↓
OBSERVE
 ↓
UNDERSTAND
 ↓
DECIDE
 ↓
ACT
 ↓
OBSERVE RESULT
 ↓
EVALUATE
 ↓
CONTINUE / REPLAN / VERIFY
 ↓
DONE
```

更完整的概念：

```text
Goal
 ↓
Observe
 ↓
Understand
 ↓
Hypothesize
 ↓
Act
 ↓
Observe
 ↓
Evaluate
 ↓
Revise Understanding
 ↓
Act Again
 ↓
Verify
 ↓
Done
```

其中：

```text
Failure
```

是新的 Observation。

而不是必然的：

```text
Task Failed
```

---

# 5. Failure Handling

最重要原則：

> Failure is an Observation.

例如：

```text
cargo build
→ failed
```

不應直接：

```text
Task Failed
```

而應形成：

```text
Observation:
cargo build failed.

Evidence:
error message = ...

Hypothesis:
dependency/version/configuration may be incorrect.

Next Decision:
inspect relevant project state.
```

Agent 應根據新資訊重新決策。

錯誤後可能選擇：

```text
Retry
Alternative Action
Gather Information
Modify Environment
Modify Code
Install Dependency
Use Another Executable
Use GUI
Use CLI
Wait
Ask User
```

實際選擇由 Agent 根據當前狀態決定。

---

# 6. Agent State

Agent 必須維護持續性的 Task State。

概念資料結構：

```rust
struct AgentState {
    goal: Goal,

    understanding: Understanding,

    environment: EnvironmentState,

    active_problems: Vec<Problem>,

    unknowns: Vec<Unknown>,

    hypotheses: Vec<Hypothesis>,

    evidence: Vec<Evidence>,

    recent_actions: Vec<ActionRecord>,

    recent_observations: Vec<Observation>,

    running_jobs: Vec<JobState>,

    remaining_work: Vec<TaskItem>,

    verification: VerificationState,

    status: TaskStatus,
}
```

實際資料結構可以調整，但必須保留上述概念。

Agent State 的目的不是保存完整歷史，而是保存 Agent 目前完成工作所需要的 Working Memory。

---

# 7. Knowledge States

Agent 不應把所有資訊都視為事實。

至少區分：

```text
FACT
OBSERVED
INFERRED
HYPOTHESIS
UNKNOWN
```

例如：

```text
FACT:
Windows 11 is running.

OBSERVED:
cargo build returned exit code 101.

INFERRED:
The project currently cannot compile.

HYPOTHESIS:
A dependency version may be incompatible.

UNKNOWN:
Whether changing the dependency will solve the problem.
```

這些分類描述的是：

> Agent 對資訊的認知狀態。

不是 Observation 本身不可變的資料類型。

因此同一項資訊可以隨著新的 Evidence 改變認知狀態：

```text
Hypothesis
→ Supported
→ Confirmed Fact
```

或：

```text
Hypothesis
→ Refuted
```

Agent 不得把：

```text
Hypothesis
```

直接當成：

```text
Fact
```

---

# 8. Unknown Is Valid

Agent 可以處於：

```text
UNKNOWN
```

狀態。

當 Agent 不知道答案時，不應強迫自己產生答案。

可以選擇：

```text
Observe
Search
Inspect
Experiment
Test
Compare
```

例如：

```text
Unknown:
Why does program X crash?

Action:
Run program and collect stderr.

Observation:
Missing DLL.

New hypothesis:
Dependency is missing.
```

Unknown 應被視為 Agent 決策系統中的合法狀態，而不是錯誤。

---

# 9. Agent Decision Model

每一輪 Agent 必須決定下一個合理行動。

概念上可以考慮：

```text
probability of progress
+
information value
+
reversibility
-
execution cost
-
risk
```

不要求實作成固定數學公式。

Agent 應能判斷：

* 哪個行動最可能解決問題。
* 哪個行動可以取得最多資訊。
* 哪個行動成本低。
* 哪個行動容易回復。
* 哪個行動具有不可逆風險。
* 是否應該先觀察再操作。
* 是否已經有足夠資訊可以直接操作。
* 是否應該改變假設。
* 是否應該停止目前策略。

這些因素是決策參考，不應變成固定 Planner Workflow。

---

# 10. Observation System

Observation System 是 Agent 的環境感知層。

核心原則：

```text
Observation ≠ Context
```

整台電腦可能有：

```text
數百萬個檔案
數百個程序
大量環境變數
大量日誌
大量程式碼
大量瀏覽器資訊
```

不能每輪全部塞進 LLM Context。

因此：

```text
Windows Environment
       ↓
Observation Store
       ↓
Relevant Observations
       ↓
Context Compiler
       ↓
LLM Context
```

Observation Store 可以保留比目前 Context 更多的資訊。

Context Compiler 負責選擇目前真正相關的資訊。

---

# 11. Observation Types

至少支援以下 Observation。

## 11.1 Environment Observation

包括：

```text
OS version
CPU
RAM
GPU
VRAM
Disk
Free space
Python
Rust
Cargo
Node
npm
Git
FFmpeg
CUDA
GPU driver
Installed software
Environment variables
PATH
Network state
```

不需要每輪重新掃描。

應採用：

```text
cache + refresh
```

模式。

---

## 11.2 Filesystem Observation

可以觀察：

```text
file existence
directory structure
file size
file timestamp
file type
file metadata
file content
directory changes
```

支援：

```text
list
search
inspect
compare
watch
```

大型目錄不應無限制一次輸出全部內容。

---

## 11.3 Process Observation

包括：

```text
PID
process name
command line
status
CPU usage
RAM usage
start time
runtime
stdout
stderr
exit code
```

---

## 11.4 Action Result Observation

每次 Runtime Action 都應產生結構化結果：

```rust
struct ActionResult {
    success: bool,

    status: ActionStatus,

    exit_code: Option<i32>,

    stdout: String,

    stderr: String,

    duration_ms: u64,

    affected_resources: Vec<ResourceChange>,

    error: Option<RuntimeError>,
}
```

Runtime 不應只回傳：

```text
true / false
```

而應提供足夠資訊讓 Agent 判斷下一步。

---

## 11.5 Visual Observation

GUI 操作需要：

```text
screenshot
window state
UI elements
OCR
visual analysis
```

Vision 不應是 GUI 的唯一方式。

優先：

```text
Windows UI Automation / Accessibility
```

必要時：

```text
OCR
Vision
Coordinate Interaction
```

---

## 11.6 Browser Observation

至少包括：

```text
URL
page title
DOM
visible text
interactive elements
forms
download state
navigation state
errors
```

---

## 11.7 Environment Delta

Agent 不一定需要重新掃描全部環境。

應支援：

```text
created
modified
deleted
started
stopped
changed
```

例如：

```text
Before:
output.mp4 does not exist.

Action:
FFmpeg encoding.

After:
output.mp4 created.
size = 3.2GB
duration = 01:32:10
```

---

# 12. Computer Runtime

Runtime 是：

```text
LLM / Agent Core
        ↕
Computer Runtime
        ↕
Windows
```

之間的執行層。

Runtime 負責：

```text
execute
interact
observe
monitor
return result
```

Runtime 不負責：

```text
選擇解法
判斷 bug 原因
決定修改哪個檔案
決定安裝哪個套件
決定最終策略
```

這些屬於 Agent Core。

---

# 13. Minimal LLM / Runtime Abstraction

LLM-facing Runtime abstraction 應盡可能少。

核心概念：

```text
OBSERVE
ACT
```

必要時支援：

```text
WAIT
```

以及由 Agent Core 控制的：

```text
VERIFY
```

其中：

```text
Verify
```

不是 Runtime Action 類型，而是 Agent Core 的內部驗證行為。

同樣：

```text
Decision
```

與：

```text
Task Status
```

必須保持概念分離。

不要為每個程式建立一個 Tool：

```text
run_python()
run_ffmpeg()
run_cargo()
run_git()
install_package()
search_google()
open_chrome()
click_button()
read_file()
write_file()
```

這些都是 Runtime 的實際能力。

Agent 應使用通用 Action 表達需要完成的操作。

---

# 14. Generic Action Model

概念上：

```rust
enum Action {
    Observe(ObserveRequest),

    Execute(ExecuteRequest),

    Interact(InteractionRequest),

    Wait(WaitRequest),
}
```

其中：

```text
Observe
```

代表取得資訊。

```text
Execute
```

代表執行程式、命令或其他可執行操作。

```text
Interact
```

代表 GUI、Browser 或其他互動式操作。

```text
Wait
```

代表等待環境或 Background Job 發生變化。

```text
Verify
```

不是 Action。

Verify 是 Agent Core 對目標是否完成的判斷階段。

Agent 不需要知道 Windows API 的細節。

例如：

```text
Execute:
program = "ffmpeg"
arguments = [...]
working_directory = ...
```

Runtime 自己處理：

```text
CreateProcess
stdout pipe
stderr pipe
process lifecycle
timeout
exit code
```

---

# 15. Filesystem Runtime

Runtime 必須支援通用檔案系統操作：

```text
create
read
write
append
modify
delete
rename
copy
move
search
list
compare
create_directory
remove_directory
metadata
watch
```

需要支援：

```text
large files
binary files
UTF-8
non-UTF-8
Unicode paths
long paths
batch operations
```

對於程式碼與文字檔案，Runtime 應能提供適合 Agent 理解的：

```text
diff
changed region
line range
```

而不是每次修改後都要求重新傳送整個檔案。

---

# 16. Process Runtime

支援：

```text
start process
foreground process
background process
capture stdout
capture stderr
wait
poll
terminate
restart
monitor
timeout
```

Background Process 必須具有：

```rust
JobId
```

例如：

```text
start → job_id = 42

observe job 42

wait

observe job 42

terminate job 42
```

---

# 17. Shell Runtime

至少支援：

```text
PowerShell
CMD
Windows executables
```

並能執行一般 Windows CLI。

例如：

```text
git
python
pip
cargo
rustc
node
npm
ffmpeg
7z
curl
winget
```

Agent 不需要為每個 CLI 建立專用 Tool。

Shell Runtime 應處理：

```text
process lifecycle
stdout
stderr
exit code
timeout
working directory
environment
interactive input
```

對需要互動式終端的程式，Runtime 應能使用 Windows ConPTY 或等效機制。

---

# 18. Program Discovery

如果 Agent 發現：

```text
ffmpeg not found
```

不能立即：

```text
NEED_USER
```

應該：

```text
Observe environment
→ determine whether executable exists
→ search known installation paths
→ check PATH
→ check package managers
→ determine install possibility
→ attempt installation if possible
→ refresh environment
→ retry
```

同理：

```text
Python
Rust
Node
Git
7-Zip
CUDA
```

Agent 應具有自行調查與處理環境問題的能力。

---

# 19. Development / Engineering Runtime

Agent 必須能處理一般軟體工程工作：

```text
inspect project
discover repository structure
search source
search symbols
find references
read source
create source
modify source
delete source
rename
refactor
inspect dependencies
modify configuration
build
run
test
debug
inspect logs
install dependencies
inspect Git state
inspect diff
checkpoint
rollback
```

對程式碼修改，Runtime 應支援：

```text
file changes
patches
diffs
affected regions
change tracking
```

對 Repository 應能取得：

```text
project structure
source files
configuration
dependencies
build system
tests
Git state
recent changes
```

Runtime 不應硬編碼：

```text
Rust Workflow
Python Workflow
Node Workflow
```

Agent 應根據 Repository 的實際狀態自行決定：

```text
如何調查
修改什麼
如何 Build
如何 Test
如何 Debug
如何驗證
```

---

# 20. Code Modification and Change Management

程式碼修改必須具備可觀察性。

每次重要修改應能追蹤：

```text
what changed
which files changed
which regions changed
why the change was made
what action produced the change
```

Runtime / Observation System 應能產生：

```text
Diff
Patch
Changed Files
Affected Regions
```

Agent 不應只能看到：

```text
file modified
```

而應能觀察：

```text
Before
↓
Change
↓
After
```

重要修改應具備合理的：

```text
checkpoint
rollback
```

能力。

具體實作方式不在本需求中固定。

---

# 21. GUI Runtime

GUI 必須支援：

```text
screen capture
window discovery
window focus
application launch
click
double click
right click
drag
type
keyboard input
hotkeys
scroll
wait
```

優先使用：

```text
Windows UI Automation / Accessibility
```

如果無法取得語意化 UI 元素：

```text
OCR
Vision
coordinate interaction
```

例如 Agent 可以表達：

```text
Interact:
target = "Install"
action = click
```

Runtime 再決定使用：

```text
UI Automation
→ OCR
→ Vision
→ Coordinate fallback
```

GUI Runtime 必須依賴 Interactive User Desktop Session。

不能假設：

```text
Windows Session 0
```

具有正常互動式 GUI 能力。

如果遇到：

```text
UAC Secure Desktop
```

或其他需要實際使用者介入的 Windows 安全邊界，Runtime 應回報明確狀態，而不是無限重試。

---

# 22. Browser Runtime

Browser 是獨立於一般 GUI 的高階 Runtime。

至少支援：

```text
navigate
back
forward
reload
observe
click
type
select
scroll
submit
download
upload
wait
```

優先使用：

```text
DOM
Browser Automation
```

必要時使用：

```text
screen
mouse
keyboard
Vision
```

因此 Agent 可以根據實際情況：

```text
Browser DOM
→ GUI
→ Vision
```

互相 fallback。

---

# 23. Network Runtime

支援：

```text
HTTP
HTTPS
API
GET
POST
download
upload
streaming
```

Agent 可以取得：

```text
documentation
Git repositories
package information
web data
files
API results
```

Network failure 應成為 Observation，而不是立即終止 Task。

---

# 24. Media / Document Runtime

架構不得限制 Agent 只能處理純文字。

應能透過 Runtime 處理：

```text
TXT
MD
JSON
CSV
XML
HTML
PDF
Word
Excel
images
audio
video
archives
```

必要時透過：

```text
external programs
CLI tools
Python
OCR
Vision
FFmpeg
archive tools
```

完成操作。

Agent 不需要為每一種格式建立專用 Agent Tool。

---

# 25. System Environment Runtime

Agent 應能觀察及處理：

```text
environment variables
PATH
installed software
services
startup programs
network configuration
Windows settings
application configuration
```

原則是：

> 如果一般 Windows 使用者可以透過 Windows、PowerShell、GUI 或已安裝程式完成，Runtime 原則上不應預先排除。

---

# 26. External Devices

架構應保留未來支援：

```text
USB devices
external drives
network drives
printers
camera
microphone
other peripherals
```

不要求第一版全部完成。

但 Runtime abstraction 不應將系統限制在：

```text
filesystem + shell
```

---

# 27. Long-Running Jobs

Agent 必須支援長時間工作。

例如：

```text
video encoding
large file copy
cargo build
model generation
download
training
compression
```

不能要求 LLM 一直阻塞等待。

概念模式：

```text
START
 ↓
JobId
 ↓
OBSERVE
 ↓
WAIT
 ↓
OBSERVE
 ↓
COMPLETE / FAILED
```

Job State 至少需要能表達：

```text
job id
associated action
status
start time
elapsed time
process id if applicable
progress if available
stdout
stderr
exit code
result
```

實際 Rust 結構不固定。

---

# 28. Runtime Error Model

Runtime 必須將常見 Windows / Process / Browser / Network 錯誤標準化。

例如：

```text
AccessDenied
NotFound
AlreadyExists
InvalidArgument
ProcessFailed
ProcessCrashed
Timeout
InteractiveInputRequired
NetworkError
BrowserError
UiError
EncodingError
Unknown
```

錯誤不能只有：

```text
false
```

必須保留足夠資訊供 Agent 判斷下一步。

---

# 29. Permission Handling

預設策略：

```text
No per-action permission prompt.
```

Agent 在目前 Windows 使用者權限範圍內自主工作。

如果遇到：

```text
Access Denied
```

Agent 應先嘗試合理替代方法，例如：

```text
alternative path
alternative command
different application
user-owned directory
existing supported elevated mechanism
```

但不得假設 Agent 自動擁有：

```text
Administrator
SYSTEM
Kernel
```

權限。

如果 Windows 明確要求：

```text
Administrator
password
UAC confirmation
external account
```

而 Runtime 無法自行完成：

```text
NEED_USER
```

---

# 30. Context Management

LLM Context 不應包含整個 Observation Store。

每輪 Context 應根據當前任務狀態動態產生。

主要來源：

```text
Goal
Current Understanding
Active Problems
Unknowns
Active Hypotheses
Relevant Environment State
Relevant Observations
Recent Actions
Recent Failures
Remaining Work
Verification State
```

Context Compiler 應根據：

```text
Goal relevance
Current problem
Active hypothesis
Recent changes
Recent failures
Verification needs
```

選擇資訊。

大型輸出應優先使用：

```text
summary
head/tail
relevant region
diff
affected lines
error extraction
```

而不是無限制塞入完整內容。

具體壓縮演算法與 Context Budget 分配由實作者決定，但不得讓大型 Observation 無限制消耗 LLM Context。

---

# 31. Memory Model

至少區分：

```text
Task State
Observation Store
Action History
Job State
```

其中：

```text
Task State
```

保存目前工作記憶。

```text
Observation Store
```

保存環境觀察。

```text
Action History
```

保存 Agent 已經採取的行動及結果。

```text
Job State
```

保存 Background Job。

v3 不要求：

```text
Vector Database
RAG
Embedding Memory
Knowledge Graph
```

這些可以未來加入，但不是核心依賴。

---

# 32. Action History

每個重要 Action 至少應能保存：

```rust
struct ActionRecord {
    action_id: ActionId,

    timestamp: DateTime,

    action: Action,

    result: ActionResult,

    purpose: Option<String>,

    precondition_snapshot: Option<EnvironmentSnapshot>,

    environment_change: Option<EnvironmentDelta>,

    undo_information: Option<UndoInformation>,

    job_id: Option<JobId>,
}
```

實際結構可以調整。

其中重要概念包括：

```text
Action
Result
Purpose
Precondition
Environment Change
Undo Information
Job
Timestamp
```

不需要保存 LLM Chain-of-Thought。

---

# 33. Action Reversibility

Agent 應能區分不同 Action 的可逆程度。

概念分類：

```text
Reversible
PartiallyReversible
Irreversible
Unknown
```

例如：

```text
Reversible:
有備份的檔案修改

PartiallyReversible:
Git commit

Irreversible:
沒有備份的刪除
某些外部 POST
```

預設未知的 Action 不應被錯誤假設為可逆。

對高風險或不可逆操作，Agent 應在決策時考慮：

```text
risk
necessity
verification
rollback possibility
```

具體安全政策不在本文件中固定。

---

# 34. Checkpoint / Rollback

對重要工作或重要修改，系統應具備合理的：

```text
checkpoint
snapshot
rollback
```

能力。

目的：

```text
錯誤修改
→ 發現問題
→ 回到合理狀態
→ 改用其他策略
```

具體實作方式可以使用：

```text
Git
filesystem snapshot
copy
backup
transaction-like mechanism
```

或其他適當方式。

本需求不強制指定：

```text
Git stash
SQLite
JSON
特定 Snapshot Library
```

但系統必須避免讓重要修改完全失去可恢復性。

---

# 35. Goal Verification

最終成功不能只由：

```text
last action succeeded
```

判定。

例如：

```text
cargo build succeeded
```

不一定代表：

```text
User's application works.
```

因此 Verification 必須針對：

```text
Original Goal
```

進行。

驗證至少可以分成：

```text
Action Success
Subtask Success
Goal Success
```

例如：

```text
FFmpeg process exit code = 0
```

代表：

```text
Action Success
```

檢查輸出影片存在：

```text
Subtask Success
```

確認：

```text
格式
解析度
音訊
時長
```

均符合要求，才可能：

```text
Goal Success
```

---

# 36. Task Status

Task Status 與 Agent Decision 必須分離。

Task Status 概念上：

```rust
enum TaskStatus {
    Continue,
    Done,
    Blocked,
    Impossible,
    NeedUser,
}
```

其中：

```text
Continue
```

表示 Task 尚未完成，Agent 應繼續工作。

```text
Done
```

表示原始 Goal 已經被驗證完成。

```text
Blocked
```

表示理論上可以完成，但目前缺少必要條件。

```text
Impossible
```

表示在目前環境與能力限制下沒有合理完成方法。

```text
NeedUser
```

表示需要實際使用者介入。

Process 的：

```text
Running
```

不屬於 Task Status。

Process / Job 是否正在執行，應由：

```text
JobState
ProcessState
```

表示。

---

# 37. Agent Decision

Agent Decision 是：

> Agent Core 決定下一步要做什麼。

它與 Task Status 不同。

概念上可以包含：

```text
Act
Observe
Wait
Finish
```

例如：

```text
Act
→ Execute cargo test

Observe
→ inspect Cargo.toml

Wait
→ wait for background job

Finish
→ Done
```

Verification 是 Agent Core 的內部判斷階段，不需要成為獨立的 Runtime Action。

---

# 38. DONE

只有在：

```text
Original Goal verified
```

之後才能：

```text
DONE
```

Agent 不得因為：

```text
最後一個 command exit code = 0
```

就直接判定：

```text
DONE
```

---

# 39. BLOCKED

表示：

> 理論上可以完成，但目前缺少必要條件。

例如：

```text
網站需要登入
需要使用者提供密碼
需要插入 USB 裝置
需要 Administrator 權限
```

Blocked 不代表 Agent 已經放棄。

如果阻塞條件仍有可能由 Agent 自己解決，應繼續調查。

---

# 40. IMPOSSIBLE

表示：

> 在目前環境與能力限制下，沒有合理方法完成。

例如：

```text
要求不存在的硬體功能
要求不存在的外部資料
要求無法取得的硬體
```

Agent 應提供證據，而不是單純聲稱 Impossible。

---

# 41. NEED_USER

只在真正需要使用者介入時使用。

Agent 不應因為：

```text
不知道
命令失敗
缺少套件
找不到檔案
網路暫時失敗
第一次嘗試失敗
```

就要求使用者。

應先自主：

```text
Observe
Search
Inspect
Experiment
Retry
Alternative Action
```

只有確實需要：

```text
credentials
confirmation
physical interaction
privileged operation
external account authorization
```

等情況時，才進入：

```text
NEED_USER
```

---

# 42. Autonomous Recovery

遇到問題時：

```text
Observe Error
 ↓
Classify Error
 ↓
Generate Possible Causes
 ↓
Choose Information-Gathering or Recovery Action
 ↓
Execute
 ↓
Observe
 ↓
Evaluate
 ↓
Continue / Replan
```

例如：

```text
cargo build
 ↓
error: linker not found
 ↓
inspect toolchain
 ↓
check installed linker
 ↓
inspect PATH
 ↓
install required dependency if possible
 ↓
refresh environment
 ↓
cargo build
```

---

# 43. Loop Avoidance

Agent 不應無限重複沒有進展的操作。

如果出現：

```text
same/similar action
+
same/similar result
+
no meaningful environment change
+
no progress
```

Agent 應：

```text
gather more information
change hypothesis
change action
change strategy
```

而不是無限 retry。

短期實作可以存在過渡性的 retry / timeout 保護，但不得把固定 retry 次數視為長期 Agent 的主要 Loop Detection 機制。

長期目標是根據：

```text
Action History
Observation
Environment Change
Progress
Hypothesis
```

判斷是否陷入 Loop。

---

# 44. No Fixed Planning Tree

Agent 不需要在開始時建立完整：

```text
Plan A
 ├── Step 1
 ├── Step 2
 ├── Step 3
 └── Step 4
```

因為真實電腦環境具有：

```text
uncertainty
external state
errors
unexpected results
```

Agent 應採：

```text
incremental planning
```

即：

```text
現在知道什麼
→ 下一步最合理的是什麼
→ 執行
→ 觀察結果
→ 更新理解
→ 再決策
```

可以存在短期計畫，但計畫必須能被 Observation 推翻與修改。

---

# 45. LLM Interaction Protocol

每輪 LLM 應收到足以做出下一步決策的資訊：

```text
SYSTEM INSTRUCTIONS
+
USER GOAL
+
CURRENT AGENT STATE
+
RELEVANT OBSERVATIONS
+
AVAILABLE RUNTIME CAPABILITIES
```

LLM 回傳：

```text
Decision
+
Action
```

或：

```text
Final Status
```

具體 Prompt Template 不在本需求中固定。

但協議必須保持：

```text
structured input
→ structured decision
→ validation
→ execution
```

---

# 46. Structured LLM Output

LLM 應優先使用結構化輸出。

概念格式：

```json
{
  "type": "action",
  "action": {
    "kind": "execute",
    "program": "cargo",
    "args": ["test"]
  }
}
```

或：

```json
{
  "type": "observe",
  "request": {
    "target": "filesystem",
    "path": "C:\\project"
  }
}
```

或：

```json
{
  "type": "finish",
  "status": "done",
  "verification": "All tests passed."
}
```

實際 schema 可以在 Interface Specification 中定義。

Runtime 不得直接信任 LLM 輸出。

必須：

```text
parse
→ validate
→ normalize
→ execute
```

如果 Local LLM 輸出：

```text
markdown wrapper
invalid JSON
missing field
invalid enum
malformed arguments
```

系統應具有合理的：

```text
parse recovery
validation error
correction retry
```

機制。

---

# 47. Runtime Must Not Trust LLM Output Blindly

LLM 回傳的 Action 必須經過：

```text
parse
validate
normalize
execute
```

Runtime 必須防止：

```text
invalid arguments
invalid paths
malformed requests
invalid process configuration
resource exhaustion
```

但不要把正常能力限制成大量 hard-coded allowlist。

Capability 應保持廣泛。

Policy / Safety 應作為獨立的控制邊界。

---

# 48. State Persistence

Long-running Task 必須具備持久化能力。

至少應能保存：

```text
Agent State
Action History
Important Observations
Job State
Verification State
```

目的包括：

```text
process crash
Agent restart
Windows restart
long-running task
resume
```

實際儲存方式不固定。

可以使用：

```text
JSON
JSONL
SQLite
其他適當 persistence mechanism
```

第一版可以使用較簡單的持久化方式，但架構不能假設 Agent 永遠只執行數秒鐘。

---

# 49. Logging

至少提供：

```text
INFO
WARN
ERROR
DEBUG
```

每個重要 Action 都應有：

```text
timestamp
action id
action type
result
duration
```

例如：

```text
[16:08:22] ACTION execute cargo test
[16:08:31] RESULT exit_code=101
[16:08:31] OBSERVE stderr
[16:08:32] DECISION inspect Cargo.toml
```

Logging 不得要求輸出 LLM Chain-of-Thought。

可以記錄：

```text
Decision
Action
Observation
Evidence
Hypothesis summary
Result
Verification
```

但不要求保存模型的私有推理過程。

---

# 50. Cancellation

使用者必須能停止 Agent。

例如：

```text
Ctrl+C
```

Agent 應：

```text
stop scheduling new actions
cancel/wind down active work when possible
terminate managed jobs when configured
persist current state
exit cleanly
```

如果某個外部操作無法立即取消，Agent 應正確記錄：

```text
Running
Stopping
Completed
Unknown
```

而不是假裝操作已經停止。

---

# 51. Timeout

每個 Runtime Action 應支援 timeout。

例如：

```rust
ExecuteRequest {
    timeout: Option<Duration>,
}
```

Timeout 不等於 Task Failure。

應產生：

```text
Observation:
Action timed out.
```

然後由 Agent 決定：

```text
wait longer
inspect process
terminate
retry
alternative method
```

---

# 52. Resource Awareness

Agent 應知道基本資源狀態：

```text
CPU
RAM
GPU
VRAM
disk
network
processes
```

例如：

```text
VRAM = 16GB
```

Agent 應能將其作為決策資訊。

例如：

```text
避免同時啟動大量 GPU 工作
選擇適合目前資源的模型
等待其他 Job
降低並行度
```

Resource awareness 不應被硬編碼成特定硬體 Workflow。

---

# 53. Concurrency

Runtime 可以支援多個 Background Job。

Agent Core 第一階段可以維持：

```text
single primary decision loop
```

即：

```text
one LLM decision at a time
```

Runtime 可以同時管理：

```text
job A
job B
job C
```

Agent 再觀察其狀態。

不要求 Multi-Agent。

---

# 54. No Multi-Agent Requirement

v3 不要求：

```text
Planner Agent
Executor Agent
Reviewer Agent
Research Agent
Coder Agent
```

多 Agent 架構。

核心優先：

```text
One strong LLM
+
Computer Runtime
+
Observation
+
State
+
Verification
```

而不是：

```text
many specialized agents
```

未來如果模型能力或實際需求證明 Multi-Agent 有價值，可以擴充，但不得成為 v3 核心依賴。

---

# 55. No Mandatory RAG

v3 不要求：

```text
Vector DB
Embedding
RAG
Knowledge Graph
```

如果 Agent 需要資訊：

```text
Observe
Search
Read
Inspect
Experiment
Test
```

即可。

未來可以加入長期 Memory，但不得成為核心 Agent Loop 的必要依賴。

---

# 56. Search

Web Search 應被視為 Runtime 能力，而不是固定 Workflow。

Agent 可以自行判斷：

```text
需要外部資訊
→ search
→ inspect result
→ evaluate evidence
→ act
```

Search backend 可以替換。

Search 結果應被視為：

```text
Evidence
```

而不是自動成為：

```text
Fact
```

---

# 57. Browser + Web Search

Browser 和 Search 不應完全綁定。

Agent 可以：

```text
HTTP
Browser
Search API
CLI
```

選擇最適合的方法。

例如：

```text
查 documentation
→ HTTP

需要登入網站
→ Browser

搜尋公開資訊
→ Search

網站沒有 API
→ Browser
```

---

# 58. Verification Hierarchy

驗證至少分三層：

```text
Action Success
Subtask Success
Goal Success
```

例如：

```text
FFmpeg process exit code = 0
```

代表：

```text
Action Success
```

檢查輸出影片存在：

```text
Subtask Success
```

確認：

```text
影片格式
解析度
音訊
時長
```

均符合使用者要求，才可能：

```text
Goal Success
```

Verification 可以結合：

```text
Runtime observation
tests
file inspection
process inspection
browser state
semantic evaluation
```

具體驗證方法由 Agent 根據 Goal 決定。

---

# 59. Repository / Engineering Context

對軟體工程任務，Context Compiler 應能形成專門的 Repository Context。

不要求將整個 Repository 放入 Context。

應優先選取：

```text
Current problem
Relevant files
Relevant symbols
References
Dependencies
Build configuration
Tests
Recent errors
Recent changes
Current diff
```

例如修復 Bug 時，理想 Context 應接近：

```text
Goal
↓
Observed failure
↓
Relevant source
↓
Relevant dependency/configuration
↓
Related tests
↓
Recent modifications
↓
Current diff
```

而不是：

```text
Entire Repository
```

---

# 60. Safety and Runtime Boundary

Runtime 應保持廣泛能力，但仍必須遵守作業系統本身的權限與安全邊界。

基本原則：

```text
Capability
    ↓
Runtime
    ↓
Current User Permissions
    ↓
Windows
```

Agent 不得假設：

```text
Administrator
SYSTEM
Kernel
```

能力。

Runtime 可以提供：

```text
error detection
permission detection
path normalization
resource limits
process cleanup
```

等基礎保護。

但不應透過大量硬編碼 allowlist 將 Runtime 變成只能執行少數固定命令的系統。

---

# 61. Runtime Internals

建議 Rust 模組：

```text
src/
├── main.rs
│
├── agent/
│   ├── mod.rs
│   ├── core.rs
│   ├── state.rs
│   ├── decision.rs
│   ├── verification.rs
│   ├── recovery.rs
│   └── loop_detection.rs
│
├── llm/
│   ├── mod.rs
│   ├── client.rs
│   ├── message.rs
│   ├── parser.rs
│   └── schema.rs
│
├── observation/
│   ├── mod.rs
│   ├── store.rs
│   ├── environment.rs
│   ├── filesystem.rs
│   ├── process.rs
│   ├── browser.rs
│   ├── visual.rs
│   ├── delta.rs
│   └── compression.rs
│
├── context/
│   ├── mod.rs
│   ├── compiler.rs
│   └── budget.rs
│
├── runtime/
│   ├── mod.rs
│   ├── executor.rs
│   ├── filesystem.rs
│   ├── process.rs
│   ├── shell.rs
│   ├── gui.rs
│   ├── browser.rs
│   ├── network.rs
│   ├── system.rs
│   ├── jobs.rs
│   └── safety.rs
│
├── engineering/
│   ├── mod.rs
│   ├── repository.rs
│   ├── search.rs
│   ├── editing.rs
│   ├── diff.rs
│   ├── build.rs
│   ├── test.rs
│   └── git.rs
│
├── models/
│   ├── action.rs
│   ├── observation.rs
│   ├── result.rs
│   ├── job.rs
│   └── error.rs
│
├── persistence/
│   └── ...
│
└── config/
    └── mod.rs
```

上述只是建議架構。

實際目錄可以調整，但以下責任邊界應保持：

```text
Agent
LLM
Observation
Context
Runtime
Engineering
Persistence
Models
```

彼此分離。

---

# 62. Separation of Responsibility

## Agent Core

負責：

```text
goal
understanding
reasoning
decision
hypothesis
evaluation
replanning
recovery strategy
verification
```

## Observation

負責：

```text
collect
normalize
store
query
refresh
detect changes
```

## Context

負責：

```text
select relevant information
compress history
summarize observations
manage context budget
```

## Runtime

負責：

```text
execute
interact
manage processes
access filesystem
control browser
control GUI
```

## Engineering Runtime

負責：

```text
repository operations
code search
symbol navigation
code modification
diff
build
test
debug support
Git
checkpoint / rollback support
```

## LLM Client

負責：

```text
Ollama communication
model requests
response parsing
structured output handling
```

## Persistence

負責：

```text
save state
restore state
resume jobs/state
```

## Models

負責：

```text
shared data structures
serialization
state representation
```

---

# 63. Ollama Integration

v3 第一階段以 Ollama 為主要 LLM backend。

LLM Provider 應抽象成：

```rust
trait LlmProvider {
    async fn generate(
        &self,
        request: LlmRequest
    ) -> Result<LlmResponse, LlmError>;
}
```

第一個 implementation：

```text
OllamaProvider
```

未來可以加入：

```text
OpenAI-compatible API
DeepSeek
其他 local model
```

而不需要修改 Agent Core。

---

# 64. Model Context

Context size 應可配置。

例如：

```toml
[llm]
provider = "ollama"
model = "qwen3:14b"
context_size = 65536
temperature = 0.2
```

其中：

```text
context_size
```

代表模型 Provider 可使用的 Context 能力。

而：

```text
Context Budget
```

是 Context Compiler 實際決定目前這一輪要放多少資訊。

兩者是不同概念。

實際預設值可以依模型能力調整。

---

# 65. Configuration

重要 Runtime 行為應可設定：

```text
model
context size
temperature
timeouts
job retention
logging
browser backend
vision backend
working directory
persistence
resource limits
```

但核心 Autonomous Loop 不應依賴大量設定。

配置應該是：

```text
Runtime Parameters
```

而不是：

```text
Workflow Definition
```

---

# 66. Windows Implementation

優先使用可靠的 Rust Windows ecosystem 與 Windows API。

可能使用：

```text
windows-rs
tokio
serde
serde_json
reqwest
tracing
```

具體 dependencies 由實作階段決定。

不得因為某項 Windows 能力需要 Windows API 就退回大量 PowerShell hack。

應優先使用可靠的原生 API。

反過來，如果：

```text
PowerShell
CLI
Windows executable
```

是最可靠的方法，也可以使用。

---

# 67. Runtime Strategy

Runtime 可以混合：

```text
Windows API
PowerShell
CMD
CLI
Browser automation
GUI automation
HTTP
external programs
Python
```

重點不是所有事情都必須用 Rust API 完成。

重點是：

```text
LLM / Agent Core
        ↓
Unified Runtime Abstraction
        ↓
Real Computer
```

---

# 68. Runtime Must Be Extensible

未來可以新增：

```text
Docker
WSL
SSH
GPU tools
Cloud APIs
Database
Serial devices
USB
VM
```

而不需要修改 Agent Core 的基本決策模型。

---

# 69. Agent Core Must Not Know Windows Details

Agent Core 不應直接依賴：

```text
CreateProcess
PowerShell
Win32
UI Automation
HTTP client
Chrome
FFmpeg
```

它只需要知道：

```text
Observe
Act
Result
```

以及：

```text
State
Observation
Decision
Verification
```

這是重要架構邊界。

---

# 70. Runtime Must Not Make High-Level Decisions

Runtime 不應決定：

```text
應該修哪個 bug
應該安裝哪個 package
應該修改哪個檔案
應該採用哪種策略
應該使用哪個假設
```

Runtime 只負責：

```text
execute requested operation
return accurate observation
```

---

# 71. Persistence and Resume

Agent 必須能支援：

```text
Long Task
↓
Save State
↓
Process Crash / Restart
↓
Restore State
↓
Observe Current Environment
↓
Resume Decision Loop
```

Resume 時不得盲目假設上一個 Action 一定成功或一定失敗。

尤其對：

```text
Irreversible Action
```

應先重新觀察環境，判斷：

```text
Action 是否已經發生
```

再決定是否繼續。

---

# 72. Core Design Goal

最終形成：

```text
        ┌─────────────────────┐
        │        LLM          │
        │                     │
        │ Understand          │
        │ Reason              │
        │ Decide              │
        │ Adapt               │
        │ Verify              │
        └──────────┬──────────┘
                   │
            Observe / Act
                   │
        ┌──────────▼──────────┐
        │    Agent Core       │
        │                     │
        │ State               │
        │ Hypothesis          │
        │ Decision            │
        │ Recovery            │
        │ Verification       │
        └──────────┬──────────┘
                   │
        ┌──────────▼──────────┐
        │   Computer Runtime  │
        │                     │
        │ Windows             │
        │ Filesystem          │
        │ Processes           │
        │ Shell               │
        │ GUI                 │
        │ Browser             │
        │ Network             │
        │ Programs            │
        │ Engineering         │
        └──────────┬──────────┘
                   │
                   ▼
              Real Computer
```

LLM 是：

```text
brain
```

Agent Core 是：

```text
working mind / decision system
```

Runtime 是：

```text
hands + actuators
```

Observation 是：

```text
sensors + perception interface
```

State 是：

```text
working memory
```

Verification 是：

```text
checking whether reality matches the goal
```

---

# 73. Long-Term Model Evolution

Agent v3 的架構應允許以下演化：

```text
Current:
Local 14B Model
        ↓
Basic Agent Reasoning
        ↓
Runtime

Future:
Stronger Local Model
        ↓
Better Reasoning
        ↓
Better Planning
        ↓
Better Code Understanding
        ↓
Better Environment Understanding
        ↓
Better Autonomous Problem Solving
        ↓
Same Runtime
```

因此，模型變強不應要求：

```text
重新設計 Runtime
重新設計所有 Tools
重新建立 Workflow
```

理想狀態是：

```text
Model Capability ↑
        ↓
Agent Problem-Solving Capability ↑
```

而：

```text
Computer Runtime
Observation
State
Persistence
Verification
```

保持相對穩定。

---

# 74. Final Non-Goals

v3 不以以下項目為核心目標：

```text
Multi-Agent
Vector Database
RAG
Complex workflow engine
Huge planner tree
Hundreds of specialized tools
Fully autonomous unrestricted administrator privilege
Perfect GUI understanding
Perfect reasoning
Perfect long-term memory
```

也不要求：

```text
完全複製 Codex 的內部架構
```

真正目標是建立一個：

```text
Local
General-purpose
Goal-oriented
Observation-driven
Adaptive
Computer-operating
Autonomous Agent
```

---

# 75. Primary Success Definition

AI Agent v3 的成功標準不是：

```text
Tool 數量很多
```

也不是：

```text
LLM 能產生漂亮的回答
```

而是：

> 使用者給 Agent 一個實際的電腦工作目標後，Agent 能夠在真實 Windows 環境中自行觀察、理解、推理、操作、處理錯誤、調整策略並驗證結果，直到目標完成，或有充分證據證明自己受到環境限制。

核心能力：

```text
SEE
↓
UNDERSTAND
↓
HYPOTHESIZE
↓
ACT
↓
CHECK
↓
EVALUATE
↓
ADAPT
↓
ACT AGAIN
↓
VERIFY
```

而不是：

```text
THINK
↓
CALL TOOL
↓
ANSWER
```

---

# 76. Implementation Priority

Implementation Priority 只描述建議的能力發展方向，不代表固定 Workflow。

## Phase 1 — Foundation

建立：

```text
Rust project
configuration
logging
error system
LLM provider
basic Agent State
basic persistence
```

核心目標：

```text
LLM ↔ Agent Core ↔ Runtime
```

能形成基本閉環。

---

## Phase 2 — Core Computer Runtime

建立：

```text
Filesystem
Process
Shell
PowerShell
CMD
stdout
stderr
exit code
timeout
background jobs
basic environment observation
```

並建立：

```text
Observe
→ Act
→ Result
→ Observe
```

的可靠循環。

Interactive Shell / ConPTY 可在此階段加入。

---

## Phase 3 — Agent Intelligence

建立：

```text
Goal
Understanding
Hypothesis
Decision
Recovery
Loop Avoidance
Observation Store
Context Compiler
Verification
```

使 Agent 從：

```text
LLM + Runtime
```

成為真正：

```text
Goal-oriented autonomous loop
```

---

## Phase 4 — Engineering Runtime

建立：

```text
Repository Discovery
Code Search
Symbol Search
Code Navigation
Code Editing
Diff
Build
Test
Debugging
Git
Checkpoint / Rollback
```

使 Agent 能處理：

```text
inspect project
→ understand failure
→ modify code
→ build
→ test
→ diagnose
→ modify again
→ verify
```

---

## Phase 5 — GUI / Browser

建立：

```text
GUI
UI Automation
Screenshot
OCR
Vision
Browser
DOM
Browser Automation
```

---

## Phase 6 — Advanced Runtime

擴展：

```text
Network
Media
Documents
System Configuration
External Devices
WSL
Docker
SSH
GPU Tools
```

實際階段可以依實作進度調整。

---

# 77. First Working Milestone

第一個真正可工作的 Milestone 不需要：

```text
GUI
Browser
Vision
Multi-Agent
RAG
Vector Database
```

優先建立：

```text
User Goal
 ↓
Local LLM
 ↕
Agent Core
 ↕
Observation
 ↕
Filesystem + Process + Shell
 ↓
Windows
```

並能完成例如：

```text
讀取專案
→ 找問題
→ 修改檔案
→ 執行命令
→ 讀取錯誤
→ 修改
→ 再執行
→ 測試
→ 驗證
```

這個 Loop 能可靠運作後，再逐步擴展：

```text
Engineering
→ Browser
→ GUI
→ System
→ External Devices
```

---

# 78. Core Requirement Summary

AI Agent v3 必須遵守以下核心原則：

1. Agent 以 Goal 為中心，而不是以 Tool 為中心。

2. LLM 負責理解、推理、決策、重新規劃與問題解決。

3. Agent Core 負責維護狀態、假設、決策、恢復與驗證。

4. Runtime 負責實際操作 Windows。

5. Observation 負責將真實環境轉換成 Agent 可以使用的資訊。

6. Context Compiler 負責從大量 Observation 中選擇目前真正相關的資訊。

7. Runtime-facing abstraction 應保持精簡，核心概念為 Observe / Act，必要時支援 Wait。

8. 不應建立大量針對單一程式或單一任務的硬編碼 Tool。

9. Failure 是 Observation，不是立即終止。

10. Unknown 是合法狀態，Agent 可以透過觀察、搜尋、實驗與測試消除 Unknown。

11. Hypothesis 不得直接視為 Fact。

12. Agent 不應盲目重複沒有進展的操作。

13. Agent 必須能根據新的 Observation 修改原本的理解與策略。

14. DONE 必須根據原始 User Goal 驗證，而不是最後一個 Action 是否成功。

15. Action Success、Subtask Success、Goal Success 必須保持概念區分。

16. Runtime 必須提供廣泛的 Windows 操作能力，但不得假設超出目前使用者權限的能力。

17. Agent Core 與 Runtime 必須保持責任分離。

18. Engineering Runtime 應支援 Repository、Code Search、Code Editing、Diff、Build、Test、Debug、Git 與 Recovery。

19. 重要修改應具備合理的 Change Tracking、Checkpoint 或 Rollback 能力。

20. Long-running Task 必須能保存狀態並支援 Resume。

21. LLM Output 必須經過 Parse、Validate、Normalize 後才能執行。

22. Local Model 能力不足時可以使用過渡性可靠性機制，但不得將固定 Workflow 硬編碼成 Agent 的核心問題解決方式。

23. 不要求 Multi-Agent、RAG、Vector Database 或其他複雜架構作為核心依賴。

24. Agent Runtime 應與模型能力解耦。

25. 未來模型越強，應主要表現在 Agent 的理解、推理、規劃、適應與問題解決能力提升，而不是需要重新設計 Runtime。

最終架構應接近：

```text
             ┌───────────────┐
             │  Local Model  │
             │ Intelligence  │
             └───────┬───────┘
                     │
                     ▼
             ┌───────────────┐
             │  Agent Core   │
             │               │
             │ Understand    │
             │ Hypothesize   │
             │ Decide        │
             │ Adapt         │
             │ Verify        │
             └───────┬───────┘
                     │
              Observe / Act
                     │
             ┌───────▼───────┐
             │    Runtime    │
             │               │
             │ Windows       │
             │ Files         │
             │ Process       │
             │ Shell         │
             │ GUI           │
             │ Browser       │
             │ Network       │
             │ Engineering   │
             └───────┬───────┘
                     │
                     ▼
                Real World
```

核心理念：

```text
Model provides intelligence.

Agent Core provides autonomous reasoning and adaptation.

Observation provides perception.

Runtime provides computer capability.

State provides working memory.

Verification checks reality against the goal.
```

最終目標：

> 建立一個不依賴固定 Workflow、能隨本地模型能力提升而持續增強，並逐漸接近人類專家使用電腦解決實際問題方式的本地 Autonomous Computer Agent。


補充

 ### Deterministic Core Testing

Agent Core 必須能在不依賴真實 Windows 環境與非確定性 LLM 的情況下進行測試。

因此第一階段應提供：

- Mock LLM
- Fake Runtime
- Deterministic Observation
- Deterministic Agent Scenario

使 Agent Core 可以在固定輸入下重現相同的 Decision / Action / Observation 流程。

Fake Runtime 與 Mock LLM 是測試基礎設施，不是正式 Runtime 的替代品。