import assert from 'node:assert/strict';
import { promises as fs } from 'node:fs';
import path from 'node:path';

export async function checkModelFiltering(page, artifactRoot, invoke, record) {
  const state = await invoke('model_library_list');
  const models = state.entries.filter(e => e.discovery === 'model');
  const candidates = state.entries.filter(e => e.discovery === 'candidate');
  const excluded = state.entries.filter(e => e.discovery === 'excluded');
  assert.equal(state.entries.length, 2617, 'All legacy records retained');
  assert.ok(models.length > 20 && models.length < 100);
  assert.ok(excluded.length > 2000);
  for (const name of ['distutils-precedence.pth', 'tutor1.pt', 'WordFluencyV5.onnx']) {
    assert.ok(excluded.some(e => e.name === name), name);
    assert.ok(!models.some(e => e.name === name));
  }
  assert.ok(models.some(e => e.name === 'Juggernaut-XL_v9_RunDiffusionPhoto_v2.safetensors'));
  assert.ok(models.some(e => e.name.includes('Qwen') && e.format === 'gguf'));
  assert.ok(candidates.some(e => e.discoveryReason === 'local_tensor_data'));
  const stats = { total: state.entries.length, models: models.length, candidates: candidates.length, excluded: excluded.length };
  await fs.writeFile(path.join(artifactRoot, 'legacy-filtering.json'), JSON.stringify(stats, null, 2));
  record('Actual 0.5.1 library migration retains all 2617 records and separates models, uncertain candidates and application/test files', JSON.stringify(stats));
  await page.getByRole('button', { name: 'Models', exact: true }).first().click();
  await page.getByRole('button', { name: 'Stored locally', exact: true }).click();
  const filter = page.getByLabel('Show findings', { exact: true });
  await filter.waitFor(); assert.equal(await filter.inputValue(), 'model');
  await page.locator('[data-model-id]').first().waitFor();
  const ids = await page.locator('[data-model-id]').evaluateAll(cards => cards.map(card => card.getAttribute('data-model-id')));
  assert.ok(ids.length <= 50 && ids.every(id => models.some(e => e.id === id)));
  await page.screenshot({ path: path.join(artifactRoot, 'filtered-models.png') });
  record('Default native model list shows only identified model formats and limits rendered cards to 50');
  await filter.selectOption('candidate');
  await page.waitForFunction(() => [...document.querySelectorAll('[data-model-id]')].length > 0);
  const uncertain = await page.locator('[data-model-id]').evaluateAll(cards => cards.map(card => card.getAttribute('data-model-id')));
  assert.ok(uncertain.every(id => candidates.some(e => e.id === id)));
  await filter.selectOption('excluded');
  await page.getByRole('button', { name: 'Next', exact: true }).click();
  await page.getByText(/Page 2 \/ \d+/, { exact: true }).waitFor();
  assert.equal(await page.locator('[data-model-id]').count(), 50);
  await page.screenshot({ path: path.join(artifactRoot, 'hidden-legacy-findings.png') });
  record('Unconfirmed and hidden legacy findings remain reviewable in separate paginated views');
  await filter.selectOption('model');
  const juggernaut = models.find(e => e.name === 'Juggernaut-XL_v9_RunDiffusionPhoto_v2.safetensors');
  const modelCard = page.locator(`[data-model-id="${juggernaut.id}"]`);
  while (!(await modelCard.count())) await page.getByRole('button', { name: 'Next', exact: true }).click();
  await modelCard.getByRole('button', { name: 'Check in Studio', exact: true }).click();
  assert.equal(await page.getByLabel('SDXL model file', { exact: true }).inputValue(), juggernaut.path);
  await page.getByRole('button', { name: 'Check readiness', exact: true }).click();
  await page.getByTestId('image-readiness').getByText('Ready to attempt generation', { exact: true }).waitFor({ timeout: 30000 });
  record('Real Juggernaut model survives legacy filtering and opens in Studio with successful runtime/GPU readiness check');
  return stats;
}

export async function checkModelFilteringRestart(page, invoke, record, stats) {
  const state = await invoke('model_library_list');
  assert.equal(state.entries.length, stats.total);
  for (const [category, count] of [['model', stats.models], ['candidate', stats.candidates], ['excluded', stats.excluded]]) {
    assert.equal(state.entries.filter(e => e.discovery === category).length, count);
  }
  record('Library classification persists across restart without discarding old references');
}
