import assert from 'node:assert/strict';
import path from 'node:path';

export async function checkCivitai31({getPage,invoke,artifactRoot,record}) {
  const page=getPage();
  const comfy=await invoke('comfy_status');
  assert.equal(typeof comfy.installed,'boolean');
  assert.equal(typeof comfy.running,'boolean');
  record('ComfyUI command is permitted for the local main window',comfy.installed?'installed':'not configured');

  const result=await invoke('civitai_model_search',{
    query:'Realistic Vision',kind:'Checkpoint',baseModel:'',sort:'Most Downloaded',
    period:'AllTime',cursor:'',includeNsfw:false,
  });
  assert.ok(result.models.length>0,'Expected a public Civitai search result');
  assert.ok(result.models.every(model=>model.nsfw===false));
  const detail=await invoke('civitai_model_detail',{id:result.models[0].id});
  assert.equal(detail.model.id,result.models[0].id);
  assert.ok(detail.versions.length>0);
  assert.ok(detail.versions.some(version=>version.files.some(file=>file.sha256&&file.sizeBytes>0)));
  record('Live Civitai search and model detail through typed desktop IPC',`${detail.model.name} (${detail.versions.length} versions)`);

  const setupLater=page.getByRole('button',{name:/Set up later|Später einrichten/});
  if(await setupLater.isVisible().catch(()=>false)) await setupLater.click();
  await page.getByRole('button',{name:/Models|Modelle/,exact:true}).first().click();
  const sourceNav=page.getByRole('navigation',{name:/Model sources|Modellquellen/});
  await sourceNav.waitFor();
  assert.equal(await sourceNav.getByRole('button').count(),3);
  await sourceNav.getByRole('button',{name:'Civitai',exact:true}).click();
  await page.getByRole('heading',{name:'Civitai',exact:true}).waitFor();
  await page.getByRole('button',{name:/Sign in with Civitai|Mit Civitai anmelden/,exact:true}).click();
  await page.locator('.civitai-browser-surface').waitFor({state:'visible'});
  for(let attempt=0;attempt<40;attempt++){
    const browser=await invoke('civitai_browser_state').catch(()=>null);
    if(browser?.visible&&browser.url.includes('civitai.com'))break;
    await new Promise(resolve=>setTimeout(resolve,250));
  }
  const browser=await invoke('civitai_browser_state');
  assert.equal(browser.visible,true);
  assert.match(browser.url,/^https:\/\/(?:auth\.)?civitai\.com\//);
  await page.getByRole('button',{name:/Close browser|Browser schließen/,exact:true}).click();
  record('Civitai account opens in the isolated integrated website profile');
  await page.getByLabel('Search Civitai',{exact:true}).fill('Realistic Vision');
  await page.getByRole('button',{name:'Checkpoints',exact:true}).click();
  await page.getByRole('button',{name:'Search',exact:true}).click();
  await page.locator('.hub-results .hub-model').first().waitFor({state:'visible',timeout:30000});
  assert.ok(await page.locator('.hub-results .hub-model').count()>0);
  await page.screenshot({path:path.join(artifactRoot,'civitai-model-search.png')});
  record('Civitai source, filters and results render in the model library');
}
