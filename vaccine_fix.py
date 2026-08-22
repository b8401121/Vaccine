import shutil
import subprocess
import os
from update_sri import update_html_sri

src_dir = r'f:\Vaccine\vaccine-app\src'
repo_dir = r'f:\Vaccine'
worktree_dir = r'f:\vaccine-ghp-deploy'

# 1. 自動更新 SRI (Subresource Integrity)
update_html_sri(os.path.join(src_dir, 'index.html'))
root_index = os.path.join(repo_dir, 'index.html')
if os.path.exists(root_index):
    update_html_sri(root_index, src_dir)

# 2. 提交並推送至 master
subprocess.run(['git', 'add', '-A'], cwd=repo_dir)
subprocess.run(['git', 'commit', '-m', 'feat(security): Add Subresource Integrity (SRI) to static assets'], cwd=repo_dir)
subprocess.run(['git', 'push', 'origin', 'master'], cwd=repo_dir)

# 3. 部署至 gh-pages 分支
if os.path.exists(worktree_dir):
    subprocess.run(['git', 'worktree', 'remove', worktree_dir, '--force'], cwd=repo_dir, capture_output=True)

subprocess.run(['git', 'worktree', 'add', worktree_dir, 'gh-pages'], cwd=repo_dir)

# 複製必要的前端檔案
files_to_copy = ['index.html', 'app.js', 'styles_final.css', 'app_styles.css', 'styles.css', 'qrcode.js', 'app-icon.png']
for f in files_to_copy:
    src_f = os.path.join(src_dir, f)
    if os.path.exists(src_f):
        shutil.copy2(src_f, os.path.join(worktree_dir, f))

# 複製 wasm3
wasm3_src = os.path.join(src_dir, 'wasm3')
wasm3_dst = os.path.join(worktree_dir, 'wasm3')
if os.path.exists(wasm3_dst):
    shutil.rmtree(wasm3_dst)
if os.path.exists(wasm3_src):
    shutil.copytree(wasm3_src, wasm3_dst)
    # 確保移除 .gitignore 避免被 git 忽略
    gi = os.path.join(wasm3_dst, '.gitignore')
    if os.path.exists(gi):
        os.remove(gi)

subprocess.run(['git', 'add', '-A'], cwd=worktree_dir)
subprocess.run(['git', 'commit', '-m', 'feat(security): Deploy Subresource Integrity (SRI) to gh-pages'], cwd=worktree_dir)
subprocess.run(['git', 'push', 'origin', 'gh-pages'], cwd=worktree_dir)
subprocess.run(['git', 'worktree', 'remove', worktree_dir, '--force'], cwd=repo_dir)
print('✅ Deployed with SRI verification successfully!')
