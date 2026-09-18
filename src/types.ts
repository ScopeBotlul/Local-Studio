export type Language = 'de' | 'en';
export type Theme = 'system' | 'light' | 'dark';
export interface Settings {
  autoUpdateCheck: boolean;
  language: Language; theme: Theme; accentColor: string; uiScale: number;
  dataRoot: string; restoreSession: boolean; setupComplete: boolean;
  maxUndo: number; tempRetentionDays: number; autoCleanup: boolean;
  storageOverrides: Partial<StoragePaths>;
  shortcuts: import('./shortcuts').Shortcuts;
}
export interface StoragePaths {
  models: string; assistantModels: string; visionModels: string; downloads: string;
  gallery: string; projects: string; temporary: string; recovery: string; cache: string; proxies: string;
}
export interface GpuInfo { name: string; vendor: string; vramBytes: number | null; driver: string | null; }
export interface DiskInfo { name: string; mountPoint: string; totalBytes: number; availableBytes: number; }
export interface HardwareInfo {
  cpu: string; logicalCores: number; totalMemoryBytes: number; availableMemoryBytes: number;
  os: string; gpus: GpuInfo[]; disks: DiskInfo[]; warnings: string[];
}
export type JobStatus = 'queued' | 'running' | 'completed' | 'failed' | 'cancelled' | 'interrupted';
export interface Job {
  id: string; kind: string; inputPath: string; status: JobStatus; progress: number | null;
  createdAt: string; startedAt: string | null; finishedAt: string | null;
  result: string | null; error: string | null;
}
export interface AppSnapshot {
  version: string; settings: Settings; paths: StoragePaths; hardware: HardwareInfo;
  jobs: Job[]; recoveryAvailable: boolean; databasePath: string; portable: boolean;
}
// Rust IPC: bootstrap() -> AppSnapshot; save_settings({settings}) -> Settings;
// get_hardware() -> HardwareInfo; list_jobs() -> Job[];
// enqueue_hash_job({path}) -> Job; cancel_job({id}) -> void;
// dismiss_recovery() -> void; get_logs() -> string; mark_clean_exit() -> void.
