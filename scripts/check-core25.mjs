import assert from 'node:assert/strict';
import {promises as fs} from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
const pause=ms=>new Promise(r=>setTimeout(r,ms));
async function until(fn){for(let i=0;i<200;i++){if(await fn())return;await pause(100);}throw new Error('Core condition timed out');}
export async function checkCore25({getPage,invoke,artifactRoot,record,pid,stop,launch}){
 let page=getPage();const base=await invoke('bootstrap');
 for(const key of ['liveHardware','minimizeToTray','systemAccent','parallelGeneration'])assert.equal(base.settings[key],false);
 assert.equal(await invoke('background_hide'),false);assert.deepEqual(await invoke('resource_status'),[]);
 await page.getByRole('button',{name:'Settings',exact:true}).first().click();
 await page.getByLabel('Live hardware monitor',{exact:true}).check();
 await page.getByLabel('Use Windows accent color',{exact:true}).check();await page.getByRole('button',{name:'Save changes',exact:true}).click();await until(async()=>(await invoke('bootstrap')).settings.systemAccent);
 await until(async()=>await page.locator('.global-resources').innerText().then(s=>s.includes('CPU')&&s.includes('RAM')));
 const hardware=await invoke('hardware_live');assert(hardware.totalMemoryBytes>hardware.usedMemoryBytes&&hardware.usedMemoryBytes>0);assert(hardware.cpuPercent>=0&&hardware.cpuPercent<=100);for(const g of hardware.gpus){assert(g.utilization===null||(g.utilization>=0&&g.utilization<=100));assert(g.usedBytes===null||g.usedBytes>=0);}
 const accent=await invoke('desktop_accent');if(accent)await until(async()=>await page.evaluate(c=>getComputedStyle(document.documentElement).getPropertyValue('--accent').trim()===c,accent));
 record('Opt-in live CPU/RAM/GPU values and Windows accent are visible and backed by native readings',JSON.stringify(hardware));
 await page.getByLabel('Keep running in the system tray on close',{exact:true}).check();
 await page.getByLabel('Parallel processing when RAM is available',{exact:true}).check();await page.getByRole('button',{name:'Save changes',exact:true}).click();await until(async()=>(await invoke('bootstrap')).settings.parallelGeneration);
 await stop(true);await launch();page=getPage();const restored=await invoke('bootstrap');for(const key of ['liveHardware','minimizeToTray','systemAccent','parallelGeneration'])assert(restored.settings[key]);record('All four resource and Windows preferences survive a real process restart');
 const source=path.join(artifactRoot,'move-source'),destination=path.join(artifactRoot,'move-target');await fs.mkdir(source);await fs.mkdir(destination);
 const header=Buffer.from(JSON.stringify({weight:{dtype:'F32',shape:[1],data_offsets:[0,4]}}));const length=Buffer.alloc(8);length.writeBigUInt64LE(BigInt(header.length));const bytes=Buffer.concat([length,header,Buffer.from([1,2,3,4])]);const original=path.join(source,'move.safetensors');await fs.writeFile(original,bytes);
 await invoke('model_scan_start',{path:source});await until(async()=>(await invoke('model_library_list')).scan.status==='completed');const model=(await invoke('model_library_list')).entries.find(m=>m.name==='move.safetensors');assert(model);
 await page.getByRole('button',{name:'Models',exact:true}).first().click();
 // Native folder-picker return only is controlled; planning, copying, checksums, deletion and persistence remain real IPC/filesystem work.
 await page.evaluate(destination=>{const original=window.fetch;window.fetch=(input,options)=>{const u=new URL(typeof input==='string'?input:input.url);if(u.hostname==='ipc.localhost'&&decodeURIComponent(u.pathname)==='/plugin:dialog|open')return Promise.resolve(new Response(JSON.stringify(destination),{headers:{'Content-Type':'application/json','Tauri-Response':'ok'}}));return original.call(window,input,options);};},destination);
 await page.getByRole('button',{name:'Stored locally',exact:true}).click();
 await page.getByRole('button',{name:/Not clearly identified/}).click();
 const entry=page.locator(`[data-model-id="${model.id}"]`);await entry.getByRole('button',{name:'Move files',exact:true}).click();await page.getByRole('button',{name:'Choose destination',exact:true}).click();await page.getByRole('button',{name:'Move with verification',exact:true}).click();await until(async()=>(await invoke('model_move_status'))?.status==='completed');const moved=(await invoke('model_library_list')).entries.find(m=>m.name==='move.safetensors');assert(moved);assert.notEqual(moved.path,model.path);assert.deepEqual(await fs.readFile(moved.path),bytes);assert.equal(await fs.stat(original).catch(()=>null),null);assert.equal(moved.status,'checked');record('Real model-move UI previews and performs SHA-verified copy, switches registry and removes only the original test file');
 const plan=await invoke('model_move_plan',{id:moved.id,destination:source});await fs.mkdir(plan.destination);const collision=path.join(plan.destination,'keep.txt');await fs.writeFile(collision,'keep');await assert.rejects(invoke('model_move_start',{planId:plan.id,confirmed:false}),/move_confirmation/);await invoke('model_move_start',{planId:plan.id,confirmed:true});await until(async()=>(await invoke('model_move_status'))?.status==='failed');assert.deepEqual(await fs.readFile(moved.path),bytes);assert.equal(await fs.readFile(collision,'utf8'),'keep');record('Unconfirmed and colliding model moves retain original bytes and unrelated destination files');
 const check=spawnSync('python',['scripts/native-window25.py',String(pid()),'visible'],{encoding:'utf8',windowsHide:true});assert.equal(check.status,0,check.stderr);assert.equal(check.stdout.trim(),'true');
 await invoke('plugin:window|close',{label:'main'});await until(async()=>spawnSync('python',['scripts/native-window25.py',String(pid()),'visible'],{encoding:'utf8',windowsHide:true}).stdout.trim()==='false');assert((await invoke('bootstrap')).settings.minimizeToTray);record('Real native window close hides to tray while the process and IPC remain alive');
 // A second launch restores the existing instance; no user application is touched.
 const reopen=spawnSync(path.join(artifactRoot,'Local Studio.exe'),[],{encoding:'utf8',windowsHide:true,timeout:15000});assert.equal(reopen.status,0,reopen.stderr);await until(async()=>spawnSync('python',['scripts/native-window25.py',String(pid()),'visible'],{encoding:'utf8',windowsHide:true}).stdout.trim()==='true');record('Launching the same portable executable restores the hidden existing window');
 const session=await page.context().newCDPSession(page);const image=await session.send('Page.captureScreenshot',{format:'png',fromSurface:true});await fs.writeFile(path.join(artifactRoot,'core25.png'),Buffer.from(image.data,'base64'));await session.detach();
 const settings=(await invoke('bootstrap')).settings;await invoke('save_settings',{settings:{...settings,minimizeToTray:false}});
}
