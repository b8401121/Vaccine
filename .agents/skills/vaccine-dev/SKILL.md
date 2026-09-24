---
name: vaccine-dev
description: 專案開發指南：預防接種指南助手 (Rust + Wasm + JS) 的架構、編譯與部署規則
---

# 預防接種指南助手 (Vaccine App) 開發指南

本專案是一套結合 **Rust 核心演算法**、**Vanilla JavaScript 前端**、**WebAssembly (Wasm 網頁版)** 與 **Tauri v2 跨平台原生封裝 (Android APK / Windows Portable EXE)** 的專業醫療輔助系統，專門用於診所與家長的疫苗時程推算、補打間隔規範、出國留學旅遊疫苗速查、0~18 歲兒童生長曲線與 BMI 試算，以及跨平台 Google / iOS 原生行事曆提醒。

---

## 1. 專案整體架構 (Multi-Platform Architecture)

- **`vaccine-core/`**: 核心醫療與演算法層 (Rust)。
  - 實作衛福部疾管署 (Taiwan CDC) 常規與自費時程、ACIP 補打規則、旅遊醫學清單、國健署 0~18 歲生長曲線與 BMI 標準。
  - 編譯成 `rlib`（供 Tauri 桌面/手機端使用）或經 `wasm-pack` 編譯為 `WebAssembly` 靜態模組（供網頁端使用）。
- **`vaccine-app/`**: 跨平台應用主體。
  - `src/`: 前端展示層 (`index.html`, `main.js` / `app.js`, `styles.css` / `styles_final.css`, `qrcode.js`)。
  - `src-tauri/`: Tauri v2 原生後端。封裝系統視窗、`tauri-plugin-opener` 外部 Intent/URL 喚醒，以及 Android / Windows 呼叫命令。
  - `src-tauri/gen/android/`: Android 原生 Gradle 專案目錄（輸出 `arm64-v8a` 簽署 APK）。
- **`portable-launcher/`**: Windows 單一獨立免安裝可攜版 (`.exe`) 啟動器源碼。
- **`gh-pages` 分支**: GitHub Pages 靜態網站部署分支（採用 WebAssembly 離線計算架構）。

---

## 2. 核心功能與醫療法規標準 (Medical Guidelines & Standards)

1. **🗓️ 年齡接種時間軸 (Timeline)**：
   - 支援西元 / 民國雙曆法，依出生年月日推算滿月/滿歲時程，自動標註公費常規、地方縣市補助、自費建議。
   - **🍂 2026 台灣流感疫苗 8 大廠牌整合**：涵蓋傳統雞胚 (GSK/賽諾菲/國光/高端)、細胞培養 (東洋 輔流威護)、長者加強型 (賽諾菲高劑量/東洋佐劑) 與鼻噴型 (AZ 能伏鼻)。依月齡/年齡精準推薦，並直接注入「當次建議接種 (Current Visit)」與時間軸置頂卡片。
   - **💉 同次同時接種部位指南 (Co-administration Guide)**：針劑分左右上臂三角肌或左右大腿前外側；口服減毒 (輪狀病毒) 不互斥；活性減毒疫苗 (MMR/水痘/活性日腦/鼻噴流感) 若未同日施打嚴格間隔至少 28 天。
   - **📱 📅 手機相機掃碼行事曆提醒 (Google / iOS VEVENT QR Code)**：
     - **當次施打項目記錄**：完整記錄當次接種的所有疫苗品名與劑次。
     - **下次預約提醒自動試算**：自動推算下次建議接種日期與預計施打疫苗名稱。
     - **Google / iOS 雙格式即時切換**：iPhone 原生相機直掃直入行事曆；亦支援 `.ics` 檔案下載與連結複製。
     - **自訂預約日期與醫療安全警示**：支援修改日期並即時評估「提早抗體效價不足警告」與「延後順延提醒」，支援一鍵切換為下次日期。
2. **⏱️ 遲打/補打間隔試算 (Catch-Up)**：
   - 遵循衛福部疾管署與 ACIP 最小月齡與最短劑次安全間隔 (Minimum Intervals)。
3. **✈️ 出國留學旅遊疫苗速查 (Travel)**：
   - 區分 Mandatory (強制/入學證明)、Recommended (常規建議)、Booster (追加/特種) 與旅遊醫學門診衛教。
4. **📈 0~18 歲生長曲線與 BMI 試算 (Growth Curve & BMI)**：
   - **0 ~ 5 歲**：國民健康署現行最新《兒童健康手冊》(2024最新版) 採用之 **WHO 2006 國際生長標準**。
   - **5 ~ 7 歲**：國健署 2009 年公布之國人兒童生長銜接標準 (Chen & Chang)。
   - **7 ~ 18 歲**：國健署最新公告《兒童及青少年身體質量指數(BMI)與身高百分位建議值》(衛署授升字第0990700680號公告，2010 年版)。

---

## 3. 關鍵開發守則與避坑指南 (CRITICAL RULES)

### A. WebAssembly / Native 資料傳輸合約 (JSON 序列化)
- **Rust 端**：對外接口回傳值嚴格透過 `serde_json::to_string` 或結構體序列化輸出。
- **JS 端**：在 `fallbackInvoke` 接收 Wasm 回傳後，第一時間執行 `JSON.parse(res)` 還原為物件。

### B. 變數命名映射 (CamelCase vs Snake_case)
- JS 前端慣用 `camelCase`（如 `ageMonths`, `vaccineId`），Rust 嚴格要求 `snake_case`（如 `age_months`, `vaccine_id`）。
- 在 `main.js` / `app.js` 的 `fallbackInvoke` 橋樑函數中必須手動對齊變數名稱，避免傳入 `undefined` 引發記憶體錯誤。

