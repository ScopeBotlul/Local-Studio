// Real public HF API through native Rust IPC. No account or OAuth registration is created.
import assert from 'node:assert/strict';
import path from 'node:path';
import { promises as fs } from 'node:fs';

export async function checkHuggingFace(page, artifactRoot, invoke, record, appPid) {
  const status = await invoke('hf_status');
  assert.equal(status.account, null);
  assert.equal(status.pending, false);
  assert.equal(status.oauthConfigured, true, 'Uses the explicitly approved registered public client');
  assert.equal(JSON.stringify(status).includes('accessToken'), false);
  await page.getByRole('button', { name: 'Hugging Face', exact: true }).click();
  await page.getByRole('heading', { name: 'Hugging Face account', exact: true }).waitFor();
  assert.equal(await page.getByRole('button', { name: 'Sign in with Hugging Face', exact: true }).isDisabled(), false);
  record('HF account state is local and registered public OAuth is enabled');

  await page.getByText('Advanced: use your own access token', { exact: true }).click();
  const synthetic = 'hf_invalidLocalStudioTestToken000000000000';
  await page.getByLabel('Hugging Face access token', { exact: true }).fill(synthetic);
  await page.getByRole('button', { name: 'Verify token and connect', exact: true }).click();
  await page.getByRole('alert').filter({ hasText: 'Credentials are invalid or expired.' }).waitFor({ timeout: 40000 });
  assert.equal(await page.getByLabel('Hugging Face access token', { exact: true }).inputValue(), '');
  assert.equal((await invoke('hf_status')).account, null);
  const snapshot = await invoke('bootstrap');
  assert.equal((await fs.readFile(snapshot.databasePath)).includes(Buffer.from(synthetic)), false);
  assert.equal((await invoke('get_logs')).includes(synthetic), false);
  await invoke('hf_cancel_login');
  await page.screenshot({ path: path.join(artifactRoot, '05-hf-account.png') });
  record('Real HF rejects invalid token; UI clears it and neither SQLite nor logs contain it');

  await page.getByRole('button', { name: 'Models', exact: true }).click();
  await page.getByLabel('Model name or repository ID', { exact: true }).fill('gpt2');
  await page.getByRole('button', { name: 'Search models', exact: true }).click();
  await page.locator('.hub-model').first().waitFor({ timeout: 40000 });
  const firstPage = await page.locator('.hub-model h2').allTextContents();
  assert.equal(firstPage.length, 20);
  assert.ok(firstPage.includes('openai-community/gpt2'));
  const gptCard = page.locator('.hub-model').filter({ has: page.getByRole('heading', { name: 'openai-community/gpt2', exact: true }) });
  await page.waitForFunction(() => [...document.querySelectorAll('.hub-model')].some(card => card.querySelector('h2')?.textContent === 'openai-community/gpt2' && card.querySelector('[data-testid="model-size"]')?.textContent?.includes('GB')), null, { timeout: 65000 });
  assert.match(await gptCard.getByTestId('model-size').innerText(), /Repository total: [\d.,]+ GB/);
  await page.getByRole('button', { name: 'Load more models', exact: true }).click();
  await page.waitForFunction(() => document.querySelectorAll('.hub-model').length > 20, { timeout: 40000 });
  const all = await page.locator('.hub-model h2').allTextContents();
  assert.equal(new Set(all).size, all.length);
  await page.screenshot({ path: path.join(artifactRoot, '06-hf-models.png') });
  record('Real public model search and pagination through native UI', `${all.length} distinct results`);

  await page.locator('.hub-model').filter({ has: page.getByRole('heading', { name: 'openai-community/gpt2', exact: true }) }).getByRole('button', { name: 'Inspect model', exact: true }).click();
  await page.getByRole('region', { name: 'Inspect model', exact: true }).waitFor({ timeout: 65000 });
  const result = await invoke('hf_model_detail', { repo: 'openai-community/gpt2', revision: 'main' });
  assert.match(result.revision, /^[a-f0-9]{40}$/);
  assert.ok(result.files.some(file => file.path === 'config.json' && file.size > 0));
  assert.ok(result.card?.includes('GPT-2'));
  assert.equal(result.model.license, 'mit');
  const total = result.files.reduce((sum, file) => sum + file.size, 0);
  assert.equal(await invoke('hf_model_size', { repo: result.model.id, revision: result.revision }), total);
  assert.ok((await page.getByTestId('model-total-size').innerText()).includes(`${new Intl.NumberFormat('en', { minimumFractionDigits: 2, maximumFractionDigits: 2 }).format(total / 1e9)} GB`));
  await page.getByText('Model card (original text)', { exact: true }).click();
  await page.screenshot({ path: path.join(artifactRoot, '07-hf-details.png') });
  const revisionBox = page.getByLabel('Revision (branch, tag or commit)', { exact: true });
  await revisionBox.fill('local-studio-nonexistent-revision-82ca9878');
  await page.getByRole('button', { name: 'Load revision', exact: true }).click();
  await page.getByRole('alert').filter({ hasText: 'Model, file or revision not found' }).waitFor({ timeout: 40000 });
  assert.ok((await page.locator('.hub-commit').innerText()).includes(result.revision), 'Failed revision does not replace previous details');
  record('Real model details include exact commit, file sizes, license and raw model card; missing revision rejected', result.revision);

  await assert.rejects(invoke('hf_model_detail', { repo: '../escape', revision: 'main' }), value => String(value).endsWith('invalid_repo'));
  await assert.rejects(invoke('hf_open_page', { page: 'https://evil.example', repo: null }), value => String(value).endsWith('invalid_link'));
  await assert.rejects(invoke('hf_open_page', { page: 'model', repo: 'https://evil.example' }), value => String(value).endsWith('invalid_repo'));
  await page.getByRole('button', { name: 'Close details', exact: true }).click();
  await page.getByLabel('Only show models executable in Local Studio', { exact: true }).check();
  await page.getByText('No model adapter is integrated yet. No models are currently classified as executable.', { exact: true }).waitFor();
  assert.equal(await page.locator('.hub-model').count(), 0);
  record('Invalid repository and link inputs rejected; compatibility filter never claims an unimplemented runtime');

  const filtered = await invoke('hf_search', { query: { search: 'whisper', task: 'automatic-speech-recognition', sort: 'downloads', cursor: null } });
  assert.ok(filtered.models.length > 0);
  assert.ok(filtered.models.every(model => model.task === 'automatic-speech-recognition'));
  record('Real HF task filter returns matching speech models');
  await invoke('hf_logout');
  assert.equal((await invoke('hf_status')).account, null);
  await page.getByRole('button', { name: 'Settings', exact: true }).first().click();
}
