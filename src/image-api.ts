import { invoke } from '@tauri-apps/api/core';

export interface ImageMask {path:string;sha256:string;width:number;height:number}
export interface ImageReference {mask?:ImageMask|null;path:string;sha256:string;width:number;height:number;strength:number}
export interface ImageLora {path:string;strength:number;sha256?:string|null}
export type ImageBackend='auto'|'vulkan'|'comfy';
export interface ImageRequest { engine?:ImageBackend; vaeOnCpu?:boolean; reference?:ImageReference|null; loras?:ImageLora[]; modelPath: string; prompt: string; negativePrompt: string; width: number; height: number; steps: number; guidance: number; seed: number; sampler: string; }
export interface ImageJob {restricted?:boolean;locked?:boolean; batch?:{id:string;index:number;count:number}|null; samplingSteps?:number|null; id: string; request: ImageRequest; status: string; phase: string; step: number; hashedBytes: number; modelBytes: number; modelSha256: string | null; runtime: string; device: string; createdAt: string; elapsedMs: number; error: string | null; output: string | null; savedPath: string | null; logTail: string; discarded: boolean; startedAt: string | null; finishedAt: string | null; queuePosition: number | null; }
export interface ImageProbe { ready: boolean; family: string | null; modelBytes: number | null; missing: string[]; runtime: string; device: string | null; vramBytes: number | null; modelLicense: string; runtimeLicense: string; }
export interface ImageModel {id:string;name:string;path:string;totalBytes:number;restricted:boolean}
export const IMAGE_MAX_SIDE=4096, IMAGE_MAX_PIXELS=4194304;
export function imageDimensionIssue(width:number,height:number):'range'|'step'|'area'|null {
 if(![width,height].every(n=>Number.isFinite(n)&&n>=256&&n<=IMAGE_MAX_SIDE))return 'range';
 if(![width,height].every(n=>Number.isInteger(n)&&n%64===0))return 'step';
 return width*height>IMAGE_MAX_PIXELS?'area':null;
}
export const validImageDimensions=(width:number,height:number)=>imageDimensionIssue(width,height)===null;
export function suggestImageDimensions(width:number,height:number):{width:number;height:number}|null {
 if(![width,height].every(n=>Number.isFinite(n)&&n>0))return null;
 const align=(n:number)=>Math.min(IMAGE_MAX_SIDE,Math.max(256,Math.round(n/64)*64));
 let w=align(width),h=align(height);
 if(w*h>IMAGE_MAX_PIXELS){const scale=Math.sqrt(IMAGE_MAX_PIXELS/(w*h));w=align(w*scale);h=align(h*scale);}
 while(w*h>IMAGE_MAX_PIXELS){if(w>=h)w-=64;else h-=64;}
 return {width:w,height:h};
}
export interface ImageWorkspace { request: ImageRequest | null; models: Record<string, ImageRequest>; }
export interface WorkspaceSnapshot {locked?:boolean; workspace: ImageWorkspace; recoveryAvailable: boolean; unsaved: number; }
export const activeImage = (job: ImageJob) => job.status === 'running' || job.status === 'queued';
export function unavailableGalleryFolder(error:unknown){return ['gallery_missing','gallery_path'].includes(String(error));}
export async function saveWithGalleryFallback(id:string,folder:string,save:(id:string,folder:string)=>Promise<string>){
 try{return {path:await save(id,folder),folder};}
 catch(error){
  if(!folder||!unavailableGalleryFolder(error))throw error;
  return {path:await save(id,''),folder:''};
 }
}
export const imageApi = {
  reference:(path:string)=>invoke<ImageReference>('image_reference',{path}),
  generateBatch:(request:ImageRequest,count:number,incrementSeed:boolean)=>invoke<{jobs:ImageJob[];error:string|null}>('image_generate_batch',{request,count,incrementSeed}),
  resume: (id: string) => invoke<ImageJob>('image_resume', { id }),
  workspace: () => invoke<WorkspaceSnapshot>('image_workspace'),
  saveWorkspace: (workspace: ImageWorkspace) => invoke<void>('image_workspace_save', { workspace }),
  recover: () => invoke<WorkspaceSnapshot>('image_recover'),
  discard: (id: string) => invoke<void>('image_discard', { id }),
  probe: (path:string,backend:ImageBackend='auto',hasLoras=false,needsNative=false) => invoke<ImageProbe>('image_probe', { path, backend, hasLoras, needsNative }),
  jobs: () => invoke<ImageJob[]>('image_jobs'),
  generate: (request: ImageRequest) => invoke<ImageJob>('image_generate', { request }),
  cancel: (id: string) => invoke<void>('image_cancel', { id }),
  output: (id: string) => invoke<string>('image_output', { id }),
  save: (id: string, folder = '') => invoke<string>('image_save', { id, folder }),
};

