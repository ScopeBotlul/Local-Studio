import assert from 'node:assert/strict';
import { promises as fs } from 'node:fs';
import path from 'node:path';
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
async function until(check) { for (let i = 0; i < 150; i++) { if (await check()) return; await pause(100); } throw new Error('Foundation condition timed out'); }

export async function checkFoundations({ getPage, invoke, stop, launch, artifactRoot, record }) {
  let page = getPage(); const previous = await invoke('bootstrap');
  const original = path.join(previous.paths.gallery, 'original-keep.txt'); await fs.writeFile(original, 'original remains');
  const destinations = Object.fromEntries(Object.keys(previous.paths).map(key => [key, path.join(artifactRoot, 'custom-storage', key)]));
  await page.getByRole('button', { name: 'Settings', exact: true }).first().click();
  await page.getByText('Customize individual storage folders', { exact: true }).click();
  for (const [key, value] of Object.entries(destinations)) await page.locator(`#storage-${key}`).fill(value);
  await page.getByRole('button', { name: 'Save changes', exact: true }).click();
  await until(async () => !(await page.locator('#theme').isDisabled()));
  let snapshot = await invoke('bootstrap'); assert.deepEqual(snapshot.paths, destinations);
  for (const value of Object.values(destinations)) assert((await fs.stat(value)).isDirectory());
  assert.equal(await fs.readFile(original, 'utf8'), 'original remains');
  await assert.rejects(invoke('save_settings', { settings: { ...snapshot.settings, storageOverrides: { ...destinations, cache: path.join(destinations.gallery, 'nested-cache') } } }), /separate/);
  await assert.rejects(invoke('save_settings', { settings: { ...snapshot.settings, storageOverrides: { ...destinations, gallery: '..\\escape' } } }));
  const aliasRoot = path.join(artifactRoot, 'new-alias-folder');
  await assert.rejects(invoke('save_settings', { settings: { ...snapshot.settings, storageOverrides: { ...destinations, gallery: aliasRoot + '\\media', cache: aliasRoot + '\\.\\media' } } }), /separate/);
  assert.deepEqual((await invoke('bootstrap')).paths, destinations);
  record('All ten storage overrides apply through native Settings, create writable folders, reject overlaps/traversal and leave previous files untouched');

  const generate = page.locator('#shortcut-generate'); await generate.click(); await generate.press('Control+a');
  await page.getByText('This combination is already used by another action.', { exact: true }).waitFor();
  await generate.press('Control+z'); await page.getByText('This combination is reserved for Windows, text editing or projects.', { exact: true }).waitFor();
  await generate.press('Escape'); assert.equal(await generate.getAttribute('aria-pressed'), 'false');
  for (const [id, chord] of [['rename', 'Control+Shift+r'], ['delete', 'Control+d'], ['uiZoomIn', 'Control+Shift+i']]) { const button = page.locator(`#shortcut-${id}`); await button.click(); await button.press(chord); }
  await page.getByRole('button', { name: 'Save changes', exact: true }).click(); await until(async () => !(await page.locator('#theme').isDisabled()));
  snapshot = await invoke('bootstrap'); assert.equal(snapshot.settings.shortcuts.rename, 'Ctrl+Shift+R');
  await assert.rejects(invoke('save_settings', { settings: { ...snapshot.settings, shortcuts: { ...snapshot.settings.shortcuts, rename: 'Ctrl+D' } } }), /conflict/);
  await page.locator('h1').click(); await page.keyboard.press('Control+='); await pause(350); assert.equal((await invoke('bootstrap')).settings.uiScale, snapshot.settings.uiScale);
  await page.keyboard.press('Control+Shift+i'); await until(async () => (await invoke('bootstrap')).settings.uiScale === snapshot.settings.uiScale + .05);
  await page.keyboard.press('Control+0'); await until(async () => (await invoke('bootstrap')).settings.uiScale === 1);
  await page.locator('.shortcut-settings').screenshot({ path: path.join(artifactRoot, 'settings-shortcuts.png') });
  await page.locator('.storage-editor').screenshot({ path: path.join(artifactRoot, 'settings-storage.png') });
  await stop(true); await launch(); page = getPage();
  assert.deepEqual((await invoke('bootstrap')).paths, destinations); assert.equal((await invoke('bootstrap')).settings.shortcuts.rename, 'Ctrl+Shift+R');
  record('Native shortcut recording detects conflicts/reserved text keys; remapped UI zoom replaces old binding, persists with storage overrides across restart');

  const bytes = await page.evaluate(() => { const canvas = document.createElement('canvas'); canvas.width = 800; canvas.height = 1200; const ctx = canvas.getContext('2d'); ctx.fillStyle = '#614a88'; ctx.fillRect(0, 0, 800, 1200); ctx.fillStyle = '#ffffff'; ctx.fillRect(50, 50, 400, 600); return canvas.toDataURL('image/png').split(',')[1]; });
  await fs.writeFile(path.join(destinations.gallery, 'foundation-portrait.png'), Buffer.from(bytes, 'base64'));
  await page.getByRole('button', { name: 'Gallery', exact: true }).first().click();
  await until(async () => await page.locator('.gallery-file').count() === 1);
  await page.locator('.gallery-file').first().click(); await page.keyboard.press('F2'); assert.equal(await page.getByRole('dialog').count(), 0);
  await page.keyboard.press('Control+Shift+r'); await page.getByRole('dialog', { name: 'Rename file', exact: true }).waitFor(); await page.keyboard.press('Escape');
  await until(() => page.getByRole('button', { name: 'Select all on this page', exact: true }).isEnabled());
  await page.locator('.gallery-file').first().focus(); await page.keyboard.press('Delete'); assert.equal(await page.getByRole('dialog').count(), 0);
  await page.keyboard.press('Control+d'); await page.getByRole('dialog', { name: 'Move to trash?', exact: true }).waitFor(); await page.keyboard.press('Escape');
  await until(() => page.getByRole('button', { name: 'Select all on this page', exact: true }).isEnabled());
  const search = page.getByLabel('Search files', { exact: true }); await search.fill('foundation'); await search.press('Control+Shift+r'); assert.equal(await page.getByRole('dialog').count(), 0);
  const viewport = page.getByLabel('Media preview', { exact: true }); await viewport.focus(); await page.keyboard.press('1');
  await until(() => page.locator('.gallery-image-scroll img').evaluate(img => img.complete && img.naturalWidth === 800 && Math.abs(img.getBoundingClientRect().width - 800) < 2));
  for(let i=0;i<5;i++)await page.keyboard.press('Control+Shift+i');
  await until(async () => (await invoke('bootstrap')).settings.uiScale === 1.25);
  await until(() => page.locator('.gallery-image-scroll img').evaluate(img => Math.abs(img.getBoundingClientRect().width - 800) < 2));
  await page.keyboard.press('Control+0');await until(async () => (await invoke('bootstrap')).settings.uiScale === 1);
  await viewport.focus(); await page.keyboard.press('f');
  await until(() => page.locator('.gallery-image-scroll').evaluate(el => { const img = el.querySelector('img').getBoundingClientRect(), box = el.getBoundingClientRect(); return img.width <= box.width + 1 && img.height <= box.height + 1; }));
  await page.screenshot({ path: path.join(artifactRoot, 'gallery-portrait-fit.png') });
  record('Remapped gallery rename/trash retain confirmation, old keys no longer trigger actions, text input remains untouched; portrait fits entirely and 100% uses actual pixel dimensions');
  assert.deepEqual(await fs.readFile(path.join(destinations.gallery, 'foundation-portrait.png')), Buffer.from(bytes, 'base64'));
  await page.getByRole('button', { name: 'Settings', exact: true }).first().click(); await page.getByRole('button', { name: 'Restore default shortcuts', exact: true }).click(); await page.getByRole('button', { name: 'Save changes', exact: true }).click(); await until(async () => !(await page.locator('#theme').isDisabled()));
  assert.deepEqual((await invoke('bootstrap')).settings.shortcuts, previous.settings.shortcuts);
  record('Default shortcuts can be restored through Settings without changing custom storage or original media');
}
