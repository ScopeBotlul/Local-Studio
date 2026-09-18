import assert from 'node:assert/strict';
import { promises as fs } from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
const a = String.raw`D:\LocalAI\ComfyUI_windows_portable\ComfyUI\models\checkpoints\Juggernaut-XL_v9_RunDiffusionPhoto_v2.safetensors`;
const b = String.raw`D:\LocalAI\ComfyUI_windows_portable\ComfyUI\models\checkpoints\DreamShaperXL_Turbo_V2.safetensors`;
const normalize = p => p.replace(/^\\\\\?\\/, '').toLowerCase();
const requestA = { modelPath: a, prompt: 'Queue A: a quiet alpine lake, morning sunlight, landscape photograph', negativePrompt: 'text, watermark', width: 1024, height: 1024, steps: 60, guidance: 5, seed: 71, sampler: 'euler' };
const requestB = { ...requestA, modelPath: b, prompt: 'Queue B: a ceramic blue teapot on a wooden table, studio photograph', negativePrompt: 'letters, blur', width: 512, height: 512, steps: 4, guidance: 2, seed: 93, sampler: 'dpm++2m' };
async function until(fn, timeout = 180000) { const end = Date.now() + timeout; while (Date.now() < end) { const value = await fn(); if (value) return value; await pause(120); } throw new Error('Image queue condition timed out'); }
function sameRequest(actual, expected) { assert.deepEqual({ ...actual, modelPath: normalize(actual.modelPath) }, { ...expected, modelPath: normalize(expected.modelPath) }); }

