// Targeted native verification of the GIF workspace. Uses isolated app data.
import assert from 'node:assert/strict';
import {chromium} from '@playwright/test';
import {spawn} from 'node:child_process';
import {promises as fs} from 'node:fs';
import path from 'node:path';
import net from 'node:net';

const root=path.resolve(import.meta.dirname,'..');
const exe=process.env.LOCAL_STUDIO_TEST_EXE||path.join(root,'src-tauri/target/release/local-studio.exe');
const artifacts=path.join(root,'.artifacts',`gif-studio32-${Date.now()}`);
await fs.mkdir(artifacts,{recursive:true});
const isolatedExe=path.join(artifacts,'Local Studio.exe');
await fs.copyFile(exe,isolatedExe);
await fs.writeFile(path.join(artifacts,'portable.marker'),'isolated GIF studio check\n');
const server=net.createServer();
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const port=server.address().port;
await new Promise(resolve=>server.close(resolve));
let browser,child;
let output='';
const pause=ms=>new Promise(resolve=>setTimeout(resolve,ms));
try {
  child=spawn(isolatedExe,[],{cwd:root,windowsHide:true,stdio:['ignore','pipe','pipe'],env:{
    ...process.env,
    LOCAL_STUDIO_CONFIG_DIR:path.join(artifacts,'config'),
    WEBVIEW2_USER_DATA_FOLDER:path.join(artifacts,'webview'),
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:`--remote-debugging-port=${port} --disable-background-timer-throttling`,
  }});
  child.stdout.on('data',chunk=>{output+=chunk;});
  child.stderr.on('data',chunk=>{output+=chunk;});
  for(let attempt=0;attempt<80&&!browser;attempt++){
    if(child.exitCode!==null)throw new Error(`Native app exited: ${output}`);
    try{browser=await chromium.connectOverCDP(`http://127.0.0.1:${port}`,{timeout:1000});}
    catch{await pause(500);}
  }
  assert.ok(browser,'WebView2 did not start');
  const page=browser.contexts()[0].pages()[0];
  await page.waitForFunction(()=>typeof window.__TAURI_INTERNALS__?.invoke==='function');
  const invoke=(command,args={})=>page.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
  await page.getByRole('button',{name:/Studio einrichten|Set up studio/}).click();
  const snapshot=await invoke('bootstrap');
  const source=path.join(snapshot.paths.gallery,'gif-studio-source.png');
  const png=await page.evaluate(()=>{
    const canvas=document.createElement('canvas');canvas.width=64;canvas.height=48;
    const context=canvas.getContext('2d');context.fillStyle='#e23994';context.fillRect(0,0,64,48);
    return canvas.toDataURL('image/png').split(',')[1];
  });
  await fs.writeFile(source,Buffer.from(png,'base64'));
  await page.getByRole('button',{name:/^(?:Create|Studio)$/}).first().click();
  const tabs=page.getByRole('navigation',{name:/Studio-Werkzeuge|Studio tools/});
  await tabs.getByRole('button',{name:/GIF erstellen|^GIF$/}).click();
  const workspace=page.locator('.gif-workspace');
  await workspace.waitFor();
  assert.equal(await workspace.locator('.gif-config').count(),1);
  assert.equal(await workspace.locator('.gif-canvas').count(),1);
  assert.equal(await workspace.locator('.studio-gallery').count(),1);
  const engines=workspace.getByRole('group',{name:/KI-Engine|AI engine/});
  await engines.getByRole('button',{name:'Vulkan'}).click();
  const vulkanFiles=workspace.locator('.gif-vulkan-files');
  await vulkanFiles.waitFor();
  assert.equal(await vulkanFiles.getByRole('button').count(),3);
  await engines.getByRole('button',{name:'ComfyUI'}).click();
  const preview=await invoke('gif_source_preview',{path:source});
  assert.match(preview,/^data:image\/png;base64,/);
  await workspace.getByRole('button',{name:/gif-studio-source.png/}).click();
  await workspace.locator('.gif-canvas-viewport img').waitFor();
  await page.screenshot({path:path.join(artifacts,'gif-studio-ai.png')});
  await workspace.getByRole('button',{name:/Aus Bildern|From images/}).click();
  await workspace.getByRole('button',{name:/gif-studio-source.png/}).click();
  await workspace.locator('.gif-frame-list li').first().waitFor();
  assert.equal(await workspace.locator('.gif-frame-list li').count(),1);
  await workspace.getByRole('button',{name:/GIF aus Bildern erstellen|Create GIF from images/}).click();
  await workspace.getByText(/Ungespeichertes GIF|Unsaved GIF/).waitFor();
  const pending=await invoke('gif_pending_list');
  assert.equal(pending.length,1);
  const outputName=`Local-Studio-${pending[0].id}.gif`;
  const result=path.join(snapshot.paths.gallery,outputName);
  await assert.rejects(()=>fs.stat(result));
  await workspace.getByRole('button',{name:/In Galerie speichern|Save to gallery/}).click();
  await workspace.getByText(outputName,{exact:false}).first().waitFor();
  for(let attempt=0;attempt<50&&(await invoke('gif_pending_list')).length;attempt++)await pause(100);
  assert.equal((await invoke('gif_pending_list')).length,0);
  assert.equal((await fs.readFile(result)).subarray(0,6).toString(),'GIF89a');
  const savedTile=workspace.locator('.studio-gallery-media-tile',{hasText:outputName});
  await savedTile.waitFor();await savedTile.click({button:'right'});
  await page.getByRole('menuitem',{name:/Löschen|Delete/}).click();
  await page.getByRole('button',{name:/In Papierkorb verschieben|Move to trash/}).click();
  await savedTile.waitFor({state:'detached'});
  await assert.rejects(()=>fs.stat(result));
  const separator=workspace.getByRole('separator',{name:/Breite der Einstellungen|Settings width/});
  await separator.focus();await separator.press('ArrowRight');
  assert.equal(await separator.getAttribute('aria-valuenow'),'430');
  await page.screenshot({path:path.join(artifacts,'gif-studio-result.png')});
  await invoke('mark_clean_exit');
  console.log(`PASS native GIF layout, temporary result, UUID save, right-click trash and keyboard resize; ${artifacts}`);
} finally {
  if(child&&child.exitCode===null)child.kill();
  if(browser)await browser.close().catch(()=>{});
  await fs.writeFile(path.join(artifacts,'native-output.txt'),output).catch(()=>{});
}
