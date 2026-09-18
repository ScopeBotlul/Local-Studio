import assert from 'node:assert/strict';
import { promises as fs } from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';

const hash = data => createHash('sha256').update(data).digest('hex');
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
async function finished(invoke) {
  for (let n = 0; n < 200; n++) {
    const state = await invoke('model_library_list');
    if (state.scan.status !== 'running') return state;
    await pause(50);
  }
  throw new Error('Model scan did not finish');
}
export async function checkModelImport(page, artifactRoot, invoke, record) {
  const source = path.join(artifactRoot, 'external-models-ä');
  await fs.mkdir(source);
  const download = (await invoke('download_list')).find(j => j.status === 'completed');
  assert.ok(download, 'Use real previously downloaded public HF weights as import fixture');
  const actual = await fs.readFile(path.join(download.destination, 'model.safetensors'));
  const model = path.join(source, 'model.safetensors'); await fs.writeFile(model, actual);
  await fs.copyFile(path.join(download.destination, 'config.json'), path.join(source, 'config.json'));
  await fs.writeFile(path.join(source, 'broken.safetensors'), 'not a model');
  await fs.writeFile(path.join(source, 'candidate.ckpt'), Buffer.from([0x80, 0x02, 0x00]));
  await fs.writeFile(path.join(source, 'python-path.pth'), 'import site; configure_python_path()');
  await fs.writeFile(path.join(source, 'tutor.pt'), 'Portuguese tutorial text');
  const shards = path.join(source, 'sharded'); await fs.mkdir(shards);
  await fs.writeFile(path.join(shards, 'one.safetensors'), actual);
  await fs.writeFile(path.join(shards, 'model.safetensors.index.json'), JSON.stringify({ weight_map: { a: 'one.safetensors', b: 'missing.safetensors' } }));
  const outside = path.join(artifactRoot, 'outside-scan'); await fs.mkdir(outside);
  await fs.writeFile(path.join(outside, 'must-not-import.safetensors'), actual);
  await fs.symlink(outside, path.join(source, 'redirected'), 'junction');

  await page.getByRole('button', { name: 'Models', exact: true }).first().click();
  await page.getByRole('button', { name: 'Stored locally', exact: true }).click();
  const beforeFull = await invoke('model_library_list');
  await page.getByRole('button', { name: 'Full scan', exact: true }).click();
  await page.getByRole('button', { name: 'Cancel scan', exact: true }).waitFor();
  let full = await invoke('model_library_list');
  assert.equal(full.scan.mode, 'full'); assert.ok(full.scan.roots.length > 0);
  assert.ok(full.scan.roots.every(root => /^[A-Z]:\\$/.test(root)));
  assert.equal(await page.getByRole('button', { name: 'Full scan', exact: true }).isDisabled(), true);
  for (let n = 0; n < 80; n++) {
    if (/[1-9]\d* paths checked/.test(await page.getByTestId('model-scan-status').innerText())) break;
    await pause(100);
  }
  assert.match(await page.getByTestId('model-scan-status').innerText(), /[1-9]\d* paths checked/);
  await page.screenshot({ path: path.join(artifactRoot, 'full-scan.png') });
  await assert.rejects(invoke('model_scan_start', { path: source }));
  await page.getByRole('button', { name: 'Cancel scan', exact: true }).click();
  full = await finished(invoke); assert.equal(full.scan.status, 'cancelled');
  assert.deepEqual(full.entries, beforeFull.entries);
  record('Full scan button discovers real local drives, reports progress, prevents concurrent scan and cancels without importing partial results');
  await page.getByLabel('Model folder', { exact: true }).fill(source);
  await page.getByRole('button', { name: 'Scan folder', exact: true }).click();
  await page.getByTestId('model-scan-status').filter({ hasText: 'Scan completed' }).waitFor({ timeout: 30000 });
  let state = await finished(invoke); assert.equal(state.scan.status, 'completed'); assert.equal(state.entries.length, 4);
  let imported = state.entries.find(e => e.name === 'model.safetensors'); assert.equal(imported.status, 'checked'); assert.equal(imported.family, 'gpt2');
  assert.equal(imported.totalBytes, actual.length); assert.equal(hash(await fs.readFile(model)), hash(actual));
  assert.ok(imported.path.includes('external-models-ä')); assert.ok(state.scan.notes.some(n => n.code === 'local_link'));
  assert.ok(!state.entries.some(e => e.name === 'must-not-import.safetensors'));
  assert.equal(state.entries.find(e => e.name === 'broken.safetensors').status, 'invalid');
  assert.equal(state.entries.find(e => e.name === 'candidate.ckpt').status, 'unverified');
  assert.ok(!state.entries.some(e => ['python-path.pth', 'tutor.pt'].includes(e.name)));
  assert.equal(state.entries.find(e => e.name === 'candidate.ckpt').discovery, 'candidate');
  assert.equal(state.entries.find(e => e.format === 'safetensors-index').status, 'incomplete');
  assert.equal(state.entries.filter(e => e.name === 'one.safetensors').length, 0);
  const card = page.locator(`[data-model-id="${imported.id}"]`);
  await card.getByText('File structure checked', { exact: true }).waitFor();
  assert.ok((await card.innerText()).includes('< 0.01 GB'));
  assert.ok((await card.innerText()).includes('Detection does not confirm execution.'));
  await card.scrollIntoViewIfNeeded(); await page.screenshot({ path: path.join(artifactRoot, 'model-import.png') });
  record('Import real HF Safetensors in place; format, GB, family, missing shards and invalid candidates shown; junction skipped');

  await invoke('model_scan_start', { path: source }); state = await finished(invoke); assert.equal(state.entries.length, 4);
  record('Repeat scan updates stable IDs without duplicating local models');
  await fs.rename(model, model + '.moved');
  await card.getByText('Not found', { exact: true }).waitFor({ timeout: 12000 });
  await fs.writeFile(model, 'damaged file');
  await card.getByRole('button', { name: 'Recheck', exact: true }).click();
  await card.getByText('Invalid file structure', { exact: true }).waitFor();
  await fs.writeFile(model, actual);
  await card.getByRole('button', { name: 'Recheck', exact: true }).click();
  await card.getByText('File structure checked', { exact: true }).waitFor();
  record('Known imported path is checked periodically; missing file and corruption are visible and explicit recheck recovers');

  const large = path.join(artifactRoot, 'cancel-scan'); await fs.mkdir(large);
  await Promise.all(Array.from({ length: 250 }, (_, n) => fs.writeFile(path.join(large, `fixture-${n}.safetensors`), actual)));
  await invoke('model_scan_start', { path: large }); await invoke('model_scan_cancel');
  state = await finished(invoke); assert.equal(state.scan.status, 'cancelled'); assert.equal(state.entries.length, 4);
  await assert.rejects(invoke('model_scan_start', { path: '../relative' }));
  record('Scan cancellation preserves existing library; relative import roots rejected');

  await page.getByLabel('Show findings', { exact: true }).selectOption('candidate');
  const candidate = state.entries.find(e => e.name === 'candidate.ckpt');
  await page.locator(`[data-model-id="${candidate.id}"]`).getByRole('button', { name: 'Remove from list', exact: true }).click();
  await page.waitForFunction(id => !document.querySelector(`[data-model-id="${id}"]`), candidate.id);
  assert.deepEqual(await fs.readFile(path.join(source, 'candidate.ckpt')), Buffer.from([0x80, 0x02, 0x00]));
  record('Remove from list deletes only database reference and preserves external file');
  return { id: imported.id, digest: hash(actual), model };
}

export async function checkModelImportRestart(page, invoke, record, fixture) {
  const state = await invoke('model_library_list');
  assert.equal(state.entries.length, 3); assert.equal(state.entries.find(e => e.id === fixture.id).status, 'checked');
  assert.equal(hash(await fs.readFile(fixture.model)), fixture.digest);
  await page.getByRole('button', { name: 'Models', exact: true }).first().click();
  await page.getByRole('button', { name: 'Stored locally', exact: true }).click();
  await page.locator(`[data-model-id="${fixture.id}"]`).getByText('File structure checked', { exact: true }).waitFor();
  record('Imported references and unchanged source files survive app restart');
}
