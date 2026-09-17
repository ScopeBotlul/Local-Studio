// Runs the real Windows executable with an isolated database and WebView2 profile.
// CDP is enabled only for this child process, never in normal app configuration.
import { chromium } from '@playwright/test';
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { promises as fs } from 'node:fs';
import path from 'node:path';
import net from 'node:net';
import assert from 'node:assert/strict';
import { checkHuggingFace } from './check-huggingface.mjs';
import { checkDownloads, checkDownloadRestart } from './check-downloads.mjs';
import { checkIntegratedOAuth } from './check-integrated-oauth.mjs';
import { checkHfBrowser, checkHfBrowserRestart } from './check-hf-browser.mjs';
import { checkSettingsRace } from './check-settings-race.mjs';
import { checkCloseDuringSave } from './check-close-during-save.mjs';

const root = path.resolve(import.meta.dirname, '..');
const exe = process.env.LOCAL_STUDIO_TEST_EXE || path.join(root, 'src-tauri/target/debug/local-studio.exe');
const artifactRoot = path.join(root, '.artifacts', `native-${Date.now()}`);
await fs.mkdir(artifactRoot, { recursive: true });
const launchExe = path.join(artifactRoot, 'Local Studio.exe');
await fs.copyFile(exe, launchExe);
await fs.writeFile(path.join(artifactRoot, 'portable.marker'), 'Local Studio isolated native test\n');
const report = { startedAt: new Date().toISOString(), executable: exe, checks: [], artifacts: artifactRoot };
const record = (name, detail = '') => { report.checks.push({ name, passed: true, detail }); console.log(`PASS ${name}${detail ? `: ${detail}` : ''}`); };
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
let app;
let browser;
let page;
const errors = [];

async function availablePort() {
  const server = net.createServer();
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const port = server.address().port;
  await new Promise(resolve => server.close(resolve));
  return port;
}
async function launch() {
  const port = await availablePort();
  app = spawn(launchExe, [], { cwd: root, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'], env: {
    ...process.env,
    LOCAL_STUDIO_CONFIG_DIR: path.join(artifactRoot, 'config'),
    WEBVIEW2_USER_DATA_FOLDER: path.join(artifactRoot, 'webview'),
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}`,
  } });
  let output = '';
  app.stdout.on('data', chunk => { output += chunk; });
  app.stderr.on('data', chunk => { output += chunk; });
  app.on('error', error => { output += String(error); });
  for (let attempt = 0; attempt < 80; attempt++) {
    if (app.exitCode !== null) throw new Error(`Native app exited ${app.exitCode}: ${output}`);
    try { browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`, { timeout: 1000 }); break; }
    catch { await pause(500); }
  }
  if (!browser) throw new Error(`WebView2 CDP did not start: ${output}`);
  for (let attempt = 0; attempt < 40; attempt++) {
    page = browser.contexts()[0]?.pages()[0];
    if (page) break;
    await pause(250);
  }
  assert.ok(page, 'Native WebView page exists');
  page.on('pageerror', error => errors.push(error.message));
  await page.waitForFunction(() => typeof window.__TAURI_INTERNALS__?.invoke === 'function', { timeout: 30000 });
  await page.waitForFunction(() => document.body.innerText.length > 100, { timeout: 30000 });
  await fs.writeFile(path.join(artifactRoot, 'native-output.txt'), output);
}
async function invoke(command, args = {}) {
  return page.evaluate(({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args), { command, args });
}
async function stop(clean) {
  try {
    if (clean && page && !page.isClosed()) await invoke('mark_clean_exit');
  } finally {
    if (app && app.exitCode === null) {
      const exited = new Promise(resolve => app.once('exit', resolve));
      app.kill();
      await exited;
    }
    if (browser) await browser.close().catch(() => {});
    browser = undefined;
    page = undefined;
  }
}
async function waitJob(id, statuses, timeout = 15000) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    const jobs = await invoke('list_jobs');
    const job = jobs.find(item => item.id === id);
    if (job && statuses.includes(job.status)) return job;
    await pause(75);
  }
  throw new Error(`Job ${id} did not reach ${statuses.join('/')}`);
}