### C. Android APK 編譯與簽署管線 (Android Build Pipeline)
在 Windows 環境下編譯 Android arm64-v8a 原生 APK 時，需設置專屬 NDK 與 Clang 環境變數：
```powershell
$env:JAVA_HOME = "E:\android-env\jdk-17"
$env:ANDROID_HOME = "E:\android-env\sdk"
$env:NDK_HOME = "E:\android-env\sdk\ndk\26.1.10909125"
$env:PATH = "E:\mingw64\mingw64\bin;E:\android-env\sdk\ndk\26.1.10909125\toolchains\llvm\prebuilt\windows-x86_64\bin;" + $env:PATH
$env:CC_aarch64_linux_android = "E:\android-env\sdk\ndk\26.1.10909125\toolchains\llvm\prebuilt\windows-x86_64\bin\aarch64-linux-android34-clang.cmd"
$env:AR_aarch64_linux_android = "E:\android-env\sdk\ndk\26.1.10909125\toolchains\llvm\prebuilt\windows-x86_64\bin\llvm-ar.exe"
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = "E:\android-env\sdk\ndk\26.1.10909125\toolchains\llvm\prebuilt\windows-x86_64\bin\aarch64-linux-android34-clang.cmd"

# 1. 編譯 Rust 為 Android arm64 .so
cargo build --target aarch64-linux-android --release --lib --features "tauri/custom-protocol"

# 2. 複製 .so 至 Gradle jniLibs 並產出 APK
Copy-Item "E:\Vaccine\vaccine-app\src-tauri\target\aarch64-linux-android\release\libvaccine_app_lib.so" "E:\Vaccine\vaccine-app\src-tauri\gen\android\app\src\main\jniLibs\arm64-v8a\libvaccine_app_lib.so" -Force
cd "E:\Vaccine\vaccine-app\src-tauri\gen\android"
.\gradlew.bat assembleArm64Release

# 3. 簽署 APK
Copy-Item ".\app\build\outputs\apk\arm64\release\app-arm64-release-unsigned.apk" "E:\台灣疫苗指南助手.apk" -Force
& "E:\android-env\sdk\build-tools\35.0.0\apksigner.bat" sign --ks "E:\android-env\debug.keystore" --ks-pass pass:android --key-pass pass:android "E:\台灣疫苗指南助手.apk"
```

### D. 靜態資源防護：Cache-Buster 取代脆弱 SRI (避坑核心)
- **避坑警示**：`integrity="sha384-..."` (SRI) 極易因 Git 換行符號（CRLF vs LF）或 CDN 壓縮導致雜湊不符，促使瀏覽器**直接阻斷整份 JS 與 CSS 載入（全頁變白、登入畫面消失、排版全毀）**。
- **現行架構**：已全面改用動態時間戳 **Cache-Buster（如 `?v=20260924j`）**，兼顧強制破快取與極致穩定性。

### E. QR Code 庫相容性規範
- 專案使用輕量原生 `qrcode.js`，構造函數僅接收 `{ text, width, height }`。
- **嚴禁直接存取 `window.QRCode.CorrectLevel.M`**（易造成 `TypeError: Cannot read properties of undefined` 中斷全域執行序）。

---

## 4. 系統高可用、不可修改穩定版本與秒級回滾體系 (High Availability & Rollback)

為防止代碼修改導致網頁突發毀損，專案建置了四大防護防線：

1. **🥇 不可修改黃金穩定版快照**：
   - 實體快照目錄：`releases/gold-stable-v1/`（封存 100% 驗證通過之靜態檔）。
   - Git 不可變標籤：`v1.0.0-gold-stable`。
   - 保護分支：`stable-release`。
2. **🔍 上線前自動健康檢查防線**：
   - 指令：`python3 scripts/verify_app.py`
   - 自動檢驗 Node.js 語法、24 個關鍵核心 DOM 節點、日曆 URL/QR 生成防護。發布前必須 100% 綠燈通過。
3. **🚨 10 秒一鍵緊急回滾 (One-Click Emergency Rollback)**：
   - **Linux / macOS**：`bash scripts/rollback_to_stable.sh`
   - **Windows 桌面**：滑鼠雙點 `scripts/rollback_to_stable.bat`
   - 自動從黃金快照還原，並於 10 秒內同步推送至 `gh-pages` 修復線上服務。
4. **🚀 安全自動發布程序 (Safe Deploy Pipeline)**：
   - 指令：`bash scripts/deploy_safely.sh`
   - 自動執行健康防線 ➔ 更新時間戳版本號 ➔ 備份舊版至 `releases/backup-<時間戳>/` ➔ 同步推送至 `master` 與 `gh-pages`。

---

## 5. 常用維護與編譯指令速查

- **線上緊急回滾 (10秒恢復穩定版)**：
  ```bash
  bash scripts/rollback_to_stable.sh
  ```
- **健康檢查驗證**：
  ```bash
  python3 scripts/verify_app.py
  ```
- **安全發布上線**：
  ```bash
  bash scripts/deploy_safely.sh
  ```
- **Windows 桌面版啟動測試**：
  ```powershell
  cd vaccine-app/src-tauri
  cargo tauri dev
  ```
- **Windows 單一免安裝獨立版編譯**：
  ```powershell
  cd portable-launcher
  cargo build --release
  Copy-Item "target/release/portable-launcher.exe" "台灣疫苗指南助手_單一獨立版.exe" -Force
  ```
- **GitHub 手動同步推送**：
  ```powershell
  git add .
  git commit -m "feat/fix: <說明>"
  git push origin master
  ```
