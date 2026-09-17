import assert from 'node:assert/strict';
import path from 'node:path';

// Inject response latency into real IPC; no settings or backend results are mocked.
export async function checkSettingsRace(page, artifactRoot, invoke) {
  const previous = (await invoke('bootstrap')).settings;
  const newRoot = path.join(artifactRoot, 'settings-race-data');
  await page.locator('#theme').selectOption('light');
  await page.getByRole('textbox', { name: 'Data folder', exact: true }).fill(newRoot);
  await page.evaluate(() => {
    const original = window.fetch;
    const state = { original, started: false, completed: 0, calls: [], release: null };
    window.__settingsRaceTest = state;
    window.fetch = async (input, options) => {
      const url = new URL(typeof input === 'string' ? input : input.url);
      if (url.hostname === 'ipc.localhost' && url.pathname === '/save_settings') {
        state.calls.push(JSON.parse(options.body).settings);
        if (!state.started) {
          state.started = true;
          await new Promise(resolve => { state.release = resolve; });
        }
        const result = await original.call(window, input, options);
        state.completed++;
        return result;
      }
      return original.call(window, input, options);
    };
  });
  try {
    await page.getByRole('button', { name: 'Save changes', exact: true }).click();
    await page.waitForFunction(() => window.__settingsRaceTest.started);
    const disabledFields = await page.locator('#language, #theme, #accent, #scale, .settings-page .path-field input, .settings-page .path-field button').evaluateAll(
      elements => elements.every(element => element.matches(':disabled')),
    );
    await page.keyboard.press('Control+=');
    // Exceed the old 220 ms debounce while the initial save remains held.
    await page.waitForTimeout(350);
    await page.evaluate(() => window.__settingsRaceTest.release());
    await page.waitForFunction(() => window.__settingsRaceTest.completed >= 1);
    await page.getByText('Settings saved.', { exact: true }).waitFor();
    // Let any queued stale write finish; then inspect the actual persisted database.
    await page.waitForTimeout(700);
    const current = (await invoke('bootstrap')).settings;
    assert.equal(current.theme, 'light', 'Concurrent zoom must not restore the previous theme');
    assert.equal(current.dataRoot, newRoot, 'Concurrent zoom must not restore the previous data folder');
    assert.equal(current.uiScale, previous.uiScale, 'Zoom remains unchanged while explicit settings save is pending');
    assert.equal(disabledFields, true, 'Settings fields must not accept edits that would be discarded during save');
    assert.equal(await page.locator('#theme').inputValue(), 'light');
    assert.equal(await page.evaluate(() => window.__settingsRaceTest.calls.length), 1, 'No stale second settings write may be queued');
  } finally {
    await page.evaluate(() => {
      const state = window.__settingsRaceTest;
      state?.release?.();
      if (state) window.fetch = state.original;
      delete window.__settingsRaceTest;
    });
  }
  // Restore the intended test session through the UI, using the same real save path.
  await page.locator('#theme').selectOption(previous.theme);
  await page.getByRole('textbox', { name: 'Data folder', exact: true }).fill(previous.dataRoot);
  await page.getByRole('button', { name: 'Save changes', exact: true }).click();
  await page.waitForFunction(() => !document.querySelector('#theme').matches(':disabled'));
  const restored = (await invoke('bootstrap')).settings;
  assert.equal(restored.theme, previous.theme);
  assert.equal(restored.dataRoot, previous.dataRoot);
}
