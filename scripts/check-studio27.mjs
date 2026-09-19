import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {popupMenu} from './popup-menu.mjs';
const pause=ms=>new Promise(r=>setTimeout(r,ms));
async function until(fn,timeout=30000){const end=Date.now()+timeout;while(Date.now()<end){const v=await fn();if(v)return v;await pause(150);}throw Error('Studio27 condition timed out');}
export async function checkStudio27({getPage,invoke,artifactRoot,record,defer,pid}){
 const page=getPage();
 await page.evaluate(async()=>{window.menuEvents=[];await window.__TAURI_INTERNALS__.invoke('plugin:event|listen',{event:'app-menu',target:{kind:'Any'},handler:window.__TAURI_INTERNALS__.transformCallback(e=>window.menuEvents.push(e))});});
 async function shot(name){const c=await page.context().newCDPSession(page);await fs.writeFile(path.join(artifactRoot,name),Buffer.from((await c.send('Page.captureScreenshot',{format:'png',fromSurface:true})).data,'base64'));await c.detach();}
 async function menu(group,label){return popupMenu(page,pid(),group,label);}
 assert.deepEqual(await page.getByRole('menubar').getByRole('menuitem').allTextContents(),['File','Edit','View','Help']);
 assert.equal(await page.locator('.window-title').innerText(),'Local Studio');
 // Inspect the same popup that is selected, avoiding a second tracking loop.
 await menu('File','New project …');const dialog=page.getByRole('dialog',{name:'New project',exact:true});await dialog.getByLabel('Project name',{exact:true}).fill('Canvas 27');await dialog.getByRole('button',{name:'Create project',exact:true}).click();await until(async()=>(await page.locator('.window-title').innerText()).includes('Canvas 27'));
 await page.getByRole('button',{name:'Studio',exact:true}).first().click();assert.equal(await page.locator('.project-details-page').count(),0);
 await menu('View','Project media and details …');await page.locator('.project-details-page').waitFor();await page.locator('.project-details-page .project-details-heading button').click();await page.locator('.image-canvas').waitFor();
 await menu('Help','User guide');await page.getByRole('dialog',{name:'User guide',exact:true}).getByRole('button',{name:'Close',exact:true}).click();
 record('Real titlebar popup menus create a project, update centered title, open project details and guide; redundant project panel stays hidden');
 await page.getByRole('button',{name:'Maximize',exact:true}).click();await page.getByRole('button',{name:'Restore',exact:true}).waitFor();await page.getByRole('button',{name:'Restore',exact:true}).click();await page.getByRole('button',{name:'Maximize',exact:true}).waitFor();
 record('Custom window maximize and restore controls operate the real native window');
 const fixture=path.join(artifactRoot,'models');await fs.mkdir(path.join(fixture,'vae_approx'),{recursive:true});await fs.mkdir(path.join(fixture,'loras'));
 async function tensor(file,weights,metadata={}){let offset=0;const h={__metadata__:metadata};for(const name of weights){h[name]={dtype:'F32',shape:[1],data_offsets:[offset,offset+4]};offset+=4;}const b=Buffer.from(JSON.stringify(h)),len=Buffer.alloc(8);len.writeBigUInt64LE(BigInt(b.length));await fs.writeFile(file,Buffer.concat([len,b,Buffer.alloc(offset)]));}
 await tensor(path.join(fixture,'vae_approx','taesd_encoder.safetensors'),['0.weight']);
 await tensor(path.join(fixture,'loras','style.safetensors'),['lora_unet_x.weight'],{ss_base_model_version:'sdxl_base_v1-0'});
 await tensor(path.join(fixture,'unidentified.safetensors'),['unusual.weight']);
 const tagged=path.join(fixture,'tagged.safetensors');await tensor(tagged,['unusual.weight'],{nsfw:'true'});
 await invoke('model_scan_start',{path:fixture});await until(async()=>(await invoke('model_library_list')).scan.status==='completed');const entries=(await invoke('model_library_list')).entries;
 assert.equal(entries.find(e=>e.name==='taesd_encoder.safetensors').profile.role,'component');assert.equal(entries.find(e=>e.name==='style.safetensors').profile.role,'extension');assert.equal(entries.find(e=>e.name==='style.safetensors').profile.baseFamily,'SDXL');assert.equal(entries.find(e=>e.name==='unidentified.safetensors').profile.role,'unknown');assert.equal(await invoke('privacy_model_status',{path:tagged}),true);
 assert(!(await invoke('image_model_catalog')).some(m=>m.path.includes(fixture)));
 await page.getByRole('button',{name:'Models',exact:true}).first().click();await page.getByRole('button',{name:'Stored locally',exact:true}).click();await page.getByRole('button',{name:/Technical components/}).click();await page.getByRole('heading',{name:'taesd_encoder.safetensors',exact:true}).waitFor();assert.equal(await page.getByRole('heading',{name:'style.safetensors',exact:true}).count(),0);
 await page.getByRole('button',{name:/Extensions/}).click();await page.getByRole('heading',{name:'style.safetensors',exact:true}).waitFor();await shot('27-library.png');
 record('Real library imports distinguish encoders, LoRAs and unknown tensors; category filters and base family work; Studio excludes all components; explicit 18+ metadata is enforced');
 const model=String.raw`D:\LocalAI\ComfyUI_windows_portable\ComfyUI\models\checkpoints\DreamShaperXL_Turbo_V2.safetensors`;
 const request={modelPath:model,prompt:'a small red cabin beside a calm lake, daylight',negativePrompt:'text',width:640,height:960,steps:4,guidance:2,seed:270,sampler:'euler',vaeOnCpu:true};
 await invoke('project_close',{confirmed:true});await invoke('image_workspace_save',{workspace:{request,models:{}}});await page.reload();await page.getByRole('button',{name:'Studio',exact:true}).first().click();await page.getByRole('button',{name:'Generate image',exact:true}).waitFor();await until(()=>page.getByRole('button',{name:'Generate image',exact:true}).isEnabled(),90000);
 assert.equal(await page.getByLabel('Image width',{exact:true}).inputValue(),'640');await page.getByLabel('Image width',{exact:true}).fill('641');assert(await page.getByRole('button',{name:'Generate image',exact:true}).isDisabled());await page.getByLabel('Image width',{exact:true}).fill('640');
 await assert.rejects(invoke('image_generate',{request:{...request,width:641}}),/image_dimensions/);await assert.rejects(invoke('image_generate',{request:{...request,width:2048,height:2048}}),/image_dimensions/);
 await page.getByRole('button',{name:'Generate image',exact:true}).click();const job=await until(async()=>(await invoke('image_jobs'))[0],90000);
 const completed=await until(async()=>{const j=(await invoke('image_jobs')).find(j=>j.id===job.id);if(j?.error&&j.error!=='resource_memory')throw Error(j.error);return j?.error==='resource_memory'||j?.status==='completed'?j:false;},600000);
 if(completed.error==='resource_memory'){
  defer('Actual SDXL custom-resolution generation, preview zoom and gallery save','resource_memory: insufficient free system RAM; admission control rejected the worker before loading.');
 }else{
 await page.getByAltText('Locally generated image',{exact:true}).waitFor();const size=await page.getByAltText('Locally generated image',{exact:true}).evaluate(i=>({width:i.naturalWidth,height:i.naturalHeight}));assert.deepEqual(size,{width:640,height:960});
 await page.getByRole('button',{name:'Zoom in',exact:true}).click();await page.getByRole('button',{name:'Fit to canvas',exact:true}).click();await shot('27-canvas-dark.png');
 await page.getByRole('button',{name:'Save to gallery',exact:true}).click();await until(async()=>!!(await invoke('image_jobs')).find(j=>j.id===job.id)?.savedPath);
 record('Actual SDXL generation at custom 640×960 succeeds through the Canvas UI; automatic readiness, invalid-size rejection, preview zoom and gallery save work');
 }
 await page.locator('.image-control-details').filter({has:page.getByText('Model information and file path',{exact:true})}).locator('summary').click();assert(!(await page.locator('#image-model').inputValue()).startsWith('\\\\?\\'));
 const settings=(await invoke('bootstrap')).settings;await invoke('save_settings',{settings:{...settings,language:'de',theme:'light',uiScale:1.25}});await page.reload();await page.getByRole('button',{name:'Studio',exact:true}).first().click();await page.locator('.image-canvas').waitFor();await shot('27-canvas-light-125.png');
 record('German/light UI at 125% keeps the Canvas and unified titlebar available; Windows internal path prefix is hidden');
}
