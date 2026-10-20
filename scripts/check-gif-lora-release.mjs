// Release smoke: real bundled UI and IPC, isolated data, no GPU generation.
import assert from 'node:assert/strict';
import {chromium} from '@playwright/test';
import {spawn} from 'node:child_process';
import {promises as fs} from 'node:fs';
import path from 'node:path';
import net from 'node:net';
const root=path.resolve(import.meta.dirname,'..'),artifacts=path.join(root,'.artifacts',`gif-lora-release-${Date.now()}`);
await fs.mkdir(artifacts,{recursive:true});const exe=path.join(artifacts,'Local Studio.exe');
await fs.copyFile(path.join(root,'src-tauri/target/release/local-studio.exe'),exe);
await fs.writeFile(path.join(artifacts,'portable.marker'),'isolated release check');
const server=net.createServer();await new Promise(r=>server.listen(0,'127.0.0.1',r));const port=server.address().port;await new Promise(r=>server.close(r));
let browser,child;const report={passed:false},errors=[];
try{
 child=spawn(exe,[],{cwd:root,windowsHide:true,stdio:'ignore',env:{...process.env,LOCAL_STUDIO_CONFIG_DIR:path.join(artifacts,'config'),WEBVIEW2_USER_DATA_FOLDER:path.join(artifacts,'webview'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:`--remote-debugging-port=${port}`}});
 for(let i=0;i<80&&!browser;i++){try{browser=await chromium.connectOverCDP(`http://127.0.0.1:${port}`,{timeout:1000});}catch{await new Promise(r=>setTimeout(r,500));}}
 assert.ok(browser);const page=browser.contexts()[0].pages()[0];page.on('pageerror',e=>errors.push(String(e)));
 await page.getByRole('button',{name:/Studio einrichten|Set up studio/}).click({timeout:60000});
 const version=await page.evaluate(async()=>{localStorage.setItem('gif-wan-loras',JSON.stringify([{path:'C:\\test-only\\wan-motion.safetensors',strength:0.7}]));return (await window.__TAURI_INTERNALS__.invoke('bootstrap')).version;});
 assert.equal(version,JSON.parse(await fs.readFile(path.join(root,'package.json'),'utf8')).version);
 await page.getByRole('button',{name:/^Studio$/}).first().click();const tabs=page.getByRole('navigation',{name:/Studio-Werkzeuge|Studio tools/});
 await tabs.getByRole('button',{name:/GIF erstellen|^GIF$/}).click();const loras=page.locator('.wan-loras');
 assert.equal(await loras.getByRole('spinbutton').inputValue(),'0.7');
 await loras.getByRole('spinbutton').fill('0.55');
 assert.equal(await page.evaluate(()=>JSON.parse(localStorage.getItem('gif-wan-loras'))[0].strength),0.55);
 await page.locator('.gif-workspace').getByRole('group',{name:/KI-Engine|AI engine/}).getByRole('button',{name:'Vulkan',exact:true}).click();
 assert.equal(await loras.getByRole('spinbutton').inputValue(),'0.55');
 await page.screenshot({path:path.join(artifacts,'gif-loras.png')});
 await loras.getByRole('button',{name:/LoRA entfernen|Remove LoRA/}).click();assert.equal(await loras.getByRole('spinbutton').count(),0);
 assert.deepEqual(errors,[]);Object.assign(report,{passed:true,version,scope:'Bundled release UI, real IPC version, LoRA controls, strength, persistence, engine switching and removal; no inference'});
 console.log('PASS release GIF LoRA UI:',artifacts);
}finally{await fs.writeFile(path.join(artifacts,'report.json'),JSON.stringify({...report,errors},null,2));await browser?.close().catch(()=>{});child?.kill();}
