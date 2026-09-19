import {build} from 'vite';
import {chromium,expect} from '@playwright/test';
import {mkdir,writeFile} from 'node:fs/promises';
await build({configFile:false,define:{'process.env.NODE_ENV':JSON.stringify('production')},build:{outDir:'.artifacts/update-dialog-fixture',emptyOutDir:false,lib:{entry:'scripts/update-dialog-fixture.jsx',name:'UpdateFixture',formats:['iife'],fileName:()=> 'fixture.js'}}});
let browser;
const passed=[];
try{
 browser=await chromium.launch({channel:'msedge',headless:true});
 console.log('Headless Edge ready');
 const page=await browser.newPage();
 page.on('pageerror',error=>console.error('Browser:',error));
 page.on('requestfailed',request=>console.error('Request failed:',request.url(),request.failure()));
 page.setDefaultTimeout(15000);
 await page.setContent('<!doctype html><html><body><div id="root"></div></body></html>');
 await page.addScriptTag({path:'.artifacts/update-dialog-fixture/fixture.js'});
 await page.clock.install();
 console.log('Fixture page ready');
 async function mount(options={}){
  await page.evaluate(options=>window.UpdateFixture.mount(options),options);
  await expect(page.getByRole('heading',{name:options.de===false?'Update available':'Update verfügbar',exact:true})).toBeVisible();
 }
 const installed=()=>page.evaluate(()=>window.updateFixture.installed);
 await mount();
 await expect(page.getByRole('dialog').getByRole('button')).toHaveCount(2);
 await page.getByRole('button',{name:'Abbrechen',exact:true}).click();
 await expect(page.getByRole('dialog')).toHaveCount(0);
 expect(await page.evaluate(()=>window.updateFixture.calls.includes('update_download'))).toBe(false);
 passed.push('Cancel closes without downloading or installing');

 await mount();
 await page.getByRole('button',{name:'Update installieren',exact:true}).dblclick();
 await page.clock.runFor(500);
 await expect(page.getByRole('progressbar')).toBeVisible();
 expect(await installed()).toBe(0);
 expect(await page.evaluate(()=>window.updateFixture.calls.filter(c=>c==='update_download').length)).toBe(1);
 await page.evaluate(()=>window.updateFixture.finish());
 await expect.poll(installed).toBe(1);
 passed.push('One click downloads once; install callback only after verified ready status');

 await mount();
 await page.getByRole('button',{name:'Update installieren',exact:true}).click();
 await page.clock.runFor(500);
 await expect(page.getByRole('progressbar')).toBeVisible();
 await page.getByRole('button',{name:'Abbrechen',exact:true}).click();
 await page.evaluate(()=>window.updateFixture.finish());
 expect(await installed()).toBe(0);
 expect(await page.evaluate(()=>window.updateFixture.calls.includes('update_cancel'))).toBe(true);
 passed.push('Cancel during download blocks installation even if a ready response arrives later');

 await mount();
 await page.getByRole('button',{name:'Update installieren',exact:true}).click();
 await page.evaluate(()=>window.updateFixture.finish('error'));
 await expect(page.getByRole('alert')).toContainText('stimmt nicht');
 expect(await installed()).toBe(0);
 passed.push('Backend integrity error is shown and never installs');

 await mount({portable:true,de:false});
 await expect(page.getByRole('button',{name:'Install update',exact:true})).toHaveCount(0);
 await page.getByRole('button',{name:'Open download on GitHub',exact:true}).click();
 expect(await page.evaluate(()=>window.updateFixture.calls.includes('update_open_download'))).toBe(true);
 expect(await page.evaluate(()=>window.updateFixture.calls.includes('update_download'))).toBe(false);
 await page.keyboard.press('Escape');
 await expect(page.getByRole('dialog')).toHaveCount(0);
 passed.push('English portable dialog opens GitHub only; Escape dismisses');
 await mkdir('.artifacts',{recursive:true});
 await writeFile('.artifacts/update-popup-browser.json',JSON.stringify({passed,scope:'React UI in headless Edge with mocked update IPC; no installer execution'},null,2));
 console.log('PASS',passed.length,'update dialog browser scenarios');
}catch(error){console.error(error);throw error;}finally{await browser?.close();}
