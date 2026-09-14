# AI Agent v3 — Development Workflow

本文件只定義「如何工作」。它不定義 Agent Runtime 功能，也不取代 `AGENTS.md` 或 `docs/` 的穩定設計。

## 1. 新 AI Session 的固定啟動流程

每次新的 AI coding session，固定依以下順序處理，不自行增加 startup reading：

```text
1. AGENTS.md
2. DEVELOPMENT_WORKFLOW.md
3. IMPLEMENTATION_STATUS.md
4. README.md
5. docs/README.md
```

以上 5 份是唯一的「必讀入口文件」。

第 1–5 步完成後，才由 `docs/README.md` 判斷本次任務需要哪些其他文件。

禁止把以下內容列為 startup 必讀：

```text
docs/ 全部文件
src/ 全部 source
tests/ 全部 tests
history/ 全部歷史
```

這些都是「依目前任務才讀」的內容。

同時檢查：

```text
git status
git log --oneline --decorate -5
```

## 2. 文件責任只有四層

```text
AGENTS.md
    = AI 必須遵守的規則與架構約束

DEVELOPMENT_WORKFLOW.md
    = AI 如何開始、實作、驗證、中斷、恢復

IMPLEMENTATION_STATUS.md
    = Repository 現在實際做到什麼

README.md
    = 人類閱讀的專案介紹與目標架構

docs/README.md
    = Stable Documents 的唯一導航入口

docs/*.md
    = Stable Requirements / Design / Interface / Acceptance / Test / Roadmap

history/
    = 歷史資料，不是目前指令

active conversation
    = 一次性 implementation prompt、debugging、暫時決策與未批准想法
```

不要讓不同文件重複成為同一規則的 source of truth。

特別是：

- `AGENTS.md` 不負責追蹤進度。
- `DEVELOPMENT_WORKFLOW.md` 不負責描述目前實作細節。
- `IMPLEMENTATION_STATUS.md` 不負責定義 target architecture。
- `README.md` 不負責定義 implementation status。
- `docs/README.md` 只負責導航，不重新定義架構。
- `history/` 不負責提供目前工作指令。

## 3. 「必讀」與「目前需要讀」必須分開

新 AI 進 repo 時，不需要推理哪些入口文件重要。

固定規則是：

```text
必讀：
AGENTS.md
DEVELOPMENT_WORKFLOW.md
IMPLEMENTATION_STATUS.md
README.md
docs/README.md
```

```text
目前需要讀：
由 docs/README.md 根據這一次的 task 選出的 stable docs
+ 直接相關 source
+ 直接相關 tests
```

例如目前只修一個測試失敗，不需要讀完整 Requirements、Roadmap、所有 Runtime 文件。

例如要實作 Agent Core increment，才讀該 increment 的 design、直接依賴的 interface / requirements / acceptance，以及相關 source/tests。

## 4. 如何判斷目前狀態

不得只相信任何單一文件。

判斷 implementation reality 時使用：

```text
IMPLEMENTATION_STATUS.md
        +
Source Code
        +
Tests
        +
實際 Compile / Test / Execution Results
```

其中實際可驗證結果優先於狀態文字。

Stable design 與 current implementation 是不同概念：

```text
Stable Design
= 系統應該是什麼

Current Implementation
= Repository 現在實際是什麼
```

兩者不同不代表文件錯誤；可能只是尚未實作完成。

## 5. Implementation Increment

本專案一次只處理一個已批准的 implementation boundary。

固定流程：

```text
確認 current state
    ↓
讀取 task-specific stable design
    ↓
檢查直接相關 source / tests
    ↓
定義 scope / non-goals / acceptance criteria
    ↓
取得使用者對 increment 的明確批准
    ↓
在 active conversation 產生一次性 implementation prompt
    ↓
實作
    ↓
Compile / Test / 必要驗證
    ↓
只修目前 increment 範圍內的問題
    ↓
更新 IMPLEMENTATION_STATUS.md
    ↓
Git checkpoint
    ↓
報告結果
    ↓
STOP
```

完成一個 increment 後不得自行開始下一個 increment。

不存在永久的 `09_IMPLEMENTATION_TASK.md` 或同類 task 文件。

## 6. Stable Design 不因實作完成而例行修改

Implementation 完成後：

1. 檢查是否符合適用的 stable design。
2. 若符合，只更新 `IMPLEMENTATION_STATUS.md`。
3. 必要時新增 history record。
4. 不因「這次實作完成」而修改 stable design。

只有以下情況才修改 stable design：

```text
需求改變
架構原則改變
穩定介面改變
驗收標準改變
長期依賴關係改變
既有設計被證明錯誤或不完整
```

若發現真正的 stable-design conflict：

```text
STOP
→ 描述 conflict
→ 說明影響
→ 取得批准
→ 修改 stable design
→ 重新定義 implementation boundary
→ 才能繼續
```

不得為了讓目前程式通過而偷偷改寫設計。

## 7. IMPLEMENTATION_STATUS.md 的用途

`IMPLEMENTATION_STATUS.md` 是 current-state 文件。

至少要能回答：

```text
目前完成什麼？
目前哪個 increment 完成？
哪些長期能力仍未完成？
目前驗證證據是什麼？
目前已知問題是什麼？
下一個 increment 是否已批准？
```

狀態使用：

```text
NOT_STARTED
IN_PROGRESS
BLOCKED
COMPLETED
```

「某個第一版基礎能力已完成」與「該長期能力全部完成」必須明確區分，不得使用會讓 AI 自行推理的模糊措辭。

## 8. 中斷與恢復

如果 session 中斷，不得直接重做，也不得假設已完成。

先確認：

```text
已完成什麼
未完成什麼
目前 source 是否可編譯
哪些 tests 通過 / 失敗
目前 Git 狀態
IMPLEMENTATION_STATUS.md 記錄
```

然後從實際 checkpoint 繼續。

## 9. 發現無關問題

不要因為看到其他問題就擴大 scope。

```text
目前 increment 必須處理的問題 → 本次處理
與目前 increment 無關的問題 → 記錄，不順便重構
```

只有無關問題會阻止目前 increment 正確完成時，才可以納入本次工作。

## 10. 最小 Context 原則

Context 的目標不是「讀最多」，而是「讀到足以正確完成這一次工作」。

固定順序：

```text
入口文件
→ docs/README.md 導航
→ task-specific stable docs
→ direct source
→ direct tests
→ verification
```

只有發現實際 dependency 或 stable-design conflict，才擴大 context。
