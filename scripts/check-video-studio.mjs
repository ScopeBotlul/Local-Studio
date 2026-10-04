// Real Windows WebView, isolated data, actual MP4 decoding/encoding. Optional
// GPU checks use existing local ComfyUI models; no dependencies are installed.
import assert from 'node:assert/strict';
import {chromium} from '@playwright/test';
import {spawn,spawnSync} from 'node:child_process';
import {promises as fs} from 'node:fs';
import path from 'node:path';
import net from 'node:net';
import {randomUUID} from 'node:crypto';
const root=path.resolve(import.meta.dirname,'..');
const artifacts=path.join(root,'.artifacts',`video-studio-${Date.now()}`);
await fs.mkdir(artifacts,{recursive:true});
const exe=path.join(artifacts,'Local Studio.exe');
await fs.copyFile(process.env.LOCAL_STUDIO_TEST_EXE||path.join(root,'src-tauri/target/release/local-studio.exe'),exe);
await fs.cp(path.join(root,'.tools/video-runtime'),path.join(artifacts,'video-runtime'),{recursive:true});
await fs.writeFile(path.join(artifacts,'portable.marker'),'isolated video verification');
const server=net.createServer();await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const port=server.address().port;await new Promise(resolve=>server.close(resolve));
let browser,child;const errors=[];const evidence={passed:false,ai:[]};
const pause=ms=>new Promise(resolve=>setTimeout(resolve,ms));
try{
  child=spawn(exe,[],{cwd:root,windowsHide:true,env:{...process.env,LOCAL_STUDIO_CONFIG_DIR:path.join(artifacts,'config'),WEBVIEW2_USER_DATA_FOLDER:path.join(artifacts,'webview'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:`--remote-debugging-port=${port} --disable-background-timer-throttling`},stdio:'ignore'});
  for(let i=0;i<80&&!browser;i++){try{browser=await chromium.connectOverCDP(`http://127.0.0.1:${port}`,{timeout:1000});}catch{await pause(500);}}
  assert.ok(browser);const page=browser.contexts()[0].pages()[0];page.on('pageerror',e=>errors.push(String(e)));
  await page.waitForFunction(()=>!!window.__TAURI_INTERNALS__?.invoke);
  const invoke=(command,args={})=>page.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
  const setup=page.getByRole('button',{name:/Studio einrichten|Set up studio/});
  await page.waitForFunction(()=>[...document.querySelectorAll('button')].some(button=>/Studio einrichten|Set up studio/.test(button.textContent)),{},{timeout:60000});
  if(await setup.isVisible())await setup.click();
  const snapshot=await invoke('bootstrap');
  await invoke('video_generation_cancel');assert.equal(await invoke('video_generation_status'),null);
  const source=path.join(snapshot.paths.gallery,'video-source.png');
  const png=await page.evaluate(()=>{const c=document.createElement('canvas');c.width=160;c.height=128;const ctx=c.getContext('2d');ctx.fillStyle='#405ca8';ctx.fillRect(0,0,160,128);ctx.fillStyle='#fff';ctx.fillRect(50,40,50,40);return c.toDataURL('image/png').split(',')[1];});
  await fs.writeFile(source,Buffer.from(png,'base64'));
  const id=randomUUID(),pendingRoot=path.join(snapshot.paths.temporary,'video-results');await fs.mkdir(pendingRoot,{recursive:true});
  const fixture=path.join(pendingRoot,`${id}.mp4`);
  const encoded=spawnSync(path.join(root,'.tools/video-runtime/ffmpeg.exe'),['-hide_banner','-v','error','-nostdin','-f','lavfi','-i','testsrc=size=160x128:rate=10:duration=0.5','-an','-c:v','libopenh264','-b:v','1M','-pix_fmt','yuv420p','-movflags','+faststart',fixture],{windowsHide:true,encoding:'utf8'});
  assert.equal(encoded.status,0,encoded.stderr);
  await page.getByRole('button',{name:/^Studio$/}).first().click();
  const tabs=page.getByRole('navigation',{name:/Studio-Werkzeuge|Studio tools/});await tabs.getByRole('button',{name:/Video erstellen|Create video/}).click();
  const workspace=page.locator('.video-generation-studio');await workspace.waitFor();
  assert.equal(await workspace.getByRole('button',{name:/Prompt zu Video|Prompt to video/}).getAttribute('aria-pressed'),'true');
  const player=workspace.locator('video');await player.waitFor();await player.evaluate(v=>new Promise((resolve,reject)=>{if(v.readyState>=1)return resolve();v.onloadedmetadata=resolve;v.onerror=()=>reject(v.error?.message);setTimeout(()=>reject(Error('MP4 metadata timeout')),10000);}));
  assert.deepEqual(await player.evaluate(v=>[v.videoWidth,v.videoHeight]),[160,128]);
  await player.evaluate(v=>v.play());await page.waitForTimeout(100);assert.ok(await player.evaluate(v=>v.currentTime>0));
  await workspace.getByRole('button',{name:/In Galerie speichern|Save to gallery/}).click();
  await page.waitForFunction(async()=>!(await window.__TAURI_INTERNALS__.invoke('video_pending_list')).length);
  assert.ok((await fs.stat(path.join(snapshot.paths.gallery,`Local-Studio-${id}.mp4`))).isFile());
  await workspace.getByRole('button',{name:/Im Videoschnitt öffnen|Open in video editor/}).click();
  await page.getByRole('button',{name:/Clips hinzufügen|Add clips/}).waitFor();
  const project=await invoke('project_snapshot');
  assert.ok(project.creative.video.clips.some(clip=>clip.name===`Local-Studio-${id}.mp4`));
  await tabs.getByRole('button',{name:/Video erstellen|Create video/}).click();
  await workspace.getByRole('button',{name:/video-source.png/}).click();await workspace.locator('.gif-canvas-media img').waitFor();
  assert.equal(await workspace.getByRole('button',{name:/Bild zu Video|Image to video/}).getAttribute('aria-pressed'),'true');
  assert.equal(await workspace.getByLabel(/^Breite$|^Width$/).inputValue(),'160');
  await workspace.getByRole('button',{name:/Bild entfernen|Remove image/}).click();
  assert.equal(await workspace.getByRole('button',{name:/Prompt zu Video|Prompt to video/}).getAttribute('aria-pressed'),'true');
  await workspace.getByLabel('FPS',{exact:true}).fill('0');assert.equal(await workspace.getByRole('button',{name:/Video generieren|Generate Video/}).isDisabled(),true);
  await workspace.getByLabel('FPS',{exact:true}).fill('10');
  if(process.env.LOCAL_STUDIO_REFERENCE_DROP==='1'){
    const reference=workspace.locator('[data-file-drop="reference"]');await reference.scrollIntoViewIfNeeded();
    const emit=paths=>page.evaluate(paths=>window.dispatchEvent(new CustomEvent('studio-reference-drop',{detail:{paths}})),paths);
    await emit([source]);await workspace.locator('.gif-canvas-media img').waitFor();
    assert.equal(await workspace.getByRole('button',{name:/Bild zu Video|Image to video/}).getAttribute('aria-pressed'),'true');
    assert.equal(await workspace.getByLabel(/^Breite$|^Width$/).inputValue(),'160');
    await workspace.getByRole('button',{name:/Bild entfernen|Remove image/}).click();
    await emit([path.join(snapshot.paths.gallery,'not-image.txt')]);
    assert.equal(await workspace.getByRole('button',{name:/Prompt zu Video|Prompt to video/}).getAttribute('aria-pressed'),'true');
    evidence.drop='controlled drop event in native WebView, real image validation, I2V mode and automatic dimensions passed; physical Explorer drag not automated';
    await tabs.getByRole('button',{name:/Bild erstellen|Create image/}).click();
    assert.equal(await page.locator('.image-reference').count(),0);await tabs.getByRole('button',{name:/Bild bearbeiten|Edit image/}).click();await page.waitForFunction(()=>!!document.querySelector('.image-edit-source[data-file-drop]'));
    const imageSource=path.join(snapshot.paths.gallery,'image-reference.png');
    const imagePng=await page.evaluate(()=>{const c=document.createElement('canvas');c.width=256;c.height=256;const ctx=c.getContext('2d');ctx.fillStyle='#45a';ctx.fillRect(0,0,256,256);return c.toDataURL('image/png').split(',')[1];});
    await fs.writeFile(imageSource,Buffer.from(imagePng,'base64'));
    await emit([imageSource]);await page.locator('.edit-image-surface img').waitFor();
    assert.ok(await page.locator('.image-edit-source').getByRole('button',{name:/Bild entfernen|Remove image/}).isVisible());
    evidence.imageDrop='image editing studio accepts a real validated PNG working copy via controlled drop event';
    await tabs.getByRole('button',{name:/Video erstellen|Create video/}).click();
  }
  await page.screenshot({path:path.join(artifacts,'video-workspace.png')});
  evidence.ui='real MP4 playback, explicit modes, reference sizing/removal, FPS validation, confirmed gallery save and timeline transfer passed';
  if(process.env.LOCAL_STUDIO_VIDEO_AI==='1'){
    const comfyRoot=process.env.LOCAL_STUDIO_COMFY_ROOT;
    assert.ok(comfyRoot,'Set LOCAL_STUDIO_COMFY_ROOT for real GPU checks');
    const queue=await (await fetch('http://127.0.0.1:8188/queue')).json();assert.equal(queue.queue_running.length,0,'Do not interfere with an existing ComfyUI job');assert.equal(queue.queue_pending.length,0);
    await invoke('comfy_set_path',{path:comfyRoot});
    const catalog=await invoke('gif_model_catalog');
    const wan21=catalog.find(m=>m.name==='wan2.1_t2v_1.3B_bf16.safetensors');
    const wan22=catalog.find(m=>m.name==='wan2.2_ti2v_5B_fp16.safetensors');
    assert.ok(wan21&&wan22,'Two existing Wan models are required for model-switch verification');
    assert.equal(wan21.supportsImage,false);assert.equal(wan21.supportsPrompt,true);
    const nsfw=catalog.find(m=>m.name==='nsfw_wan_14b_e15_q4_k.gguf');if(nsfw){evidence.unmarkedModel={supportsImage:nsfw.supportsImage,supportsPrompt:nsfw.supportsPrompt};}
    const base={engine:'comfy',mode:'prompt',sourcePath:null,modelPath:wan21.path,encoderPath:null,vaePath:null,prompt:'A paper boat floats slowly on calm blue water, soft daylight, stationary camera.',negativePrompt:'distorted, noisy, blurry',width:128,height:128,frames:5,fps:10,steps:2,guidance:5,seed:42};
    await assert.rejects(invoke('video_generation_create',{request:{...base,mode:'image'}}),/gif_ai_source/);
    for(const [model,mode] of [[wan21,'prompt'],[wan22,'prompt'],[wan22,'image']]){
      const started=Date.now();const result=await invoke('video_generation_create',{request:{...base,modelPath:model.path,mode,sourcePath:mode==='image'?source:null}});
      assert.ok(result.bytes>1000);assert.equal(result.request.seed,42);
      const output=await invoke('video_pending_save',{id:result.id,folder:''});
      assert.equal((await invoke('video_result_info',{path:path.basename(output)})).mode,mode);
      await fs.copyFile(output,path.join(artifacts,`${model.name}-${mode}.mp4`));
      evidence.ai.push({model:model.name,mode,bytes:result.bytes,seconds:(Date.now()-started)/1000});
    }
    // Cancellation must target the submitted job and leave another app's queue alone.
    const task=invoke('video_generation_create',{request:{...base,seed:43}}).then(()=>({completed:true}),error=>({error:String(error)}));
    await page.waitForFunction(async()=>['generating','encoding'].includes((await window.__TAURI_INTERNALS__.invoke('video_generation_status'))?.phase));
    await invoke('video_generation_cancel');const cancelled=await task;assert.match(cancelled.error,/video_generation_cancelled|resource_cancelled/);evidence.cancel='real submitted job cancelled';
  }
  assert.deepEqual(errors,[]);evidence.passed=true;
  console.log(`PASS Video Studio; ${artifacts}`);
}finally{
  if(browser){const page=browser.contexts()[0]?.pages()[0];if(page){await page.screenshot({path:path.join(artifacts,'final.png')}).catch(()=>{});evidence.page=await page.locator('body').innerText().catch(()=> '');}}
  await fs.writeFile(path.join(artifacts,'report.json'),JSON.stringify({...evidence,errors},null,2));
  await browser?.close().catch(()=>{});child?.kill();
}
