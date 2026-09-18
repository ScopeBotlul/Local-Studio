import {checkPrivacy26} from './check-privacy26.mjs';
import {checkCore25} from './check-core25.mjs';
import {checkReference25} from './check-reference25.mjs';
import {checkUpdatesBench25} from './check-updates-bench25.mjs';
import {checkCreative25} from './check-creative25.mjs';
import {checkAi24} from './check-ai24.mjs';
import {checkMenuUpdates} from './check-menu-updates.mjs';
const suites={privacy:checkPrivacy26,core:checkCore25,references:checkReference25,models:checkUpdatesBench25,editors:checkCreative25,ai:checkAi24,menu:checkMenuUpdates};
const suite=suites[process.argv[2]];
if(!suite)throw Error('Usage: node scripts/native-25.mjs core|references|models|editors|ai|menu');
// Runs the real Windows executable with an isolated database and WebView2 profile.
// CDP is enabled only for this child process, never in normal app configuration.
import { chromium } from '@playwright/test';
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { promises as fs } from 'node:fs';
import path from 'node:path';
import net from 'node:net';
import assert from 'node:assert/strict';

const root = path.resolve(import.meta.dirname, '..');
const exe = process.env.LOCAL_STUDIO_TEST_EXE || path.join(root, 'src-tauri/target/debug/local-studio.exe');
const artifactRoot = path.join(root, '.artifacts', `native-${Date.now()}`);
await fs.mkdir(artifactRoot, { recursive: true });
const launchExe = path.join(artifactRoot, 'Local Studio.exe');
await fs.copyFile(exe, launchExe);
const imageRuntime = path.join(path.dirname(exe), 'image-runtime');
if ((await fs.stat(imageRuntime).catch(() => null))?.isDirectory()) await fs.cp(imageRuntime, path.join(artifactRoot, 'image-runtime'), { recursive: true });
for(const kind of ['assistant','speech','video']) {const d=path.join(path.dirname(exe),kind+'-runtime');await fs.cp(d,path.join(artifactRoot,kind+'-runtime'),{recursive:true});}
await fs.writeFile(path.join(artifactRoot, 'portable.marker'), 'Local Studio isolated native test\n');
if (process.env.LOCAL_STUDIO_TEST_LIBRARY) {
  const config = path.join(artifactRoot, 'Local-Studio-Data/config');
  await fs.mkdir(config, { recursive: true });
  await fs.copyFile(process.env.LOCAL_STUDIO_TEST_LIBRARY, path.join(config, 'model-library.sqlite3'));
}

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
async function launch(args = []) {
  const port = await availablePort();
  app = spawn(launchExe, args, { cwd: root, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'], env: {
    ...process.env,
    LOCAL_STUDIO_CONFIG_DIR: path.join(artifactRoot, 'config'),
    WEBVIEW2_USER_DATA_FOLDER: path.join(artifactRoot, 'webview'),
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port} --disable-background-timer-throttling --disable-renderer-backgrounding --disable-backgrounding-occluded-windows --disable-features=CalculateNativeWinOcclusion`,
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
 await launch();const snapshot=await invoke('bootstrap');report.version=snapshot.version;
 await invoke('save_settings',{settings:{...snapshot.settings,setupComplete:true,language:'en',theme:'dark',autoUpdateCheck:false}});await page.reload();await page.getByRole('button',{name:'Gallery',exact:true}).first().waitFor();
 await suite({getPage:()=>page,invoke,stop,launch,artifactRoot,record,pid:()=>app.pid});
 assert.deepEqual(errors,[]);record('No uncaught frontend runtime errors');report.passed=true;
} catch (error) {
  report.passed = false;
  report.error = String(error.stack || error);
  console.error(report.error);
  if (page && !page.isClosed()) {
    report.visibleText = await page.locator('body').innerText().catch(() => 'unavailable');
    
  }
  process.exitCode = 1;
} finally {
  if (page && !page.isClosed()) await invoke('hf_logout').catch(error => console.error('Isolated credential cleanup:', String(error)));
  await stop(true).catch(error => console.error('Cleanup:', String(error)));
  report.finishedAt = new Date().toISOString();
  await fs.writeFile(path.join(artifactRoot, 'report.json'), JSON.stringify(report, null, 2));
  console.log(`Report: ${path.join(artifactRoot, 'report.json')}`);
}
