import shutil, subprocess, os

src_dir = r'f:\Vaccine\vaccine-app\src'
repo_dir = r'f:\Vaccine'
worktree_dir = r'f:\vaccine-ghp21'

# update index.html to point to styles_final.css
with open(src_dir + r'\index.html', 'r', encoding='utf-8') as f:
    html = f.read()

html = html.replace('href="app_styles.css"', 'href="styles_final.css"')

with open(src_dir + r'\index.html', 'w', encoding='utf-8') as f:
    f.write(html)

subprocess.run(['git', 'add', '-A'], cwd=repo_dir)
subprocess.run(['git', 'commit', '-m', 'style: Revert to original color scheme and bust cache with styles_final.css'], cwd=repo_dir)
subprocess.run(['git', 'push', 'origin', 'master'], cwd=repo_dir)

if os.path.exists(worktree_dir):
    subprocess.run(['git', 'worktree', 'remove', worktree_dir, '--force'], cwd=repo_dir, capture_output=True)

subprocess.run(['git', 'worktree', 'add', worktree_dir, 'gh-pages'], cwd=repo_dir)
shutil.copy2(src_dir + r'\styles_final.css', worktree_dir + r'\styles_final.css')
shutil.copy2(src_dir + r'\index.html', worktree_dir + r'\index.html')

subprocess.run(['git', 'add', '-A'], cwd=worktree_dir)
subprocess.run(['git', 'commit', '-m', 'style: Revert to original color scheme and bust cache'], cwd=worktree_dir)
subprocess.run(['git', 'push', 'origin', 'gh-pages'], cwd=worktree_dir)
subprocess.run(['git', 'worktree', 'remove', worktree_dir, '--force'], cwd=repo_dir)
print('Deployed revert successfully!')
