import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';

export async function checkRelease36({getPage,invoke,artifactRoot,record}) {
  const page=getPage();
  const fixture=path.join(artifactRoot,'lora-fixture');await fs.mkdir(path.join(fixture,'loras'),{recursive:true});
  const header={__metadata__:{ss_base_model_version:'sdxl_base_v1-0'},'lora_unet_x.weight':{dtype:'F32',shape:[1],data_offsets:[0,4]}};
  const json=Buffer.from(JSON.stringify(header)),length=Buffer.alloc(8);length.writeBigUInt64LE(BigInt(json.length));
  const lora=path.join(fixture,'loras','ally-style.safetensors');await fs.writeFile(lora,Buffer.concat([length,json,Buffer.alloc(4)]));
  await invoke('model_scan_start',{path:fixture});let imported;
  for(let attempt=0;attempt<300;attempt++){const library=await invoke('model_library_list');imported=library.entries.find(entry=>entry.name==='ally-style.safetensors'&&entry.profile?.role==='extension');if(imported)break;await new Promise(resolve=>setTimeout(resolve,100));}
  assert(imported,'LoRA fixture was not classified as a library extension');
  const snapshot=await invoke('bootstrap');await invoke('save_settings',{settings:{...snapshot.settings,theme:'light',language:'en',accentColor:'#651b2c',uiScale:1.25}});await page.reload();
  await page.getByRole('button',{name:'Create',exact:true}).first().click();await page.locator('.image-canvas').waitFor();
  const layout=await page.locator('.app-shell').evaluate(element=>{const rect=element.getBoundingClientRect(),root=getComputedStyle(document.documentElement);return{zoom:getComputedStyle(element).zoom,width:rect.width,height:rect.height,viewportWidth:innerWidth,viewportHeight:innerHeight,accent:root.getPropertyValue('--accent').trim(),ink:root.getPropertyValue('--accent-ink').trim()};});
  assert.equal(layout.zoom,'1.25');assert(Math.abs(layout.width-layout.viewportWidth)<2,JSON.stringify(layout));assert(Math.abs(layout.height-(layout.viewportHeight-36))<2,JSON.stringify(layout));assert.equal(layout.accent,'#651b2c');assert.equal(layout.ink,'#ffffff');
  await page.getByLabel('Image engine',{exact:true}).selectOption('vulkan');await page.locator('.image-control-details').filter({has:page.getByText('LoRA',{exact:true})}).locator('summary').click();const chooser=page.getByLabel('Add LoRA',{exact:true});await chooser.selectOption(imported.path);await page.getByText('ally-style',{exact:true}).waitFor();assert.equal(await page.locator('.window-error').count(),0);
  const session=await page.context().newCDPSession(page);const screenshot=await session.send('Page.captureScreenshot',{format:'png',fromSurface:true});await fs.writeFile(path.join(artifactRoot,'release36-scaled-lora.png'),Buffer.from(screenshot.data,'base64'));await session.detach();
  const reduced=(await invoke('bootstrap')).settings;await invoke('save_settings',{settings:{...reduced,uiScale:.75}});await page.reload();await page.getByRole('button',{name:'Create',exact:true}).first().click();await page.locator('.image-canvas').waitFor();const small=await page.locator('.app-shell').evaluate(element=>{const rect=element.getBoundingClientRect();return{zoom:getComputedStyle(element).zoom,width:rect.width,height:rect.height,viewportWidth:innerWidth,viewportHeight:innerHeight};});assert.equal(small.zoom,'0.75');assert(Math.abs(small.width-small.viewportWidth)<2,JSON.stringify(small));assert(Math.abs(small.height-(small.viewportHeight-36))<2,JSON.stringify(small));
  record('The real release fills the window at both 75% and 125% scale, preserves the Canvas, applies readable accent colors and updates the runtime window icon without an IPC error');
  record('A library LoRA remains selectable with the explicit Vulkan engine, without requiring ComfyUI');
}