try {
  await launch();
  let snapshot = await invoke('bootstrap');
  assert.ok(snapshot.hardware.totalMemoryBytes > 0);
  assert.ok(snapshot.hardware.logicalCores > 0);
  assert.ok(snapshot.hardware.disks.length > 0);
  assert.ok(snapshot.databasePath.startsWith(artifactRoot));
  assert.equal(snapshot.settings.setupComplete, false);
  record('Native launch, real hardware and isolated SQLite', snapshot.hardware.cpu);
  report.hardware = snapshot.hardware;
  report.version = snapshot.version;
  await page.screenshot({ path: path.join(artifactRoot, '01-setup.png') });

  const setup = page.getByRole('button', { name: /Studio einrichten|Set up studio/ });
  await setup.waitFor({ state: 'visible' });
  await setup.click();
  await page.getByRole('button', { name: /Einstellungen|Settings/, exact: true }).first().waitFor();
  for (let attempt = 0; attempt < 30; attempt++) {
    snapshot = await invoke('bootstrap');
    if (snapshot.settings.setupComplete) break;
    await pause(100);
  }
  assert.equal(snapshot.settings.setupComplete, true);
  record('First-run setup through real UI');

  await page.getByRole('button', { name: /Einstellungen|Settings/, exact: true }).first().click();
  await page.getByLabel(/Sprache|Language/, { exact: true }).selectOption('en');
  await page.getByLabel(/Design|Theme/, { exact: true }).selectOption('dark');
  await page.getByRole('button', { name: /Änderungen speichern|Save changes/ }).click();
  await page.getByText('Settings saved.', { exact: true }).waitFor();
  snapshot = await invoke('bootstrap');
  assert.equal(snapshot.settings.language, 'en');
  assert.equal(snapshot.settings.theme, 'dark');
  record('Language and theme saved through real UI');
  await page.screenshot({ path: path.join(artifactRoot, '02-settings-dark.png') });
  await page.keyboard.press('Control+=');
  await pause(500);
  assert.equal((await invoke('bootstrap')).settings.uiScale, 1.05);
  await page.keyboard.press('Control+0');
  await pause(500);
  assert.equal((await invoke('bootstrap')).settings.uiScale, 1);
  record('Keyboard zoom and reset persist');
  await page.getByRole('button', { name: 'Settings', exact: true }).first().click();
  const originalRoot = snapshot.settings.dataRoot;
  const originalDb = snapshot.databasePath;
  const newDataRoot = path.join(artifactRoot, 'changed-data-root');
  await page.getByRole('textbox', { name: 'Data folder', exact: true }).fill(newDataRoot);
  await page.getByRole('button', { name: 'Save changes', exact: true }).click();
  for (let attempt = 0; attempt < 30; attempt++) {
    snapshot = await invoke('bootstrap');
    if (snapshot.settings.dataRoot === newDataRoot) break;
    await pause(100);
  }
  assert.equal(snapshot.settings.dataRoot, newDataRoot);
  assert.equal(snapshot.databasePath, originalDb);
  await fs.access(originalRoot);
  await fs.access(snapshot.paths.models);
  record('Data-root change preserves original directory and stable database');
  await checkSettingsRace(page, artifactRoot, invoke);
  record('Delayed settings save cannot be overwritten by concurrent zoom or form edits');

  await assert.rejects(invoke('save_settings', { settings: { ...snapshot.settings, uiScale: -10 } }));
  assert.equal((await invoke('bootstrap')).settings.uiScale, snapshot.settings.uiScale);
  record('Invalid settings rejected without altering persisted settings');

  const fixture = path.join(artifactRoot, 'hash-fixture.txt');
  const content = 'Local Studio: real isolated worker\nUTF-8: ÄÖÜ\n';
  await fs.writeFile(fixture, content);
  const expected = createHash('sha256').update(content).digest('hex');
  const queued = await invoke('enqueue_hash_job', { path: fixture });
  const completed = await waitJob(queued.id, ['completed', 'failed']);
  assert.equal(completed.status, 'completed', completed.error || 'Hash worker failed');
  assert.ok(completed.result?.includes(expected), `Expected SHA256 ${expected}; got ${completed.result}`);
  record('Isolated native worker computes exact SHA256', expected);
  await assert.rejects(invoke('enqueue_hash_job', { path: path.join(artifactRoot, 'absent-file') }));
  record('Nonexistent input rejected');

  await page.getByRole('button', { name: 'Jobs', exact: true }).first().click();
  await page.getByText('hash-fixture.txt', { exact: true }).first().waitFor();
  await page.screenshot({ path: path.join(artifactRoot, '03-jobs.png') });
  record('Completed worker job appears in native UI');

  const large = path.join(artifactRoot, 'cancel-fixture.bin');
  const handle = await fs.open(large, 'w');
  await handle.truncate(1024 * 1024 * 1024);
  await handle.close();
  const running = await invoke('enqueue_hash_job', { path: large });
  await waitJob(running.id, ['running']);
  const pending = await invoke('enqueue_hash_job', { path: large });
  await invoke('cancel_job', { id: pending.id });
  assert.equal((await waitJob(pending.id, ['cancelled'])).status, 'cancelled');
  await invoke('cancel_job', { id: running.id });
  assert.equal((await waitJob(running.id, ['cancelled'])).status, 'cancelled');
  record('Queued and running job cancellation');

  await stop(true);
  await launch();
  snapshot = await invoke('bootstrap');
  assert.equal(snapshot.settings.language, 'en');
  assert.equal(snapshot.settings.theme, 'dark');
  assert.equal(snapshot.recoveryAvailable, false);
  assert.ok(snapshot.jobs.some(job => job.id === completed.id && job.status === 'completed'));
  record('Settings and completed jobs persist across clean restart');

  const interrupted = await invoke('enqueue_hash_job', { path: large });
  await waitJob(interrupted.id, ['running']);
  await stop(false);
  await launch();
  snapshot = await invoke('bootstrap');
  assert.equal(snapshot.recoveryAvailable, true);
  assert.equal(snapshot.jobs.find(job => job.id === interrupted.id)?.status, 'interrupted');
  await invoke('dismiss_recovery');
  assert.equal((await invoke('bootstrap')).recoveryAvailable, false);
  record('Crash recovery detects interrupted worker without fake resume');
  await fs.unlink(large);
  await page.screenshot({ path: path.join(artifactRoot, '04-recovered.png') });
  const closeSettings = await checkCloseDuringSave(page, invoke);
  await stop(false);
  await launch();
  const afterNativeClose = await invoke('bootstrap');
  assert.equal(afterNativeClose.settings.theme, closeSettings.theme);
  assert.equal(afterNativeClose.settings.dataRoot, closeSettings.dataRoot);
  assert.equal(afterNativeClose.recoveryAvailable, false);
  record('Native close waits for settings write and path refresh; saved state survives restart');
  if (process.env.LOCAL_STUDIO_TEST_HF === '1') await checkHuggingFace(page, artifactRoot, invoke, record, app.pid);
  if (process.env.LOCAL_STUDIO_TEST_BROWSER === '1') {
    await checkIntegratedOAuth(page, browser, artifactRoot, invoke, record);
    await checkHfBrowser(page, browser, artifactRoot, invoke, record);
    await stop(true);
    await launch();
    await checkHfBrowserRestart(page, browser, invoke, record);
  }
  if (process.env.LOCAL_STUDIO_TEST_DOWNLOADS === '1') {
    const id = await checkDownloads(page, artifactRoot, invoke, record);
    await stop(true); await launch();
    await checkDownloadRestart(page, invoke, record, id);
  }
  assert.deepEqual(errors, [], 'No uncaught WebView errors');
  record('No uncaught frontend runtime errors');
  report.passed = true;
} catch (error) {
  report.passed = false;
  report.error = String(error.stack || error);
  console.error(report.error);
  if (page && !page.isClosed()) {
    report.visibleText = await page.locator('body').innerText().catch(() => 'unavailable');
    await page.screenshot({ path: path.join(artifactRoot, 'failure.png') }).catch(() => {});
  }
  process.exitCode = 1;
} finally {
  if (page && !page.isClosed()) await invoke('hf_logout').catch(error => console.error('Isolated credential cleanup:', String(error)));
  await stop(true).catch(error => console.error('Cleanup:', String(error)));
  report.finishedAt = new Date().toISOString();
  await fs.writeFile(path.join(artifactRoot, 'report.json'), JSON.stringify(report, null, 2));
  console.log(`Report: ${path.join(artifactRoot, 'report.json')}`);
}
