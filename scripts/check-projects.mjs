import assert from 'node:assert/strict';
import { promises as fs } from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
const pause = ms => new Promise(r => setTimeout(r, ms));
async function until(check) { for (let i = 0; i < 200; i++) { if (await check()) return; await pause(100); } throw new Error('Project condition timed out'); }
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
export async function checkProjects({ getPage, invoke, stop, launch, artifactRoot, record }) {
  let page = getPage();
  const source = path.join(artifactRoot, 'project-fixtures'); await fs.mkdir(source);
  const target = path.join(source, 'Arbeitsstand ü.localstudio');
  // Dialog return values are controlled; all UI actions, IPC, archive IO and playback are real.
  async function dialogs() { await page.evaluate(() => {
    const original = window.fetch;
    window.projectDialog = { open: null, save: null, confirm: true };
    window.fetch = (input, options) => {
      const url = new URL(typeof input === 'string' ? input : input.url);
      const command = decodeURIComponent(url.pathname).slice(1);
      if (url.hostname === 'ipc.localhost' && ['plugin:dialog|open', 'plugin:dialog|save', 'plugin:dialog|message'].includes(command)) {
        const value = command.endsWith('|message') ? (window.projectDialog.confirm ? 'Ok' : 'Cancel') : command.endsWith('|open') ? window.projectDialog.open : window.projectDialog.save;
        return Promise.resolve(new Response(JSON.stringify(value), { headers: { 'Content-Type': 'application/json', 'Tauri-Response': 'ok' } }));
      }
      return original.call(window, input, options);
    };
  }); }
  await dialogs();
  const encoded = await page.evaluate(async () => {
    const c = document.createElement('canvas'); c.width = 320; c.height = 180; const ctx = c.getContext('2d'); ctx.fillStyle = '#6550a8'; ctx.fillRect(0, 0, 320, 180);
    const png = c.toDataURL('image/png').split(',')[1];
    const stream = c.captureStream(10), chunks = []; const recorder = new MediaRecorder(stream, { mimeType: 'video/webm;codecs=vp8' });
    const done = new Promise(resolve => recorder.onstop = async () => { const data = new Uint8Array(await new Blob(chunks).arrayBuffer()); resolve(btoa(String.fromCharCode(...data))); });
    recorder.ondataavailable = event => chunks.push(event.data); recorder.start();
    for (let i = 0; i < 8; i++) { ctx.fillStyle = i % 2 ? '#cc8877' : '#6550a8'; ctx.fillRect(0, 0, 320, 180); await new Promise(r => setTimeout(r, 100)); }
    recorder.stop(); const webm = await done; stream.getTracks().forEach(track => track.stop()); return { png, webm };
  });
  const names = ['Bild ü.png', 'Clip.webm', 'Ton.wav'];
  const samples = 22050, wav = Buffer.alloc(44 + samples * 2); wav.write('RIFF'); wav.writeUInt32LE(wav.length - 8, 4); wav.write('WAVEfmt ', 8); wav.writeUInt32LE(16, 16); wav.writeUInt16LE(1, 20); wav.writeUInt16LE(1, 22); wav.writeUInt32LE(22050, 24); wav.writeUInt32LE(44100, 28); wav.writeUInt16LE(2, 32); wav.writeUInt16LE(16, 34); wav.write('data', 36); wav.writeUInt32LE(samples * 2, 40);
  for (let i = 0; i < samples; i++) wav.writeInt16LE(Math.round(Math.sin(i / 8) * 500), 44 + i * 2);
  const bytes = [Buffer.from(encoded.png, 'base64'), Buffer.from(encoded.webm, 'base64'), wav];
  for (let i = 0; i < names.length; i++) await fs.writeFile(path.join(source, names[i]), bytes[i]);
  const model = path.join(source, 'reference-only.safetensors'); await fs.writeFile(model, 'checksum fixture, never used for inference');
  await page.getByRole('button', { name: 'Studio', exact: true }).first().click();
  if (await page.getByRole('button', { name: 'Restore workspace', exact: true }).count()) await page.getByRole('button', { name: 'Restore workspace', exact: true }).click();
  await page.getByLabel('Image prompt', { exact: true }).fill('Project roundtrip ü');
  await page.locator('#image-model').fill(model);
  await page.locator('.project-title').click();
  await page.getByLabel('Project name', { exact: true }).fill('Arbeitsstand ü');
  await page.getByRole('button', { name: 'Create project', exact: true }).click();
  await until(async () => (await invoke('project_snapshot'))?.name === 'Arbeitsstand ü');
  await page.evaluate(sources => { window.projectDialog.open = sources; }, names.map(n => path.join(source, n)));
  await page.getByRole('button', { name: 'Add media', exact: true }).click();
  await until(async () => (await invoke('project_snapshot'))?.assets.length === 3);
  await page.evaluate(value => { window.projectDialog.save = value; }, target);
  await page.getByLabel('Image prompt', { exact: true }).press('Control+s');
  await until(async () => (await invoke('project_snapshot'))?.path === target && !(await invoke('project_snapshot')).dirty);
  const saved = await invoke('project_snapshot'); assert.equal(saved.request.prompt, 'Project roundtrip ü'); assert.equal(saved.model.sha256, hash(await fs.readFile(model))); assert((await fs.stat(target)).size < 1024 * 1024);
  record('Native project creation, image/video/audio embedding and Ctrl+S save actual container with model checksum; file picker results supplied by harness');
  for (let i = 0; i < names.length; i++) {
    await page.locator('.project-asset').filter({ hasText: names[i] }).click();
    const selector = i === 0 ? 'img' : i === 1 ? 'video' : 'audio'; const preview = page.locator(`.project-preview ${selector}`);
    await until(() => preview.evaluate(el => el instanceof HTMLImageElement ? el.complete && el.naturalWidth === 320 : el.readyState >= 1));
    if (i) { await preview.evaluate(el => el.play()); await until(() => preview.evaluate(el => el.currentTime > .1)); await preview.evaluate(el => el.pause()); }
  }
  const url = `http://project.localhost/${saved.id}/${saved.assets[0].id}`;
  const denied = await page.evaluate(async value => (await fetch(value)).status, `http://project.localhost/${saved.id}/not-a-listed-asset`); assert.equal(denied, 404);
  await page.screenshot({ path: path.join(artifactRoot, 'project-media.png') });
  record('Embedded image decodes and video/audio actually play through restricted project protocol; unlisted asset denied');
  await page.getByLabel('Image prompt', { exact: true }).fill('Saved immediately before debounce');
  await page.getByLabel('Image prompt', { exact: true }).press('Control+s');
  await until(async () => { const p = await invoke('project_snapshot'); return p.request.prompt === 'Saved immediately before debounce' && !p.dirty; });
  await page.getByRole('button', { name: 'Close project', exact: true }).click(); await until(async () => await invoke('project_snapshot') === null);
  await page.evaluate(value => { window.projectDialog.open = value; }, target);
  await page.getByRole('button', { name: 'Open project', exact: true }).click(); await until(async () => (await invoke('project_snapshot'))?.path === target);
  assert.equal(await page.getByLabel('Image prompt', { exact: true }).inputValue(), 'Saved immediately before debounce');
  assert.equal(await page.locator('#image-model').inputValue(), ''); assert.equal(await page.locator('[data-image-generate]').isDisabled(), true);
  assert.equal((await invoke('project_snapshot')).assets.length, 3);
  record('Immediate edit/save and close/reopen restore exact inputs; unresolved model keeps generation disabled without blocking media');
  await page.evaluate(value => { window.projectDialog.open = value; }, path.join(source, names[0]));
  await page.getByRole('button', { name: 'Link original model', exact: true }).click(); await page.getByText('The checksum does not match the saved model.', { exact: true }).waitFor();
  await page.evaluate(value => { window.projectDialog.open = value; }, model);
  await page.getByRole('button', { name: 'Link original model', exact: true }).click(); await until(() => page.locator('#image-model').inputValue().then(value => value === model));
  await page.getByRole('button', { name: 'Copy media to gallery', exact: true }).click(); await page.getByText('3 media files copied to gallery. 0 errors.', { exact: true }).waitFor();
  const gallery = (await invoke('bootstrap')).paths.gallery; const exported = (await fs.readdir(gallery)).find(n => n.startsWith('Project-')); assert(exported);
  for (let i = 0; i < names.length; i++) assert.equal(hash(await fs.readFile(path.join(gallery, exported, names[i]))), hash(bytes[i]));
  record('Incorrect model link refused; checksum-identical local model linked; gallery export preserves friendly names and all bytes');
  await page.getByRole('button', { name: `Remove from project: ${names[0]}`, exact: true }).click(); await until(async () => (await invoke('project_snapshot'))?.assets.length === 2);
  assert.equal(await page.evaluate(async value => (await fetch(value)).status, url), 404);
  await page.getByRole('button', { name: 'Save project', exact: true }).click(); await until(async () => !(await invoke('project_snapshot')).dirty);
  assert.equal(hash(await fs.readFile(path.join(source, names[0]))), hash(bytes[0]));
  await page.getByLabel('Image prompt', { exact: true }).fill('Crash recovery prompt'); await until(async () => (await invoke('project_snapshot'))?.request.prompt === 'Crash recovery prompt');
  await stop(false); await launch(); page = getPage(); await dialogs();
  await page.getByRole('button', { name: 'Resume project', exact: true }).click(); await until(async () => !(await invoke('project_snapshot'))?.recovery);
  await page.getByRole('button', { name: 'Studio', exact: true }).first().click();
  assert.equal(await page.getByLabel('Image prompt', { exact: true }).inputValue(), 'Crash recovery prompt'); assert.equal((await invoke('project_snapshot')).assets.length, 2);
  record('Project removal preserves source; unsaved project inputs and media survive actual process termination and explicit resume');
  // Real native window close triggers the one in-app exit dialog, including project checkbox.
  await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('plugin:window|close', { label: 'main' }));
  const dialog = page.locator('.image-exit-dialog'); await dialog.waitFor();
  assert.equal(await dialog.getByLabel('Save project file before closing', { exact: true }).isChecked(), true);
  await dialog.getByRole('button', { name: 'Cancel', exact: true }).click();
  // The compact project header now leaves file commands to the native menu.
  // Expand details to exercise the retained project-panel controls here.
  if(await page.locator('.project-title').getAttribute('aria-expanded')==='false')await page.locator('.project-title').click();
  const before = await fs.readFile(target); await fs.writeFile(target, 'external modification');
  await page.getByRole('button', { name: 'Save project', exact: true }).click(); await page.getByText('The project or file changed. Save under a new name.', { exact: true }).waitFor();
  assert.equal(await fs.readFile(target, 'utf8'), 'external modification'); await fs.writeFile(target, before);
  await page.getByRole('button', { name: 'Save project', exact: true }).click(); await until(async () => !(await invoke('project_snapshot')).dirty);
  await page.getByRole('button', { name: 'Close project', exact: true }).click(); await until(async () => await invoke('project_snapshot') === null);
  record('One exit dialog includes checked project save option and Cancel keeps app open; changed external archive is never overwritten');
}
