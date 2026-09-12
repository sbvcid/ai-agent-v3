# IMPLEMENTATION_STATUS.md

## Current Stage
* **Stage 5: Deterministic Closed-Loop & Event Trace (AT-CORE-001)** (COMPLETED)

## Overall Status
* **IN_PROGRESS**

## Completed Stages
* **Stage 1: 專案基底、規格審查與 IMPLEMENTATION_STATUS.md** (COMPLETED)
  - 完成所有規格文件（AGENTS.md、DEVELOPMENT_WORKFLOW.md、docs/01~05、Agent Runtime 實作任務.txt）之深度審查。
  - 完成三個關鍵 SPEC CONFLICT 的正式裁決，並已全面同步回寫至相關規格文件。
  - 建立最小必要的 `Cargo.toml` 與 `src/lib.rs`。
  - 執行 `cargo check` 驗證專案基底編譯通過。
  - 建立本進度追蹤基準文件。
* **Stage 2: Canonical Core Data Model** (COMPLETED)
  - 建立 Agent Core 的唯一 canonical domain model（定義於 `src/core/types.rs` 並由 `src/core/mod.rs`、`src/lib.rs` 匯出）。
  - 實作型別：`Goal`, `Action`, `ActionType`, `ActionResult`, `Observation`, `ObservationKind`, `ExecutionState`, `FinalTaskStatus`, `AgentDecision`, `KnowledgeState`, `VerificationState`, `AgentState`。
  - 排除 `ToolCall` 與 `ToolResult`，嚴格遵守領域邊界。
* **Stage 3: Schema Serialization, Strict Validation & State Checkpoint** (COMPLETED)
  - 為 Core domain types 加入 Serde `Serialize, Deserialize` 衍生。
  - 採用 `#[serde(deny_unknown_fields)]` 實作嚴格架構驗證，拒絕未知/未定義屬性。
  - 實作領域不變條件驗證：`ValidationError` 與 `.validate()` 方法（驗證必填非空、ActionResult 失敗一致性等）。
  - 實作 `StateCheckpoint`：封裝 `checkpoint_id`, `timestamp_epoch_ms`, `step_index`, `state: AgentState`。
  - 定義 `CheckpointStore` 抽象 trait（`save_checkpoint`, `load_checkpoint`, `load_latest_checkpoint`）。
  - 實作 `JsonFileCheckpointStore`：純 JSON 檔案儲存與還原，完全與 Windows Runtime / Tools / LLM 解耦，排除 SQLite。
  - 建立 6 個 focused integration tests 驗證 round-trip、缺漏欄位拒絕、未知欄位防護、存取與 crash recovery 流程。
  - Code Review: PASS WITH CONCERNS（deferred issues 見 Known Issues）。
* **Stage 4: Deterministic Mock LLM & Fake Runtime** (COMPLETED)
  - 建立 `MockLlm`：scripted `AgentDecision` queue，FIFO 順序返回，deterministic fallback（`Finish(Blocked)`），記錄 call count。
  - 建立 `FakeRuntime`：scripted `ActionResult` by action_id，deterministic fallback failure，記錄所有收到的 `Action`。
  - 純 in-memory，無 OS side effects，無 network，無 random，無 time dependency。
  - 維持 Core boundary：`AgentDecision::Act(Action)` → Runtime → `ActionResult`。不引入 `ToolCall` / `ToolResult`。
  - 不建立 Agent Loop / Closed Loop / Event Trace / Observation Pipeline（Stage 5 範圍）。
  - 新增 11 個 unit tests（`src/core/test_doubles.rs`）+ 6 個 integration tests（`tests/test_doubles_tests.rs`）。
  - Code Review: PASS WITH CONCERNS。
* **Stage 5: Deterministic Closed-Loop & Event Trace (Phase 0 核心)** (COMPLETED)
  - 建立最小確定性 `AgentLoop`：串聯 `AgentState` → `MockLlm` → `AgentDecision` → `Action` → `FakeRuntime` → `ActionResult` → `Observation` → `AgentState` 更新 → `Finish`。
  - 建立純記憶體確定性 `EventTrace` 與 `LoopEvent`：記錄有序生命週期事件，無 OS/網路/時鐘副作用，支援兩次相同輸入產生完全相同 Trace 的確定性驗證。
  - 正式實作並強制 `DF-01` Invariant：`ExecutionState::Running + final_status = Some(...)` 嚴格回傳 `ValidationError::InvariantViolation`；合法終止態統一為 `ExecutionState::Waiting + final_status = Some(...)`。
  - 解決 `SF-02`：為 `MockLlm` 與 `FakeRuntime` 實作 `Default` trait，消除全部 clippy warnings。
  - 實作 AT-CORE-001 驗收場景：驗證 Action A 失敗 → 產生 Failure Observation → 更新 AgentState → LLM 收到失敗後調整策略採取 Action B → Action B 成功 → 驗證 → Finish(Done)。
  - 實作中斷恢復（Crash-Resume）閉環驗證：中途儲存 Checkpoint、清除記憶體 context、從 JSON 檔案還原狀態並續跑直到 Finish(Done)。
  - 新增 8 個 focused integration tests（`tests/closed_loop_tests.rs`），總計 38 個測試全數通過。

