import os
import re
import hashlib
import base64

def calculate_sri(file_path, algorithm="sha384"):
    if not os.path.exists(file_path):
        return None
    with open(file_path, "rb") as f:
        data = f.read()
    digest = getattr(hashlib, algorithm)(data).digest()
    b64 = base64.b64encode(digest).decode("utf-8")
    return f"{algorithm}-{b64}"

def update_html_sri(html_path, base_dir=None):
    if base_dir is None:
        base_dir = os.path.dirname(html_path)
    
    with open(html_path, "r", encoding="utf-8") as f:
        content = f.read()

    # 1. Update <link rel="stylesheet" href="...">
    def replace_link(match):
        full_tag = match.group(0)
        href = match.group(1)
        if href.startswith("http://") or href.startswith("https://") or href.startswith("//"):
            return full_tag
        # Clean query string if present
        clean_file = href.split("?")[0]
        target_path = os.path.join(base_dir, clean_file)
        sri = calculate_sri(target_path)
        if not sri:
            return full_tag
        
        # Remove existing integrity and crossorigin if any
        tag_no_sri = re.sub(r'\s+integrity="[^"]*"', '', full_tag)
        tag_no_sri = re.sub(r'\s+crossorigin="[^"]*"', '', tag_no_sri)
        
        # Insert integrity and crossorigin before closing >
        if tag_no_sri.endswith("/>"):
            return tag_no_sri[:-2] + f' integrity="{sri}" crossorigin="anonymous" />'
        elif tag_no_sri.endswith(">"):
            return tag_no_sri[:-1] + f' integrity="{sri}" crossorigin="anonymous">'
        return full_tag

    content = re.sub(r'<link\s+[^>]*rel=["\']stylesheet["\'][^>]*href=["\']([^"\']+)["\'][^>]*>', replace_link, content, flags=re.IGNORECASE)
    content = re.sub(r'<link\s+[^>]*href=["\']([^"\']+)["\'][^>]*rel=["\']stylesheet["\'][^>]*>', replace_link, content, flags=re.IGNORECASE)

    # 2. Update <script src="...">
    def replace_script(match):
        full_tag = match.group(0)
        src = match.group(1)
        if src.startswith("http://") or src.startswith("https://") or src.startswith("//"):
            return full_tag
        clean_file = src.split("?")[0]
        target_path = os.path.join(base_dir, clean_file)
        sri = calculate_sri(target_path)
        if not sri:
            return full_tag
        
        tag_no_sri = re.sub(r'\s+integrity="[^"]*"', '', full_tag)
        tag_no_sri = re.sub(r'\s+crossorigin="[^"]*"', '', tag_no_sri)
        
        if tag_no_sri.endswith(">"):
            return tag_no_sri[:-1] + f' integrity="{sri}" crossorigin="anonymous">'
        return full_tag

    content = re.sub(r'<script\s+[^>]*src=["\']([^"\']+)["\'][^>]*>', replace_script, content, flags=re.IGNORECASE)

    with open(html_path, "w", encoding="utf-8") as f:
        f.write(content)
    print(f"Updated SRI for {html_path}")

if __name__ == "__main__":
    src_html = r"f:\Vaccine\vaccine-app\src\index.html"
    update_html_sri(src_html)
    root_html = r"f:\Vaccine\index.html"
    if os.path.exists(root_html):
        update_html_sri(root_html, r"f:\Vaccine\vaccine-app\src")
