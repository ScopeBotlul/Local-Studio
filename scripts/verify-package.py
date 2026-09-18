"""Verify distributable bytes against successful native creative and AI tests."""
from pathlib import Path
import hashlib,json,zipfile
r=Path(__file__).resolve().parent.parent
v=json.loads((r/'package.json').read_text())['version']
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
exe=sha(r/'src-tauri/target/release/local-studio.exe')
candidates=[]
ai_candidates=[]
privacy_candidates=[]
for f in (r/'.artifacts').glob('native-*/report.json'):
 q=json.loads(f.read_text(encoding='utf-8'))
 if q.get('passed') and q.get('version')==v and any('Native layered canvas' in c.get('name','') for c in q.get('checks',[])) and sha(f.parent/'Local Studio.exe')==exe:candidates.append(f)
 if q.get('passed') and q.get('version')==v and any('Actual CPU llama.cpp inference' in c.get('name','') for c in q.get('checks',[])) and sha(f.parent/'Local Studio.exe')==exe:ai_candidates.append(f)
 if q.get('passed') and q.get('version')==v and any('Normal Windows close works with a locked Studio' in c.get('name','') for c in q.get('checks',[])) and sha(f.parent/'Local Studio.exe')==exe:privacy_candidates.append(f)
assert candidates,'No successful creative report for the exact release EXE'
native=sorted(candidates)[-1]
assert ai_candidates,'No successful AI report for the exact release EXE'
ai_native=sorted(ai_candidates)[-1]
assert privacy_candidates,'No successful privacy and locked-close report for the exact release EXE'
privacy_native=sorted(privacy_candidates)[-1]
portable=r/f'releases/Local Studio Hub {v}'
archive=r/f'releases/Local-Studio-{v}-hub-portable.zip'
installer=r/f'releases/Local-Studio-{v}-hub-setup.exe'
assert sha(portable/'Local Studio.exe')==exe
assert sha(installer)==sha(r/f'src-tauri/target/release/bundle/inno/Local-Studio-{v}-hub-setup.exe')
manifests={kind:json.loads((r/f'src-tauri/{kind}-runtime.json').read_text())['files'] for kind in ['image','video','assistant','speech']}
expected={'Local Studio.exe','portable.marker','LIESMICH.txt'}|{f'{kind}-runtime/{name}' for kind,files in manifests.items() for name in files}
with zipfile.ZipFile(archive) as z:
 assert z.testzip() is None
 assert {n.replace('\\','/') for n in z.namelist() if not n.endswith('/')}==expected
 assert hashlib.sha256(z.read('Local Studio.exe')).hexdigest()==exe
 assert f'Neu in {v}' in z.read('LIESMICH.txt').decode('utf-8')
 for kind,files in manifests.items():
  for name,digest in files.items():
   assert sha(portable/f'{kind}-runtime'/name)==digest
   assert hashlib.sha256(z.read(f'{kind}-runtime/{name}')).hexdigest()==digest
spec='2e3232ef5a607123326d3732679e28a5758106a5d8db61a7420b2e47b71d4bdd'
assert sha(r/'SPEC.md')==sha(r/'prompt')==spec
for name in ['package.json','package-lock.json','src-tauri/tauri.conf.json']:assert json.loads((r/name).read_text())['version']==v
sums=(r/f'releases/SHA256SUMS-{v}.txt').read_text();assert sha(archive) in sums and sha(installer) in sums
result=dict(passed=True,version=v,nativeReport=str(native.relative_to(r)),aiReport=str(ai_native.relative_to(r)),privacyReport=str(privacy_native.relative_to(r)),executableSha256=exe,zipSha256=sha(archive),installerSha256=sha(installer),zipFiles=len(expected),runtimeFiles={k:len(f) for k,f in manifests.items()},specAndPromptSha256=spec)
(r/f'.artifacts/package-verification-{v}.json').write_text(json.dumps(result,indent=2),encoding='utf-8');print(json.dumps(result,indent=2))
