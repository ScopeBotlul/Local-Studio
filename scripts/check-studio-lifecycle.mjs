import assert from 'node:assert/strict';
import { promises as fs } from 'node:fs';
import path from 'node:path';
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
async function until(fn, timeout = 20000) { const end = Date.now() + timeout; while (Date.now() < end) { const value = await fn(); if (value) return value; await pause(100); } throw new Error('Studio lifecycle condition timed out'); }
async function closeRequest(page, invoke) { const label = await page.evaluate(() => window.__TAURI_INTERNALS__.metadata.currentWindow.label); await invoke('plugin:window|close', { label }); await page.getByRole('dialog', { name: 'Close Local Studio?' }).waitFor(); }

export async function checkStudioLifecycle({ getPage, invoke, stop, launch, artifactRoot, record }) {
  let page = getPage();
  const recovered = await invoke('image_workspace'); assert.equal(recovered.recoveryAvailable, true); assert.ok(recovered.unsaved >= 2);
  await page.getByRole('button', { name: 'Restore workspace', exact: true }).click();
  await until(() => page.getByLabel('Image prompt', { exact: true }).inputValue().then(v => v.includes('Changed form')));
  assert.equal((await invoke('image_workspace')).recoveryAvailable, false);
  record('Native crash recovery offers and restores prompt, parameters and actual unsaved generated images');
  await page.locator('#image-model-library option').filter({ hasText: 'DreamShaperXL_Turbo_V2' }).waitFor({ state: 'attached' });
  const models = await page.locator('#image-model-library option').evaluateAll(options => options.map(o => ({ value: o.value, label: o.textContent })));
  const a = models.find(o => /Juggernaut-XL_v9_RunDiffusionPhoto_v2/.test(o.label));
  const b = models.find(o => /DreamShaperXL_Turbo_V2/.test(o.label)); assert.ok(a && b); assert.match(a.label, /GB/); assert.match(b.label, /GB/);
  await page.locator('#image-model-library').selectOption(a.value);
  await page.getByLabel('Image steps', { exact: true }).fill('9');
  await page.getByLabel('Image prompt', { exact: true }).fill('Studio lifecycle recovery: mountains reflected in a lake');
  await page.locator('#image-model-library').selectOption(b.value);
  await page.getByLabel('Image steps', { exact: true }).fill('4');
  await page.getByLabel('Image guidance', { exact: true }).fill('2');
  await page.locator('#image-model-library').selectOption(a.value);
  assert.equal(await page.getByLabel('Image steps', { exact: true }).inputValue(), '9');
  assert.equal(await page.getByLabel('Image guidance', { exact: true }).inputValue(), '5');
  const prompt = await page.getByLabel('Image prompt', { exact: true }).inputValue();
  await page.getByRole('button', { name: 'Jobs', exact: true }).first().click();
  const jobs = await invoke('image_jobs'); const cancelled = jobs.find(j => j.status === 'cancelled'); assert.ok(cancelled);
  await page.locator(`[data-image-job-id="${cancelled.id}"]`).waitFor();
  const completed = jobs.find(j => j.status === 'completed');
  await page.locator(`[data-image-job-id="${completed.id}"]`).getByRole('button', { name: 'Open in Studio', exact: true }).click();
  await page.getByAltText('Locally generated image', { exact: true }).waitFor();
  assert.equal(await page.getByLabel('Image steps', { exact: true }).inputValue(), '9');
  assert.equal(await page.getByLabel('Image prompt', { exact: true }).inputValue(), prompt);
  await page.screenshot({ path: path.join(artifactRoot, 'studio-model-dropdown.png') });
  record('Model dropdown shows real library entries with GB; A/B/A restores model parameters, keeps prompt and survives navigation; Jobs opens real image result');
  const drafts = jobs.filter(j => j.status === 'completed' && !j.savedPath && !j.discarded);
  await closeRequest(page, invoke); await page.getByRole('dialog').getByRole('button', { name: 'Cancel', exact: true }).click();
  assert.equal(page.isClosed(), false); for (const job of drafts) await fs.access(job.output);
  record('Consolidated close dialog Cancel retains all unsaved image files and keeps the native app usable');

  // A native child WebView must not cover the HTML modal; cancelling restores the browser.
  await page.getByRole('button', { name: 'Hugging Face', exact: true }).first().click();
  await page.getByRole('button', { name: 'Open website in Studio', exact: true }).click();
  await until(async () => (await invoke('hf_browser_state')).visible);
  await closeRequest(page, invoke); await until(async () => !(await invoke('hf_browser_state')).visible);
  await page.getByRole('dialog').getByRole('button', { name: 'Cancel', exact: true }).click();
  await until(async () => (await invoke('hf_browser_state')).visible);
  record('Exit modal hides embedded HF WebView and Cancel restores its existing browser session');

  await page.getByRole('button', { name: 'Settings', exact: true }).first().click();
  await page.locator('#restore-session').selectOption('true');
  await page.getByRole('button', { name: 'Save changes', exact: true }).click();
  await until(async () => (await invoke('bootstrap')).settings.restoreSession);
  await closeRequest(page, invoke); const kept = page.waitForEvent('close', { timeout: 30000 });
  await page.getByRole('button', { name: 'Keep workspace and close', exact: true }).click(); await kept;
  await stop(false); await launch(); page = getPage();
  const keptState = await invoke('image_workspace'); assert.equal(keptState.unsaved, drafts.length); assert.equal(keptState.recoveryAvailable, false); assert.equal(keptState.workspace.request.prompt, prompt);
  for (const job of drafts) { await fs.access(job.output); assert.equal((await invoke('image_jobs')).find(j => j.id === job.id).savedPath, null); }
  record('Enabled session restore can retain unsaved actual images through a clean exit without publishing them to the gallery');
  await closeRequest(page, invoke); const closed = page.waitForEvent('close', { timeout: 30000 });
  await page.getByRole('button', { name: 'Save images and close', exact: true }).click(); await closed;
  await stop(false); await launch(); page = getPage();
  let after = await invoke('image_jobs');
  for (const job of drafts) { const saved = after.find(j => j.id === job.id); assert.ok(saved.savedPath); await fs.access(saved.savedPath); }
  let state = await invoke('image_workspace'); assert.equal(state.unsaved, 0); assert.equal(state.recoveryAvailable, false); assert.equal(state.workspace.request.prompt, prompt); assert.equal(state.workspace.request.steps, 9);
  record('Save images and close saves all actual drafts with metadata; clean restart restores image inputs when enabled');
  const savedPaths = after.filter(j => j.savedPath).map(j => j.savedPath);
  await page.getByRole('button', { name: 'Studio', exact: true }).first().click();
  await until(() => page.getByLabel('Image prompt', { exact: true }).inputValue().then(v => v === prompt));
  const generated = await invoke('image_generate', { request: { ...state.workspace.request, steps: 2 } });
  const done = await until(async () => { const j = (await invoke('image_jobs')).find(j => j.id === generated.id); if (j?.status === 'failed') throw new Error(j.error); return j?.status === 'completed' ? j : false; }, 180000);
  await page.getByRole('button', { name: 'Settings', exact: true }).first().click();
  await page.locator('#restore-session').selectOption('false'); await page.getByRole('button', { name: 'Save changes', exact: true }).click();
  await until(async () => !(await invoke('bootstrap')).settings.restoreSession);
  await closeRequest(page, invoke); const discarded = page.waitForEvent('close', { timeout: 30000 });
  await page.getByRole('button', { name: 'Discard images and close', exact: true }).click(); await discarded;
  await stop(false); await launch(); page = getPage();
  await assert.rejects(fs.access(done.output)); for (const filename of savedPaths) await fs.access(filename);
  assert.equal((await invoke('image_jobs')).find(j => j.id === done.id).discarded, true);
  state = await invoke('image_workspace'); assert.equal(state.workspace.request, null); assert.equal(state.unsaved, 0); assert.equal(state.recoveryAvailable, false);
  await page.getByRole('button', { name: 'Studio', exact: true }).first().click();
  await page.locator('#image-model-library').selectOption(a.value);
  assert.equal(await page.getByLabel('Image steps', { exact: true }).inputValue(), '9');
  assert.equal(await page.getByLabel('Image prompt', { exact: true }).inputValue(), '');
  await page.getByRole('button', { name: 'Gallery', exact: true }).first().click(); await page.getByAltText('Gallery image', { exact: true }).waitFor();
  await page.screenshot({ path: path.join(artifactRoot, 'studio-gallery-after-discard.png') });
  record('Discard images and close deletes only the unsaved real PNG, preserves all saved gallery images and starts empty while retaining per-model parameters');
  await page.getByRole('button', { name: 'Studio', exact: true }).first().click();
  await page.getByLabel('Image prompt', { exact: true }).fill('Only a draft prompt; no generation');
  await closeRequest(page, invoke);
  await page.getByRole('dialog').getByText('Current Studio inputs will be cleared on the next start. Prompts of completed jobs remain in their metadata.', { exact: true }).waitFor();
  await page.getByRole('dialog').getByRole('button', { name: 'Cancel', exact: true }).click();
  assert.equal(await page.getByLabel('Image prompt', { exact: true }).inputValue(), 'Only a draft prompt; no generation');
  await page.getByLabel('Image prompt', { exact: true }).fill('');
  await until(async () => (await invoke('image_workspace')).workspace.request.prompt === '');
  record('A prompt-only workspace also receives the consolidated close warning; Cancel retains the input');
}
