"""Explicit build setup: download the pinned, hash-verified official image runtime.

No model weights are downloaded and no system packages are installed.
"""
from pathlib import Path
import hashlib
import json
import urllib.request
import zipfile

root = Path(__file__).resolve().parent.parent
manifest = json.loads((root / 'src-tauri/image-runtime.json').read_text(encoding='utf-8'))
destination = root / '.tools/image-runtime'
destination.mkdir(parents=True, exist_ok=True)
archive = root / '.tools/image-runtime-vulkan.zip'
url = 'https://github.com/leejet/stable-diffusion.cpp/releases/download/master-872-cc515a0/sd-master-cc515a0-bin-win-vulkan-x64.zip'
if not archive.exists():
    urllib.request.urlretrieve(url, archive)
if hashlib.sha256(archive.read_bytes()).hexdigest() != manifest['archiveSha256']:
    raise RuntimeError('Runtime archive hash mismatch; no files extracted.')
with zipfile.ZipFile(archive) as z:
    for name, digest in manifest['files'].items():
        if Path(name).name != name or any(c in name for c in '/\\:'):
            raise RuntimeError('Unsafe runtime filename')
        vendored = root / 'third-party/image-runtime' / name
        data = vendored.read_bytes() if vendored.exists() else z.read(name)
        if hashlib.sha256(data).hexdigest() != digest:
            raise RuntimeError(f'Runtime file hash mismatch: {name}')
        (destination / name).write_bytes(data)
unexpected = set(p.name for p in destination.iterdir()) - set(manifest['files'])
if unexpected:
    raise RuntimeError(f'Unexpected runtime files: {sorted(unexpected)}')
print('Pinned image runtime ready in .tools/image-runtime; no models downloaded.')
