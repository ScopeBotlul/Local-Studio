"""Check distribution bytes only; this does not grant native release acceptance.

The stricter verify-package.py still requires successful generation/privacy,
creative and AI reports for the exact executable.
"""
from pathlib import Path
import hashlib,json,zipfile
root=Path(__file__).resolve().parent.parent
version=json.loads((root/'package.json').read_text())['version']
def sha(file):
 with file.open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest()
exe=root/'src-tauri/target/release/local-studio.exe'
portable=root/f'releases/Local Studio Hub {version}'
archive=root/f'releases/Local-Studio-{version}-hub-portable.zip'
installer=root/f'releases/Local-Studio-{version}-hub-setup.exe'
assert sha(exe)==sha(portable/'Local Studio.exe')
assert sha(installer)==sha(root/f'src-tauri/target/release/bundle/inno/Local-Studio-{version}-hub-setup.exe')
manifests={kind:json.loads((root/f'src-tauri/{kind}-runtime.json').read_text())['files'] for kind in ['image','video','assistant','speech']}
expected={'Local Studio.exe','portable.marker','LIESMICH.txt'}|{f'{kind}-runtime/{name}' for kind,files in manifests.items() for name in files}
with zipfile.ZipFile(archive) as z:
 assert z.testzip() is None
 assert {n.replace('\\','/') for n in z.namelist() if not n.endswith('/')}==expected
 assert hashlib.sha256(z.read('Local Studio.exe')).hexdigest()==sha(exe)
 assert f'Neu in {version}' in z.read('LIESMICH.txt').decode('utf-8')
 for kind,files in manifests.items():
  for name,digest in files.items():
   assert sha(portable/f'{kind}-runtime'/name)==digest
   assert hashlib.sha256(z.read(f'{kind}-runtime/{name}')).hexdigest()==digest
for name in ['package.json','package-lock.json','src-tauri/tauri.conf.json']:assert json.loads((root/name).read_text())['version']==version
assert sha(root/'SPEC.md')==sha(root/'prompt')=='2e3232ef5a607123326d3732679e28a5758106a5d8db61a7420b2e47b71d4bdd'
sums=(root/f'releases/SHA256SUMS-{version}.txt').read_text();assert sha(archive) in sums and sha(installer) in sums
result=dict(passed=True,scope='Distribution bytes only; native release acceptance is separate',version=version,executableSha256=sha(exe),zipSha256=sha(archive),installerSha256=sha(installer),zipFiles=len(expected),runtimeFiles={k:len(f) for k,f in manifests.items()})
(root/f'.artifacts/package-bytes-verification-{version}.json').write_text(json.dumps(result,indent=2),encoding='utf-8')
print(json.dumps(result,indent=2))
