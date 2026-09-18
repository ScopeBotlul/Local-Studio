import assert from 'node:assert/strict';
import {promises as fs} from 'node:fs';
import path from 'node:path';
import {spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
const pause=ms=>new Promise(r=>setTimeout(r,ms));
async function until(fn,timeout=20000){const end=Date.now()+timeout;while(Date.now()<end){if(await fn())return;await pause(80);}throw new Error('Media 0.18 condition timed out');}
const hash=b=>createHash('sha256').update(b).digest('hex');
export async function checkMedia18({getPage,invoke,stop,launch,artifactRoot,record}){
 let page=getPage();await page.reload();await page.getByRole('button',{name:'Gallery',exact:true}).first().click();
 await page.locator('[data-gallery-watch="active"]').waitFor();
 const gallery=(await invoke('bootstrap')).paths.gallery;const folder=path.join(gallery,'Media18');await fs.mkdir(folder,{recursive:true});
 const encoded=await page.evaluate(async()=>{const c=document.createElement('canvas');c.width=320;c.height=180;const x=c.getContext('2d');x.fillStyle='#23b882';x.fillRect(0,0,320,180);x.fillStyle='#f4aa34';x.fillRect(35,35,100,90);const png=c.toDataURL().split(',')[1];const stream=c.captureStream(12),chunks=[];const r=new MediaRecorder(stream,{mimeType:'video/webm;codecs=vp8'});const finished=new Promise(resolve=>{r.onstop=async()=>resolve(btoa(String.fromCharCode(...new Uint8Array(await new Blob(chunks).arrayBuffer()))));});r.ondataavailable=e=>chunks.push(e.data);r.start();for(let n=0;n<12;n++){x.fillStyle=n%2?'#23b882':'#21906c';x.fillRect(180,0,140,180);await new Promise(r=>setTimeout(r,90));}r.stop();const webm=await finished;stream.getTracks().forEach(t=>t.stop());return{png,webm};});
 await page.evaluate(async()=>{window.media18Events=0;await window.__TAURI_INTERNALS__.invoke('plugin:event|listen',{event:'gallery-changed',target:{kind:'Any'},handler:window.__TAURI_INTERNALS__.transformCallback(()=>window.media18Events++)});});
 const source=path.join(folder,'source.png'),clip=path.join(folder,'clip.webm');await fs.writeFile(source,Buffer.from(encoded.png,'base64'));await fs.writeFile(clip,Buffer.from(encoded.webm,'base64'));
 await until(()=>page.evaluate(()=>window.media18Events>0),3000);
 await page.getByLabel('Search files',{exact:true}).fill('Media18');await until(async()=>await page.locator('.gallery-file').count()===2);
 await page.getByRole('button',{name:'Grid view',exact:true}).click();const card=page.locator('[data-gallery-path="Media18/clip.webm"]');await card.scrollIntoViewIfNeeded();
 await until(async()=>await card.locator('.gallery-thumbnail img').count()===1&&await card.locator('.gallery-thumbnail img').evaluate(i=>i.complete&&i.naturalWidth===320),18000);
 const listing=await invoke('gallery_list',{query:{folder:'Media18',search:'',kind:'all',recursive:true,offset:0}});const video=listing.entries.find(e=>e.kind==='video');const videoQuery={rootId:listing.rootId,path:video.path,version:video.thumbnailVersion};
 const thumb=await invoke('gallery_thumbnail',{query:videoQuery});assert(thumb.cached);assert.deepEqual([thumb.width,thumb.height],[320,180]);assert.equal(hash(await fs.readFile(clip)),hash(Buffer.from(encoded.webm,'base64')));
 const extra=path.join(folder,'watch-only.png');await fs.writeFile(extra,Buffer.from(encoded.png,'base64'));await until(async()=>await page.locator('[data-gallery-path="Media18/watch-only.png"]').count()===1,2500);await fs.rename(extra,path.join(folder,'watch-renamed.png'));await until(async()=>await page.locator('[data-gallery-path="Media18/watch-renamed.png"]').count()===1,2500);await fs.unlink(path.join(folder,'watch-renamed.png'));await until(async()=>await page.locator('.gallery-file').count()===2,2500);
 await page.locator('.gallery-items').screenshot({path:path.join(artifactRoot,'media18-video-thumbnail.png')});record('Real native Windows folder event refreshes gallery; actual VP8 frame decodes into a cached 320px thumbnail without changing the original');
 await page.locator('[data-gallery-path="Media18/source.png"]').click();await until(async()=>await page.locator('[data-gallery-path="Media18/source.png"]').getAttribute('aria-pressed')==='true');await page.getByRole('button',{name:'Create variant copy',exact:true}).click();
 const list=()=>invoke('gallery_list',{query:{folder:'Media18',search:'',kind:'all',recursive:true,offset:0}});await until(async()=>(await list()).entries.filter(e=>e.kind==='image').length===2);
 const copy=(await list()).entries.find(e=>e.kind==='image'&&e.path!=='Media18/source.png');assert.equal(hash(await fs.readFile(path.join(gallery,copy.path))),hash(await fs.readFile(source)));
 await until(async()=>await page.locator('.lineage-chain li').count()===2);
 let originalRow=page.locator('.lineage-chain li').filter({hasText:'Original / import'});await originalRow.getByRole('button',{name:'Set as main',exact:true}).click();await until(async()=>(await originalRow.innerText()).includes('Main version'));
 await page.locator('.gallery-lineage').screenshot({path:path.join(artifactRoot,'media18-lineage.png')});
 const query={rootId:listing.rootId,target:{path:copy.path,fileId:copy.fileId,version:copy.thumbnailVersion}};const before=await invoke('gallery_lineage',{query});assert.equal(before.versions.length,2);assert.equal(before.versions[1].node.parent,before.versions[0].node.id);
 await fs.rename(source,path.join(folder,'renamed.png'));const renamed=await invoke('gallery_lineage',{query});assert.equal(renamed.versions[0].node.path,'Media18/renamed.png');assert(renamed.versions[0].available);
 const sourceEntry=(await list()).entries.find(e=>e.path==='Media18/renamed.png');const target={path:sourceEntry.path,fileId:sourceEntry.fileId,version:sourceEntry.thumbnailVersion};
 const trashed=await invoke('gallery_file_action',{request:{rootId:listing.rootId,targets:[target],action:{type:'trash',confirmed:true}}});assert.equal(trashed.errors.length,0);
 let trash=await invoke('gallery_trash_list',{offset:0});const deleted=trash.entries.find(e=>e.originalPath==='Media18/renamed.png');assert(deleted);
 const purged=await invoke('gallery_trash_action',{request:{rootId:trash.rootId,ids:[deleted.id],action:'purge',confirmed:true}});assert.equal(purged.errors.length,0);
 const after=await invoke('gallery_lineage',{query});assert(!after.versions[0].available);assert(after.versions[1].available);assert.equal(after.primary,after.current);assert.equal(hash(await fs.readFile(path.join(gallery,copy.path))),hash(Buffer.from(encoded.png,'base64')));
 await assert.rejects(invoke('gallery_create_variant',{query:{...query,target:{...query.target,version:'stale'}}}),/gallery_changed/);
 record('Native variant UI creates an independent byte-identical file and real lineage; primary choice, external rename, source trash/purge and stale-version rejection preserve remaining variants and metadata');
 // Permanent deletion intentionally clears the shared preview cache. Rebuild a real video frame before testing restart persistence.
 await page.getByRole('button',{name:'Rebuild thumbnails',exact:true}).click();await page.locator('[data-gallery-path="Media18/clip.webm"]').scrollIntoViewIfNeeded();
 await until(async()=>await invoke('gallery_thumbnail',{query:videoQuery}).then(t=>t.cached,()=>false),18000);
 await stop(true);await launch();page=getPage();assert((await invoke('gallery_thumbnail',{query:videoQuery})).cached);assert.equal((await invoke('gallery_lineage',{query})).versions.length,2);
 record('Video thumbnail cache and origin relationships survive native process restart');
 const a=path.join(artifactRoot,'Open A.localstudio'),b=path.join(artifactRoot,'Open B.localstudio');
 await invoke('project_new',{name:'Open A',request:null,confirmed:true});await invoke('project_save',{path:a});await invoke('project_new',{name:'Open B',request:null,confirmed:true});await invoke('project_save',{path:b});await invoke('project_close',{confirmed:true});
 async function dialogs(){await page.addInitScript(()=>{const original=window.fetch;window.media18={answer:'Cancel',dialogs:0};window.fetch=(input,options)=>{const u=new URL(typeof input==='string'?input:input.url);if(u.hostname==='ipc.localhost'&&decodeURIComponent(u.pathname)==='/plugin:dialog|message'){window.media18.dialogs++;return Promise.resolve(new Response(JSON.stringify(window.media18.answer),{headers:{'Content-Type':'application/json','Tauri-Response':'ok'}}));}return original.call(window,input,options);};});await page.reload();await page.getByRole('button',{name:'Gallery',exact:true}).first().waitFor();}
 await dialogs();await page.locator('.project-title').click();await page.getByText('Recent projects',{exact:true}).click();await page.locator('.project-recent').getByRole('button',{name:/^Open A/}).click();await until(async()=>(await invoke('project_snapshot'))?.name==='Open A');
 await page.getByRole('button',{name:'Studio',exact:true}).first().click();await page.getByLabel('Image prompt',{exact:true}).fill('Keep this unsaved input');await until(async()=>(await invoke('project_snapshot'))?.request?.prompt==='Keep this unsaved input');
 async function secondary(file){await new Promise((resolve,reject)=>{const child=spawn(path.join(artifactRoot,'Local Studio.exe'),[file],{cwd:artifactRoot,windowsHide:true,stdio:'ignore'});const timer=setTimeout(()=>{child.kill();reject(new Error('Second instance did not exit'));},10000);child.once('error',reject);child.once('exit',code=>{clearTimeout(timer);code===0?resolve():reject(new Error(`Second instance ${code}`));});});}
 await secondary(b);await until(()=>page.evaluate(()=>window.media18.dialogs>0));await until(async()=>await invoke('project_take_open')===null);assert.equal((await invoke('project_snapshot')).name,'Open A');assert.equal(await page.getByLabel('Image prompt',{exact:true}).inputValue(),'Keep this unsaved input');
 await page.evaluate(()=>{window.media18.answer='Ok';});await secondary(b);await until(async()=>(await invoke('project_snapshot'))?.name==='Open B');assert.equal(await page.getByLabel('Image prompt',{exact:true}).inputValue(),'');
 await secondary(clip);assert.equal((await invoke('project_snapshot')).name,'Open B');
 record('Recent-project UI opens saved containers; a second native launch forwards one project to the running app, Cancel preserves dirty inputs and confirmation opens the requested project; unrelated arguments are ignored');
 await invoke('project_close',{confirmed:true});await stop(true);await launch([a]);page=getPage();await until(async()=>(await invoke('project_snapshot'))?.name==='Open A'&&!(await invoke('project_snapshot')).recovery);
 await page.locator('.project-title').click();await page.getByText('Recent projects',{exact:true}).click();const row=page.locator('.project-recent .project-message').filter({hasText:'Open B'});await row.getByRole('button',{name:'Remove from list',exact:true}).click();await until(async()=>!(await invoke('project_recent')).some(p=>p.path===b));await fs.access(a);await fs.access(b);
 await page.locator('.project-panel').screenshot({path:path.join(artifactRoot,'media18-recent-projects.png')});
 record('Cold native launch with a project argument opens through the validated project path; recent-project removal affects only the list and both archives remain');
}
