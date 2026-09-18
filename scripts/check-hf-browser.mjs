import assert from 'node:assert/strict';
import path from 'node:path';

const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
async function until(work, message) {
  for (let i = 0; i < 100; i++) { const result = await work(); if (result) return result; await pause(250); }
  throw new Error(message);
}
async function open(page, browser) {
  await page.getByRole('button', { name: 'Hugging Face', exact: true }).first().click();
  await page.getByRole('button', { name: 'Website in Studio', exact: true }).click();
  return until(() => browser.contexts().flatMap(context => context.pages()).find(candidate => candidate !== page && candidate.url().startsWith('https://huggingface.co/')), 'HF child WebView did not appear');
}

export async function checkHfBrowser(page, browser, artifactRoot, invoke, record) {
  const website = await open(page, browser);
  await website.waitForLoadState('domcontentloaded');
  await website.locator('body').waitFor();
  assert.ok((await website.locator('body').innerText()).length > 100);
  assert.equal((await invoke('hf_browser_state')).visible, true);
  await website.screenshot({ path: path.join(artifactRoot, 'hf-embedded-website.png') });
  await page.screenshot({ path: path.join(artifactRoot, 'hf-browser-controls.png') });
  record('Real Hugging Face website renders in separate native child WebView');

  const denied = await website.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    if (!invoke) return { bridgeAbsent: true };
    const result = {};
    for (const command of ['gallery_compare','gallery_file_action','gallery_trash_list','gallery_trash_action','gallery_trash_detail','gallery_annotate_batch', 'gallery_thumbnail', 'gallery_thumbnail_clear', 'gallery_annotate', 'gallery_list', 'gallery_detail', 'gallery_import', 'gallery_create_folder', 'gallery_open_folder', 'bootstrap', 'hf_status', 'hf_browser_state', 'list_jobs', 'download_list', 'hf_model_size', 'model_library_list', 'model_scan_start', 'model_scan_full', 'model_scan_quick', 'image_workspace', 'image_workspace_save', 'image_recover', 'image_discard', 'image_probe', 'image_generate', 'image_resume', 'image_cancel', 'image_jobs', 'image_output', 'image_save', 'model_library_forget']) {
      try { await invoke(command); result[command] = 'ALLOWED'; }
      catch (error) { result[command] = String(error); }
    }
    return result;
  });
  assert.ok(denied.bridgeAbsent || Object.values(denied).every(value => value !== 'ALLOWED'));
  record('Website cannot invoke privileged local app commands', JSON.stringify(denied));

  const address = page.getByRole('textbox', { name: 'Hugging Face address' });
  await address.fill('https://example.org');
  await address.press('Enter');
  await page.getByText('This address cannot be opened here.', { exact: false }).waitFor();
  assert.ok(website.url().startsWith('https://huggingface.co/'));
  await page.getByRole('button', { name: 'Dismiss notice' }).click();
  await address.fill('https://huggingface.co/openai-community/gpt2');
  await address.press('Enter');
  await until(async () => (await invoke('hf_browser_state')).modelRepo === 'openai-community/gpt2', 'Model URL was not recognized');
  await page.getByRole('button', { name: 'Open in Local Studio', exact: true }).click();
  await until(async () => !(await invoke('hf_browser_state')).visible, 'Child WebView was not hidden');
  await page.getByRole('heading', { name: 'openai-community/gpt2', exact: true }).waitFor({ timeout: 30000 });
  record('HF-only address validation and model-page handoff to real repository details');

  const reopened = await open(page, browser);
  await invoke('hf_browser_hide', { owner: crypto.randomUUID() });
  assert.equal((await invoke('hf_browser_state')).visible, true, 'Stale component cleanup cannot hide active browser');
  assert.equal(reopened, website);
  assert.ok(website.url().includes('openai-community/gpt2'));
  await page.keyboard.press('Control+=');
  await pause(1000);
  const rect = await page.getByTestId('hf-browser-surface').boundingBox();
  const inner = await website.evaluate(() => ({ width: innerWidth, height: innerHeight }));
  assert.ok(Math.abs(rect.width - inner.width) < 5, JSON.stringify({ rect, inner }));
  assert.ok(Math.abs(rect.height - inner.height) < 5, JSON.stringify({ rect, inner }));
  await page.keyboard.press('Control+0');
  await pause(700);
  record('Browser survives tab switches and follows scaled native view bounds');

  // Synthetic non-auth cookie in the isolated test profile; never inspect user cookies.
  await website.evaluate(() => { document.cookie = 'local_studio_persistence_test=present; Path=/; Max-Age=3600; Secure; SameSite=Lax'; });
  assert.ok(await website.evaluate(() => document.cookie.split('; ').includes('local_studio_persistence_test=present')));
  await pause(2000);
}

export async function checkHfBrowserRestart(page, browser, invoke, record) {
  const website = await open(page, browser);
  await website.waitForLoadState('domcontentloaded');
  assert.ok(await website.evaluate(() => document.cookie.split('; ').includes('local_studio_persistence_test=present')), 'Synthetic cookie should survive restart');
  await website.evaluate(() => { document.cookie = 'local_studio_persistence_test=; Path=/; Max-Age=0; Secure; SameSite=Lax'; });
  assert.equal((await invoke('hf_status')).account, null);
  record('Website cookie persists across app restart independently of app authentication', 'Synthetic cookie only; no real website login performed');
}
