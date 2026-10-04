// Isolated native UI, real input/mask files; optional real Vulkan inpainting.
import assert from 'node:assert/strict';
import {chromium} from '@playwright/test';
import {spawn} from 'node:child_process';
import {promises as fs} from 'node:fs';
import path from 'node:path';
import net from 'node:net';
import {createHash} from 'node:crypto';
const root=path.resolve(import.meta.dirname,'..'),artifacts=path.join(root,'.artifacts',`image-edit-studio-${Date.now()}`);
await fs.mkdir(artifacts,{recursive:true});const exe=path.join(artifacts,'Local Studio.exe');
await fs.copyFile(process.env.LOCAL_STUDIO_TEST_EXE||path.join(root,'src-tauri/target/debug/local-studio.exe'),exe);
await fs.cp(path.join(root,'.tools/image-runtime'),path.join(artifacts,'image-runtime'),{recursive:true});
await fs.writeFile(path.join(artifacts,'portable.marker'),'isolated image editing test');
const server=net.createServer();await new Promise(r=>server.listen(0,'127.0.0.1',r));const port=server.address().port;await new Promise(r=>server.close(r));
let browser,child;const evidence={passed:false},errors=[];const pause=ms=>new Promise(r=>setTimeout(r,ms));
try{
 child=spawn(exe,[],{cwd:root,windowsHide:true,stdio:'ignore',env:{...process.env,LOCAL_STUDIO_CONFIG_DIR:path.join(artifacts,'config'),WEBVIEW2_USER_DATA_FOLDER:path.join(artifacts,'webview'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:`--remote-debugging-port=${port} --disable-background-timer-throttling`}});
 for(let i=0;i<80&&!browser;i++){try{browser=await chromium.connectOverCDP(`http://127.0.0.1:${port}`,{timeout:1000});}catch{await pause(500);}}
 assert.ok(browser);const page=browser.contexts()[0].pages()[0];page.on('pageerror',e=>errors.push(String(e)));
 await page.getByRole('button',{name:/Studio einrichten|Set up studio/}).click({timeout:60000});
 const invoke=(command,args={})=>page.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
 const snapshot=await invoke('bootstrap'),source=path.join(artifacts,'source.png');
 const data=await page.evaluate(()=>{const c=document.createElement('canvas');c.width=c.height=512;const x=c.getContext('2d');x.fillStyle='#b7cbdc';x.fillRect(0,0,512,512);x.fillStyle='#314555';x.fillRect(140,140,220,220);return c.toDataURL('image/png').split(',')[1];});
 await fs.writeFile(source,Buffer.from(data,'base64'));const original=await fs.readFile(source);
 await page.getByRole('button',{name:/^Studio$/}).first().click();const tabs=page.getByRole('navigation',{name:/Studio-Werkzeuge|Studio tools/});
 await tabs.getByRole('button',{name:/Bild erstellen|Create image/}).click();
 assert.equal(await page.locator('.image-reference,.image-edit-source').count(),0);
 assert.equal(await page.getByText(/Referenzbild und Inpainting|Reference image and inpainting/).count(),0);
 await tabs.getByRole('button',{name:/Bild bearbeiten|Edit image/}).click();
 await page.getByRole('button',{name:/Mit KI bearbeiten|Edit with AI/}).waitFor();
 await page.waitForFunction(()=>!!document.querySelector('.image-edit-source[data-file-drop]'));
 await page.evaluate(source=>window.dispatchEvent(new CustomEvent('studio-reference-drop',{detail:{paths:[source]}})),source);
 await page.locator('.edit-image-surface img').waitFor();
 assert.equal(await page.locator('.edit-image-surface img').evaluate(img=>img.naturalWidth),512);
 const area=page.locator('.edit-image-surface');await area.scrollIntoViewIfNeeded();
 await page.getByRole('button',{name:/^Rechteck$|^Rectangle$/}).click();const box=await area.boundingBox();
 await page.mouse.move(box.x+box.width*.3,box.y+box.height*.3);await page.mouse.down();await page.mouse.move(box.x+box.width*.6,box.y+box.height*.6,{steps:5});await page.mouse.up();
 assert.ok(await page.locator('.edit-tools').getByRole('button',{name:/^Rückgängig$|^Undo$/}).isEnabled());
 await page.locator('.edit-tools').getByRole('button',{name:/^Rückgängig$|^Undo$/}).click();assert.ok(await page.locator('.edit-tools').getByRole('button',{name:/^Wiederholen$|^Redo$/}).isEnabled());await page.locator('.edit-tools').getByRole('button',{name:/^Wiederholen$|^Redo$/}).click();
 await page.getByRole('button',{name:/Ebenen und Werkzeuge|Layers and tools/}).click();await page.getByRole('button',{name:/Neue Arbeitsfläche|New canvas/}).waitFor();await page.getByRole('button',{name:/Mit KI bearbeiten|Edit with AI/}).click();await page.locator('.edit-image-surface img').waitFor();
 await page.screenshot({path:path.join(artifacts,'editing.png')});
 evidence.ui='generation has no reference controls; real image drop, rectangle marking, undo/redo, draft restoration and retained layer editor passed';
 if(process.env.LOCAL_STUDIO_EDIT_AI==='1'){
  const model=process.env.LOCAL_STUDIO_EDIT_MODEL||String.raw`D:\LocalAI\ComfyUI_windows_portable\ComfyUI\models\checkpoints\DreamShaperXL_Turbo_V2.safetensors`;
  await page.getByText(/Engine und Modellinformationen|Engine and model details/,{exact:true}).click();
  await page.locator('#image-model').fill(model);await page.locator('#image-engine').selectOption('vulkan');
  await page.getByRole('checkbox',{name:/VAE auf der CPU|Run VAE on CPU/}).check();
  await page.getByLabel('Image prompt',{exact:true}).fill('A small red flower painted on a blue square, crisp details');
  await page.getByText(/Generierungseinstellungen|Generation settings/,{exact:false}).first().click();await page.getByLabel('Image steps',{exact:true}).fill('2');
  await page.locator('[data-image-generate]').waitFor();await page.waitForFunction(()=>!document.querySelector('[data-image-generate]').disabled,{},{timeout:60000});
  await page.locator('[data-image-generate]').click();
  let done;for(let i=0;i<600;i++){const jobs=await invoke('image_jobs');const j=jobs.find(j=>j.request.reference);if(j&&['failed','cancelled'].includes(j.status))throw Error(JSON.stringify({error:j.error,log:j.logTail}));if(j?.status==='completed'){done=j;break;}await pause(500);}
  assert.ok(done);assert.ok(done.request.reference.mask);
  const sourcePreview=(await invoke('image_reference',{path:done.request.reference.path})).preview;
  const maskPreview=(await invoke('image_reference',{path:done.request.reference.mask.path,mask:true})).preview;
  const output=await invoke('image_output',{id:done.id});
  const compared=await page.evaluate(async({source,mask,output})=>{const load=async url=>{const img=new Image();img.src=url;await img.decode();const c=document.createElement('canvas');c.width=img.width;c.height=img.height;c.getContext('2d').drawImage(img,0,0);return c.getContext('2d').getImageData(0,0,c.width,c.height).data;};const [a,b,m]=await Promise.all([load(source),load(output),load(mask)]);let outside=0,changed=0;for(let i=0;i<a.length;i+=4){if(m[i]===0){outside++;if(a[i]!==b[i]||a[i+1]!==b[i+1]||a[i+2]!==b[i+2]||a[i+3]!==b[i+3])throw Error('Unselected pixel changed');}else if(a[i]!==b[i]||a[i+1]!==b[i+1]||a[i+2]!==b[i+2])changed++;}return {outside,changed};},{source:sourcePreview,mask:maskPreview,output});
  assert.ok(compared.outside>100000&&compared.changed>0);assert.equal(createHash('sha256').update(await fs.readFile(source)).digest('hex'),createHash('sha256').update(original).digest('hex'));
  await invoke('image_save',{id:done.id,folder:''});evidence.ai={job:done.id,...compared};
 }
 assert.deepEqual(errors,[]);evidence.passed=true;console.log(`PASS image editing studio; ${artifacts}`);
}finally{
 if(browser){const page=browser.contexts()[0]?.pages()[0];if(page){await page.screenshot({path:path.join(artifacts,'final.png')}).catch(()=>{});evidence.page=await page.locator('body').innerText().catch(()=> '');}}
 await fs.writeFile(path.join(artifacts,'report.json'),JSON.stringify({...evidence,errors},null,2));await browser?.close().catch(()=>{});child?.kill();
}
