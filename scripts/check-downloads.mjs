import assert from 'node:assert/strict';
import { promises as fs } from 'node:fs';
import { createHash } from 'node:crypto';
import path from 'node:path';
const repo = 'hf-internal-testing/tiny-random-gpt2';
const expected = '8111d5afb0715dbf5a31396d31432cb56370ba23f6650a035ea0fc8a20b4e500';
async function wait(invoke, id, states) {
  for (let i=0;i<600;i++) {
    const item=(await invoke('download_list')).find(j=>j.id===id);
    if(item && states.includes(item.status)) return item;
    await new Promise(resolve=>setTimeout(resolve,100));
  }
  throw new Error('Download did not reach expected state');
}
function owns(root, target) {
  const relative=path.relative(path.toNamespacedPath(root),target);
  assert.ok(relative && !relative.startsWith('..') && !path.isAbsolute(relative), 'Test writes remain inside isolated artifact folder');
}
export async function checkDownloads(page, artifactRoot, invoke, record) {
  // HF's internal test repository is public but excluded from search results.
  // Use the real website-to-model handoff rather than depending on its search index.
  await page.getByRole('button',{name:'Hugging Face',exact:true}).first().click();
  await page.getByRole('button',{name:'Website in Studio',exact:true}).click();
  const address=page.getByRole('textbox',{name:'Hugging Face address',exact:true});
  // Clicking waits for the button to become enabled after initial native WebView creation.
  await address.fill(`https://huggingface.co/${repo}`);await page.getByRole('button',{name:'Go',exact:true}).click();
  for(let i=0;i<100;i++){if((await invoke('hf_browser_state')).modelRepo===repo)break;await new Promise(r=>setTimeout(r,100));}
  assert.equal((await invoke('hf_browser_state')).modelRepo,repo);
  await page.getByRole('button',{name:'Open in Local Studio',exact:true}).click();
  await page.getByRole('checkbox',{name:'model.safetensors',exact:true}).waitFor({timeout:40000});
  assert.equal(await page.locator('.download-selection input:checked').count(),0);
  assert.equal(await page.getByRole('button',{name:'Prepare download',exact:true}).isDisabled(),true);
  await page.getByRole('checkbox',{name:'config.json',exact:true}).check();
  await page.getByRole('checkbox',{name:'model.safetensors',exact:true}).check();
  await page.getByRole('button',{name:'Prepare download',exact:true}).click();
  await page.getByTestId('download-plan').waitFor({timeout:40000});
  assert.ok((await page.getByTestId('download-plan').innerText()).includes('< 0.01 GB'));
  assert.equal((await invoke('download_list')).length,0,'Preparing a download must not start it');
  await page.getByRole('button',{name:'Download selection',exact:true}).scrollIntoViewIfNeeded();
  await page.screenshot({path:path.join(artifactRoot,'download-plan.png')});
  record('Explicit file selection and pinned download preview; no file or variant preselected');
  await page.getByRole('button',{name:'Download selection',exact:true}).click();
  await page.getByRole('heading',{name:'Downloads',exact:true}).waitFor();
  const first=(await invoke('download_list'))[0];assert.equal(first.repo,repo);assert.equal(first.totalBytes,454671);assert.match(first.revision,/^[a-f0-9]{40}$/);
  const completed=await wait(invoke,first.id,['completed','failed']);assert.equal(completed.status,'completed',completed.error);
  owns(artifactRoot,completed.destination);
  const weights=await fs.readFile(path.join(completed.destination,'model.safetensors'));
  assert.equal(weights.length,453864);assert.equal(createHash('sha256').update(weights).digest('hex'),expected);
  assert.equal(completed.files.find(f=>f.path==='model.safetensors').actualSha256,expected);
  assert.equal(JSON.parse(await fs.readFile(path.join(completed.destination,'config.json'),'utf8')).model_type,'gpt2');
  await assert.rejects(fs.access(path.join(completed.partialDirectory,'0.part')));
  record('Real HF model weights and config downloaded, source hashes verified, selection published locally',`${weights.length} weight bytes; SHA-256 ${expected}`);
  await page.screenshot({path:path.join(artifactRoot,'downloads-completed.png')});

  await page.getByRole('button',{name:'Models',exact:true}).first().click();
  await page.getByRole('button',{name:'Stored locally',exact:true}).click();
  await page.getByRole('heading',{name:repo,exact:true}).waitFor();
  await page.getByText('File selection stored. Runtime completeness and execution have not been tested.',{exact:true}).waitFor();
  assert.ok((await page.getByTestId('local-model-size').innerText()).includes('< 0.01 GB'));
  await page.screenshot({path:path.join(artifactRoot,'local-model-files.png')});
  // Only alter our own downloaded test copy; restore it after proving corruption detection.
  const target=path.join(completed.destination,'model.safetensors');owns(artifactRoot,target);
  const corrupt=Buffer.from(weights);corrupt[corrupt.length-1]^=1;await fs.writeFile(target,corrupt);
  await page.getByRole('button',{name:'Verify local files',exact:true}).click();
  const invalid=await wait(invoke,first.id,['invalid']);assert.equal(invalid.error,'download_hash');
  await fs.writeFile(target,weights);await invoke('download_action',{id:first.id,action:'verify',priority:null});
  assert.equal((await wait(invoke,first.id,['completed','invalid'])).status,'completed');
  record('Local file inventory does not claim runtime compatibility; recheck detects same-size corruption');

  await assert.rejects(invoke('download_plan',{repo:'../escape',revision:'main',files:['config.json']}),e=>String(e).endsWith('invalid_repo'));
  await assert.rejects(invoke('download_plan',{repo,revision:completed.revision,files:['../escape']}),e=>String(e).endsWith('download_unsafe_path'));
  await assert.rejects(invoke('download_start',{planId:first.id}),e=>String(e).endsWith('download_plan_expired'));
  const duplicate=await invoke('download_plan',{repo,revision:completed.revision,files:['config.json','model.safetensors']});
  await assert.rejects(invoke('download_start',{planId:duplicate.id}),e=>String(e).endsWith('download_duplicate'));
  record('Unsafe download paths, consumed previews and duplicate selections rejected');
  return first.id;
}
export async function checkDownloadRestart(page,invoke,record,id) {
  const item=(await invoke('download_list')).find(j=>j.id===id);assert.equal(item.status,'completed');
  assert.equal(createHash('sha256').update(await fs.readFile(path.join(item.destination,'model.safetensors'))).digest('hex'),expected);
  await page.getByRole('button',{name:'Models',exact:true}).first().click();await page.getByRole('button',{name:'Stored locally',exact:true}).click();await page.getByRole('heading',{name:repo,exact:true}).waitFor();
  record('Completed download and local file inventory survive app restart without redownloading');
}
