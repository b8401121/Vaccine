import urllib.request
req = urllib.request.Request('https://b8401121.github.io/Vaccine/app.js')
req.add_header('User-Agent', 'Mozilla/5.0')
with urllib.request.urlopen(req) as f:
    content = f.read().decode('utf-8')

# find wasm import
for i, line in enumerate(content.split('\n')):
    if 'wasm' in line.lower() and 'import' in line.lower():
        print(f'{i}: {line[:200]}')
