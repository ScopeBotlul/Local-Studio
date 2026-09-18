// Explicit developer setup only. The app never installs model-provided dependencies.
import {readFile,mkdir,writeFile,copyFile,readdir} from 'node:fs/promises';
import path from 'node:path';import {createHash,randomUUID} from 'node:crypto';import {spawnSync} from 'node:child_process';
const root=path.resolve(import.meta.dirname,'..'),hash=b=>createHash('sha256').update(b).digest('hex');
for(const kind of ['assistant','speech']){
 const manifest=JSON.parse(await readFile(path.join(root,`src-tauri/${kind}-runtime.json`)));const target=path.join(root,`.tools/${kind}-runtime`);
 try{for(const [name,digest]of Object.entries(manifest.files))if(hash(await readFile(path.join(target,name)))!==digest)throw Error('changed');console.log(`${kind} runtime already verified.`);continue;}catch{}
 const work=path.join(root,'.tools',`${kind}-bootstrap-${randomUUID()}`);await mkdir(work,{recursive:true});const response=await fetch(manifest.url);if(!response.ok)throw Error(`Runtime download: ${response.status}`);const bytes=Buffer.from(await response.arrayBuffer());if(bytes.length!==manifest.size||hash(bytes)!==manifest.sha256)throw Error('Runtime archive integrity mismatch');await writeFile(path.join(work,'runtime.zip'),bytes);
 const result=spawnSync('powershell.exe',['-NoProfile','-NonInteractive','-Command','Expand-Archive -LiteralPath $env:LS_AI_ARCHIVE -DestinationPath $env:LS_AI_EXTRACT'],{env:{...process.env,LS_AI_ARCHIVE:path.join(work,'runtime.zip'),LS_AI_EXTRACT:path.join(work,'extracted')},windowsHide:true,encoding:'utf8'});if(result.status!==0)throw Error(result.stderr);
 const staging=path.join(work,'verified');await mkdir(staging);for(const [name,digest]of Object.entries(manifest.files)){if(!/^[A-Za-z0-9_.-]+$/.test(name))throw Error('Invalid manifest');const source=['LICENSE.txt','SOURCE.txt'].includes(name)?path.join(root,`docs/${kind}-runtime`,name):path.join(work,'extracted',manifest.archiveRoot,name);const data=await readFile(source);if(hash(data)!==digest)throw Error(`Runtime file mismatch: ${name}`);await writeFile(path.join(staging,name),data);}
 await mkdir(target,{recursive:true});for(const name of await readdir(staging))await copyFile(path.join(staging,name),path.join(target,name));console.log(`${kind} pinned runtime verified. Download archive retained in ${work}`);
}
