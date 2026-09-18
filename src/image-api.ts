import { invoke } from '@tauri-apps/api/core';

export interface ImageRequest { modelPath: string; prompt: string; negativePrompt: string; width: number; height: number; steps: number; guidance: number; seed: number; sampler: string; }
export interface ImageJob { id: string; request: ImageRequest; status: string; phase: string; step: number; hashedBytes: number; modelBytes: number; modelSha256: string | null; runtime: string; device: string; createdAt: string; elapsedMs: number; error: string | null; output: string | null; savedPath: string | null; logTail: string; discarded: boolean; startedAt: string | null; finishedAt: string | null; queuePosition: number | null; }
export interface ImageProbe { ready: boolean; family: string | null; modelBytes: number | null; missing: string[]; runtime: string; device: string | null; vramBytes: number | null; modelLicense: string; runtimeLicense: string; }
export interface ImageWorkspace { request: ImageRequest | null; models: Record<string, ImageRequest>; }
export interface WorkspaceSnapshot { workspace: ImageWorkspace; recoveryAvailable: boolean; unsaved: number; }
export const activeImage = (job: ImageJob) => job.status === 'running' || job.status === 'queued';
export const imageApi = {
  resume: (id: string) => invoke<ImageJob>('image_resume', { id }),
  workspace: () => invoke<WorkspaceSnapshot>('image_workspace'),
  saveWorkspace: (workspace: ImageWorkspace) => invoke<void>('image_workspace_save', { workspace }),
  recover: () => invoke<WorkspaceSnapshot>('image_recover'),
  discard: (id: string) => invoke<void>('image_discard', { id }),
  probe: (path: string) => invoke<ImageProbe>('image_probe', { path }),
  jobs: () => invoke<ImageJob[]>('image_jobs'),
  generate: (request: ImageRequest) => invoke<ImageJob>('image_generate', { request }),
  cancel: (id: string) => invoke<void>('image_cancel', { id }),
  output: (id: string) => invoke<string>('image_output', { id }),
  save: (id: string) => invoke<string>('image_save', { id }),
};

