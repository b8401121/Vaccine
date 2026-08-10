---
name: vaccine-dev
description: 專案開發指南：預防接種指南助手 (Rust + Wasm + JS) 的架構、編譯與部署規則
---

# 預防接種指南助手 (Vaccine App) 開發指南

本專案是一個結合 Rust WebAssembly 與 Vanilla JavaScript 的前端網頁應用程式，主要用於診所內的疫苗時程、生長曲線與補打間隔試算。

## 1. 專案架構
- **`vaccine-core/`**: 核心邏輯層 (Rust)。所有的醫療計算、時程推算都在這裡進行，並編譯成 WebAssembly。
- **`vaccine-app/src/`**: 前端展示層 (HTML/CSS/JS)。負責 UI 互動與 Wasm 模組的載入。
- **`gh-pages` 分支**: 靜態網站部署分支。

## 2. 關鍵架構地雷與避坑指南 (CRITICAL RULES)

### A. WebAssembly 傳輸合約 (JSON 化)
**絕對不要**直接從 Rust 傳遞 JS 對象 (如 `js_sys::Object` 或 `Map`) 給前端，這會導致型別轉換的例外錯誤。
- **Rust 端**: 所有的 Wasm 匯出函數必須回傳 `String`。請使用 `serde_json::to_string` 將 Rust struct 序列化為 JSON 字串。
- **JS 端**: 在 `app.js` 呼叫 Wasm 函數後，必須第一時間呼叫 `JSON.parse(res)` 來還原物件。

### B. JS 與 Rust 的變數名稱轉換
- JS 端習慣使用 `camelCase` (例如: `vaccineId`, `lastDoseNum`)。
- Rust Wasm 端嚴格要求 `snake_case` (例如: `vaccine_id`, `last_dose_num`)。
- **規則**: `app.js` 中的 `fallbackInvoke` (或直接呼叫的橋樑函式) 必須手動把變數名稱對應好，否則傳遞給 Rust 的值會是 `undefined`，導致 `Cannot read properties of undefined (reading 'length')` 的記憶體錯誤。

### C. GitHub Pages 強制快取破解法
GitHub Pages 的 CDN 快取極度頑固，會無視 `?v=123` 的 Query String 快取破解法。
當你修改了 Wasm、JS 或 CSS 後，如果前端沒有更新，必須採取以下激進的快取破解手段：
1. **Wasm 檔案**: 不要覆蓋舊的 Wasm 資料夾。每次更新 Wasm，請建立一個全新的資料夾 (例如從 `wasm2/` 改名為 `wasm3/`)，並更新 `app.js` 中的 import 路徑。
2. **JS/CSS 檔案**: 直接重新命名實體檔案 (例如 `main.js` -> `app.js`，`styles.css` -> `styles_final.css`)，然後去 `index.html` 更新 `<script>` 和 `<link>` 標籤。

## 3. 開發與編譯流程

### 編譯 Rust 為 Wasm
在 `vaccine-core/` 目錄下執行：
```bash
wasm-pack build --target web
```
然後手動將 `pkg/` 裡面的檔案複製到 `vaccine-app/src/wasm2/` (或其他新命名的資料夾)。

### 測試前端
在 `vaccine-app/src/` 啟動任意本機伺服器即可測試 (例如: `python -m http.server`)。

## 4. 部署流程 (Deploy)
部署需要將 `vaccine-app/src` 的內容推送到 `gh-pages` 分支的根目錄。
目前已經有寫好的 Python 部署腳本 `f:\vaccine_fix.py`，你也可以隨時自己手寫 `git worktree` 的部署腳本來覆蓋 `gh-pages` 分支。
