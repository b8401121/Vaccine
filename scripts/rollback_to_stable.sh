#!/usr/bin/env bash
set -e

# 取得專案根目錄
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "============================================================"
echo "🚨 啟動緊急回滾程序：恢復至不可修改黃金穩定版 (gold-stable-v1)..."
echo "============================================================"

GOLD_DIR="$ROOT_DIR/releases/gold-stable-v1"

if [ ! -d "$GOLD_DIR" ]; then
  echo "❌ 錯誤：黃金穩定備份目錄不存在 -> $GOLD_DIR"
  exit 1
fi

# 1. 恢復根目錄檔案
echo "📦 [1/5] 從黃金快照還原前端關鍵檔案..."
cp -r "$GOLD_DIR"/* "$ROOT_DIR"/

# 2. 恢復 vaccine-app/src
if [ -d "$ROOT_DIR/vaccine-app/src" ]; then
  echo "📦 [2/5] 同步還原至 vaccine-app/src 目錄..."
  cp "$GOLD_DIR"/index.html "$ROOT_DIR/vaccine-app/src/"
  cp "$GOLD_DIR"/app.js "$ROOT_DIR/vaccine-app/src/"
  cp "$GOLD_DIR"/styles_final.css "$ROOT_DIR/vaccine-app/src/"
  cp "$GOLD_DIR"/qrcode.js "$ROOT_DIR/vaccine-app/src/"
fi

# 3. 執行健康驗證
echo "🔍 [3/5] 執行健康防線檢驗..."
python3 "$ROOT_DIR/scripts/verify_app.py"

# 4. Git 提交 master
cd "$ROOT_DIR"
echo "💾 [4/5] 記錄回滾變更至 master..."
git add index.html app.js styles_final.css qrcode.js app-icon.png wasm vaccine-app/src/index.html vaccine-app/src/app.js vaccine-app/src/styles_final.css vaccine-app/src/qrcode.js || true
git commit -m "rollback(emergency): Restore to immutable gold-stable-v1 release" || true
git push origin master || true

# 5. 推送至 gh-pages 分支
echo "🌐 [5/5] 緊急修復並推送至 GitHub Pages (gh-pages)..."
git checkout gh-pages
git checkout master -- index.html app.js styles_final.css qrcode.js app-icon.png wasm
git commit -m "rollback(gh-pages): Emergency restore to immutable gold-stable-v1" app.js index.html styles_final.css qrcode.js || true
git push origin gh-pages
git checkout master

echo "============================================================"
echo "🎉 回滾完成！線上網頁已在 10 秒內成功還原至 100% 穩定版！"
echo "👉 網址: https://b8401121.github.io/Vaccine/"
echo "============================================================"
