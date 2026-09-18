// Exercises the real release EXE's signed updater and Inno handoff in isolated
// directories, with a separate installer AppId and harmless fixture executable.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawn,spawnSync} from 'node:child_process';
import {createHash,createPrivateKey,sign} from 'node:crypto';
const root=path.resolve(import.meta.dirname,'..');
const artifact=path.join(root,'.artifacts',`updater-${Date.now()}`);
await fs.mkdir(artifact,{recursive:true});
const report={passed:false,checks:[],artifact};
const record=s=>{report.checks.push(s);console.log('PASS',s);};
const hash=b=>createHash('sha256').update(b).digest('hex');
const wait=ms=>new Promise(r=>setTimeout(r,ms));
async function until(fn,timeout=20000){const end=Date.now()+timeout;while(Date.now()<end){if(await fn())return;await wait(100);}throw new Error('Timed out');}
async function exists(p){return fs.stat(p).then(()=>true,()=>false);}
function ps(code,input){const p=spawnSync('powershell',['-NoProfile','-NonInteractive','-Command',code],{input,encoding:'utf8',windowsHide:true});assert.equal(p.status,0,p.stderr);return p.stdout;}
const quote=s=>`'${s.replaceAll("'","''")}'`;
async function processDone(child,timeout=120000){return new Promise((resolve,reject)=>{const timer=setTimeout(()=>{child.kill();reject(new Error('Child timeout'));},timeout);child.once('error',e=>{clearTimeout(timer);reject(e);});child.once('exit',code=>{clearTimeout(timer);resolve(code);});});}
let uninstall;
try{
 for(const name of ['old','new']){
  const code=`public static class Fixture${name} { public static void Main() { System.IO.File.WriteAllText(System.IO.Path.Combine(System.AppDomain.CurrentDomain.BaseDirectory, "${name}-restarted.txt"), "local test"); } }`;
  ps(`Add-Type -TypeDefinition ${quote(code)} -OutputAssembly ${quote(path.join(artifact,`${name}.exe`))} -OutputType WindowsApplication`);
 }
 const ident=`updater-${Date.now()}`;
 const compiler=spawn(path.join(root,'.tools/inno-setup/ISCC.exe'),['--quiet','--define=AppVersion=99.0.0',`--define=ProjectRoot=${root}`,`--define=TestBuild=${ident}`,`--define=TestBinary=${path.join(artifact,'new.exe')}`,path.join(root,'installer/local-studio.iss')],{windowsHide:true,stdio:'inherit'});
 assert.equal(await processDone(compiler),0);
 const installer=await fs.readFile(path.join(root,`src-tauri/target/release/bundle/inno/installer-test-${ident}.exe`));
 const protectedKey=await fs.readFile(path.join(root,'.tools/update-signing/private.dpapi'));
 const raw=Buffer.from(ps('Add-Type -AssemblyName System.Security; $b=[Convert]::FromBase64String([Console]::In.ReadToEnd()); [Console]::Write([Convert]::ToBase64String([Security.Cryptography.ProtectedData]::Unprotect($b,$null,[Security.Cryptography.DataProtectionScope]::CurrentUser)))',protectedKey.toString('base64')),'base64');
 const key=createPrivateKey({key:raw,type:'pkcs8',format:'der'});raw.fill(0);
 for(const mode of ['valid','tampered-package','tampered-signature']){
  const stage=path.join(artifact,mode,'stage'),install=path.join(artifact,mode,'installed');
  await fs.mkdir(stage,{recursive:true});await fs.mkdir(install,{recursive:true});
  await fs.copyFile(path.join(root,'src-tauri/target/release/local-studio.exe'),path.join(stage,'helper.exe'));
  const old=await fs.readFile(path.join(artifact,'old.exe'));await fs.writeFile(path.join(install,'local-studio.exe'),old);
  await fs.writeFile(path.join(install,'installed.marker'),'test');await fs.mkdir(path.join(install,'Local-Studio-Data'));await fs.writeFile(path.join(install,'Local-Studio-Data/keep.txt'),'user data');
  const asset=kind=>({url:`https://github.com/ScopeBotlul/Local-Studio/releases/download/v99.0.0/Local-Studio-99.0.0-hub-${kind}`,size:installer.length,sha256:hash(installer)});
  const bytes=Buffer.from(JSON.stringify({schema:1,version:'99.0.0',publishedAt:new Date().toISOString(),notes:'LOCAL ISOLATED TEST ONLY',installer:asset('setup.exe'),portable:asset('portable.zip')}));
  const signature=sign(null,bytes,key);if(mode==='tampered-signature')signature[0]^=1;
  const packageBytes=Buffer.from(installer);if(mode==='tampered-package')packageBytes[packageBytes.length-1]^=1;
  await fs.writeFile(path.join(stage,'setup.exe'),packageBytes);await fs.writeFile(path.join(stage,'update.json'),bytes);await fs.writeFile(path.join(stage,'update.sig'),signature.toString('base64'));
  const parent=spawn('powershell',['-NoProfile','-NonInteractive','-Command','Start-Sleep -Seconds 180'],{windowsHide:true,stdio:'ignore'});
  const helper=spawn(path.join(stage,'helper.exe'),['--update-helper',String(parent.pid),stage,install,hash(old)],{windowsHide:true,stdio:'ignore'});const done=processDone(helper);
  try{await until(()=>exists(path.join(stage,'helper.ready')));assert.equal(hash(await fs.readFile(path.join(install,'local-studio.exe'))),hash(old));assert(!(await exists(path.join(stage,'install.log'))));parent.kill();assert.equal(await done,mode==='valid'?0:1);}finally{parent.kill();helper.kill();}
  assert.equal(await fs.readFile(path.join(install,'Local-Studio-Data/keep.txt'),'utf8'),'user data');
  const result=await fs.readFile(path.join(stage,'result.txt'),'utf8');
  if(mode==='valid'){
   assert.equal(result,'Update installed');assert.equal(hash(await fs.readFile(path.join(install,'local-studio.exe'))),hash(await fs.readFile(path.join(artifact,'new.exe'))));await until(()=>exists(path.join(install,'new-restarted.txt')));uninstall=path.join(install,'unins000.exe');
   record('Real helper waits for parent exit, verifies signed installer, installs to exact original directory, preserves user data and restarts the new executable');
  }else{
   assert.equal(result,mode==='tampered-package'?'update_integrity':'update_signature');assert.equal(hash(await fs.readFile(path.join(install,'local-studio.exe'))),hash(old));await until(()=>exists(path.join(install,'old-restarted.txt')));assert(!(await exists(path.join(stage,'install.log'))));
   record(`${mode}: no installer execution or replacement; original executable restarted`);
  }
 }
 report.passed=true;
}catch(e){report.error=String(e.stack||e);console.error(report.error);process.exitCode=1;}
finally{
 if(uninstall){const child=spawn(uninstall,['/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART'],{windowsHide:true,stdio:'ignore'});assert.equal(await processDone(child),0);}
 await fs.writeFile(path.join(artifact,'report.json'),JSON.stringify(report,null,2));console.log(`Report: ${path.join(artifact,'report.json')}`);
}