const errors: Record<string, [string, string]> = {
  image_closing: ['Die Anwendung wird gerade beendet.', 'The application is closing.'],
  image_unsaved: ['Ein Bild wurde während des Beendens fertig. Bitte erneut speichern oder verwerfen.', 'An image completed while closing. Please choose save or discard again.'],
  image_recovery_pending: ['Bitte zuerst den Bild-Arbeitsstand wiederherstellen.', 'Restore the image workspace first.'],
  image_model_missing: ['Bitte eine lokale SDXL-Modelldatei wählen.', 'Choose a local SDXL model file.'],
  image_format: ['Dieser erste Bildadapter benötigt einen vollständigen SDXL-Safetensors-Checkpoint.', 'This first image adapter requires a complete SDXL Safetensors checkpoint.'],
  image_structure: ['Die Dateistruktur ist fehlerhaft oder wird von diesem Adapter nicht unterstützt.', 'The file structure is invalid or unsupported by this adapter.'],
  image_path: ['Die Datei ist nicht verfügbar, gesperrt oder über einen umgeleiteten Pfad erreichbar.', 'The file is unavailable, locked or reached through a redirected path.'],
  image_runtime_missing: ['Bild-Runtime fehlt. Bitte das vollständige Local-Studio-Paket inklusive image-runtime installieren.', 'Image runtime is missing. Install the complete Local Studio package including image-runtime.'],
  image_runtime_invalid: ['Die Bild-Runtime wurde verändert. Bitte die Originaldateien aus dem Local-Studio-Paket wiederherstellen.', 'Image runtime has changed. Restore the original files from the Local Studio package.'],
  image_runtime_start: ['Die Bild-Runtime konnte nicht gestartet werden. Prüfe den Grafiktreiber; Details stehen im Auftrag.', 'Image runtime could not start. Check your graphics driver; see job details.'],
  image_runtime_timeout: ['Die Runtime antwortet nicht. Grafiktreiber prüfen und erneut versuchen.', 'The runtime is not responding. Check your graphics driver and retry.'],
  image_gpu: ['Dieser Adapter benötigt derzeit eine NVIDIA-GPU mit funktionierendem Vulkan-Treiber.', 'This adapter currently requires an NVIDIA GPU with a working Vulkan driver.'],
  image_vram: ['Für diesen ersten SDXL-Pfad sind mindestens 8 GB Grafikspeicher vorgesehen.', 'This first SDXL path requires at least 8 GB of graphics memory.'],
  image_prompt: ['Bitte einen Prompt mit höchstens 4.000 UTF-8-Bytes eingeben.', 'Enter a prompt of at most 4,000 UTF-8 bytes.'],
  image_parameters: ['Ungültige Bildparameter. Auflösung, Schritte, Guidance und Sampler prüfen.', 'Invalid image parameters. Check resolution, steps, guidance and sampler.'],
  image_queue_full: ['Es warten bereits 20 Bildaufträge. Bitte einen Auftrag abwarten oder abbrechen.', '20 image jobs are already waiting. Wait for a job or cancel one.'],
  image_not_paused: ['Dieser Auftrag wartet nicht auf eine Wiederaufnahme.', 'This job is not awaiting resumption.'],
  image_model_changed: ['Die Modelldatei stimmt nicht mehr mit dem gespeicherten Auftrag überein. Stelle das ursprüngliche Modell wieder her oder übernimm die Einstellungen für einen neuen Auftrag.', 'The model file no longer matches the saved job. Restore the original model or reuse the settings for a new job.'],
  image_environment_changed: ['Runtime oder GPU-Zuordnung hat sich geändert. Bitte die Einstellungen für einen neuen Auftrag übernehmen und erneut prüfen.', 'The runtime or GPU mapping changed. Reuse the settings for a new job and check readiness again.'],
  image_queue_paused: ['Dieser wartende Auftrag wurde beim Beenden pausiert. Er startet erst nach bewusstem erneutem Einreihen.', 'This waiting job was paused when the app closed. It will only start after you explicitly resume it.'],
  image_busy: ['Es läuft bereits eine Bildgenerierung.', 'An image generation is already running.'],
  image_cancelled: ['Generierung abgebrochen; Modell entladen.', 'Generation cancelled; model unloaded.'],
  image_timeout: ['Zeitlimit erreicht. Der Worker wurde beendet und der Grafikspeicher freigegeben.', 'Time limit reached. The worker was stopped and graphics memory released.'],
  image_execution: ['Generierung fehlgeschlagen. Details prüfen; bei Speichermangel eine kleinere Auflösung wählen.', 'Generation failed. Check details; choose a smaller resolution if memory was exhausted.'],
  image_output: ['Der Worker hat kein gültiges PNG in der gewünschten Auflösung geliefert.', 'The worker did not return a valid PNG at the requested resolution.'],
  image_storage: ['Lokale Bilddaten konnten nicht gespeichert oder gelesen werden.', 'Unable to read or save local image data.'],
  image_missing: ['Das Ergebnis ist nicht mehr verfügbar.', 'The result is no longer available.'],
  image_interrupted: ['Dieser Auftrag wurde durch einen App-Abbruch unterbrochen. Er wird nicht automatisch neu gestartet.', 'This job was interrupted by an app crash. It will not restart automatically.'],
  image_worker_guard: ['Der Bildworker konnte nicht sicher an die App gebunden werden.', 'The image worker could not be attached safely to the app.'],
};
export function imageError(value: unknown, de: boolean) {
  const message = String(value);
  if (message.startsWith('image_not_ready: ')) return (de ? 'Noch nicht bereit: ' : 'Not ready: ') + message.slice('image_not_ready: '.length).split(', ').map(code => errors[code]?.[de ? 0 : 1] ?? code).join(' · ');
  return errors[message]?.[de ? 0 : 1] ?? message;
}
