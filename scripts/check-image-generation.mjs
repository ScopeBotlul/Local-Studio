// Real model execution, only enabled explicitly on a machine with local test weights.
import assert from 'node:assert/strict';
import { promises as fs } from 'node:fs';
import { createHash } from 'node:crypto';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import path from 'node:path';
const exec = promisify(execFile);
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const modelA = process.env.LOCAL_STUDIO_IMAGE_MODEL_A || String.raw`D:\LocalAI\ComfyUI_windows_portable\ComfyUI\models\checkpoints\Juggernaut-XL_v9_RunDiffusionPhoto_v2.safetensors`;
const modelB = process.env.LOCAL_STUDIO_IMAGE_MODEL_B || String.raw`D:\LocalAI\ComfyUI_windows_portable\ComfyUI\models\checkpoints\DreamShaperXL_Turbo_V2.safetensors`;
const request = { modelPath: modelA, prompt: 'A quiet alpine lake at sunrise, pine trees, clear reflections, landscape photography, no people', negativePrompt: 'text, watermark, blurry', width: 512, height: 512, steps: 10, guidance: 5, seed: 42, sampler: 'euler' };

async function waitJob(invoke, id, predicate, timeout = 180000) {
  const deadline = Date.now() + timeout; let job;
  while (Date.now() < deadline) {
    job = (await invoke('image_jobs')).find(j => j.id === id);
    if (job && predicate(job)) return job;
    if (job && !['running', 'queued'].includes(job.status)) throw new Error(JSON.stringify(job));
    await pause(150);
  }
  throw new Error(`Timed out waiting for image job: ${JSON.stringify(job)}`);
}
async function workers(root) {
  const { stdout } = await exec('powershell', ['-NoProfile', '-Command', "Get-CimInstance Win32_Process -Filter \"Name='sd-cli.exe'\" | Select-Object ProcessId,ExecutablePath | ConvertTo-Json -Compress"], { windowsHide: true });
  const data = stdout.trim() ? JSON.parse(stdout) : [];
  return (Array.isArray(data) ? data : [data]).filter(p => p.ExecutablePath?.toLowerCase().startsWith(root.toLowerCase() + path.sep));
}
async function noWorker(root) {
  for (let i = 0; i < 15; i++) { if (!(await workers(root)).length) return; await pause(200); }
  assert.fail('Image worker survived completion/cancellation/parent crash');
}
async function scanDone(invoke) {
  for (let i = 0; i < 600; i++) { const s = await invoke('model_library_list'); if (s.scan.mode === 'quick' && s.scan.status !== 'running') return s; await pause(100); }
  assert.fail('Quick scan did not finish');
}
export async function checkImageGeneration(page, artifactRoot, invoke, record) {
  await fs.access(modelA); await fs.access(modelB);
  await page.getByRole('button', { name: 'Models', exact: true }).first().click();
  await page.getByRole('button', { name: 'Stored locally', exact: true }).click();
  await page.getByRole('button', { name: 'Quick scan', exact: true }).click();
  let library = await scanDone(invoke);
  assert.equal(library.scan.mode, 'quick'); assert.equal(library.scan.status, 'completed');
  const configuredPaths = (await invoke('bootstrap')).paths;
  for (const name of ['models', 'assistantModels', 'visionModels']) {
    const expected = path.resolve(configuredPaths[name]).toLowerCase();
    assert(library.scan.roots.some(root => expected === path.resolve(root).toLowerCase() || expected.startsWith(path.resolve(root).toLowerCase() + path.sep)), `${name} is included in quick scan`);
  }
  await fs.writeFile(path.join(artifactRoot, 'quick-scan.json'), JSON.stringify(library, null, 2));
  const entry = library.entries.find(e => path.resolve(e.path).replace(/^\\\\\?\\/, '').toLowerCase() === modelA.toLowerCase());
  assert.ok(entry, 'Quick scan finds existing ComfyUI SDXL checkpoint');
  const count = library.entries.length; await invoke('model_scan_quick'); library = await scanDone(invoke);
  assert.equal(library.entries.length, count);
  record('Quick scan finds real model folders and HF cache; repeat scan does not duplicate references', `${count} entries, ${library.scan.roots.length} roots`);

  const runtime = path.join(artifactRoot, 'image-runtime');
  const extra = path.join(runtime, 'untrusted.dll'); await fs.writeFile(extra, 'not executable');
  let probe = await invoke('image_probe', { path: modelA }); assert.equal(probe.ready, false); assert.ok(probe.missing.includes('image_runtime_invalid'));
  await fs.unlink(extra);
  const manifest = JSON.parse(await fs.readFile(path.join(import.meta.dirname, '../src-tauri/image-runtime.json'), 'utf8'));
  const licenseName = Object.keys(manifest.files).find(n => n.toLowerCase().includes('license'));
  const licenseFile = path.join(runtime, licenseName), originalLicense = await fs.readFile(licenseFile);
  try {
    await fs.appendFile(licenseFile, '\nchanged');
    probe = await invoke('image_probe', { path: modelA }); assert.equal(probe.ready, false); assert.ok(probe.missing.includes('image_runtime_invalid'));
  } finally { await fs.writeFile(licenseFile, originalLicense); }
  record('Image preflight rejects extra DLLs and changed pinned runtime files before execution');

  const incomplete = path.join(artifactRoot, 'incomplete-image.safetensors');
  const header = Buffer.from(JSON.stringify({ weight: { dtype: 'F32', shape: [1], data_offsets: [0, 4] } }));
  const length = Buffer.alloc(8); length.writeBigUInt64LE(BigInt(header.length)); await fs.writeFile(incomplete, Buffer.concat([length, header, Buffer.alloc(4)]));
  probe = await invoke('image_probe', { path: incomplete }); assert.equal(probe.ready, false);
  assert.deepEqual(probe.missing, ['SDXL UNet', 'CLIP-L', 'CLIP-G', 'VAE encoder', 'VAE decoder']);
  await assert.rejects(invoke('image_generate', { request: { ...request, modelPath: incomplete } }), /image_not_ready/);
  await assert.rejects(invoke('image_generate', { request: { ...request, width: 4160 } }), /image_dimensions/);
  await assert.rejects(invoke('image_generate', { request: { ...request, sampler: '--rpc-servers' } }), /image_parameters/);
  record('Preflight names missing SDXL components; incomplete model and invalid generation parameters cannot execute');

  await page.locator(`[data-model-id="${entry.id}"]`).getByRole('button', { name: 'Check in Studio', exact: true }).click();
  assert.equal(await page.getByLabel('SDXL model file', { exact: true }).inputValue(), entry.path);
  await page.getByRole('button', { name: 'Check readiness', exact: true }).click();
  await page.getByTestId('image-readiness').getByText('Ready to attempt generation', { exact: true }).waitFor({ timeout: 30000 });
  await page.getByLabel('Image prompt', { exact: true }).fill(request.prompt);
  await page.getByLabel('Negative prompt', { exact: true }).fill(request.negativePrompt);
  await page.getByLabel('Image steps', { exact: true }).fill(String(request.steps));
  const beforeGeneration = await invoke('bootstrap');
  await page.getByLabel('Image prompt', { exact: true }).press('Control+Enter');
  let first;
  for (let i = 0; i < 200; i++) { first = (await invoke('image_jobs'))[0]; if (first) break; await pause(100); }
  assert.ok(first);
  const alternateTemporary = path.join(artifactRoot, 'temporary-after-enqueue');
  await invoke('save_settings', { settings: { ...beforeGeneration.settings, storageOverrides: { ...beforeGeneration.settings.storageOverrides, temporary: alternateTemporary } } });
  await page.getByLabel('Image prompt', { exact: true }).fill('Changed form; running job must keep its original prompt');
  await waitJob(invoke, first.id, j => j.phase === 'sampling' && j.step > 0);
  assert.equal((await invoke('image_jobs'))[0].request.prompt, request.prompt);
  first = await waitJob(invoke, first.id, j => j.status === 'completed');
  assert.equal(path.resolve(first.workingDirectory), path.resolve(beforeGeneration.paths.temporary, 'image-results', first.id));
  assert.equal(path.resolve(first.output), path.resolve(first.workingDirectory, 'image.png'));
  assert.match(first.modelSha256, /^[a-f0-9]{64}$/); assert.equal(first.request.steps, 10); assert.equal(first.device.split('\t')[0].startsWith('Vulkan'), true);
  const png = await fs.readFile(first.output); assert.equal(png.readUInt32BE(16), 512); assert.equal(png.readUInt32BE(20), 512);
  assert.equal(hash(Buffer.from((await invoke('image_output', { id: first.id })).split(',')[1], 'base64')), hash(png));
  await page.getByAltText('Locally generated image', { exact: true }).waitFor({ timeout: 10000 });
  await page.getByTestId('image-result').scrollIntoViewIfNeeded(); await page.screenshot({ path: path.join(artifactRoot, 'image-generated.png') });
  await noWorker(artifactRoot);
  record('Actual Ctrl+Enter generation through native Studio UI returns a 512x512 PNG in configured temporary storage; measured steps and immutable prompt; worker exits', `${(first.elapsedMs / 1000).toFixed(1)} seconds, SHA-256 ${first.modelSha256}`);

  await page.getByRole('button', { name: 'Save to gallery', exact: true }).click();
  await page.getByRole('button', { name: 'Saved to gallery', exact: true }).waitFor();
  first = (await invoke('image_jobs')).find(j => j.id === first.id);
  assert.ok(first.savedPath); assert.equal(hash(await fs.readFile(first.savedPath)), hash(png));
  const metadata = JSON.parse(await fs.readFile(path.join(path.dirname(first.savedPath), 'metadata.json'), 'utf8'));
  assert.equal(metadata.modelSha256, first.modelSha256); assert.deepEqual(metadata.request, first.request);
  assert.equal(await invoke('image_save', { id: first.id }), first.savedPath);
  await fs.rename(first.output, first.output + '.test-moved');
  assert.equal(hash(Buffer.from((await invoke('image_output', { id: first.id })).split(',')[1], 'base64')), hash(png));
  await page.getByRole('button', { name: 'Gallery', exact: true }).first().click();
  await page.getByAltText('Gallery image', { exact: true }).waitFor();
  record('Gallery saves separate PNG and immutable metadata; repeat save is idempotent; preview survives missing temporary output');

  const second = await invoke('image_generate', { request: { ...request, modelPath: modelB, steps: 4, guidance: 2, sampler: 'dpm++2m' } });
  const secondDone = await waitJob(invoke, second.id, j => j.status === 'completed');
  assert.equal(path.resolve(secondDone.workingDirectory), path.resolve(alternateTemporary, 'image-results', second.id));
  await invoke('save_settings', { settings: beforeGeneration.settings });
  const third = await invoke('image_generate', { request: { ...request, steps: 2 } });
  const thirdDone = await waitJob(invoke, third.id, j => j.status === 'completed');
  assert.equal(path.resolve(thirdDone.workingDirectory), path.resolve(beforeGeneration.paths.temporary, 'image-results', third.id));
  assert.equal(thirdDone.modelSha256, first.modelSha256); assert.notEqual(secondDone.modelSha256, first.modelSha256);
  await fs.writeFile(path.join(artifactRoot, 'image-results.json'), JSON.stringify([first, secondDone, thirdDone], null, 2));
  await noWorker(artifactRoot);
  record('Real model A/B/A switch completes with both existing SDXL checkpoints, no weights downloaded', `DreamShaper ${secondDone.modelSha256}`);

  const cancelled = await invoke('image_generate', { request: { ...request, steps: 60 } });
  await waitJob(invoke, cancelled.id, j => j.phase === 'sampling' && j.step > 0);
  assert.equal((await workers(artifactRoot)).length, 1);
  await page.getByRole('button', { name: /^Jobs/ }).first().click();
  const row = page.locator(`[data-image-job-id="${cancelled.id}"]`); await row.waitFor();
  await row.getByRole('progressbar', { name: 'Image job progress' }).waitFor();
  const label = await page.evaluate(() => window.__TAURI_INTERNALS__.metadata.currentWindow.label);
  await invoke('plugin:window|close', { label });
  await page.getByRole('dialog', { name: 'Close Local Studio?' }).getByRole('button', { name: 'Cancel', exact: true }).click();
  assert.equal((await invoke('image_jobs')).find(j => j.id === cancelled.id).status, 'running');
  assert.equal((await workers(artifactRoot)).length, 1);
  record('Cancel in consolidated exit dialog keeps the actual denoising worker running');
  await row.getByRole('button', { name: 'Cancel image job', exact: true }).click();
  assert.equal((await waitJob(invoke, cancelled.id, j => j.status === 'cancelled')).status, 'cancelled');
  await noWorker(artifactRoot); await assert.rejects(invoke('image_output', { id: cancelled.id }));
  record('Cancellation from central Jobs during real denoising stops the native worker and exposes no finished image');
  const interrupted = await invoke('image_generate', { request: { ...request, steps: 60 } });
  await waitJob(invoke, interrupted.id, j => j.phase === 'sampling' && j.step > 0);
  assert.equal((await workers(artifactRoot)).length, 1);
  return { interrupted: interrupted.id, saved: first.id, pngHash: hash(png) };
}

export async function checkImageRestart(page, artifactRoot, invoke, record, fixture) {
  await noWorker(artifactRoot);
  const jobs = await invoke('image_jobs');
  assert.deepEqual(jobs.map(j => j.createdAt), jobs.map(j => j.createdAt).sort().reverse());
  assert.equal(jobs.find(j => j.id === fixture.interrupted).status, 'interrupted');
  assert.equal(hash(Buffer.from((await invoke('image_output', { id: fixture.saved })).split(',')[1], 'base64')), fixture.pngHash);
  await page.getByRole('button', { name: 'Gallery', exact: true }).first().click();
  await page.getByAltText('Gallery image', { exact: true }).waitFor();
  await page.screenshot({ path: path.join(artifactRoot, 'image-gallery-restart.png') });
  record('Parent crash kills inference child; restart marks interrupted job and restores saved gallery image without restarting inference');
}
