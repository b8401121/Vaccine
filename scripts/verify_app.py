#!/usr/bin/env python3
"""
台灣疫苗指南助手 - 自動化健康驗證防線 (Pre-Deploy Verification Gate)
在發布或部署前執行，確保核心 DOM 節點、JS 語法、QR Code 元件均 100% 健全，避免任何突發性網頁毀損。
"""
import os
import sys
import subprocess
import re

ROOT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def check_file_exists(filename):
    path = os.path.join(ROOT_DIR, filename)
    if not os.path.exists(path):
        print(f"❌ 錯誤: 必要檔案不存在 -> {filename}")
        return False
    return True

def check_syntax_node(filename):
    path = os.path.join(ROOT_DIR, filename)
    res = subprocess.run(["node", "-c", path], capture_output=True, text=True)
    if res.returncode != 0:
        print(f"❌ 語法錯誤 in {filename}:\n{res.stderr}")
        return False
    print(f"✅ {filename} 語法檢查通過 (Node.js syntax OK)")
    return True

def check_html_integrity(filename="index.html"):
    path = os.path.join(ROOT_DIR, filename)
    with open(path, "r", encoding="utf-8") as f:
        html = f.read()

    # 1. 核心 DOM ID 檢驗
    critical_ids = [
        "login-overlay",
        "login-form",
        "login-account",
        "results",
        "timeline-container",
        "catchup-result",
        "travel-result",
        "growth-results",
        "calendar-modal",
        "calendar-modal-close",
        "qrcode-container",
        "qr-tab-google",
        "qr-tab-apple",
        "qr-hint-text",
        "cal-modal-title",
        "cal-modal-current-vax",
        "cal-modal-next-box",
        "cal-modal-next-info",
        "cal-set-next-date-btn",
        "cal-date-input",
        "cal-reset-date-btn",
        "cal-direct-link",
        "cal-apple-btn",
        "cal-copy-link-btn"
    ]

    missing_ids = []
    for cid in critical_ids:
        if f'id="{cid}"' not in html and f"id='{cid}'" not in html:
            missing_ids.append(cid)

    if missing_ids:
        print(f"❌ index.html 缺少關鍵 DOM 元素 ID: {missing_ids}")
        return False

    print(f"✅ index.html 核心 DOM 結構完整 ({len(critical_ids)} 個必備關鍵節點均存在)")

    # 2. 檢查是否有脆弱的 SRI
    if "integrity=" in html:
        print("⚠️ 警告: index.html 包含 integrity 屬性，若有更新未重算可能導致 CDN 拒絕載入！")
    else:
        print("✅ index.html 採用 Cache-Buster 架構，無脆弱 SRI 拒載風險")

    # 3. 檢查版本號 (Cache-Buster)
    if not re.search(r'app\.js\?v=\d+', html):
        print("⚠️ 提醒: app.js 建議帶有版本號參數以防瀏覽器強快取")

    return True

def check_app_js_safety(filename="app.js"):
    path = os.path.join(ROOT_DIR, filename)
    with open(path, "r", encoding="utf-8") as f:
        js = f.read()

    # 檢查是否含有未受保護的 CorrectLevel 直接存取
    if "window.QRCode.CorrectLevel." in js:
        print("❌ 錯誤: app.js 含有未防護的 window.QRCode.CorrectLevel 直接存取，易引發 TypeError！")
        return False

    # 檢查必備核心函式
    required_fns = [
        "openCalendarModal",
        "generateGoogleCalendarUrl",
        "generateIcsFileContent",
        "generateIcsQrString",
        "displayVaccines"
    ]
    for fn in required_fns:
        if fn not in js:
            print(f"❌ 錯誤: app.js 遺失核心函式: {fn}")
            return False

    print("✅ app.js 核心邏輯安全防護與函式完整度檢驗通過")
    return True

def main():
    print("=" * 60)
    print("🛡️  正在執行 預防接種指南助手 健康檢查防線...")
    print("=" * 60)

    files_to_check = [
        "index.html",
        "app.js",
        "styles_final.css",
        "qrcode.js"
    ]

    all_ok = True
    for f in files_to_check:
        if not check_file_exists(f):
            all_ok = False

    if not all_ok:
        sys.exit(1)

    if not check_syntax_node("app.js") or not check_syntax_node("qrcode.js"):
        sys.exit(1)

    if not check_html_integrity("index.html"):
        sys.exit(1)

    if not check_app_js_safety("app.js"):
        sys.exit(1)

    print("=" * 60)
    print("🎉 恭喜！所有靜態語法與核心 DOM 檢驗全數通過，系統處於健康穩定狀態！")
    print("=" * 60)
    sys.exit(0)

if __name__ == "__main__":
    main()
