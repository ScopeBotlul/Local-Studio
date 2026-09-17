import type { Job, Language } from './types';

export const MIN_SCALE = 0.75;
export const MAX_SCALE = 1.5;
export const ZOOM_STEP = 0.05;

export function clampScale(scale: number): number {
  if (!Number.isFinite(scale)) return 1;
  return Math.round(Math.min(MAX_SCALE, Math.max(MIN_SCALE, scale)) * 100) / 100;
}

export function isActiveJob(job: Pick<Job, 'status'>): boolean {
  return job.status === 'queued' || job.status === 'running';
}

export function fileName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).at(-1) ?? path;
}

export function formatBytes(bytes: number, language: Language = 'en'): string {
  if (!Number.isFinite(bytes) || bytes < 0) return '—';
  const units = ['B', 'KiB', 'GiB'];
  if (bytes < 1024) return `${Math.round(bytes)} ${units[0]}`;
  const power = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), 4);
  return `${new Intl.NumberFormat(language, { maximumFractionDigits: 1 }).format(bytes / 1024 ** power)} ${['B', 'KiB', 'MiB', 'GiB', 'TiB'][power]}`;
}

// Decimal GB for model storage; never turn a missing size into zero.
export function formatGigabytes(bytes: number | null | undefined, language: Language = 'en'): string {
  if (bytes == null || !Number.isFinite(bytes) || bytes < 0) return language === 'de' ? 'Unbekannt' : 'Unknown';
  const format = new Intl.NumberFormat(language, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  if (bytes > 0 && bytes < 10_000_000) return `< ${format.format(0.01)} GB`;
  return `${format.format(bytes / 1_000_000_000)} GB`;
}

export function totalFileBytes(files: { size: number | null }[]): number | null {
  if (!files.length || files.some(file => file.size === null || !Number.isSafeInteger(file.size) || file.size < 0)) return null;
  const total = files.reduce((sum, file) => sum + file.size!, 0);
  return Number.isSafeInteger(total) ? total : null;
}

export function formatDate(value: string, language: Language): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(language, { dateStyle: 'short', timeStyle: 'short' }).format(date);
}

export function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  return typeof error === 'string' ? error : String(error);
}

export function initialLanguage(): Language {
  return navigator.language.toLowerCase().startsWith('de') ? 'de' : 'en';
}
