import assert from 'node:assert/strict';
import { promises as fs } from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
const exec = promisify(execFile), pause = ms => new Promise(r => setTimeout(r, ms));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
async function until(fn, timeout = 25000) { const end = Date.now() + timeout; while (Date.now() < end) { if (await fn()) return; await pause(100); } throw new Error('Media workflow condition timed out'); }
export async function checkMediaWorkflow({ getPage, invoke, stop, launch, artifactRoot, record }) {
  let page = getPage();
  // Route a controlled native drag payload through Tauri's real registered callback.
  // This tests application routing and disk copies, not physical Explorer/OLE delivery.
  async function prepare() {
    await page.addInitScript(() => {
      const original = window.fetch;
      window.workflowTest = { dragHandler: null, leaveHandler: null, dialog: 'Ok' };
      window.fetch = (input, options) => {
        const url = new URL(typeof input === 'string' ? input : input.url);
        const command = decodeURIComponent(url.pathname).slice(1);
        if (url.hostname === 'ipc.localhost') {
          if (command === 'plugin:event|listen') { const body = JSON.parse(options.body); if (body.event === 'tauri://drag-drop') window.workflowTest.dragHandler = body.handler; if (body.event === 'tauri://drag-leave') window.workflowTest.leaveHandler = body.handler; }
          if (command === 'plugin:dialog|message') return Promise.resolve(new Response(JSON.stringify(window.workflowTest.dialog), { headers: { 'Content-Type': 'application/json', 'Tauri-Response': 'ok' } }));
        }
        return original.call(window, input, options);
      };
    });
    await page.reload(); await page.getByRole('button', { name: 'Gallery', exact: true }).first().waitFor();
  }
  await prepare();
  const png = await page.evaluate(() => { const c = document.createElement('canvas'); c.width = 160; c.height = 90; c.getContext('2d').fillRect(0, 0, 160, 90); return c.toDataURL('image/png').split(',')[1]; });
  const source = path.join(artifactRoot, 'drop-source.png'); await fs.writeFile(source, Buffer.from(png, 'base64'));
  async function drop(selector, paths) {
    await page.locator(selector).scrollIntoViewIfNeeded();
    await until(() => page.evaluate(() => typeof window.workflowTest?.dragHandler === 'number' && typeof window.workflowTest?.leaveHandler === 'number'));
    const bounds = await page.locator(selector).boundingBox(); const factor = await invoke('plugin:window|scale_factor', { label: 'main' });
    await page.evaluate(({ paths, x, y }) => {
      window.__TAURI_INTERNALS__.runCallback(window.workflowTest.dragHandler, { event: 'tauri://drag-drop', id: 0, payload: { paths, position: { x, y } } });
      window.__TAURI_INTERNALS__.runCallback(window.workflowTest.leaveHandler, { event: 'tauri://drag-leave', id: 0, payload: null });
    }, { paths, x: (bounds.x + 30) * factor, y: (bounds.y + 30) * factor });
  }
  await drop('.project-toolbar', [source]);
  await until(async () => (await invoke('project_snapshot'))?.assets.length === 1);
  const project = await invoke('project_snapshot'); assert.equal(project.assets[0].sha256, hash(Buffer.from(png, 'base64')));
  if (await page.locator('.project-title').getAttribute('aria-expanded') !== 'true') await page.locator('.project-title').click();
  await page.getByLabel('Rename project', { exact: true }).fill('Workflow renamed'); await page.getByRole('button', { name: 'Change name', exact: true }).click();
  await until(async () => (await invoke('project_snapshot'))?.name === 'Workflow renamed');
  await page.getByRole('button', { name: 'Create recovery point', exact: true }).count().then(async count => { if (!count || !await page.getByRole('button', { name: 'Create recovery point', exact: true }).isVisible()) await page.locator('.project-recovery summary').filter({ hasText: 'Recovery points' }).click(); });
  await page.getByRole('button', { name: 'Create recovery point', exact: true }).click(); await page.getByText('Recovery point created.', { exact: true }).waitFor();
  const checkpoint = (await invoke('project_history'))[0]; assert.equal(checkpoint.media, 1);
  await page.getByRole('button', { name: 'Remove from project: drop-source.png', exact: true }).click(); await until(async () => !(await invoke('project_snapshot')).assets.length);
  await page.getByText('Restore removed media (1)', { exact: true }).click(); await page.getByRole('button', { name: 'Restore media', exact: true }).click();
  await until(async () => (await invoke('project_snapshot')).assets.length === 1);
  record('Native project file-drop route copies bytes and can create a project; rename, explicit recovery point and removed-media restoration work through UI');
  await page.locator('.project-title').click();
  await page.getByRole('button', { name: 'Gallery', exact: true }).first().click();
  await drop('.gallery-page h1', [source]);
  const gallery = (await invoke('bootstrap')).paths.gallery; await until(async () => !!await fs.stat(path.join(gallery, 'drop-source.png')).catch(() => null));
  await page.getByLabel('Search files', { exact: true }).fill('drop-source'); await until(async () => await page.locator('.gallery-file').count() === 1);
  await page.getByRole('button', { name: 'Add selection to project', exact: true }).click(); await until(async () => (await invoke('project_snapshot')).assets.length === 2);
  const listing = await invoke('gallery_list', { query: { folder: '', search: 'drop-source', kind: 'all', recursive: true, offset: 0 } }); const entry = listing.entries[0];
  await assert.rejects(invoke('project_add_gallery', { id: project.id, rootId: listing.rootId, targets: [{ path: entry.path, fileId: entry.fileId, version: 'stale' }] }));
  assert.equal(hash(await fs.readFile(source)), hash(await fs.readFile(path.join(gallery, 'drop-source.png'))));
  record('Native gallery file-drop route copies originals; current gallery selection goes straight to project; stale gallery identity is rejected');
  const generated = (await invoke('image_jobs')).find(j => j.status === 'completed' && !j.discarded && (j.savedPath || j.output));
  if (generated) {
    await page.getByRole('button', { name: /^Jobs/ }).first().click();
    await page.locator(`[data-image-job-id="${generated.id}"]`).getByRole('button', { name: 'Open in Studio', exact: true }).click();
    await page.getByRole('button', { name: 'Add image to project', exact: true }).click(); await until(async () => (await invoke('project_snapshot')).assets.length === 3);
    const p = await invoke('project_snapshot'); assert.equal(p.assets.at(-1).sha256, hash(await fs.readFile(generated.savedPath || generated.output)));
    record('An actual locally generated SDXL image is copied directly from Studio into project with identical bytes');
  }
  await stop(false); await launch(); page = getPage(); await prepare();
  await page.getByRole('button', { name: 'Resume project', exact: true }).click(); await until(async () => !(await invoke('project_snapshot')).recovery);
  assert.equal((await invoke('project_snapshot')).name, 'Workflow renamed');
  if (await page.locator('.project-title').getAttribute('aria-expanded') !== 'true') await page.locator('.project-title').click();
  await page.locator('.project-recovery summary').filter({ hasText: 'Recovery points' }).click();
  const pointRow = page.locator('.project-history-point').filter({ hasText: checkpoint.name }).filter({ hasText: `${checkpoint.media} media` }).first();
  await pointRow.getByRole('button', { name: 'Restore snapshot', exact: true }).click(); await until(async () => (await invoke('project_snapshot')).assets.length === 1);
  await page.locator('.project-panel').screenshot({ path: path.join(artifactRoot, 'workflow-recovery.png') });
  record('Project name, media and recovery history survive actual process termination; an older one-media snapshot restores through the UI');
  await page.getByRole('button', { name: 'Close project', exact: true }).click(); await until(async () => await invoke('project_snapshot') === null);
  // Age only isolated test records. No wall-clock waiting and no user database modifications.
  const snapshot = await invoke('bootstrap'); assert(snapshot.databasePath.startsWith(artifactRoot));
  const sentinels = [snapshot.paths.models, snapshot.paths.downloads, snapshot.paths.gallery].map((folder, i) => path.join(folder, `keep-${i}.bin`)); for (const file of sentinels) await fs.writeFile(file, 'must stay');
  for (let n = 0; n < 6; n++) { await invoke('project_new', { name: `Cleanup fixture ${n}`, request: null, confirmed: true }); await invoke('project_add', { sources: [source] }); await invoke('project_checkpoint'); }
  await invoke('project_close', { confirmed: true }); await stop(true);
  const db = path.join(path.dirname(snapshot.databasePath), 'projects.sqlite3');
  await exec('python', ['-c', "import sqlite3,sys,time; c=sqlite3.connect(sys.argv[1]); old=int(time.time())-9*86400; c.execute('UPDATE project_locations SET touched=?',(old,)); c.execute('UPDATE project_history SET at=?',(old,)); c.commit()", db]);
  await launch(); page = getPage(); await prepare();
  await page.getByRole('button', { name: 'Settings', exact: true }).first().click();
  const automatic = page.getByLabel('Automatically clean old, unneeded working files', { exact: true }); await automatic.uncheck(); await page.getByRole('button', { name: 'Save changes', exact: true }).click();
  await until(async () => !(await invoke('bootstrap')).settings.autoCleanup);
  const preview = await invoke('storage_cleanup_preview'); assert(preview.files.length >= 3); assert(preview.protectedBytes > 0);
  assert.equal((await invoke('storage_cleanup_auto')).deleted, 0); for (const f of preview.files) await fs.access(f.path);
  await page.getByRole('button', { name: 'Inspect storage', exact: true }).click(); await page.getByText('Review affected files', { exact: true }).waitFor();
  await page.getByText('Review affected files', { exact: true }).click(); assert(await page.locator('.storage-cleanup-list li').count() >= 3);
  await page.locator('.storage-cleanup').screenshot({ path: path.join(artifactRoot, 'workflow-storage.png') });
  await page.evaluate(() => { window.workflowTest.dialog = 'Cancel'; }); await page.getByRole('button', { name: 'Clean now …', exact: true }).click();
  for (const f of preview.files) await fs.access(f.path);
  await page.evaluate(() => { window.workflowTest.dialog = 'Ok'; }); await page.getByRole('button', { name: 'Clean now …', exact: true }).click();
  await page.locator('.storage-cleanup [role="status"]').waitFor();
  for (const f of preview.files) await assert.rejects(fs.access(f.path)); for (const file of sentinels) assert.equal(await fs.readFile(file, 'utf8'), 'must stay');
  assert.equal((await invoke('storage_cleanup_preview')).files.length, 0); assert((await invoke('project_history')).filter(p => p.protected).length === 3);
  record('Native storage preview shows real eligible bytes/files; Cancel preserves all; confirmed cleanup removes only old registered copies and retains latest three points plus models/downloads/gallery');
  await page.locator('#temp-retention').fill('14'); await page.locator('#project-undo-limit').fill('50'); await automatic.check(); await page.getByRole('button', { name: 'Save changes', exact: true }).click();
  await until(async () => (await invoke('bootstrap')).settings.tempRetentionDays === 14);
  // Newly created empty snapshots release previously protected old sessions.
  for (let n = 0; n < 3; n++) { await invoke('project_new', { name: `Auto cleanup guard ${n}`, request: null, confirmed: true }); await invoke('project_checkpoint'); }
  await invoke('project_close', { confirmed: true });
  await stop(true);
  await exec('python', ['-c', "import sqlite3,sys,time; c=sqlite3.connect(sys.argv[1]); old=int(time.time())-20*86400; c.execute('UPDATE project_locations SET touched=? WHERE id IN (SELECT project_id FROM project_files)',(old,)); c.execute('UPDATE project_history SET at=? WHERE project_id IN (SELECT project_id FROM project_files)',(old,)); c.commit()", db]);
  await launch(); page = getPage();
  const autoPreview = await invoke('storage_cleanup_preview'); assert(autoPreview.files.length > 0);
  const auto = await invoke('storage_cleanup_auto'); assert(auto.deleted > 0); assert.equal(auto.errors.length, 0);
  assert.equal((await invoke('storage_cleanup_auto')).deleted, 0);
  for (const file of sentinels) assert.equal(await fs.readFile(file, 'utf8'), 'must stay');
  record('Enabled automatic-cleanup command removes eligible old working copies, preserves original sentinels and throttles repeated calls; disabled mode deletes nothing');
  const settings = (await invoke('bootstrap')).settings; assert(settings.autoCleanup); assert.equal(settings.tempRetentionDays, 14); assert.equal(settings.maxUndo, 50);
  record('Automatic-cleanup preference, retention and project-media restoration limit persist across native restart');
}
