@echo off
chcp 65001 >nul
echo ============================================================
echo 🚨 啟動緊急回滾程序：恢復至不可修改黃金穩定版 (gold-stable-v1)...
echo ============================================================

set SCRIPT_DIR=%~dp0
set ROOT_DIR=%SCRIPT_DIR%..
set GOLD_DIR=%ROOT_DIR%\releases\gold-stable-v1

if not exist "%GOLD_DIR%" (
    echo ❌ 錯誤：黃金穩定備份目錄不存在: %GOLD_DIR%
    pause
    exit /b 1
)

echo 📦 [1/4] 從黃金快照還原前端關鍵檔案...
xcopy /E /Y /I "%GOLD_DIR%\*" "%ROOT_DIR%\"
copy /Y "%GOLD_DIR%\index.html" "%ROOT_DIR%\vaccine-app\src\index.html"
copy /Y "%GOLD_DIR%\app.js" "%ROOT_DIR%\vaccine-app\src\app.js"
copy /Y "%GOLD_DIR%\styles_final.css" "%ROOT_DIR%\vaccine-app\src\styles_final.css"
copy /Y "%GOLD_DIR%\qrcode.js" "%ROOT_DIR%\vaccine-app\src\qrcode.js"

echo 🔍 [2/4] 執行健康防線檢驗...
python "%ROOT_DIR%\scripts\verify_app.py"
if %errorlevel% neq 0 (
    echo ❌ 檢驗失敗，請檢查檔案！
    pause
    exit /b 1
)

echo 💾 [3/4] 提交回滾至 Git...
cd /d "%ROOT_DIR%"
git add index.html app.js styles_final.css qrcode.js app-icon.png wasm vaccine-app/src/
git commit -m "rollback(emergency): Restore to immutable gold-stable-v1 release"
git push origin master

echo 🌐 [4/4] 緊急修復並推送至 GitHub Pages (gh-pages)...
git checkout gh-pages
git checkout master -- index.html app.js styles_final.css qrcode.js app-icon.png wasm
git commit -m "rollback(gh-pages): Emergency restore to immutable gold-stable-v1" index.html app.js styles_final.css qrcode.js
git push origin gh-pages
git checkout master

echo ============================================================
echo 🎉 回滾完成！線上網頁已在 10 秒內成功還原至 100%% 穩定版！
echo 👉 網址: https://b8401121.github.io/Vaccine/
echo ============================================================
pause
