import { describe, expect, it, vi, afterEach } from 'vitest';
import { clampScale, errorMessage, fileName, formatBytes, formatGigabytes, totalFileBytes, formatDate, initialLanguage, isActiveJob } from './helpers';
import { translations } from './i18n';
import type { JobStatus } from './types';

afterEach(() => vi.unstubAllGlobals());

describe('job presentation and exit protection', () => {
  it('treats both queued and running jobs as active, and only those jobs', () => {
    const statuses: JobStatus[] = ['queued', 'running', 'completed', 'cancelled', 'failed', 'interrupted'];
    expect(statuses.filter(status => isActiveJob({ status }))).toEqual(['queued', 'running']);
  });

  it('handles native Windows and Unix file paths without hiding Unicode filenames', () => {
    expect(fileName('D:\\Meine Modelle\\überprüfung.safetensors')).toBe('überprüfung.safetensors');
    expect(fileName('/home/media/hello.world.png')).toBe('hello.world.png');
    expect(fileName('C:\\some folder\\file with spaces.bin')).toBe('file with spaces.bin');
    expect(fileName('')).toBe('');
  });

  it('preserves backend diagnostics and native errors', () => {
    expect(errorMessage(new Error('Access denied'))).toBe('Access denied');
    expect(errorMessage('Worker exited unexpectedly')).toBe('Worker exited unexpectedly');
  });
});

describe('persisted interface preferences', () => {
  it('bounds repeated keyboard zoom and resets invalid values safely', () => {
    expect(clampScale(0.7)).toBe(0.75);
    expect(clampScale(1.55)).toBe(1.5);
    expect(clampScale(1.15000000002)).toBe(1.15);
    expect(clampScale(Number.NaN)).toBe(1);
    expect(clampScale(Number.POSITIVE_INFINITY)).toBe(1);
  });

  it('uses German for supported German locales and English as the fallback', () => {
    for (const locale of ['de-DE', 'de-AT', 'de-CH']) {
      vi.stubGlobal('navigator', { language: locale });
      expect(initialLanguage()).toBe('de');
    }
    for (const locale of ['en-US', 'fr-FR', 'ja-JP']) {
      vi.stubGlobal('navigator', { language: locale });
      expect(initialLanguage()).toBe('en');
    }
  });
});

describe('real hardware and history formatting', () => {
  it('keeps missing, empty and fractional capacity distinct and localized', () => {
    expect(formatBytes(0, 'en')).toBe('0 B');
    expect(formatBytes(-1, 'en')).toBe('—');
    expect(formatBytes(Number.NaN, 'de')).toBe('—');
    expect(formatBytes(1.5 * 1024 ** 3, 'en')).toBe('1.5 GiB');
    expect(formatBytes(1.5 * 1024 ** 3, 'de')).toBe('1,5 GiB');
    expect(formatBytes(8 * 1024 ** 4, 'en')).toBe('8 TiB');
  });

  it('does not invent a date when an unexpected backend timestamp arrives', () => {
    expect(formatDate('invalid timestamp', 'de')).toBe('invalid timestamp');
    expect(formatDate('2026-09-17T12:30:00Z', 'en')).not.toBe('Invalid Date');
  });

  it('provides nonempty labels for every backend state in both languages', () => {
    const statuses: JobStatus[] = ['queued', 'running', 'completed', 'cancelled', 'failed', 'interrupted'];
    for (const language of ['de', 'en'] as const) {
      const text = translations(language);
      for (const status of statuses) expect(text[status].length).toBeGreaterThan(0);
      expect(text.connectionText).toBeTruthy();
      expect(text.recoveryText).toBeTruthy();
    }
  });
});

describe('model sizes in decimal GB', () => {
  it('uses decimal units and localizes without rounding tiny models down to zero', () => {
    expect(formatGigabytes(1_500_000_000, 'de')).toBe('1,50 GB');
    expect(formatGigabytes(1_500_000_000, 'en')).toBe('1.50 GB');
    expect(formatGigabytes(454671, 'en')).toBe('< 0.01 GB');
    expect(formatGigabytes(0, 'de')).toBe('0,00 GB');
  });
  it('does not present unknown or incomplete metadata as an exact size', () => {
    for (const value of [null, undefined, NaN, -1, Infinity]) expect(formatGigabytes(value, 'en')).toBe('Unknown');
    expect(totalFileBytes([{ size: 10 }, { size: null }])).toBeNull();
    expect(totalFileBytes([])).toBeNull();
    expect(totalFileBytes([{ size: 0 }])).toBe(0);
    expect(totalFileBytes([{ size: 453864 }, { size: 807 }])).toBe(454671);
    expect(totalFileBytes([{ size: Number.MAX_SAFE_INTEGER }, { size: 1 }])).toBeNull();
  });
});
