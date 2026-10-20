// Isolated native UI. Controlled library/request interception; no GPU inference.
import assert from 'node:assert/strict';
import {chromium} from '@playwright/test';
import {spawn} from 'node:child_process';
import {promises as fs} from 'node:fs';
import path from 'node:path';
import net from 'node:net';
const root=path.resolve(import.meta.dirname,'..'),artifacts=path.join(root,'.artifacts',`gif-loras-${Date.now()}`);
await fs.mkdir(artifacts,{recursive:true});const exe=path.join(artifacts,'Local Studio.exe');
await fs.copyFile(process.env.LOCAL_STUDIO_TEST_EXE||path.join(root,'src-tauri/target/debug/local-studio.exe'),exe);
await fs.writeFile(path.join(artifacts,'portable.marker'),'isolated GIF LoRA test');
const server=net.createServer();await new Promise(r=>server.listen(0,'127.0.0.1',r));const port=server.address().port;await new Promise(r=>server.close(r));
let browser,child;const errors=[],evidence={passed:false};
try{
 child=spawn(exe,[],{cwd:root,windowsHide:true,stdio:'ignore',env:{...process.env,LOCAL_STUDIO_CONFIG_DIR:path.join(artifacts,'config'),WEBVIEW2_USER_DATA_FOLDER:path.join(artifacts,'webview'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:`--remote-debugging-port=${port}`}});
 for(let i=0;i<80&&!browser;i++){try{browser=await chromium.connectOverCDP(`http://127.0.0.1:${port}`,{timeout:1000});}catch{await new Promise(r=>setTimeout(r,500));}}
 assert.ok(browser);const page=browser.contexts()[0].pages()[0];page.on('pageerror',e=>errors.push(String(e)));
 await page.getByRole('button',{name:/Studio einrichten|Set up studio/}).waitFor({timeout:60000});
 await page.route('**/node_modules/**',async route=>{const response=await route.fetch();if(response.headers()['content-type']?.includes('javascript')){const body=(await response.text()).replaceAll('window.__TAURI_INTERNALS__.invoke(', '(window.testInvoke??window.__TAURI_INTERNALS__.invoke)(');await route.fulfill({response,body});}else await route.fulfill({response});});
 await page.reload();
 await page.getByRole('button',{name:/Studio einrichten|Set up studio/}).click({timeout:60000});
 await page.evaluate(()=>{
   const real=window.__TAURI_INTERNALS__.invoke;window.testRealInvoke=real;window.testRequests=[];
   window.testInvoke=(cmd,args)=>{
     if(cmd==='model_library_list')return Promise.resolve({entries:[{id:'wan-motion',name:'wan-motion.safetensors',path:'C:\\fixture\\wan-motion.safetensors',status:'recognized',profile:{role:'extension',family:'LoRA',baseFamily:'Wan 2.1'}}]});
     if(cmd==='gif_model_catalog')return Promise.resolve([{name:'wan2.1_t2v.safetensors',path:'C:\\fixture\\wan2.1_t2v.safetensors',bytes:1234,supportsImage:false,supportsPrompt:true}]);
     if(cmd==='gif_ai_create'||cmd==='gif_vulkan_create'){window.testRequests.push({cmd,request:args.request});return Promise.reject('gif_ai_lora_unavailable');}
     return real(cmd,args);
   };
   for(const part of ['model','encoder','vae'])localStorage.setItem('gif-vulkan-'+part,'C:\\fixture\\wan-'+part+'.safetensors');
 });
 await page.getByRole('button',{name:/^Studio$/}).first().click();const tabs=page.getByRole('navigation',{name:/Studio-Werkzeuge|Studio tools/});
 await tabs.getByRole('button',{name:/GIF erstellen|^GIF$/}).click();const workspace=page.locator('.gif-workspace');
 const loras=workspace.locator('.wan-loras');await loras.locator('summary').click();
 await loras.getByRole('combobox').selectOption('C:\\fixture\\wan-motion.safetensors');
 await loras.getByRole('spinbutton',{name:/LoRA-Stärke 1|LoRA strength 1/}).fill('0.65');
 await workspace.getByRole('group',{name:/KI-Engine|AI engine/}).getByRole('button',{name:'ComfyUI',exact:true}).click();
 await workspace.locator('#gif-model').selectOption('C:\\fixture\\wan2.1_t2v.safetensors');
 await workspace.locator('textarea').first().fill('A paper boat drifting across water');
 await workspace.getByRole('button',{name:/^GIF generieren$|^Generate GIF$/}).click();
 await page.waitForFunction(()=>window.testRequests.length===1);
 await workspace.getByRole('group',{name:/KI-Engine|AI engine/}).getByRole('button',{name:'Vulkan',exact:true}).click();
 await workspace.getByRole('button',{name:/^GIF generieren$|^Generate GIF$/}).click();
 await page.waitForFunction(()=>window.testRequests.length===2);
 const requests=await page.evaluate(()=>window.testRequests);assert.deepEqual(requests.map(r=>r.cmd),['gif_ai_create','gif_vulkan_create']);
 for(const r of requests)assert.deepEqual(r.request.loras,[{path:'C:\\fixture\\wan-motion.safetensors',strength:0.65}]);
 // Exercise the actual Rust IPC validator without contacting ComfyUI or a GPU.
 const rejected=await page.evaluate(async r=>{try{await window.testRealInvoke('gif_ai_create',{request:{...r,loras:[{path:'C:\\fixture\\missing.safetensors',strength:3}]}});return '';}catch(e){return String(e);}},requests[0].request);
 assert.equal(rejected,'image_lora');
 await tabs.getByRole('button',{name:/Bild erstellen|Create image/}).click();await tabs.getByRole('button',{name:/GIF erstellen|^GIF$/}).click();
 assert.equal(await page.locator('.wan-loras input[type=number]').inputValue(),'0.65');
 await page.locator('.wan-loras').getByRole('button',{name:/LoRA entfernen|Remove LoRA/}).click();
 assert.equal(await page.locator('.wan-loras input[type=number]').count(),0);
 assert.deepEqual(errors,[]);evidence.passed=true;evidence.requests=requests;evidence.scope='Native UI selection, strength, removal, local persistence, controlled ComfyUI/Vulkan request payloads and actual Rust IPC rejection; no GPU generation';
 console.log('PASS GIF LoRAs:',artifacts);
}finally{
 if(browser){await browser.contexts()[0]?.pages()[0]?.screenshot({path:path.join(artifacts,'final.png')}).catch(()=>{});}
 await fs.writeFile(path.join(artifacts,'report.json'),JSON.stringify({...evidence,errors},null,2));await browser?.close().catch(()=>{});child?.kill();
}
