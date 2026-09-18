// Local release signing. The private key is DPAPI-protected for this Windows user,
// stays outside source control, and is never printed or included in a release.
import {generateKeyPairSync,createPrivateKey,createHash,sign,verify} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {readFileSync,writeFileSync,mkdirSync,existsSync} from 'node:fs';
import path from 'node:path';
const root=path.resolve(import.meta.dirname,'..');
const keyPath=path.join(root,'.tools/update-signing/private.dpapi');
const publicPath=path.join(root,'src-tauri/update-public-key.txt');
function protect(bytes,unprotect=false){const command=`Add-Type -AssemblyName System.Security; $data=[Convert]::FromBase64String([Console]::In.ReadToEnd()); $result=[Security.Cryptography.ProtectedData]::${unprotect?'Unprotect':'Protect'}($data,$null,[Security.Cryptography.DataProtectionScope]::CurrentUser); [Console]::Write([Convert]::ToBase64String($result))`;const p=spawnSync('powershell',['-NoProfile','-NonInteractive','-Command',command],{input:bytes.toString('base64'),encoding:'utf8',windowsHide:true});if(p.status!==0)throw new Error('Windows key protection failed');return Buffer.from(p.stdout,'base64');}
if(process.argv[2]==='init'){
 if(existsSync(keyPath)||existsSync(publicPath))throw new Error('Signing key already exists; refusing to rotate it');
 const pair=generateKeyPairSync('ed25519');const secret=pair.privateKey.export({type:'pkcs8',format:'der'});mkdirSync(path.dirname(keyPath),{recursive:true});writeFileSync(keyPath,protect(secret),{flag:'wx'});secret.fill(0);writeFileSync(publicPath,pair.publicKey.export({type:'spki',format:'der'}).subarray(-32).toString('hex')+'\n',{flag:'wx'});console.log('Update signing initialized; public key saved, private key protected by Windows.');
}else{
 const version=JSON.parse(readFileSync(path.join(root,'package.json'),'utf8')).version;if(!/^\d+\.\d+\.\d+$/.test(version))throw new Error('Invalid version');
 const asset=kind=>{const name=`Local-Studio-${version}-hub-${kind==='installer'?'setup.exe':'portable.zip'}`;const bytes=readFileSync(path.join(root,'releases',name));return {url:`https://github.com/ScopeBotlul/Local-Studio/releases/download/v${version}/${name}`,size:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex')};};
 const notesPath=process.argv[2];if(!notesPath)throw new Error('Pass the release notes file');
 const notes=readFileSync(notesPath,'utf8');if(Buffer.byteLength(notes)>16000)throw new Error('Release notes exceed 16000 UTF-8 bytes');
 const bytes=Buffer.from(JSON.stringify({schema:1,version,publishedAt:new Date().toISOString(),notes,installer:asset('installer'),portable:asset('portable')},null,2));
 if(bytes.length>65536)throw new Error('Manifest too large');const secret=protect(readFileSync(keyPath),true);const key=createPrivateKey({key:secret,format:'der',type:'pkcs8'});secret.fill(0);const signature=sign(null,bytes,key);const publicKey=Buffer.concat([Buffer.from('302a300506032b6570032100','hex'),Buffer.from(readFileSync(publicPath,'utf8').trim(),'hex')]);if(!verify(null,bytes,{key:publicKey,format:'der',type:'spki'},signature))throw new Error('Signing key does not match embedded public key');
 writeFileSync(path.join(root,'releases/update.json'),bytes);writeFileSync(path.join(root,'releases/update.sig'),signature.toString('base64'));console.log(`Signed update manifest for ${version}. Private key was not exported.`);
}
