import { invoke } from '@tauri-apps/api/core';
import type { ImageRequest } from './image-api';
import type {EditOperation,ProjectEditQuery} from './editor-state';
export interface ProjectAsset { id: string; name: string; kind: 'image' | 'video' | 'audio'; bytes: number; sha256: string; archiveName: string; edit?:EditOperation[] }
export interface StudioProject { id: string; name: string; path: string | null; version: string | null; request: ImageRequest | null; model: { name: string; sha256: string; source: string } | null; assets: ProjectAsset[]; removed: ProjectAsset[]; dirty: boolean; recovery: boolean }
export interface RecoveryPoint { id: number; at: number; name: string; media: number; prompt: string; protected: boolean }
export interface ProjectGallerySelection { rootId: string; targets: { path: string; fileId: string; version: string }[] }
export const projectApi = {
  addEdit:(id:string,selection:ProjectGallerySelection,operations:EditOperation[])=>invoke<StudioProject>('project_add_edit',{id,...selection,operations}),
  saveEdit:(query:ProjectEditQuery,expected:EditOperation[],operations:EditOperation[])=>invoke<StudioProject>('project_editor_save',{query,expected,operations}),
  recent: () => invoke<RecentProject[]>("project_recent"),
  forgetRecent: (path:string) => invoke<void>("project_forget_recent",{path}),
  ackOpen: (path:string) => invoke<void>("project_ack_open",{path}),
  takeOpen: () => invoke<string|null>("project_take_open"),
  history: () => invoke<RecoveryPoint[]>('project_history'),
  checkpoint: () => invoke<void>('project_checkpoint'),
  restorePoint: (id: number, confirmed: boolean) => invoke<StudioProject>('project_restore_point', { id, confirmed }),
  rename: (id: string, name: string) => invoke<StudioProject>('project_rename', { id, name }),
  restoreMedia: (id: string) => invoke<StudioProject>('project_restore_media', { id }),
  addGallery: (id: string, selection: ProjectGallerySelection) => invoke<StudioProject>('project_add_gallery', { id, ...selection }),
  addImage: (id: string, jobId: string) => invoke<StudioProject>('project_add_image', { id, jobId }),
  snapshot: () => invoke<StudioProject | null>('project_snapshot'),
  create: (name: string, request: ImageRequest, confirmed: boolean) => invoke<StudioProject>('project_new', { name, request, confirmed }),
  open: (path: string, confirmed: boolean) => invoke<StudioProject>('project_open', { path, confirmed }),
  save: (path: string) => invoke<StudioProject>('project_save', { path }),
  update: (id: string, request: ImageRequest) => invoke<StudioProject>('project_update', { id, request }),
  add: (sources: string[]) => invoke<StudioProject>('project_add', { sources }),
  remove: (id: string) => invoke<StudioProject>('project_remove', { id }),
  close: (confirmed: boolean) => invoke<void>('project_close', { confirmed }),
  recover: () => invoke<StudioProject>('project_recover'),
  relink: (path: string) => invoke<StudioProject>('project_relink', { path }),
  exportGallery: () => invoke<{ imported: string[]; errors: string[] }>('project_export_gallery'),
};
const errors: Record<string, [string, string]> = {
  project_history_missing: ['Dieser Wiederherstellungsstand ist nicht mehr vorhanden.', 'This recovery point is no longer available.'],
  project_storage: ['Projektdateien konnten nicht gelesen oder geschrieben werden.', 'Unable to read or write project files.'],
  project_archive: ['Der Projektcontainer ist ungültig oder beschädigt.', 'The project archive is invalid or damaged.'],
  project_manifest: ['Die Projektdaten sind ungültig.', 'The project data is invalid.'],
  project_version: ['Dieses Projektformat wird noch nicht unterstützt.', 'This project format is not supported.'],
  project_limit: ['Maximal 100 Medien und insgesamt 2 GB pro Projekt.', 'Projects support up to 100 media files and 2 GB total.'],
  project_hash: ['Eine Projektdatei wurde verändert oder ist beschädigt.', 'A project file has changed or is damaged.'],
  project_changed: ['Projekt oder Datei wurde zwischenzeitlich verändert. Bitte unter einem neuen Namen speichern.', 'The project or file changed. Save under a new name.'],
  project_collision: ['Diese Datei gehört nicht zum geöffneten Projekt. Bitte einen freien Dateinamen wählen.', 'This file is not the current project. Choose an unused filename.'],
  project_model_changed: ['Die Prüfsumme passt nicht zum gespeicherten Modell.', 'The checksum does not match the saved model.'],
  project_model: ['Für diese Modellreferenz wird eine lokale Safetensors-Datei benötigt.', 'This model reference requires a local Safetensors file.'],
  project_inactive: ['Bitte zuerst ein Projekt öffnen oder den Arbeitsstand fortsetzen.', 'Open a project or resume the workspace first.'],
  project_save_recovery: ['Ein unterbrochener Speichervorgang benötigt Wiederherstellung. Die vorhandenen Dateien bleiben erhalten.', 'An interrupted save needs recovery. Existing files are preserved.'],
  project_path: ['Der Dateipfad ist nicht zulässig oder nicht verfügbar.', 'The file path is invalid or unavailable.'],
  project_name: ['Bitte einen kurzen Projektnamen ohne Steuerzeichen eingeben.', 'Enter a short project name without control characters.'],
  project_unsaved: ['Das Projekt enthält ungespeicherte Änderungen.', 'The project has unsaved changes.'],
  project_media: ['Keine passenden Projektmedien vorhanden.', 'No matching project media available.'],
};
export function projectError(value: unknown, de: boolean) { const code = String(value); return errors[code]?.[de ? 0 : 1] ?? (de ? `Projektaktion fehlgeschlagen (${code}).` : `Project action failed (${code}).`); }

export interface RecentProject {path:string;name:string;openedAt:number;available:boolean}
