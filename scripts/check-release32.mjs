import assert from 'node:assert/strict';

export async function checkRelease32({getPage,invoke,record}) {
  const page = getPage();
  const snapshot = await invoke('bootstrap');
  assert.equal(snapshot.version, '0.32.0');
  assert.equal(await page.getByRole('button',{name:/18+.*Locked|18+.*Gesperrt/}).count(), 0);
  await assert.rejects(invoke('privacy_status'));
  record('Packaged 0.32.0 starts without an 18+ lock control or public lock IPC');

  await invoke('save_settings',{settings:{...snapshot.settings,autoUpdateCheck:true}});
  assert.equal((await invoke('bootstrap')).settings.autoUpdateCheck, true);
  const update = await invoke('update_check');
  assert(['current','available'].includes(update.phase), `unexpected update phase: ${update.phase}`);
  record('Automatic update preference persists and the signed GitHub update channel responds');
}
