import assert from 'node:assert/strict';
import path from 'node:path';

// Hold actual IPC requests/responses; neither native results nor exit dialogs are mocked.
// This closes the isolated test window and returns settings to verify after relaunch.
export async function checkCloseDuringSave(page, invoke) {
  const previous = (await invoke('bootstrap')).settings;
  const jobs = await invoke('list_jobs');
  assert.equal(jobs.some(job => ['queued', 'running'].includes(job.status)), false,
    'Close-during-save check requires no active jobs');
  const expected = {
    theme: previous.theme === 'light' ? 'dark' : 'light',
    dataRoot: path.join(previous.dataRoot, 'close-during-save'),
  };
  await page.getByRole('button', { name: /Einstellungen|Settings/, exact: true }).first().click();
  await page.locator('#theme').selectOption(expected.theme);
  await page.getByRole('textbox', { name: /Datenordner|Data folder/, exact: true }).fill(expected.dataRoot);
  await page.evaluate(async () => {
    const original = window.fetch;
    const state = {
      original, saveStarted: false, refreshStarted: false, cleanExitCalls: 0,
      closeRequests: 0, releaseSave: null, releaseRefresh: null, listenerId: null,
    };
    window.__closeSaveTest = state;
    window.fetch = async (input, options) => {
      const url = new URL(typeof input === 'string' ? input : input.url);
      if (url.hostname === 'ipc.localhost') {
        if (url.pathname === '/save_settings' && !state.saveStarted) {
          state.saveStarted = true;
          await new Promise(resolve => { state.releaseSave = resolve; });
        } else if (url.pathname === '/bootstrap' && state.saveStarted && !state.refreshStarted) {
          const response = await original.call(window, input, options);
          state.refreshStarted = true;
          await new Promise(resolve => { state.releaseRefresh = resolve; });
          return response;
        } else if (url.pathname === '/mark_clean_exit') {
          state.cleanExitCalls++;
        }
      }
      return original.call(window, input, options);
    };
    const label = window.__TAURI_INTERNALS__.metadata.currentWindow.label;
    // Same subscription payload as @tauri-apps/api/event.listen; observation only.
    state.listenerId = await window.__TAURI_INTERNALS__.invoke('plugin:event|listen', {
      event: 'tauri://close-requested', target: { kind: 'Window', label },
      handler: window.__TAURI_INTERNALS__.transformCallback(() => { state.closeRequests++; }),
    });
  });
  try {
    await page.getByRole('button', { name: /Änderungen speichern|Save changes/, exact: true }).click();
    await page.waitForFunction(() => window.__closeSaveTest.saveStarted);
    const label = await page.evaluate(() => window.__TAURI_INTERNALS__.metadata.currentWindow.label);
    // Exact command/payload used by getCurrentWindow().close(), firing the real close handler.
    await invoke('plugin:window|close', { label });
    await page.waitForFunction(() => window.__closeSaveTest.closeRequests >= 1);
    await page.waitForTimeout(350);
    assert.equal(page.isClosed(), false, 'The window must stay open until the settings write finishes');
    assert.equal(await page.evaluate(() => window.__closeSaveTest.cleanExitCalls), 0,
      'Clean exit must not run while settings are still being written');

    await page.evaluate(() => window.__closeSaveTest.releaseSave());
    await page.waitForFunction(() => window.__closeSaveTest.refreshStarted);
    await page.waitForTimeout(350);
    assert.equal(page.isClosed(), false, 'The window must also await applying the refreshed storage paths');
    assert.equal(await page.evaluate(() => window.__closeSaveTest.cleanExitCalls), 0,
      'Finishing the raw settings write alone must not allow exit');
    assert.equal(await page.locator('#theme').isDisabled(), true,
      'Settings editing stays locked until the complete transaction is applied');

    const closed = page.waitForEvent('close', { timeout: 15000 });
    await page.evaluate(() => window.__closeSaveTest.releaseRefresh());
    await closed;
    return expected;
  } finally {
    if (!page.isClosed()) {
      await page.evaluate(async () => {
        const state = window.__closeSaveTest;
        if (!state) return;
        window.fetch = state.original;
        if (state.listenerId !== null) {
          await window.__TAURI_INTERNALS__.invoke('plugin:event|unlisten', {
            event: 'tauri://close-requested', eventId: state.listenerId,
          });
        }
        state.releaseSave?.();
        state.releaseRefresh?.();
        delete window.__closeSaveTest;
      }).catch(() => {});
    }
  }
}