const errors: Record<string, [string, string]> = {
  image_mask:['Die Maske muss ein undurchsichtiges PNG mit gleichen RGB-Grauwerten und mindestens einem hellen Bereich sein. Weiß wird bearbeitet, Schwarz bleibt erhalten.','The mask must be an opaque grayscale PNG with at least one non-black area. White is edited, black is preserved.'],
  resource_memory:['Zu wenig freier RAM für diese Bild-Engine. Schließe speicherintensive Programme oder wähle ComfyUI.','Not enough free RAM for this image engine. Close memory-heavy apps or select ComfyUI.'],
  resource_timeout:['Zu lange auf Ressourcen gewartet. Den Auftrag bei Bedarf erneut einreihen.','Resource wait timed out. Queue the job again if needed.'],
  image_batch: ['Stapelgröße 1 bis 20 wählen. Bei aufsteigenden Seeds darf der letzte Seed 4294967295 nicht überschreiten.', 'Choose a batch size from 1 to 20. With increasing seeds, the last seed must not exceed 4294967295.'],
  image_dimensions: ['Breite/Höhe: 256–4096 Pixel in 64er-Schritten; maximal 4.194.304 Pixel insgesamt.', 'Width/height: 256–4096 pixels in steps of 64; at most 4,194,304 total pixels.'],
  image_reference_parameters: ['Referenz: PNG in 8 Bit, 256–4096 Pixel in 64er-Schritten, insgesamt maximal 4.194.304 Pixel. Ausgabegröße muss übereinstimmen; Stärke 0,05 bis 1.', 'Reference: 8-bit PNG with dimensions of 256–4096 in steps of 64, at most 4,194,304 total pixels. Output dimensions must match; strength must be 0.05 to 1.'],
  image_reference_changed: ['Das Referenzbild wurde verändert. Bitte erneut auswählen.', 'The reference image changed. Select it again.'],
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
  image_gpu: ['Keine nutzbare Vulkan-GPU gefunden. Aktualisiere den Grafiktreiber oder wähle ComfyUI als Engine.', 'No usable Vulkan GPU was found. Update the graphics driver or select ComfyUI as the engine.'],
  image_vram: ['Für diesen SDXL-Pfad sind mindestens 8 GB dedizierter Grafikspeicher vorgesehen. Geräte mit gemeinsamem Grafikspeicher werden stattdessen vom Vulkan-Worker dynamisch eingeteilt.', 'This SDXL path requires at least 8 GB of dedicated graphics memory. Devices with shared graphics memory are allocated dynamically by the Vulkan worker instead.'],
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
  image_execution: ['Generierung fehlgeschlagen. Bitte die technischen Auftragsdetails prüfen.', 'Generation failed. Check the technical job details.'],
  image_memory: ['Zu wenig verfügbarer Speicher für die Generierung. Bei knappem VRAM „VAE auf CPU“ wählen oder andere GPU-Aufgaben beenden und erneut starten.', 'Insufficient available memory for generation. If VRAM is limited, select “Run VAE on CPU” or finish other GPU tasks before trying again.'],
  image_output: ['Der Worker hat kein gültiges PNG in der gewünschten Auflösung geliefert.', 'The worker did not return a valid PNG at the requested resolution.'],
  image_storage: ['Lokale Bilddaten konnten nicht gespeichert oder gelesen werden.', 'Unable to read or save local image data.'],
  gallery_missing: ['Der gewählte Galerieordner existiert nicht mehr. Local Studio verwendet wieder den Galerie-Hauptordner.', 'The selected gallery folder no longer exists. Local Studio is using the main gallery folder again.'],
  gallery_path: ['Der gewählte Galerieordner ist nicht mehr sicher erreichbar. Local Studio verwendet wieder den Galerie-Hauptordner.', 'The selected gallery folder is no longer safely accessible. Local Studio is using the main gallery folder again.'],
  image_missing: ['Das Ergebnis ist nicht mehr verfügbar.', 'The result is no longer available.'],
  image_interrupted: ['Dieser Auftrag wurde durch einen App-Abbruch unterbrochen. Er wird nicht automatisch neu gestartet.', 'This job was interrupted by an app crash. It will not restart automatically.'],
  image_worker_guard: ['Der Bildworker konnte nicht sicher an die App gebunden werden.', 'The image worker could not be attached safely to the app.'],
  comfy_connection: ['Die lokale ComfyUI-API antwortet nicht.', 'The local ComfyUI API is not responding.'],
  comfy_runtime_exit: ['ComfyUI ist während der Generierung abgestürzt. Der Auftrag wurde beendet; das ComfyUI-Protokoll enthält Details.', 'ComfyUI crashed during generation. The job was stopped; check the ComfyUI log for details.'],
  comfy_workflow: ['ComfyUI hat den geprüften Workflow abgelehnt. Die konkrete Antwort steht in den technischen Auftragsdetails und im ComfyUI-Protokoll.', 'ComfyUI rejected the validated workflow. The exact response is available in the technical job details and ComfyUI log.'],
  comfy_checkpoint_unavailable: ['ComfyUI sieht den ausgewählten Checkpoint noch nicht. Die verwaltete ComfyUI-Engine neu starten; bleibt die Meldung bestehen, Installation und Modellpfad im Update-Center prüfen.', 'ComfyUI cannot see the selected checkpoint yet. Restart the managed ComfyUI engine; if this remains, check the installation and model path in Update Center.'],
  comfy_execution: ['ComfyUI konnte das Bild nicht erzeugen. Das ComfyUI-Protokoll enthält Details.', 'ComfyUI could not generate the image. Check its log for details.'],
  comfy_model_path: ['Das Modell muss im Ordner ComfyUI\\models\\checkpoints liegen.', 'The model must be inside ComfyUI\\models\\checkpoints.'],
  comfy_reference: ['Referenzbild und Inpainting verwenden vorerst die bisherige Bildengine.', 'Reference image and inpainting currently use the existing image engine.'],
  image_lora: ['Maximal acht LoRAs mit einer Stärke von -2 bis 2 auswählen.', 'Select up to eight LoRAs with a strength from -2 to 2.'],
  image_lora_path: ['Die LoRA-Datei oder ihr Ordner ist nicht sicher erreichbar.', 'The LoRA file or its folder is not safely accessible.'],
  comfy_lora_restart: ['Der LoRA-Ordner wurde gespeichert. Extern gestartete ComfyUI bitte einmal neu starten, damit sie ihn einliest.', 'The LoRA folder was saved. Restart externally started ComfyUI once so it can load it.'],
  image_lora_changed: ['Eine ausgewählte LoRA-Datei wurde seit dem Einreihen verändert.', 'A selected LoRA file changed after the job was queued.'],
  image_lora_stage: ['Die LoRA konnte nicht sicher für den Vulkan-Worker bereitgestellt werden.', 'The LoRA could not be staged safely for the Vulkan worker.'],
};
export function imageError(value: unknown, de: boolean) {
  const message = String(value);
  if (message.startsWith('image_not_ready: ')) return (de ? 'Noch nicht bereit: ' : 'Not ready: ') + message.slice('image_not_ready: '.length).split(', ').map(code => errors[code]?.[de ? 0 : 1] ?? code).join(' · ');
  return errors[message]?.[de ? 0 : 1] ?? message;
}
