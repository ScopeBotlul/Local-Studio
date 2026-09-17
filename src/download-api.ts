import { invoke } from '@tauri-apps/api/core';
export interface DownloadFile { path: string; size: number; sha256: string | null; gitSha1: string | null; downloaded: number; actualSha256: string | null; }
export interface DownloadItem {
  id: string; repo: string; revision: string; license: string | null; task: string | null; files: DownloadFile[];
  destination: string; partialDirectory: string; status: string; priority: number; totalBytes: number; downloadedBytes: number;
  bytesPerSecond: number; error: string | null; createdAt: string; verifyOnly: boolean;
}
export interface DownloadPlan { id: string; download: DownloadItem; additionalBytes: number; availableBytes: number; }
export const downloads = {
  list: () => invoke<DownloadItem[]>('download_list'),
  plan: (repo: string, revision: string, files: string[]) => invoke<DownloadPlan>('download_plan', { repo, revision, files }),
  start: (planId: string) => invoke<DownloadItem>('download_start', { planId }),
  action: (id: string, action: string, priority: number | null = null) => invoke<void>('download_action', { id, action, priority }),
};
export const activeDownload = (item: DownloadItem) => ['queued', 'downloading', 'verifying', 'installing', 'pausing', 'cancelling'].includes(item.status);