## Current Work
* Stage 5 工作已全部完成。等待使用者確認後開始 Stage 6。

## Incomplete Work
* **Stage 6: Safe Filesystem Tools** (NOT_STARTED)
* **Stage 7: Safe Process Execution** (NOT_STARTED)
* **Stage 8: Ollama Provider Adapter** (NOT_STARTED)
* **Stage 9: Real LLM Agent Integration & Release Gate** (NOT_STARTED)

## Tests
* `cargo check --lib`: PASS (Exit code 0)
* `cargo fmt --check`: PASS (Exit code 0)
* `cargo clippy --all-targets --all-features -- -D warnings`: PASS (0 warnings, Exit code 0)
* `cargo test` (all): PASS (38 tests passed, 0 failed)
  - Unit tests: 18 passed (8 core::types + 10 core::test_doubles)
  - Integration tests: 20 passed (6 schema_checkpoint + 6 test_doubles + 8 closed_loop)
  - Doc tests: 0

## Known Issues
### Resolved in Stage 5
1. `DF-01`: `AgentState::validate()` 中強制約束 `Running + final_status` 為非法，已實作並通過單元測試驗證。
2. `SF-02`: `MockLlm` 與 `FakeRuntime` 實作 `Default`，Clippy 2 warnings 已全數消除。

### Deferred to Future Stages
1. `checkpoint_id` 尚未做 filename/path traversal 防護 → Stage 6 安全邊界階段處理。
2. Checkpoint write 尚未 atomic → Phase 0 不要求 power-loss durability，目前不處理。
3. Timestamp 預設 0 → deterministic design 可接受。
4. `SF-01`: Script 耗盡時之 Fallback 語意（`MockLlm -> Blocked`、`FakeRuntime -> failure`）→ 測試中已明確斷言 decisions/runtime 完全消耗；未來引入真實 Provider 時再重構。
5. `SF-03`: `FakeRuntime` 靜態 Key-Value 查詢 → 目前 AT-CORE-001 採用 Action A/B 不同 ID，未來需要重試序列時再擴充。

## Unresolved Questions
* 無未決問題。

## Next Step
* 等待使用者明確指示後，開始 **Stage 6: Safe Filesystem Tools**。

## Important Decisions & Canonical Architecture
1. **Specification Finalization Completed**:
   - 正式定稿 9-Stage Incremental Implementation Plan。
2. **Conflict 1 (Action / ActionResult vs ToolCall / ToolResult) Resolved**:
   - `ToolCall` 嚴格屬於 LLM / Provider / Wire protocol layer。
   - `Action`（`Observe`, `Execute`, `Interact`, `Wait`）是 Agent Core 的 Canonical Domain Model。
   - `ActionResult` 是 Runtime 執行的唯一 Canonical Result。
   - `ToolResult` 正式自系統型別中廢除。
   - Provider Adapter 負責將 `ToolCall` 驗證並轉換為 `Action`。
   - Runtime 保持獨立與通用，不直接處理 `ToolCall`。
3. **Conflict 2 (ExecutionState vs FinalTaskStatus) Resolved**:
   - 嚴格分離執行中狀態與終止狀態，禁止單一 enum 混用。
   - `ExecutionState`: `Running`, `Waiting`。
   - `FinalTaskStatus`: `Done`, `Blocked`, `Impossible`, `NeedUser`。
   - `AgentDecision::Finish(FinalTaskStatus)` 必須且只能攜帶 `FinalTaskStatus`。
   - 徹底移除 `Continue` 作為狀態型別。
4. **Conflict 3 (Persistence / Checkpoint in Phase 0) Resolved**:
   - Phase 0 採用「Minimal JSON Checkpoint 抽象」，嚴格禁止引入 SQLite 或外部資料庫。
   - 僅使用 Serde + JSON 檔案儲存與還原 `AgentState`。
   - 邊界專注於支援 Closed Loop 中的 `Crash → Resume` 驗證。
5. **Stage 5 Closed-Loop & State Transition Formalized**:
   - 嚴格強制 Invariant：`Running` 時 `final_status == None`；`Finished` 狀態轉移為 `Waiting` 並設置 `final_status = Some(...)`。
   - `AgentLoop` 提供 `step()` 與 `run()`，具備最大步數邊界防護與確定性 `EventTrace` 記錄。

## 9 Stage Plan Status
* [x] **Stage 1**: 專案基底、規格審查與 IMPLEMENTATION_STATUS.md (COMPLETED)
* [x] **Stage 2**: Canonical Core Data Model (COMPLETED)
* [x] **Stage 3**: Schema Serialization, Strict Validation & State Checkpoint (COMPLETED)
* [x] **Stage 4**: Deterministic Mock LLM & Fake Runtime (COMPLETED)
* [x] **Stage 5**: Deterministic Closed-Loop & Event Trace (Phase 0 核心) (COMPLETED)
* [ ] **Stage 6**: Safe Filesystem Tools (NOT_STARTED)
* [ ] **Stage 7**: Safe Process Execution (NOT_STARTED)
* [ ] **Stage 8**: Ollama Provider Adapter (NOT_STARTED)
* [ ] **Stage 9**: Real LLM Agent Integration & Release Gate (NOT_STARTED)
