# 預防接種指南助手 (Vaccine Assistant) 開發說明書

這是一套為「桃園 吳鎮宇親子耳鼻喉科診所」量身打造的網頁應用程式，主要用於試算嬰幼兒與成人的疫苗時程、補打間隔，以及生長曲線和 BMI。

## 核心技術架構
本專案採用 **Rust (WebAssembly)** 作為運算核心，搭配 **Vanilla JavaScript / HTML / CSS** 負責前端畫面，以確保醫療邏輯計算的高效與準確，並大幅提升網頁的載入速度。

### 專案目錄結構
```text
Vaccine/
├── vaccine-core/         # Rust 核心邏輯層 (WebAssembly 的原始碼)
│   ├── src/              # Rust 演算法 (生長曲線、疫苗間隔)
│   └── Cargo.toml
├── vaccine-app/          # 前端介面層
│   └── src/
│       ├── index.html    # 主頁面
│       ├── app.js        # 前端核心邏輯與 Wasm 橋樑
│       ├── styles_final.css # 樣式表 (深色護眼模式)
│       └── wasm2/        # 由 Rust 編譯產生的 WebAssembly 模組
└── README.md             # 本開發說明書
```

---

## 避坑指南與開發守則 (必讀)

由於本專案依賴 `Rust -> WebAssembly -> JavaScript` 的跨語言橋樑，且佈署於 GitHub Pages，請在開發時嚴格遵守以下守則，避免常見錯誤：

### 1. 變數命名代溝 (CamelCase vs Snake_case)
- **錯誤原因**：JavaScript 前端預設使用 `camelCase` (如 `vaccineId`)，而 Rust 端強烈要求 `snake_case` (如 `vaccine_id`)。
- **解法**：在 `app.js` 的 `fallbackInvoke` 函數中，呼叫 Wasm 前必須手動進行變數映射，例如 `const vaccine_id = args.vaccineId;`，否則會引發 `Cannot read properties of undefined (reading 'length')` 錯誤。

### 2. 跨語言資料傳輸 (JSON)
- **錯誤原因**：直接將 JavaScript 物件傳給 Wasm，或讓 Wasm 直接回傳 `JsValue::Object`，極易引發底層指標與型別對齊錯誤。
- **解法**：Rust 端所有的對外接口，回傳值必須全部經過 `serde_json::to_string` 轉換成字串；前端 `app.js` 在接收後，必須第一時間執行 `JSON.parse()` 將其還原為 JS 物件。

### 3. GitHub Pages 快取地獄 (強制破解法)
- **錯誤原因**：GitHub Pages 擁有極度強大且固執的 CDN 快取。傳統的 `?v=123` Query String 破解法在這裡**完全無效**。使用者會一直看到舊的畫面或載入舊的 Wasm，導致 JS 與 Wasm 版本不匹配而崩潰 (`LinkError`)。
- **解法**：
  1. **針對 Wasm**：每次更新 Rust 核心後，請把編譯出來的資料夾換個名字 (例如從 `wasm2/` 換成 `wasm3/`)，並同步修改 `app.js` 裡的 import 路徑。
  2. **針對 JS / CSS**：若大幅修改樣式或邏輯，請直接把實體檔案重新命名 (例如 `styles.css` 改為 `styles_final.css`，`main.js` 改為 `app.js`)，並更新 `index.html` 的引用。

---

## 編譯與部署 SOP

### 步驟一：修改與編譯核心 (Rust)
如果你修改了 `vaccine-core/src/lib.rs` 的疫苗邏輯：
1. 開啟終端機進入 `vaccine-core/` 目錄。
2. 執行編譯指令：`wasm-pack build --target web`
3. 建立一個全新的 Wasm 資料夾 (例如 `vaccine-app/src/wasm3/`)。
4. 將 `vaccine-core/pkg/` 裡的所有檔案複製到剛剛建立的 `wasm3/` 中。
5. 打開 `vaccine-app/src/app.js`，修改載入路徑為 `import('./wasm3/vaccine_core.js')`。

### 步驟二：本機測試
在 `vaccine-app/src/` 目錄下開啟本機伺服器：
```bash
python -m http.server 8000
```
打開瀏覽器訪問 `http://localhost:8000` 進行測試，確認所有功能與變數映射正常。

### 步驟三：部署到 GitHub Pages
GitHub Pages 是由 repository 的 `gh-pages` 分支所驅動，必須將 `vaccine-app/src/` 的內容推送到該分支的根目錄。
你可以使用自訂的 Python 腳本 (搭配 `git worktree`) 來進行部署，確保 `master` 分支存放原始碼，而 `gh-pages` 只存放編譯後的靜態檔案。

---
*本系統目前採用深色沉浸護眼配色 (Dark Mocha)，以利診所人員長時間閱讀。若需修改配色，請直接調整 `styles_final.css` 中的 `:root` CSS 變數。*
