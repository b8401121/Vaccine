#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "============================================================"
echo "🚀 預防接種指南助手 - 安全自動發布程序 (Safe Deploy)"
echo "============================================================"

# 1. 執行健康防線
echo "🔍 [1/5] 執行上線前健康檢查防線..."
python3 "$ROOT_DIR/scripts/verify_app.py"

# 2. 自動更新版本號 (Cache-Buster)
NEW_VER=$(date +%Y%m%d%H%M)
echo "🏷️  [2/5] 更新快取破壞版本標籤 -> ?v=$NEW_VER..."
sed -i -E "s/app\.js\?v=[a-zA-Z0-9_]+/app.js?v=$NEW_VER/g" "$ROOT_DIR/index.html"
sed -i -E "s/qrcode\.js\?v=[a-zA-Z0-9_]+/qrcode.js?v=$NEW_VER/g" "$ROOT_DIR/index.html"
sed -i -E "s/styles_final\.css\?v=[a-zA-Z0-9_]+/styles_final.css?v=$NEW_VER/g" "$ROOT_DIR/index.html"

# 同步至 vaccine-app/src
if [ -d "$ROOT_DIR/vaccine-app/src" ]; then
  cp "$ROOT_DIR/index.html" "$ROOT_DIR/vaccine-app/src/index.html"
  cp "$ROOT_DIR/app.js" "$ROOT_DIR/vaccine-app/src/app.js"
  cp "$ROOT_DIR/qrcode.js" "$ROOT_DIR/vaccine-app/src/qrcode.js"
fi

# 3. 建立自動備份
BACKUP_DIR="$ROOT_DIR/releases/backup-$NEW_VER"
echo "📦 [3/5] 備份當前版本至 $BACKUP_DIR..."
mkdir -p "$BACKUP_DIR"
cp -r "$ROOT_DIR/index.html" "$ROOT_DIR/app.js" "$ROOT_DIR/styles_final.css" "$ROOT_DIR/qrcode.js" "$BACKUP_DIR/"

# 4. 再次驗證修改後的 index.html
python3 "$ROOT_DIR/scripts/verify_app.py"

# 5. 推送至 master 與 gh-pages
cd "$ROOT_DIR"
echo "💾 [4/5] 提交並推送到 master 分支..."
git add index.html app.js styles_final.css qrcode.js scripts/ releases/ vaccine-app/src/
git commit -m "deploy: Release version $NEW_VER with verified safety checks" || true
git push origin master

echo "🌐 [5/5] 同步推送到 GitHub Pages (gh-pages)..."
git checkout gh-pages
git checkout master -- index.html app.js styles_final.css qrcode.js
git commit -m "deploy(gh-pages): Live deploy release $NEW_VER" index.html app.js styles_final.css qrcode.js || true
git push origin gh-pages
git checkout master

echo "============================================================"
echo "🎉 發布成功！新版本 $NEW_VER 已安全部署上線！"
echo "👉 網址: https://b8401121.github.io/Vaccine/"
echo "============================================================"
