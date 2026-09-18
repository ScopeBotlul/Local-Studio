// Explicit developer setup; never called by models or by the application.
import {readFile,mkdir,writeFile,copyFile,rm} from 'node:fs/promises';
import path from 'node:path';
import {createHash,randomUUID} from 'node:crypto';
import {spawnSync} from 'node:child_process';
const root=path.resolve(import.meta.dirname,'..');
const manifest=JSON.parse(await readFile(path.join(root,'src-tauri/video-runtime.json')));
const hash=b=>createHash('sha256').update(b).digest('hex');
const destination=path.join(root,'.tools/video-runtime');
try{for(const [name,digest]of Object.entries(manifest.files))if(hash(await readFile(path.join(destination,name)))!==digest)throw Error('changed');console.log('Pinned video runtime already verified.');process.exit(0);}catch{}
const work=path.join(root,'.tools',`video-bootstrap-${randomUUID()}`);await mkdir(work,{recursive:true});
try{
 let response=await fetch(manifest.url);if(!response.ok)response=await fetch(`https://github.com/ScopeBotlul/Local-Studio/releases/download/v0.22.0/${manifest.name}`);if(!response.ok)throw Error(`Download failed: ${response.status}`);
 const bytes=Buffer.from(await response.arrayBuffer());if(bytes.length!==manifest.size||hash(bytes)!==manifest.sha256)throw Error('Runtime archive integrity mismatch');
 const archive=path.join(work,'runtime.zip');await writeFile(archive,bytes);
 const extracted=path.join(work,'extracted');
 const result=spawnSync('powershell.exe',['-NoProfile','-NonInteractive','-Command','Expand-Archive -LiteralPath $env:LS_VIDEO_ARCHIVE -DestinationPath $env:LS_VIDEO_EXTRACT'],{env:{...process.env,LS_VIDEO_ARCHIVE:archive,LS_VIDEO_EXTRACT:extracted},encoding:'utf8',windowsHide:true});if(result.status!==0)throw Error(result.stderr);
 const packageRoot=path.join(extracted,manifest.name.slice(0,-4));const staging=path.join(work,'staging');await mkdir(staging);
 for(const [name,digest]of Object.entries(manifest.files)){
  if(!/^[A-Za-z0-9_.-]+$/.test(name))throw Error('Invalid manifest file name');
  const source=name==='LICENSE.txt'?path.join(packageRoot,name):name.endsWith('.txt')?path.join(root,'docs/video-runtime',name):path.join(packageRoot,'bin',name);
  const data=await readFile(source);if(hash(data)!==digest)throw Error(`Runtime file integrity mismatch: ${name}`);await copyFile(source,path.join(staging,name));
 }
 await mkdir(destination,{recursive:true});for(const name of Object.keys(manifest.files))await copyFile(path.join(staging,name),path.join(destination,name));
 console.log('Pinned LGPL video runtime verified and installed for development.');
}finally{await rm(work,{recursive:true,force:true});}
