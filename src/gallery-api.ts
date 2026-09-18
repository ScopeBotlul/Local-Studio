import { invoke } from '@tauri-apps/api/core';
import type { ImageRequest } from './image-api';
export type MediaKind = 'image' | 'video' | 'audio';
export interface Annotation { favorite: boolean; tags: string[]; revision: number }
export interface AnnotationEdit { rootId: string; path: string; fileId: string; revision: number; favorite: boolean; tags: string[] }
export interface GalleryOrigin {jobId:string;request:ImageRequest;modelName:string;modelSha256:string|null;runtime:string;createdAt:string;association:string}
export interface GalleryEntry { origin:GalleryOrigin|null; path: string; name: string; kind: MediaKind | 'folder'; bytes: number; modified: number; fileId: string | null; annotation: Annotation; thumbnailVersion: string }
export type GallerySort='modifiedDesc'|'modifiedAsc'|'nameAsc'|'nameDesc'|'sizeAsc'|'sizeDesc';
export interface GalleryQuery { sort?:GallerySort; folder: string; search: string; kind: 'all' | MediaKind; recursive: boolean; offset: number; favoritesOnly: boolean; tag: string }
export interface GalleryListing { root: string; rootId: string; folder: string; folders: GalleryEntry[]; entries: GalleryEntry[]; total: number; skipped: number; limited: boolean; tags: string[] }
export interface GalleryDetail {origin:GalleryOrigin|null;dimensions:[number,number]|null; url: string; kind: MediaKind; request: ImageRequest | null; jobId: string | null }
export interface GalleryThumbnailData { dataUrl: string; width: number; height: number; cached: boolean; cacheStored: boolean }
export type GalleryBatchAction = {type:'favorite';value:boolean} | {type:'addTag'|'removeTag';tag:string};
export interface GalleryBatchTarget {path:string;fileId:string;revision:number;version:string}
export type GalleryFileAction={type:'rename';name:string}|{type:'move';folder:string}|{type:'trash';confirmed:boolean};
export interface FileReport {completed:string[];errors:string[]}
export interface TrashItem {id:string;originalPath:string;storedPath:string;fileId:string;deletedAt:string;bytes:number;kind:MediaKind}
export interface TrashList {rootId:string;entries:TrashItem[];total:number}
export const galleryApi = {
  lineage:(query:LineageQuery)=>invoke<LineageFamily>('gallery_lineage',{query}),
  createVariant:(query:LineageQuery)=>invoke<string>('gallery_create_variant',{query}),
  setPrimary:(query:LineageQuery)=>invoke<void>('gallery_set_primary',{query}),
  compare:(request:{rootId:string;targets:{path:string;fileId:string;version:string}[]})=>invoke<GalleryDetail[]>('gallery_compare',{request}),
  fileAction:(request:{rootId:string;targets:{path:string;fileId:string;version:string}[];action:GalleryFileAction})=>invoke<FileReport>('gallery_file_action',{request}),
  trashList:(offset=0)=>invoke<TrashList>('gallery_trash_list',{offset}),
  trashAction:(request:{rootId:string;ids:string[];action:'restore'|'purge';confirmed:boolean})=>invoke<FileReport>('gallery_trash_action',{request}),
  trashDetail:(id:string)=>invoke<GalleryDetail>('gallery_trash_detail',{id}),
  annotateBatch: (batch:{rootId:string;targets:GalleryBatchTarget[];action:GalleryBatchAction}) => invoke<number>('gallery_annotate_batch',{batch}),
  thumbnail: (query: {rootId: string; path: string; version: string}) => invoke<GalleryThumbnailData>('gallery_thumbnail', {query}),
  storeVideoThumbnail: (query: {rootId:string;path:string;version:string}, png:string) => invoke<GalleryThumbnailData>('gallery_video_thumbnail_store',{query,png}),
  clearThumbnails: () => invoke<void>('gallery_thumbnail_clear'),
  annotate: (edit: AnnotationEdit) => invoke<Annotation>('gallery_annotate', { edit }),
  list: (query: GalleryQuery) => invoke<GalleryListing>('gallery_list', { query }),
  detail: (path: string) => invoke<GalleryDetail>('gallery_detail', { path }),
  importFiles: (folder: string, sources: string[]) => invoke<{ imported: string[]; errors: string[] }>('gallery_import', { folder, sources }),
  createFolder: (folder: string, name: string) => invoke<void>('gallery_create_folder', { folder, name }),
  openFolder: (folder: string) => invoke<void>('gallery_open_folder', { folder }),
};
export function galleryError(error: unknown, de: boolean): string {
  const text = String(error);
  const errors: Record<string, [string,string]> = {
    editor_limit:['Editorgrenze erreicht: maximal 64 MiB, 32 Megapixel, 16.384 Pixel je Kante oder zu viele aufwendige Schritte. Zwischenstand exportieren und als neue Quelle öffnen.','Editor limit: 64 MiB, 32 megapixels, 16,384 pixels per edge, or too many expensive steps. Export and reopen the intermediate result.'],
    editor_format:['Der Editor öffnet derzeit statische PNG-, JPEG- und BMP-Bilder. Export: PNG oder JPEG.','The editor currently opens still PNG, JPEG and BMP images. Export: PNG or JPEG.'],
    editor_depth:['Dieser erste Editorpfad unterstützt 8-Bit-Bilder. 16-Bit/RAW/HDR folgen später; die Quelle bleibt unverändert.','This initial editor supports 8-bit images. 16-bit/RAW/HDR are planned; the source is unchanged.'],
    editor_profile:['Das Farbprofil ist kein unterstütztes RGB-Profil oder überschreitet 1 MiB. Die Quelle bleibt unverändert.','The color profile is not a supported RGB profile or exceeds 1 MiB. The source is unchanged.'],
    editor_crop:['Der Ausschnitt muss vollständig im Bild liegen und mindestens 1 × 1 Pixel groß sein.','The crop must be inside the image and at least 1 × 1 pixel.'],
    editor_image:['Die Bildverarbeitung ist fehlgeschlagen. Datei, freien Speicher und Schreibrechte prüfen.','Image processing failed. Check the file, free space and write permissions.'],
    gallery_lineage_limit:['Eine Versionsgruppe enthält höchstens 100 Medien.','A version group supports up to 100 media files.'],
    gallery_compare_selection:['Bitte genau zwei verschiedene Bilder auswählen.','Select exactly two different images.'],
    gallery_compare_format:['Dieses Bildformat kann nicht verglichen werden oder die Datei ist beschädigt. Unterstützt: PNG, JPEG, WebP, GIF und BMP.','This image format cannot be compared or the file is damaged. Supported: PNG, JPEG, WebP, GIF and BMP.'],
    gallery_compare_limit:['Der Vergleich ist auf 32 MiB und 32 Megapixel pro Bild begrenzt.','Comparison is limited to 32 MiB and 32 megapixels per image.'],
    gallery_collision:['Am Ziel existiert bereits eine Datei mit diesem Namen. Es wurde nichts überschrieben.','A file with this name already exists at the destination. Nothing was overwritten.'],
    gallery_locked:['Die Datei wird gerade verwendet. Wiedergabe schließen und erneut versuchen.','The file is in use. Close playback and try again.'],
    gallery_move:['Die Datei konnte nicht verschoben werden. Schreibrechte und belegte Dateien prüfen.','Could not move the file. Check permissions and files in use.'],
    gallery_same_path:['Quelle und Ziel sind identisch. Einen anderen Ordner oder Namen wählen.','Source and destination are identical. Choose a different folder or name.'],
    gallery_confirmation:['Die Löschaktion muss ausdrücklich bestätigt werden.','Deletion requires explicit confirmation.'],
    gallery_recovery:['Eine Dateiaktion konnte noch nicht vollständig abgeglichen werden. Ansicht aktualisieren. Bei weiterem Fehler die betroffenen Dateien unverändert lassen.','A file operation could not yet be reconciled. Refresh the view. If the error persists, leave the affected files unchanged.'],
    gallery_query:['Die Abfrage ist ungültig oder zu groß.','The query is invalid or too large.'],
    gallery_thumbnail_cache: ['Der Vorschau-Cache konnte nicht neu aufgebaut werden. Schreibrechte und freien Platz prüfen.', 'Could not rebuild the preview cache. Check permissions and free space.'],
    gallery_thumbnail_limit: ['Dieses Bild überschreitet die Grenzen für Miniaturen (64 MiB, 32 Megapixel).', 'This image exceeds the thumbnail limits (64 MiB, 32 megapixels).'],
    gallery_thumbnail: ['Keine Miniatur verfügbar. Format nicht unterstützt oder Datei beschädigt.', 'Thumbnail unavailable. Unsupported format or damaged file.'],
    gallery_batch_limit: ['Bitte 1 bis 50 Medien auswählen.', 'Select 1 to 50 media files.'],
    gallery_batch_duplicate: ['Dieselbe Datei bitte nur einmal auswählen, auch wenn sie mehrfach verknüpft ist.', 'Select the same file only once, including its hard links.'],
    gallery_identity: ['Für diese Datei ist keine stabile Zuordnung verfügbar. Markierungen sind hier nicht möglich.', 'A stable identity is unavailable for this file. Annotations are not available here.'],
    gallery_changed: ['Die Datei oder der Galerieordner wurde geändert. Ansicht aktualisieren und erneut versuchen.', 'The file or gallery folder changed. Refresh the view and try again.'],
    gallery_conflict: ['Die Markierungen wurden inzwischen geändert. Die Ansicht wird aktualisiert; bitte die gewünschte Aktion erneut ausführen.', 'Annotations changed in the meantime. The view is refreshing; please repeat the intended action.'],
    gallery_tags: ['Maximal 32 Tags mit je 64 Zeichen; keine Kommas oder Steuerzeichen.', 'Up to 32 tags, 64 characters each; no commas or control characters.'],
    gallery_path: ['Dieser Pfad ist nicht verfügbar oder liegt außerhalb der sicheren lokalen Galerie. Verknüpfungen werden übersprungen.', 'This path is unavailable or outside the safe local gallery. Links are skipped.'],
    gallery_missing: ['Datei oder Ordner nicht mehr verfügbar.', 'File or folder is no longer available.'],
    gallery_format: ['Dieses Dateiformat wird noch nicht in der Galerie unterstützt.', 'This file format is not supported in the gallery yet.'],
    gallery_name: ['Bitte einen gültigen Windows-Dateinamen mit unveränderter Dateiendung bzw. einen gültigen Ordnernamen verwenden.', 'Use a valid Windows filename with the same extension, or a valid folder name.'],
    gallery_storage: ['Dateiaktion fehlgeschlagen. Schreibrechte, freien Platz und vorhandene Namen prüfen.', 'File operation failed. Check permissions, free space and existing names.'],
    gallery_import_size: ['Kopierimport derzeit auf 8 GiB pro Datei begrenzt. Größere Medien lassen sich über den Explorer ablegen.', 'Copy import is currently limited to 8 GiB per file. Larger media can be added through Explorer.'],
    gallery_import_limit: ['Bitte 1 bis 100 Dateien auswählen.', 'Please select 1 to 100 files.'],
  };
  for (const [code, labels] of Object.entries(errors)) if (text.includes(code)) return text.replace(code, labels[de ? 0 : 1]);
  return text;
}

export interface LineageNode {id:string;parent:string|null;group:string;path:string;name:string;kind:MediaKind;fileId:string;stamp:string;createdAt:number;operation:'original'|'textToImage'|'copy'|'edit';origin:GalleryOrigin|null}
export interface LineageFamily {current:string;primary:string|null;versions:{node:LineageNode;available:boolean;version:string|null}[]}
export interface LineageQuery {rootId:string;target:{path:string;fileId:string;version:string}}
