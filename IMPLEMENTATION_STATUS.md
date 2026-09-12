# IMPLEMENTATION_STATUS.md

## Current Stage
* **Stage 4: Deterministic Mock LLM & Fake Runtime** (COMPLETED)

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
  - Code Review: PASS WITH CONCERNS（SHOULD FIX 見 Known Issues）。

## Current Work
* Stage 4 工作已全部完成。等待使用者確認後開始 Stage 5。

## Incomplete Work
* **Stage 5: Deterministic Closed-Loop & Event Trace (AT-CORE-001)** (NOT_STARTED)
* **Stage 6: Safe Filesystem Tools** (NOT_STARTED)
* **Stage 7: Safe Process Execution** (NOT_STARTED)
* **Stage 8: Ollama Provider Adapter** (NOT_STARTED)
* **Stage 9: Real LLM Agent Integration & Release Gate** (NOT_STARTED)

## Tests
* `cargo check --lib`: PASS (Exit code 0)
* `cargo fmt --check`: PASS (Exit code 0)
* `cargo test` (all): PASS (29 tests passed, 0 failed)
  - Unit tests: 17 passed (6 core::types + 11 core::test_doubles)
  - Integration tests: 12 passed (6 schema_checkpoint + 6 test_doubles)
  - Doc tests: 0
* `cargo clippy`: 2 warnings (`clippy::new_without_default`, tracked in SF-02)

## Known Issues
### Deferred from Stage 3 Code Review
1. `AgentState::validate()` 中 Running + final_status 的 invariant 檢查尚未真正回傳 error → 預計 Stage 5 前處理。
2. `checkpoint_id` 尚未做 filename/path traversal 防護 → 後續安全邊界階段處理。
3. Checkpoint write 尚未 atomic → Phase 0 不要求 power-loss durability，目前不處理。
4. Timestamp 預設 0 → deterministic design 可接受。

### Concerns from Stage 4 Code Review (SHOULD FIX)
1. `SF-01`: Script 耗盡時之 Fallback 語意（`MockLlm -> Blocked`、`FakeRuntime -> failure`）具掩蓋測試錯誤與假陽性風險 → 預計 Stage 5 實作 Agent Loop 時引入 Strict Mode 或明確檢查。
2. `SF-02`: `MockLlm` 與 `FakeRuntime` 缺少 `Default` 實作（Clippy 2 warnings） → Stage 5 開發前處理。
3. `SF-03`: `FakeRuntime` 為靜態 Key-Value 查詢，無法表達同 ID 重試情境 → Stage 5 或 7 視需求擴充。

## Unresolved Questions
* 無未決問題。

## Next Step
* 等待使用者明確指示後，開始 **Stage 5: Deterministic Closed-Loop & Event Trace**。

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

## 9 Stage Plan Status
* [x] **Stage 1**: 專案基底、規格審查與 IMPLEMENTATION_STATUS.md (COMPLETED)
* [x] **Stage 2**: Canonical Core Data Model (COMPLETED)
* [x] **Stage 3**: Schema Serialization, Strict Validation & State Checkpoint (COMPLETED)
* [x] **Stage 4**: Deterministic Mock LLM & Fake Runtime (COMPLETED)
* [ ] **Stage 5**: Deterministic Closed-Loop & Event Trace (Phase 0 核心) (NOT_STARTED)
* [ ] **Stage 6**: Safe Filesystem Tools (NOT_STARTED)
* [ ] **Stage 7**: Safe Process Execution (NOT_STARTED)
* [ ] **Stage 8**: Ollama Provider Adapter (NOT_STARTED)
* [ ] **Stage 9**: Real LLM Agent Integration & Release Gate (NOT_STARTED)
