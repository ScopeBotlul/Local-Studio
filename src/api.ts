import { invoke, isTauri } from '@tauri-apps/api/core';
import type { AppSnapshot, HardwareInfo, Job, Settings } from './types';

export const inDesktop = isTauri;

// Keep command names and payloads in one typed boundary. No browser data stand-ins.
export const api = {
  bootstrap: () => invoke<AppSnapshot>('bootstrap'),
  saveSettings: (settings: Settings) => invoke<Settings>('save_settings', { settings }),
  hardware: () => invoke<HardwareInfo>('get_hardware'),
  jobs: () => invoke<Job[]>('list_jobs'),
  enqueueHash: (path: string) => invoke<Job>('enqueue_hash_job', { path }),
  cancelJob: (id: string) => invoke<void>('cancel_job', { id }),
  dismissRecovery: () => invoke<void>('dismiss_recovery'),
  logs: () => invoke<string>('get_logs'),
  cleanExit: () => invoke<void>('mark_clean_exit'),
};
