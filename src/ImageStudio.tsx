import {PrivacyGate,ModelPrivacy,usePrivacy,useModelPrivacy} from "./Privacy";
import { useEffect, useRef, useState, type Dispatch, type SetStateAction } from 'react';
import { invoke } from '@tauri-apps/api/core';
import ImageCanvas from './ImageCanvas';
import ImageDimensions from './ImageDimensions';
import { confirm, open } from '@tauri-apps/plugin-dialog';
import { FolderOpen, ImagePlus, RefreshCw, Save, X } from 'lucide-react';
import { imagePhases as phases } from './ImageJobRow';
import { activeImage, imageApi, imageError, validImageDimensions, type ImageModel, type ImageJob, type ImageProbe, type ImageRequest } from './image-api';
import { displayPath, formatBytes, formatDate, formatGigabytes } from './helpers';
import type { Language } from './types';
import './image-studio.css';
import ImageReferenceInput from './ImageReferenceInput';
import { shortcutFor, type Shortcuts } from './shortcuts';

// Advisory UI results only. Queue admission always checks current bytes and runtime.
const readinessCache = new Map<string, {at:number;probe:ImageProbe}>();

export default function ImageStudio({ workspaceLocked=false,onAddToProject, projectDisabled, shortcuts, language, request, setRequest, selectModel, onRestore, selectedJob, disabled = false, galleryOnly = false }: { workspaceLocked?:boolean;onAddToProject: (id: string) => Promise<boolean>; projectDisabled: boolean; shortcuts: Shortcuts; language: Language; request: ImageRequest; setRequest: Dispatch<SetStateAction<ImageRequest>>; selectModel: (path: string) => void; onRestore: (request: ImageRequest) => void; selectedJob?: string | null; disabled?: boolean; galleryOnly?: boolean }) {
  const de = language === 'de';const privacy=usePrivacy();const restrictedModel=useModelPrivacy(request.modelPath);const contentLocked=privacy.status.locked&&(workspaceLocked||restrictedModel);
  const [historyLimit, setHistoryLimit] = useState(50);
  const [dimensionsEditingValid,setDimensionsEditingValid]=useState(true);
  const [batchCount,setBatchCount]=useState(1),[incrementSeed,setIncrementSeed]=useState(true);
  const [models, setModels] = useState<ImageModel[]>([]);
  const [modelsBusy,setModelsBusy]=useState(true);
  useEffect(() => { let live=true; setModelsBusy(true); void invoke<ImageModel[]>('image_model_catalog').then(data=>{if(live)setModels(data);}).catch(e=>{if(live)setError(String(e));}).finally(()=>{if(live)setModelsBusy(false);});return()=>{live=false;}; }, [privacy.status.epoch]);
  const [probe, setProbe] = useState<ImageProbe | null>(null); const [probeBusy, setProbeBusy] = useState(false);
  const [jobs, setJobs] = useState<ImageJob[]>([]); const [selected, setSelected] = useState<string | null>(selectedJob ?? null);
  const [preview, setPreview] = useState<{id:string;data:string}|null>(null); const [error, setError] = useState(''); const [busy, setBusy] = useState(false);
  const alive = useRef(true); const probeGeneration = useRef(0); const operation = useRef(false);
  const active = jobs.find(j => j.status === 'running'); const waiting = jobs.filter(j => j.status === 'queued'); const visibleJobs = galleryOnly ? jobs.filter(j => j.savedPath) : jobs;
  const current = visibleJobs.find(j => j.id === selected) ?? visibleJobs[0];
  async function refresh() { const list = await imageApi.jobs(); if (alive.current) setJobs(list); }
  useEffect(() => {
    alive.current = true; let loading = false;
    const poll = async () => { if (loading) return; loading = true; try { await refresh(); } catch (e) { if (alive.current) setError(String(e)); } finally { loading = false; } };
    void poll(); const timer = setInterval(() => void poll(), 600);
    return () => { alive.current = false; probeGeneration.current++; clearInterval(timer); };
  }, []);
  useEffect(() => {
    let live = true; setPreview(null);
    if (current?.status === 'completed' && !current.discarded && !current.locked) void imageApi.output(current.id).then(data => { if (live) setPreview({id:current.id,data}); }).catch(e => { if (live) setError(String(e)); });
    return () => { live = false; };
  }, [current?.id, current?.status, current?.discarded, current?.savedPath, current?.locked]);
  useEffect(() => {
    probeGeneration.current++; setProbe(null); setProbeBusy(!!request.modelPath);
    if (!request.modelPath || galleryOnly) {setProbeBusy(false);return;}
    const timer=setTimeout(()=>void check(false),450);
    return()=>{clearTimeout(timer);probeGeneration.current++;};
  }, [request.modelPath, galleryOnly]);
  useEffect(()=>{if(selectedJob)setSelected(selectedJob);},[selectedJob]);
  function modelPath(value: string) { if(value===request.modelPath)return; probeGeneration.current++; setProbeBusy(false); setProbe(null); selectModel(value); }
  async function check(force=true) {
    const generation = ++probeGeneration.current; setProbeBusy(true); setError('');
    try { const cached=readinessCache.get(request.modelPath); const next = !force && cached && Date.now()-cached.at<60000 ? cached.probe : await imageApi.probe(request.modelPath); if(readinessCache.size>=64)readinessCache.clear(); readinessCache.set(request.modelPath,{at:Date.now(),probe:next}); if (alive.current && generation === probeGeneration.current) setProbe(next); }
    catch (e) { if (alive.current && generation === probeGeneration.current) setError(String(e)); }
    finally { if (alive.current && generation === probeGeneration.current) setProbeBusy(false); }
  }
  async function act(action: () => Promise<unknown>) {
    if (operation.current) return; operation.current = true; setBusy(true); setError('');
    try { await action(); await refresh(); } catch (e) { if (alive.current) setError(String(e)); }
    finally { operation.current = false; if (alive.current) setBusy(false); }
  }
  const dimensionsValid=dimensionsEditingValid&&validImageDimensions(request.width,request.height);
  const referenceValid=!request.reference||(request.width===request.reference.width&&request.height===request.reference.height);
  const parametersValid=Number.isInteger(request.steps)&&request.steps>=1&&request.steps<=60&&Number.isFinite(request.guidance)&&request.guidance>=1&&request.guidance<=20&&Number.isInteger(request.seed)&&request.seed>=0&&request.seed<=4294967295&&(!incrementSeed||request.seed+batchCount-1<=4294967295);
  return <div className="page image-studio" onKeyDown={event => {
    if (event.defaultPrevented || event.repeat || disabled || busy || galleryOnly || (event.target as HTMLElement).closest('dialog')) return;
    if ((event.target as HTMLElement).closest('input,textarea,select,[contenteditable="true"]') && !event.ctrlKey && !event.altKey) return;
    if (shortcutFor(event.nativeEvent, shortcuts) === 'generate') { const button = event.currentTarget.querySelector<HTMLButtonElement>('[data-image-generate]'); if (button && !button.disabled) { event.preventDefault(); button.click(); } }
  }}>
    <header className="page-heading image-studio-heading"><div><div className="eyebrow">SDXL · {de?'LOKAL':'LOCAL'}</div><h1>{galleryOnly?(de?'Generierte Bilder':'Generated images'):(de?'Bildstudio':'Image studio')}</h1></div><span className="image-local-badge">{de?'Dein Modell. Deine Ideen. Dein PC.':'Your model. Your ideas. Your PC.'}</span></header>
    {error && <p className="notice warning" role="alert">{imageError(error, de)}</p>}
    {!galleryOnly && (active || waiting.length > 0) && <section className="notice image-queue-summary" role="status"><span>{active ? (de ? 'Ein Bildauftrag läuft.' : 'An image job is running.') : ''} {waiting.length} {de ? 'in der Warteschlange' : 'in the queue'}.</span>{active && <button className="text-button" onClick={() => setSelected(active.id)}>{de ? 'Laufenden Auftrag anzeigen' : 'Show running job'}</button>}</section>}
    <div className={`image-workspace ${galleryOnly?'gallery-only':''}`}>
    {!galleryOnly && <fieldset disabled={disabled} className="panel image-config">
      <div className="section-heading"><h2>{de?'Modell':'Model'}</h2><button type="button" className="text-button" disabled={busy} aria-label={de?'Modelldatei wählen':'Choose model file'} onClick={()=>void act(async()=>{const path=await open({multiple:false,filters:[{name:'SDXL · Safetensors',extensions:['safetensors']}]});if(typeof path==='string')modelPath(path);})}><FolderOpen size={17}/></button></div>
      <label className="field-label" htmlFor="image-model-library">{de?'SDXL-Checkpoints':'SDXL checkpoints'}</label>
      <select id="image-model-library" className="image-model-select" disabled={modelsBusy} value={models.some(m=>m.path===request.modelPath)?request.modelPath:''} onChange={e=>modelPath(e.target.value)}>
        <option value="">{modelsBusy?(de?'Modelle werden erkannt …':'Identifying models …'):request.modelPath?(displayPath(request.modelPath).split(/[\\/]/).pop()):(de?'Modell auswählen …':'Select a model …')}</option>
        {models.map(model=><option key={model.id} value={model.path}>{model.name} · {formatGigabytes(model.totalBytes,language)}{model.restricted?' · 18+':''}</option>)}
      </select>
      <div className={'image-model-status '+(probe?.ready?'ready':'')} role="status">{probeBusy?(de?'Ausführbarkeit wird automatisch geprüft …':'Checking readiness automatically …'):probe?.ready?(de?'SDXL bereit · Prüfung vor dem Start automatisch':'SDXL ready · checked automatically before starting'):probe?(de?'Modell oder Laufzeit noch nicht bereit':'Model or runtime not ready'):(de?'Vollständigen SDXL-Checkpoint wählen':'Choose a complete SDXL checkpoint')}</div>
      {!modelsBusy&&!models.length&&<p className="hub-hint">{de?'Keine passenden Checkpoints in der Bibliothek. Über das Ordnersymbol eine Datei wählen.':'No compatible checkpoints in the library. Choose a file with the folder button.'}</p>}
      {probe&&!probe.ready&&<ul className="image-model-errors">{probe.missing.map(code=><li key={code}>{imageError(code,de)}</li>)}</ul>}
      <details className="image-control-details"><summary>{de?'Modellinformationen und Dateipfad':'Model information and file path'}</summary>
      <label className="field-label" htmlFor="image-model">{de ? 'SDXL-Modelldatei' : 'SDXL model file'}</label>
      <div className="image-model-path"><input id="image-model" value={displayPath(request.modelPath)} onChange={e => { probeGeneration.current++; setProbe(null); setRequest(r => ({ ...r, modelPath: e.target.value })); }} spellCheck={false} placeholder="D:\Modelle\modell.safetensors" />
        <button className="button secondary" disabled={busy} onClick={() => void act(async () => { const path = await open({ multiple: false, filters: [{ name: 'Safetensors', extensions: ['safetensors'] }] }); if (typeof path === 'string') modelPath(path); })}><FolderOpen size={16} />{de ? 'Modell wählen' : 'Choose model'}</button>
        <button className="button secondary" disabled={probeBusy || busy} onClick={() => void check()}><RefreshCw size={16} />{probeBusy ? (de ? 'Wird geprüft …' : 'Checking …') : (de ? 'Erneut prüfen' : 'Recheck')}</button></div>
      <p className="hub-hint">{de ? 'Dieser erste Adapter unterstützt einzelne SDXL-Safetensors-Dateien mit UNet, CLIP-L, CLIP-G und VAE. Diffusers-Ordner, SD 1.5, FLUX, Z-Image und Erweiterungen sind hier noch nicht ausführbar.' : 'This first adapter supports single SDXL Safetensors files with UNet, CLIP-L, CLIP-G and VAE. Diffusers folders, SD 1.5, FLUX, Z-Image and extensions cannot run here yet.'}</p>
      {probe && <div className={`image-readiness ${probe.ready ? 'ready' : ''}`} data-testid="image-readiness">
        <strong>{probe.ready ? (de ? 'Bereit für den Generierungsversuch' : 'Ready to attempt generation') : (de ? 'Noch nicht ausführbar' : 'Not ready')}</strong>
        {probe.missing.length > 0 && <ul>{probe.missing.map(code => <li key={code}>{imageError(code, de)}</li>)}</ul>}
        <p>{probe.family ?? (de ? 'SDXL nicht vollständig erkannt' : 'Complete SDXL not recognized')} · {formatGigabytes(probe.modelBytes, language)} · {probe.runtime}</p>
        <p>{probe.device?.split('\t').slice(1).join(' ') ?? (de ? 'Keine unterstützte GPU verfügbar' : 'No supported GPU available')} · VRAM: {probe.vramBytes === null ? (de ? 'unbekannt' : 'unknown') : formatBytes(probe.vramBytes, language)}</p>
        <p>{de ? 'Runtime-Lizenz: MIT. Modelllizenz: unbekannt – Bedingungen der Modellquelle beachten. Die Vorprüfung ersetzt keinen erfolgreichen Modelllauf.' : 'Runtime license: MIT. Model license: unknown — check the model source terms. Preflight does not replace a successful model run.'}</p>
      </div>}
      </details>
      {request.modelPath&&<ModelPrivacy path={request.modelPath} de={de}/>}
      <p className="hub-hint image-privacy-hint">{de?'18+-Tags werden automatisch übernommen. Ohne Kennzeichnung bleibt der Inhalt unbekannt; bei Bedarf manuell zuordnen.':'18+ tags are detected automatically. Untagged content remains unknown; classify it manually when needed.'}</p>
      {contentLocked?<PrivacyGate de={de}/>:<>
      <div className="image-prompt-grid"><label className="field-label">Prompt<textarea aria-label="Image prompt" value={request.prompt} onChange={e=>setRequest(r=>({...r,prompt:e.target.value}))} rows={5} maxLength={4000} placeholder={de?'Motiv, Stil, Licht und Details …':'Subject, style, lighting and details …'}/></label><details className="image-control-details" open><summary>{de?'Negativer Prompt':'Negative prompt'}{request.negativePrompt?' •':''}</summary><textarea aria-label="Negative prompt" value={request.negativePrompt} onChange={e=>setRequest(r=>({...r,negativePrompt:e.target.value}))} rows={3} maxLength={4000} placeholder={de?'Was soll im Bild vermieden werden?':'What should the image avoid?'}/></details></div>
      <ImageDimensions request={request} onChange={patch=>setRequest(r=>({...r,...patch}))} onValidityChange={setDimensionsEditingValid} de={de}/>
      <details className="image-control-details"><summary>{de?'Referenzbild und Inpainting':'Reference image and inpainting'}{request.reference?' •':''}</summary>
      <ImageReferenceInput request={request} onChange={patch=>setRequest(r=>({...r,...patch}))} de={de} disabled={disabled||busy}/>
      </details>
      <details className="image-control-details"><summary>{de?'Generierungseinstellungen':'Generation settings'} · {request.steps} {de?'Schritte':'steps'}</summary>
      <div className="image-parameters">
        <label className="field-label">{de ? 'Schritte' : 'Steps'}<input aria-label="Image steps" type="number" min={1} max={60} value={request.steps} onChange={e => setRequest(r => ({ ...r, steps: Number(e.target.value) }))} /></label>
        <label className="field-label">Guidance<input aria-label="Image guidance" type="number" min={1} max={20} step={0.5} value={request.guidance} onChange={e => setRequest(r => ({ ...r, guidance: Number(e.target.value) }))} /></label>
        <label className="field-label">Seed<input aria-label="Image seed" type="number" min={0} max={4294967295} value={request.seed} onChange={e => setRequest(r => ({ ...r, seed: Number(e.target.value) }))} /></label>
        <label className="field-label">Sampler<select aria-label="Image sampler" value={request.sampler} onChange={e => setRequest(r => ({ ...r, sampler: e.target.value }))}><option value="euler">Euler · Karras</option><option value="dpm++2m">DPM++ 2M · Karras</option></select></label>
      </div>
      <div className="image-parameters"><label className="field-label">{de?'Bilder im Stapel':'Images in batch'}<input aria-label={de?'Bilder im Stapel':'Images in batch'} type="number" min={1} max={20} value={batchCount} disabled={busy} onChange={e=>setBatchCount(Number(e.target.value))}/></label><label className="field-label">{de?'Seeds im Stapel':'Batch seeds'}<select aria-label={de?'Seeds im Stapel':'Batch seeds'} value={incrementSeed?'increment':'same'} disabled={busy||batchCount===1} onChange={e=>setIncrementSeed(e.target.value==='increment')}><option value="increment">{de?'Startseed, danach jeweils +1':'Starting seed, then +1 per image'}</option><option value="same">{de?'Gleicher Seed für alle Bilder':'Same seed for all images'}</option></select></label></div>
      <p className="hub-hint">{de ? 'Ein Bild pro Auftrag; maximal 20 wartende Aufträge. Ausführung nacheinander. Beim Einreihen werden Modellbytes geprüft und bis zum Ende des Auftrags gegen Änderungen gesperrt. Der Worker gibt seinen Speicher nach jedem Auftrag frei.' : 'One image per job; up to 20 waiting jobs. Jobs run sequentially. Enqueuing verifies the model bytes and protects them from changes until the job ends. The worker releases its memory after each job.'}</p>
      </details>
      {!parametersValid&&<p className="notice warning" role="alert">{de?'Schritte, Guidance oder Seed liegen außerhalb des gültigen Bereichs.':'Steps, guidance or seed are outside the valid range.'}</p>}
      {!dimensionsValid&&<p className="notice warning" role="alert">{de?'Zum Generieren bitte die Bildgröße oben korrigieren oder den vorgeschlagenen Wert übernehmen.':'To generate, correct the image size above or apply the suggested dimensions.'}</p>}
      <div className="image-generate-bar"><button data-image-generate className="button primary" disabled={busy || probeBusy || !dimensionsValid || !referenceValid || !parametersValid || !Number.isInteger(batchCount) || batchCount<1 || batchCount>20 || waiting.length+batchCount>20 || !probe?.ready || !request.prompt.trim()} onClick={() => void act(async () => { if(batchCount===1){const job=await imageApi.generate({...request});setSelected(job.id);}else{const result=await imageApi.generateBatch({...request},batchCount,incrementSeed);if(result.jobs[0])setSelected(result.jobs[0].id);if(result.error)setError((de?`${result.jobs.length} Aufträge eingereiht. `:`${result.jobs.length} jobs queued. `)+imageError(result.error,de));} })}><ImagePlus size={17} />{busy ? (de ? 'Modell für Auftrag prüfen …' : 'Verifying model for job …') : (batchCount>1?(de?`${batchCount} Bilder einreihen`:`Queue ${batchCount} images`):active || waiting.length ? (de ? 'Bild einreihen' : 'Queue image') : (de ? 'Bild generieren' : 'Generate image'))}</button>
      </div>
      </>}
    </fieldset>}
    <div className="image-workspace-output">
    {current?.locked?<PrivacyGate de={de}/>:<ImageCanvas preview={preview?.id===current?.id&&!current?.discarded?preview?.data??'':''} job={current} width={request.width} height={request.height} de={de}/>}
    {current && !current.locked && <section className="panel image-result" data-testid="image-result">
      <div className="section-heading"><h2>{phases[current.phase]?.[de ? 0 : 1] ?? current.phase}</h2>{(activeImage(current) || current.status === 'paused') && <button className="button secondary" disabled={busy} onClick={() => void act(() => imageApi.cancel(current.id))}><X size={15} />{de ? 'Generierung abbrechen' : 'Cancel generation'}</button>}</div>
      {current.status === 'queued' && <p>{de ? `Wartet auf Ausführung · Position ${current.queuePosition ?? '…'}. Dieser Auftrag behält seinen eigenen Prompt und seine Modellwahl.` : `Waiting to run · position ${current.queuePosition ?? '…'}. This job keeps its own prompt and model selection.`}</p>}
      {current.status === 'paused' && <button className="button secondary" disabled={busy || disabled} onClick={() => void act(() => imageApi.resume(current.id))}>{de ? 'Auftrag erneut einreihen' : 'Resume queued job'}</button>}
      {current.status === 'running' && <><progress max={current.phase === 'hashing' ? current.modelBytes : (current.samplingSteps??current.request.steps)} value={current.phase === 'hashing' ? current.hashedBytes : current.phase === 'sampling' ? current.step : undefined} aria-label="Image progress" /><p>{current.phase === 'hashing' ? `${formatBytes(current.hashedBytes, language)} / ${formatBytes(current.modelBytes, language)}` : current.phase === 'sampling' ? `${current.step} / ${current.samplingSteps??current.request.steps} ${de ? 'Schritte' : 'steps'}` : (de ? 'Fortschritt für diese Phase nicht messbar.' : 'Progress is not measurable for this phase.')}</p></>}
      {current.error && <p className="notice warning">{imageError(current.error, de)}</p>}
      {current.status === 'completed' && !current.discarded && <><p>{(current.elapsedMs / 1000).toFixed(1)} s · {current.request.width} × {current.request.height} · Seed {current.request.seed}</p><button className="button primary" disabled={busy || !!current.savedPath} onClick={() => void act(() => imageApi.save(current.id))}><Save size={16} />{current.savedPath ? (de ? 'In Galerie gespeichert' : 'Saved to gallery') : (de ? 'In Galerie speichern' : 'Save to gallery')}</button><button className="button secondary" disabled={busy || projectDisabled} onClick={() => void act(async () => { await onAddToProject(current.id); })}>{de ? 'Ins Projekt übernehmen' : 'Add image to project'}</button>{current.savedPath && <p className="download-path">{displayPath(current.savedPath)}</p>}{!current.savedPath && <p className="hub-hint">{de ? 'Ungespeichertes Ergebnis: bleibt beim Ansichtswechsel erhalten. Beim Beenden kannst du es speichern oder verwerfen.' : 'Unsaved result: kept when switching views. When closing, you can save or discard it.'}</p>}</>}
      {current.status === 'completed' && !current.savedPath && !current.discarded && <button className="button secondary" disabled={busy} onClick={() => void act(async () => { if (await confirm(de ? 'Dieses ungespeicherte Bild endgültig verwerfen?' : 'Permanently discard this unsaved image?', { title: de ? 'Bild verwerfen' : 'Discard image', kind: 'warning' })) await imageApi.discard(current.id); })}>{de ? 'Bild verwerfen' : 'Discard image'}</button>}
      {current.discarded && <p>{de ? 'Ergebnis verworfen. Die Auftragsdaten bleiben im Verlauf.' : 'Result discarded. Job information remains in history.'}</p>}
      <button className="button secondary image-restore" disabled={busy || disabled} onClick={() => onRestore({ ...current.request })}>{de ? 'Einstellungen wiederherstellen' : 'Restore settings'}</button>
      <p className="hub-hint">{de ? 'Übernimmt Modellpfad, Prompts und Parameter in ein neues Formular. Es startet keine Generierung; die Modellprüfung läuft automatisch.' : 'Copies the model path, prompts and parameters into a new form. It does not start generation; model readiness is checked automatically.'}</p>
      <details><summary>{de ? 'Auftrag und technische Details' : 'Job and technical details'}</summary><dl><dt>Prompt</dt><dd>{current.request.prompt}</dd><dt>{de ? 'Modell' : 'Model'}</dt><dd>{displayPath(current.request.modelPath)}</dd><dt>SHA-256</dt><dd>{current.modelSha256 ?? '—'}</dd><dt>Runtime</dt><dd>{current.runtime}</dd><dt>GPU</dt><dd>{current.device}</dd><dt>{de ? 'Parameter' : 'Parameters'}</dt><dd>{current.request.steps} steps · CFG {current.request.guidance} · {current.request.sampler} · Karras</dd></dl><pre>{current.logTail}</pre></details>
    </section>}
    <section className="image-history"><h2>{de ? 'Letzte Aufträge' : 'Recent jobs'}</h2>{visibleJobs.length === 0 && <p className="hub-hint">{galleryOnly ? (de ? 'Noch keine generierten Bilder in der Galerie gespeichert.' : 'No generated images saved to the gallery yet.') : (de ? 'Noch keine Bildgenerierung gestartet.' : 'No image generation started yet.')}</p>}{visibleJobs.slice(0, historyLimit).map(job => <button key={job.id} aria-pressed={current?.id===job.id} className={`image-history-row ${current?.id === job.id ? 'selected' : ''}`} onClick={() => setSelected(job.id)}><span>{job.locked?(de?'18+ · Gesperrt':'18+ · Locked'):job.request.prompt.slice(0,100)}</span><small>{formatDate(job.createdAt, language)} · {phases[job.phase]?.[de ? 0 : 1] ?? job.phase}</small></button>)}{visibleJobs.length > historyLimit && <button className="button secondary" onClick={() => setHistoryLimit(n => n + 50)}>{de ? 'Weitere Aufträge anzeigen' : 'Show more jobs'}</button>}</section>
    </div></div>
  </div>;
}