export async function checkImageQueue({ getPage, invoke, stop, launch, artifactRoot, record }) {
  let page = getPage();
  if ((await invoke('image_workspace')).recoveryAvailable) await invoke('image_recover');
  const job = async id => (await invoke('image_jobs')).find(j => j.id === id);
  const done = async id => until(async () => { const j = await job(id); if (['failed', 'cancelled', 'interrupted'].includes(j?.status)) throw new Error(JSON.stringify(j)); return j?.status === 'completed' ? j : false; });
  const first = await invoke('image_generate', { request: requestA });
  await until(async () => { const j = await job(first.id); if (j.status === 'failed') throw new Error(j.error); return j.phase === 'sampling' && j.step > 0; });
  await page.getByRole('button', { name: /^Studio/ }).first().click();
  await page.getByLabel('SDXL model file', { exact: true }).fill(b);
  await page.getByLabel('Image prompt', { exact: true }).fill(requestB.prompt);
  await page.getByLabel('Negative prompt', { exact: true }).fill(requestB.negativePrompt);
  await page.getByLabel('Image width', { exact: true }).selectOption('512');
  await page.getByLabel('Image height', { exact: true }).selectOption('512');
  for (const [label, value] of [['Image steps', 4], ['Image guidance', 2], ['Image seed', 93]]) await page.getByLabel(label, { exact: true }).fill(String(value));
  await page.getByLabel('Image sampler', { exact: true }).selectOption('dpm++2m');
  await page.getByRole('button', { name: 'Check readiness', exact: true }).click();
  await page.getByTestId('image-readiness').getByText('Ready to attempt generation', { exact: true }).waitFor({ timeout: 30000 });
  await page.getByRole('button', { name: 'Queue image', exact: true }).click();
  const second = await until(async () => (await invoke('image_jobs')).find(j => j.request.prompt === requestB.prompt));
  assert.equal(second.status, 'queued'); assert.equal(second.queuePosition, 1); assert.equal(second.startedAt, null);
  sameRequest(second.request, requestB); sameRequest((await job(first.id)).request, requestA);
  await page.getByLabel('Image prompt', { exact: true }).fill('Later draft C must not modify either queued snapshot');
  await page.getByLabel('Image steps', { exact: true }).fill('7');
  const third = await invoke('image_generate', { request: { ...requestB, prompt: 'Cancel this waiting job', seed: 123 } });
  assert.equal((await job(third.id)).status, 'queued');
  await page.getByRole('button', { name: /^Jobs/ }).first().click();
  const thirdRow = page.locator(`[data-image-job-id="${third.id}"]`); await thirdRow.waitFor();
  await thirdRow.getByRole('button', { name: 'Cancel image job', exact: true }).click();
  await until(async () => (await job(third.id)).status === 'cancelled');
  assert.equal((await job(third.id)).startedAt, null); assert.equal((await job(first.id)).status, 'running');
  await page.screenshot({ path: path.join(artifactRoot, 'image-queue-jobs.png') });
  record('Native UI queues model B while A generates; both retain independent prompts/parameters; cancelling waiting C leaves A running');
  let maxRunning = 0;
  const secondDone = await until(async () => {
    const jobs = await invoke('image_jobs'); maxRunning = Math.max(maxRunning, jobs.filter(j => j.status === 'running').length);
    assert.ok(maxRunning <= 1); const j = jobs.find(j => j.id === second.id);
    if (j.status === 'failed') throw new Error(JSON.stringify(j)); return j.status === 'completed' ? j : false;
  });
  const firstDone = await done(first.id);
  assert.ok(firstDone.finishedAt <= secondDone.startedAt); sameRequest(secondDone.request, requestB);
  assert.notEqual(firstDone.modelSha256, secondDone.modelSha256);
  for (const j of [firstDone, secondDone]) { const png = await fs.readFile(j.output); assert.equal(png.readUInt32BE(16), j.request.width); assert.equal(png.readUInt32BE(20), j.request.height); }
  await fs.writeFile(path.join(artifactRoot, 'queue-results.json'), JSON.stringify([firstDone, secondDone, await job(third.id)], null, 2));
  record('Actual GPU FIFO completes A before starting B; real PNG dimensions and distinct checkpoint hashes match each immutable job', `A ${firstDone.elapsedMs} ms; B ${secondDone.elapsedMs} ms`);

  await invoke('image_save', { id: second.id }); const count = (await invoke('image_jobs')).length;
  await page.getByRole('button', { name: 'Gallery', exact: true }).first().click();
  const savedB = (await job(second.id)).savedPath; const savedFolder = path.basename(path.dirname(savedB));
  await page.locator('.gallery-file').filter({ hasText: savedFolder }).click();
  await page.getByRole('button', { name: 'Restore settings', exact: true }).click();
  await until(() => page.getByLabel('Image prompt', { exact: true }).inputValue().then(v => v === requestB.prompt));
  await until(async () => (await invoke('image_workspace')).workspace.request?.prompt === requestB.prompt);
  sameRequest((await invoke('image_workspace')).workspace.request, requestB);
  assert.equal((await invoke('image_jobs')).length, count);
  assert.equal(await page.getByRole('button', { name: 'Generate image', exact: true }).isDisabled(), true);
  await page.screenshot({ path: path.join(artifactRoot, 'image-restore-settings.png') });
  record('Saved gallery image restores exact model, both prompts and all parameters into Studio without creating a job; readiness check required again');

  const interrupted = await invoke('image_generate', { request: { ...requestA, prompt: 'Crash recovery running A' } });
  await until(async () => (await job(interrupted.id)).phase === 'sampling');
  const paused = await invoke('image_generate', { request: { ...requestB, prompt: 'Crash recovery waiting B' } });
  assert.equal((await job(paused.id)).status, 'queued');
  await stop(false); await launch(); page = getPage();
  assert.equal((await job(interrupted.id)).status, 'interrupted'); assert.equal((await job(paused.id)).status, 'paused');
  assert.equal((await job(paused.id)).startedAt, null); assert.equal((await invoke('image_jobs')).filter(j => ['queued', 'running'].includes(j.status)).length, 0);
  assert.equal((await invoke('image_workspace')).recoveryAvailable, true);
  await assert.rejects(invoke('image_resume', { id: paused.id }), /image_recovery_pending/);
  await page.getByRole('button', { name: 'Restore workspace', exact: true }).click();
  await page.getByRole('button', { name: /^Jobs/ }).first().click();
  const pausedRow = page.locator(`[data-image-job-id="${paused.id}"]`); await pausedRow.waitFor();
  await page.screenshot({ path: path.join(artifactRoot, 'image-queue-paused.png') });
  await pausedRow.getByRole('button', { name: 'Resume image job', exact: true }).click();
  const resumed = await done(paused.id); assert.equal(resumed.modelSha256, paused.modelSha256); sameRequest(resumed.request, paused.request);
  await assert.rejects(invoke('image_resume', { id: paused.id }), /image_not_paused/);
  const saved = (await job(second.id)).savedPath;
  assert.equal(createHash('sha256').update(await fs.readFile(saved)).digest('hex'), createHash('sha256').update(await fs.readFile(secondDone.output)).digest('hex'));
  record('Native crash pauses waiting jobs without autostart; explicit recovery and Resume runs the original pinned B snapshot, with no duplicate resume');
  for (const j of await invoke('image_jobs')) if (j.status === 'completed' && !j.savedPath && !j.discarded) await invoke('image_discard', { id: j.id });
}
