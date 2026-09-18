import { useEffect, useRef, useState, type Dispatch, type SetStateAction } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { LocalModel, Snapshot } from './LocalModels';
import { confirm, open } from '@tauri-apps/plugin-dialog';
import { FolderOpen, ImagePlus, RefreshCw, Save, X } from 'lucide-react';
import { imagePhases as phases } from './ImageJobRow';
import { activeImage, imageApi, imageError, type ImageJob, type ImageProbe, type ImageRequest } from './image-api';
import { formatBytes, formatDate, formatGigabytes } from './helpers';
import type { Language } from './types';
import './image-studio.css';
import ImageReferenceInput from './ImageReferenceInput';
import { shortcutFor, type Shortcuts } from './shortcuts';

export default function ImageStudio({ onAddToProject, projectDisabled, shortcuts, language, request, setRequest, selectModel, onRestore, selectedJob, disabled = false, galleryOnly = false }: { onAddToProject: (id: string) => Promise<boolean>; projectDisabled: boolean; shortcuts: Shortcuts; language: Language; request: ImageRequest; setRequest: Dispatch<SetStateAction<ImageRequest>>; selectModel: (path: string) => void; onRestore: (request: ImageRequest) => void; selectedJob?: string | null; disabled?: boolean; galleryOnly?: boolean }) {
  const de = language === 'de';
  const [historyLimit, setHistoryLimit] = useState(50);
  const [batchCount,setBatchCount]=useState(1),[incrementSeed,setIncrementSeed]=useState(true);
  const [models, setModels] = useState<LocalModel[]>([]);
  useEffect(() => { let live = true; void invoke<Snapshot>('model_library_list').then(data => { if (live) setModels(data.entries.filter(m => m.discovery === 'model' && m.format === 'safetensors')); }).catch(e => { if (live) setError(String(e)); }); return () => { live = false; }; }, []);
  const [probe, setProbe] = useState<ImageProbe | null>(null); const [probeBusy, setProbeBusy] = useState(false);
  const [jobs, setJobs] = useState<ImageJob[]>([]); const [selected, setSelected] = useState<string | null>(selectedJob ?? null);
  const [preview, setPreview] = useState(''); const [error, setError] = useState(''); const [busy, setBusy] = useState(false);
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
    let live = true; setPreview('');
    if (current?.status === 'completed' && !current.discarded) void imageApi.output(current.id).then(data => { if (live) setPreview(data); }).catch(e => { if (live) setError(String(e)); });
    return () => { live = false; };
  }, [current?.id, current?.status, current?.discarded, current?.savedPath]);
  useEffect(() => { probeGeneration.current++; setProbeBusy(false); setProbe(null); }, [request.modelPath]);
  function modelPath(value: string) { probeGeneration.current++; setProbeBusy(false); setProbe(null); selectModel(value); }
  async function check() {
    const generation = ++probeGeneration.current; setProbeBusy(true); setError('');
    try { const next = await imageApi.probe(request.modelPath); if (alive.current && generation === probeGeneration.current) setProbe(next); }
    catch (e) { if (alive.current && generation === probeGeneration.current) setError(String(e)); }
    finally { if (alive.current && generation === probeGeneration.current) setProbeBusy(false); }
  }
  async function act(action: () => Promise<unknown>) {
    if (operation.current) return; operation.current = true; setBusy(true); setError('');
    try { await action(); await refresh(); } catch (e) { if (alive.current) setError(String(e)); }
    finally { operation.current = false; if (alive.current) setBusy(false); }
  }
  return <div className="page image-studio" onKeyDown={event => {
    if (event.defaultPrevented || event.repeat || disabled || busy || galleryOnly || (event.target as HTMLElement).closest('dialog')) return;
    if ((event.target as HTMLElement).closest('input,textarea,select,[contenteditable="true"]') && !event.ctrlKey && !event.altKey) return;
    if (shortcutFor(event.nativeEvent, shortcuts) === 'generate') { const button = event.currentTarget.querySelector<HTMLButtonElement>('[data-image-generate]'); if (button && !button.disabled) { event.preventDefault(); button.click(); } }
  }}>
    <header className="page-heading"><div><div className="eyebrow">{de ? 'LOKALE BILDGENERIERUNG' : 'LOCAL IMAGE GENERATION'}</div><h1>{galleryOnly ? (de ? 'Generierte Bilder' : 'Generated images') : 'Studio · Image'}</h1><p>{galleryOnly ? (de ? 'Gespeicherte Bilder aus deinen lokalen Generierungen.' : 'Saved images from your local generations.') : (de ? 'Text-to-Image und Bild-zu-Bild mit lokalen SDXL-Checkpoints. Modelle und Prompts bleiben auf deinem PC.' : 'Text-to-image and image-to-image with local SDXL checkpoints. Models and prompts stay on your PC.')}</p></div></header>
    {error && <p className="notice warning" role="alert">{imageError(error, de)}</p>}
    {!galleryOnly && (active || waiting.length > 0) && <section className="notice image-queue-summary" role="status"><span>{active ? (de ? 'Ein Bildauftrag läuft.' : 'An image job is running.') : ''} {waiting.length} {de ? 'in der Warteschlange' : 'in the queue'}.</span>{active && <button className="text-button" onClick={() => setSelected(active.id)}>{de ? 'Laufenden Auftrag anzeigen' : 'Show running job'}</button>}</section>}
    {!galleryOnly && <fieldset disabled={disabled} className="panel image-config">
      <h2>{de ? 'Modell und Ausführbarkeit' : 'Model and readiness'}</h2>
      <label className="field-label" htmlFor="image-model-library">{de ? 'Modelle aus deiner Bibliothek' : 'Models from your library'}</label>
      <select id="image-model-library" className="image-model-select" value={models.some(m => m.path === request.modelPath) ? request.modelPath : ''} onChange={e => modelPath(e.target.value)}>
        <option value="">{de ? 'Modell auswählen …' : 'Select a model …'}</option>
        {models.map(model => <option key={model.id} value={model.path} disabled={['missing','unavailable','invalid','incomplete'].includes(model.status)}>{model.name} · {formatGigabytes(model.totalBytes, language)} · {['missing','unavailable'].includes(model.status) ? (de ? 'nicht verfügbar' : 'unavailable') : ['invalid','incomplete'].includes(model.status) ? (de ? 'fehlerhaft/unvollständig' : 'invalid/incomplete') : model.path === request.modelPath && probe ? (probe.ready ? 'SDXL ✓' : (de ? 'nicht ausführbar' : 'not ready')) : (de ? 'Ausführbarkeit prüfen' : 'check readiness')}</option>)}
      </select>
      <p className="hub-hint">{de ? 'Die Auswahl lädt noch kein Modell in den Grafikspeicher. Die Prüfung unten zeigt, ob es mit diesem Adapter ausführbar ist.' : 'Selecting a model does not load it into graphics memory. The check below determines whether this adapter can run it.'}</p>
      <label className="field-label" htmlFor="image-model">{de ? 'SDXL-Modelldatei' : 'SDXL model file'}</label>
      <div className="image-model-path"><input id="image-model" value={request.modelPath} onChange={e => { probeGeneration.current++; setProbe(null); setRequest(r => ({ ...r, modelPath: e.target.value })); }} spellCheck={false} placeholder="D:\Modelle\modell.safetensors" />
        <button className="button secondary" disabled={busy} onClick={() => void act(async () => { const path = await open({ multiple: false, filters: [{ name: 'Safetensors', extensions: ['safetensors'] }] }); if (typeof path === 'string') modelPath(path); })}><FolderOpen size={16} />{de ? 'Modell wählen' : 'Choose model'}</button>
        <button className="button secondary" disabled={probeBusy || busy} onClick={() => void check()}><RefreshCw size={16} />{probeBusy ? (de ? 'Wird geprüft …' : 'Checking …') : (de ? 'Ausführbarkeit prüfen' : 'Check readiness')}</button></div>
      <p className="hub-hint">{de ? 'Dieser erste Adapter unterstützt einzelne SDXL-Safetensors-Dateien mit UNet, CLIP-L, CLIP-G und VAE. Diffusers-Ordner, SD 1.5, FLUX, Z-Image und Erweiterungen sind hier noch nicht ausführbar.' : 'This first adapter supports single SDXL Safetensors files with UNet, CLIP-L, CLIP-G and VAE. Diffusers folders, SD 1.5, FLUX, Z-Image and extensions cannot run here yet.'}</p>
      {probe && <div className={`image-readiness ${probe.ready ? 'ready' : ''}`} data-testid="image-readiness">
        <strong>{probe.ready ? (de ? 'Bereit für den Generierungsversuch' : 'Ready to attempt generation') : (de ? 'Noch nicht ausführbar' : 'Not ready')}</strong>
        {probe.missing.length > 0 && <ul>{probe.missing.map(code => <li key={code}>{imageError(code, de)}</li>)}</ul>}
        <p>{probe.family ?? (de ? 'SDXL nicht vollständig erkannt' : 'Complete SDXL not recognized')} · {formatGigabytes(probe.modelBytes, language)} · {probe.runtime}</p>
        <p>{probe.device?.split('\t').slice(1).join(' ') ?? (de ? 'Keine unterstützte GPU verfügbar' : 'No supported GPU available')} · VRAM: {probe.vramBytes === null ? (de ? 'unbekannt' : 'unknown') : formatBytes(probe.vramBytes, language)}</p>
        <p>{de ? 'Runtime-Lizenz: MIT. Modelllizenz: unbekannt – Bedingungen der Modellquelle beachten. Die Vorprüfung ersetzt keinen erfolgreichen Modelllauf.' : 'Runtime license: MIT. Model license: unknown — check the model source terms. Preflight does not replace a successful model run.'}</p>
      </div>}
      <div className="image-prompt-grid"><label className="field-label">Prompt<textarea aria-label="Image prompt" value={request.prompt} onChange={e => setRequest(r => ({ ...r, prompt: e.target.value }))} rows={4} maxLength={4000} placeholder={de ? 'Beschreibe dein Bild …' : 'Describe your image …'} /></label><label className="field-label">{de ? 'Negativer Prompt' : 'Negative prompt'}<textarea aria-label="Negative prompt" value={request.negativePrompt} onChange={e => setRequest(r => ({ ...r, negativePrompt: e.target.value }))} rows={4} maxLength={4000} /></label></div>
      <ImageReferenceInput request={request} onChange={patch=>setRequest(r=>({...r,...patch}))} de={de} disabled={disabled||busy}/>
      <div className="image-parameters">
        {(['width','height'] as const).map(key => <label className="field-label" key={key}>{key === 'width' ? (de ? 'Breite' : 'Width') : (de ? 'Höhe' : 'Height')}<select aria-label={key === 'width' ? 'Image width' : 'Image height'} value={request[key]} onChange={e => setRequest(r => ({ ...r, [key]: Number(e.target.value) }))}>{[512,768,1024].map(n => <option key={n} value={n}>{n} px</option>)}</select></label>)}
        <label className="field-label">{de ? 'Schritte' : 'Steps'}<input aria-label="Image steps" type="number" min={1} max={60} value={request.steps} onChange={e => setRequest(r => ({ ...r, steps: Number(e.target.value) }))} /></label>
        <label className="field-label">Guidance<input aria-label="Image guidance" type="number" min={1} max={20} step={0.5} value={request.guidance} onChange={e => setRequest(r => ({ ...r, guidance: Number(e.target.value) }))} /></label>
        <label className="field-label">Seed<input aria-label="Image seed" type="number" min={0} max={4294967295} value={request.seed} onChange={e => setRequest(r => ({ ...r, seed: Number(e.target.value) }))} /></label>
        <label className="field-label">Sampler<select aria-label="Image sampler" value={request.sampler} onChange={e => setRequest(r => ({ ...r, sampler: e.target.value }))}><option value="euler">Euler · Karras</option><option value="dpm++2m">DPM++ 2M · Karras</option></select></label>
      </div>
      <div className="image-parameters"><label className="field-label">{de?'Bilder im Stapel':'Images in batch'}<input aria-label={de?'Bilder im Stapel':'Images in batch'} type="number" min={1} max={20} value={batchCount} disabled={busy} onChange={e=>setBatchCount(Number(e.target.value))}/></label><label className="field-label">{de?'Seeds im Stapel':'Batch seeds'}<select aria-label={de?'Seeds im Stapel':'Batch seeds'} value={incrementSeed?'increment':'same'} disabled={busy||batchCount===1} onChange={e=>setIncrementSeed(e.target.value==='increment')}><option value="increment">{de?'Startseed, danach jeweils +1':'Starting seed, then +1 per image'}</option><option value="same">{de?'Gleicher Seed für alle Bilder':'Same seed for all images'}</option></select></label></div>
      <p className="hub-hint">{de ? 'Ein Bild pro Auftrag; maximal 20 wartende Aufträge. Ausführung nacheinander. Beim Einreihen werden Modellbytes geprüft und bis zum Ende des Auftrags gegen Änderungen gesperrt. Der Worker gibt seinen Speicher nach jedem Auftrag frei.' : 'One image per job; up to 20 waiting jobs. Jobs run sequentially. Enqueuing verifies the model bytes and protects them from changes until the job ends. The worker releases its memory after each job.'}</p>
      <button data-image-generate className="button primary" disabled={busy || !Number.isInteger(batchCount) || batchCount<1 || batchCount>20 || waiting.length+batchCount>20 || !probe?.ready || !request.prompt.trim()} onClick={() => void act(async () => { if(batchCount===1){const job=await imageApi.generate({...request});setSelected(job.id);}else{const result=await imageApi.generateBatch({...request},batchCount,incrementSeed);if(result.jobs[0])setSelected(result.jobs[0].id);if(result.error)setError((de?`${result.jobs.length} Aufträge eingereiht. `:`${result.jobs.length} jobs queued. `)+imageError(result.error,de));} })}><ImagePlus size={17} />{busy ? (de ? 'Modell für Auftrag prüfen …' : 'Verifying model for job …') : (batchCount>1?(de?`${batchCount} Bilder einreihen`:`Queue ${batchCount} images`):active || waiting.length ? (de ? 'Bild einreihen' : 'Queue image') : (de ? 'Bild generieren' : 'Generate image'))}</button>
    </fieldset>}
    {current && <section className="panel image-result" data-testid="image-result">
      <div className="section-heading"><h2>{phases[current.phase]?.[de ? 0 : 1] ?? current.phase}</h2>{(activeImage(current) || current.status === 'paused') && <button className="button secondary" disabled={busy} onClick={() => void act(() => imageApi.cancel(current.id))}><X size={15} />{de ? 'Generierung abbrechen' : 'Cancel generation'}</button>}</div>
      {current.status === 'queued' && <p>{de ? `Wartet auf Ausführung · Position ${current.queuePosition ?? '…'}. Dieser Auftrag behält seinen eigenen Prompt und seine Modellwahl.` : `Waiting to run · position ${current.queuePosition ?? '…'}. This job keeps its own prompt and model selection.`}</p>}
      {current.status === 'paused' && <button className="button secondary" disabled={busy || disabled} onClick={() => void act(() => imageApi.resume(current.id))}>{de ? 'Auftrag erneut einreihen' : 'Resume queued job'}</button>}
      {current.status === 'running' && <><progress max={current.phase === 'hashing' ? current.modelBytes : (current.samplingSteps??current.request.steps)} value={current.phase === 'hashing' ? current.hashedBytes : current.phase === 'sampling' ? current.step : undefined} aria-label="Image progress" /><p>{current.phase === 'hashing' ? `${formatBytes(current.hashedBytes, language)} / ${formatBytes(current.modelBytes, language)}` : current.phase === 'sampling' ? `${current.step} / ${current.samplingSteps??current.request.steps} ${de ? 'Schritte' : 'steps'}` : (de ? 'Fortschritt für diese Phase nicht messbar.' : 'Progress is not measurable for this phase.')}</p></>}
      {current.error && <p className="notice warning">{imageError(current.error, de)}</p>}
      {preview && <img className="generated-image" src={preview} alt={de ? 'Lokal erzeugtes Bild' : 'Locally generated image'} />}
      {current.status === 'completed' && !current.discarded && <><p>{(current.elapsedMs / 1000).toFixed(1)} s · {current.request.width} × {current.request.height} · Seed {current.request.seed}</p><button className="button primary" disabled={busy || !!current.savedPath} onClick={() => void act(() => imageApi.save(current.id))}><Save size={16} />{current.savedPath ? (de ? 'In Galerie gespeichert' : 'Saved to gallery') : (de ? 'In Galerie speichern' : 'Save to gallery')}</button><button className="button secondary" disabled={busy || projectDisabled} onClick={() => void act(async () => { await onAddToProject(current.id); })}>{de ? 'Ins Projekt übernehmen' : 'Add image to project'}</button>{current.savedPath && <p className="download-path">{current.savedPath}</p>}{!current.savedPath && <p className="hub-hint">{de ? 'Ungespeichertes Ergebnis: bleibt beim Ansichtswechsel erhalten. Beim Beenden kannst du es speichern oder verwerfen.' : 'Unsaved result: kept when switching views. When closing, you can save or discard it.'}</p>}</>}
      {current.status === 'completed' && !current.savedPath && !current.discarded && <button className="button secondary" disabled={busy} onClick={() => void act(async () => { if (await confirm(de ? 'Dieses ungespeicherte Bild endgültig verwerfen?' : 'Permanently discard this unsaved image?', { title: de ? 'Bild verwerfen' : 'Discard image', kind: 'warning' })) await imageApi.discard(current.id); })}>{de ? 'Bild verwerfen' : 'Discard image'}</button>}
      {current.discarded && <p>{de ? 'Ergebnis verworfen. Die Auftragsdaten bleiben im Verlauf.' : 'Result discarded. Job information remains in history.'}</p>}
      <button className="button secondary image-restore" disabled={busy || disabled} onClick={() => { probeGeneration.current++; setProbe(null); setProbeBusy(false); onRestore({ ...current.request }); }}>{de ? 'Einstellungen wiederherstellen' : 'Restore settings'}</button>
      <p className="hub-hint">{de ? 'Übernimmt Modellpfad, Prompts und Parameter in ein neues Formular. Es startet keine Generierung; die aktuelle lokale Modelldatei muss erneut geprüft werden.' : 'Copies the model path, prompts and parameters into a new form. It does not start generation; check the current local model file again.'}</p>
      <details><summary>{de ? 'Auftrag und technische Details' : 'Job and technical details'}</summary><dl><dt>Prompt</dt><dd>{current.request.prompt}</dd><dt>{de ? 'Modell' : 'Model'}</dt><dd>{current.request.modelPath}</dd><dt>SHA-256</dt><dd>{current.modelSha256 ?? '—'}</dd><dt>Runtime</dt><dd>{current.runtime}</dd><dt>GPU</dt><dd>{current.device}</dd><dt>{de ? 'Parameter' : 'Parameters'}</dt><dd>{current.request.steps} steps · CFG {current.request.guidance} · {current.request.sampler} · Karras</dd></dl><pre>{current.logTail}</pre></details>
    </section>}
    <section className="image-history"><h2>{de ? 'Letzte Aufträge' : 'Recent jobs'}</h2>{visibleJobs.length === 0 && <p className="hub-hint">{galleryOnly ? (de ? 'Noch keine generierten Bilder in der Galerie gespeichert.' : 'No generated images saved to the gallery yet.') : (de ? 'Noch keine Bildgenerierung gestartet.' : 'No image generation started yet.')}</p>}{visibleJobs.slice(0, historyLimit).map(job => <button key={job.id} className={`image-history-row ${current?.id === job.id ? 'selected' : ''}`} onClick={() => setSelected(job.id)}><span>{job.request.prompt.slice(0,100)}</span><small>{formatDate(job.createdAt, language)} · {phases[job.phase]?.[de ? 0 : 1] ?? job.phase}</small></button>)}{visibleJobs.length > historyLimit && <button className="button secondary" onClick={() => setHistoryLimit(n => n + 50)}>{de ? 'Weitere Aufträge anzeigen' : 'Show more jobs'}</button>}</section>
  </div>;
}
