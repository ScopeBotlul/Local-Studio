import { useEffect, useRef, useState, type CSSProperties, type Dispatch, type PointerEvent as ReactPointerEvent, type SetStateAction } from 'react';
import { invoke } from '@tauri-apps/api/core';
import ImageCanvas from './ImageCanvas';
import ImageDimensions from './ImageDimensions';
import { confirm, open } from '@tauri-apps/plugin-dialog';
import { ArrowDown, ArrowUp, FolderOpen, ImagePlus, RefreshCw, Save, Search, X } from 'lucide-react';
import { imagePhases as phases } from './ImageJobRow';
import { activeImage, imageApi, imageError, saveWithGalleryFallback, validImageDimensions, type ImageModel, type ImageJob, type ImageProbe, type ImageRequest } from './image-api';
import { displayPath, formatBytes, formatDate, formatGigabytes } from './helpers';
import type { Language } from './types';
import './image-studio.css';
import ImageEditSource from './ImageEditSource';
import ImageEditCanvas,{type EditCanvasHandle} from './ImageEditCanvas';
import StudioGallery from './StudioGallery';
import { shortcutFor, type Shortcuts } from './shortcuts';
import TagImporter from './TagImporter';
import CivitaiImport from './CivitaiImport';

interface LoraCandidate {id:string;name:string;path:string;status:string;profile?:{role:string;family:string|null;baseFamily?:string|null}|null}
interface ModelLibrarySnapshot {entries:LoraCandidate[]}
type StudioPanelWidths={left:number;right:number};
const defaultPanelWidths:StudioPanelWidths={left:410,right:300};
function savedPanelWidths():StudioPanelWidths{try{const value=JSON.parse(localStorage.getItem('image-studio-panel-widths')??'{}');return {left:Number.isFinite(value.left)?Math.max(320,Math.min(560,value.left)):defaultPanelWidths.left,right:Number.isFinite(value.right)?Math.max(250,Math.min(440,value.right)):defaultPanelWidths.right};}catch{return defaultPanelWidths;}}

// Advisory UI results only. Queue admission always checks current bytes and runtime.
const readinessCache = new Map<string, {at:number;probe:ImageProbe}>();
export const modelDisplayName=(name:string)=>name.replace(/\.safetensors$/i,'');
export function randomSeed(random:()=>number=()=>crypto.getRandomValues(new Uint32Array(1))[0]):number{return (random()>>>0)%4294967277;}

export default function ImageStudio({ onAddToProject, projectDisabled, shortcuts, language, request, setRequest, selectModel, onRestore, selectedJob, disabled = false, galleryOnly = false, removeCensorTags=false, editing=false }: { onAddToProject: (id: string) => Promise<boolean>; projectDisabled: boolean; shortcuts: Shortcuts; language: Language; request: ImageRequest; setRequest: Dispatch<SetStateAction<ImageRequest>>; selectModel: (path: string) => void; onRestore: (request: ImageRequest) => void; selectedJob?: string | null; disabled?: boolean; galleryOnly?: boolean; removeCensorTags?:boolean;editing?:boolean }) {
  const de = language === 'de';
  const editCanvas=useRef<EditCanvasHandle>(null);
  const [editValid,setEditValid]=useState(false),[showEditResult,setShowEditResult]=useState(false);
  useEffect(()=>setShowEditResult(false),[request.reference?.sha256]);
  const [historyLimit, setHistoryLimit] = useState(50);
  const [panelWidths,setPanelWidths]=useState<StudioPanelWidths>(savedPanelWidths);
  const panelDrag=useRef<{side:keyof StudioPanelWidths;startX:number;startWidth:number}|null>(null);
  const [galleryFolder,setGalleryFolder]=useState(()=>{try{return sessionStorage.getItem('studio-gallery-folder')??'';}catch{return '';}}),[galleryRefresh,setGalleryRefresh]=useState(0);
  const [dimensionsEditingValid,setDimensionsEditingValid]=useState(true);
  const [batchCount,setBatchCount]=useState(1),[incrementSeed,setIncrementSeed]=useState(true),[randomizeSeed,setRandomizeSeed]=useState(true);
  const [promptSearchOpen,setPromptSearchOpen]=useState(false),[promptSearch,setPromptSearch]=useState(''),[promptMatch,setPromptMatch]=useState(0);
  const promptRef=useRef<HTMLTextAreaElement>(null),negativeRef=useRef<HTMLTextAreaElement>(null);
  const [models, setModels] = useState<ImageModel[]>([]);
  const [loraCandidates,setLoraCandidates]=useState<LoraCandidate[]>([]);
  const [modelsBusy,setModelsBusy]=useState(true);
  useEffect(() => { let live=true; setModelsBusy(true); void Promise.all([invoke<ImageModel[]>('image_model_catalog'),invoke<ModelLibrarySnapshot>('model_library_list')]).then(([data,library])=>{if(live){setModels(data);setLoraCandidates(library.entries.filter(entry=>entry.profile?.role==='extension'&&entry.profile.family==='LoRA'&&['checked','recognized'].includes(entry.status)));}}).catch(e=>{if(live)setError(String(e));}).finally(()=>{if(live)setModelsBusy(false);});return()=>{live=false;}; }, []);
  const [probe, setProbe] = useState<ImageProbe | null>(null); const [probeBusy, setProbeBusy] = useState(false);
  const [jobs, setJobs] = useState<ImageJob[]>([]); const [selected, setSelected] = useState<string | null>(selectedJob ?? null);
  const [preview, setPreview] = useState<{id:string;data:string}|null>(null); const [error, setError] = useState(''); const [busy, setBusy] = useState(false);
  const alive = useRef(true); const probeGeneration = useRef(0); const operation = useRef(false);
  const relevant=jobs.filter(job=>galleryOnly||!!job.request.reference===editing);
  const active = relevant.find(j => j.status === 'running'); const waiting = relevant.filter(j => j.status === 'queued'); const visibleJobs = galleryOnly ? relevant.filter(j => j.savedPath) : relevant;
  const current = visibleJobs.find(j => j.id === selected) ?? (galleryOnly ? visibleJobs[0] : active ?? waiting[0]);
  async function refresh() { const list = await imageApi.jobs(); if (alive.current) setJobs(list); }
  useEffect(() => {
    alive.current = true; let loading = false;
    const poll = async () => { if (loading) return; loading = true; try { await refresh(); } catch (e) { if (alive.current) setError(String(e)); } finally { loading = false; } };
    void poll(); const timer = setInterval(() => void poll(), 600);
    return () => { alive.current = false; probeGeneration.current++; clearInterval(timer); };
  }, []);
  useEffect(() => {
    let live = true; setPreview(null);
    if (current?.status === 'completed' && !current.discarded) void imageApi.output(current.id).then(data => { if (live) setPreview({id:current.id,data}); }).catch(e => { if (live) setError(String(e)); });
    return () => { live = false; };
  }, [current?.id, current?.status, current?.discarded, current?.savedPath]);
  useEffect(() => {
    probeGeneration.current++; setProbe(null); setProbeBusy(!!request.modelPath);
    if (!request.modelPath || galleryOnly) {setProbeBusy(false);return;}
    const timer=setTimeout(()=>void check(false),450);
    return()=>{clearTimeout(timer);probeGeneration.current++;};
  }, [request.modelPath, request.engine, request.loras?.length, !!request.reference, galleryOnly]);
  useEffect(()=>{if(selectedJob)setSelected(selectedJob);},[selectedJob]);
  useEffect(()=>{try{localStorage.setItem('image-studio-panel-widths',JSON.stringify(panelWidths));}catch{/* local preference only */}},[panelWidths]);
  function adjustPanel(side:keyof StudioPanelWidths,delta:number){setPanelWidths(current=>({...current,[side]:Math.max(side==='left'?320:250,Math.min(side==='left'?560:440,current[side]+delta))}));}
  function startPanelDrag(side:keyof StudioPanelWidths,event:ReactPointerEvent<HTMLDivElement>){event.preventDefault();panelDrag.current={side,startX:event.clientX,startWidth:panelWidths[side]};event.currentTarget.setPointerCapture(event.pointerId);}
  function movePanelDrag(event:ReactPointerEvent<HTMLDivElement>){const drag=panelDrag.current;if(!drag)return;const scale=Number(getComputedStyle(document.documentElement).getPropertyValue('--ui-scale'))||1;const direction=drag.side==='left'?1:-1;const value=drag.startWidth+(event.clientX-drag.startX)/scale*direction;setPanelWidths(current=>({...current,[drag.side]:Math.max(drag.side==='left'?320:250,Math.min(drag.side==='left'?560:440,value))}));}
  function stopPanelDrag(){panelDrag.current=null;}
  function modelPath(value: string) { if(value===request.modelPath)return; probeGeneration.current++; setProbeBusy(false); setProbe(null); selectModel(value); }
  async function check(force=true) {
    const generation = ++probeGeneration.current; setProbeBusy(true); setError('');
    try { const cacheKey=[request.modelPath,request.engine??'auto',request.loras?.length?'lora':'',request.reference?'reference':''].join('|'); const cached=readinessCache.get(cacheKey); const next = !force && cached && Date.now()-cached.at<60000 ? cached.probe : await imageApi.probe(request.modelPath,request.engine??'auto',!!request.loras?.length,!!request.reference); if(readinessCache.size>=64)readinessCache.clear(); readinessCache.set(cacheKey,{at:Date.now(),probe:next}); if (alive.current && generation === probeGeneration.current) setProbe(next); }
    catch (e) { if (alive.current && generation === probeGeneration.current) setError(String(e)); }
    finally { if (alive.current && generation === probeGeneration.current) setProbeBusy(false); }
  }
  async function act(action: () => Promise<unknown>) {
    if (operation.current) return; operation.current = true; setBusy(true); setError('');
    try { await action(); await refresh(); } catch (e) { if (alive.current) setError(String(e)); }
    finally { operation.current = false; if (alive.current) setBusy(false); }
  }
  async function saveToGallery(id:string){
    const saved=await saveWithGalleryFallback(id,galleryFolder,imageApi.save);
    if(saved.folder!==galleryFolder){setGalleryFolder(saved.folder);try{sessionStorage.setItem('studio-gallery-folder',saved.folder);}catch{/* session only */}}
    setGalleryRefresh(n=>n+1);
  }
  const dimensionsValid=dimensionsEditingValid&&validImageDimensions(request.width,request.height);
  const referenceValid=!editing||!request.reference||(request.width===request.reference.width&&request.height===request.reference.height);
  const parametersValid=Number.isInteger(request.steps)&&request.steps>=1&&request.steps<=60&&Number.isFinite(request.guidance)&&request.guidance>=1&&request.guidance<=20&&Number.isInteger(request.seed)&&request.seed>=0&&request.seed<=4294967295&&(randomizeSeed||!incrementSeed||request.seed+batchCount-1<=4294967295);
  const promptMatches=(()=>{const query=promptSearch.toLocaleLowerCase();if(!query)return [] as {field:'prompt'|'negative';start:number}[];const found:{field:'prompt'|'negative';start:number}[]=[];for(const [field,value] of [['prompt',request.prompt],['negative',request.negativePrompt]] as const){const text=value.toLocaleLowerCase();let start=0;while(found.length<500&&(start=text.indexOf(query,start))>=0){found.push({field,start});start+=Math.max(1,query.length);}}return found;})();
  function showPromptMatch(index:number){if(!promptMatches.length)return;const next=(index+promptMatches.length)%promptMatches.length;setPromptMatch(next);const match=promptMatches[next];requestAnimationFrame(()=>{const field=match.field==='prompt'?promptRef.current:negativeRef.current;field?.focus();field?.setSelectionRange(match.start,match.start+promptSearch.length);});}
  return <div className="page image-studio" onKeyDown={event => {
    if((event.ctrlKey||event.metaKey)&&event.key.toLocaleLowerCase()==='f'){event.preventDefault();setPromptSearchOpen(true);requestAnimationFrame(()=>document.querySelector<HTMLInputElement>('[data-prompt-search]')?.focus());return;}
    if (event.defaultPrevented || event.repeat || disabled || busy || galleryOnly || (event.target as HTMLElement).closest('dialog')) return;
    if ((event.target as HTMLElement).closest('input,textarea,select,[contenteditable="true"]') && !event.ctrlKey && !event.altKey) return;
    if (shortcutFor(event.nativeEvent, shortcuts) === 'generate') { const button = event.currentTarget.querySelector<HTMLButtonElement>('[data-image-generate]'); if (button && !button.disabled) { event.preventDefault(); button.click(); } }
  }}>
    {error && <p className="notice warning" role="alert">{imageError(error, de)}</p>}
    <div className={`image-workspace ${galleryOnly?'gallery-only':''}`} style={galleryOnly?undefined:{'--image-left-panel':`${panelWidths.left}px`,'--image-right-panel':`${panelWidths.right}px`} as CSSProperties}>
    {!galleryOnly && <fieldset disabled={disabled} className="panel image-config">
      {editing&&<ImageEditSource request={request} onChange={patch=>setRequest(r=>({...r,...patch}))} de={de} disabled={disabled||busy}/>}
      <div className="section-heading"><h2>{editing?(de?'2. Modell und Änderung':'2. Model and change'):(de?'Modell':'Model')}</h2><button type="button" className="text-button" disabled={busy} aria-label={de?'Modelldatei wählen':'Choose model file'} onClick={()=>void act(async()=>{const path=await open({multiple:false,filters:[{name:'SDXL · Safetensors',extensions:['safetensors']}]});if(typeof path==='string')modelPath(path);})}><FolderOpen size={17}/></button></div>
      <label className="field-label" htmlFor="image-model-library">{de?'SDXL-Checkpoints':'SDXL checkpoints'}</label>
      <select id="image-model-library" className="image-model-select" disabled={modelsBusy} value={models.some(m=>m.path===request.modelPath)?request.modelPath:''} onChange={e=>modelPath(e.target.value)}>
        <option value="">{modelsBusy?(de?'Modelle werden erkannt …':'Identifying models …'):request.modelPath?modelDisplayName(displayPath(request.modelPath).split(/[\\/]/).pop()??''):(de?'Modell auswählen …':'Select a model …')}</option>
        {models.filter(model=>!editing||/\.safetensors$/i.test(model.path)).map(model=><option key={model.id} value={model.path}>{modelDisplayName(model.name)} · {formatGigabytes(model.totalBytes,language)}{model.restricted?' · 18+':''}</option>)}
      </select>
      <div className={'image-model-status '+(probe?.ready?'ready':'')} role="status">{probeBusy?(de?'Ausführbarkeit wird automatisch geprüft …':'Checking readiness automatically …'):probe?.ready?`${probe.family ?? (de?'Bildmodell':'Image model')} ${de?'bereit · Prüfung vor dem Start automatisch':'ready · checked automatically before starting'}`:probe?(de?'Modell oder Laufzeit noch nicht bereit':'Model or runtime not ready'):(editing?(de?'SDXL-Checkpoint für die Bildbearbeitung wählen':'Choose an SDXL checkpoint for editing'):(de?'Vollständigen Bild-Checkpoint oder Qwen Image 2.1 GGUF wählen':'Choose a complete image checkpoint or Qwen Image 2.1 GGUF'))}</div>
      {!modelsBusy&&!models.length&&<p className="hub-hint">{de?'Keine passenden Checkpoints in der Bibliothek. Über das Ordnersymbol eine Datei wählen.':'No compatible checkpoints in the library. Choose a file with the folder button.'}</p>}
      {probe&&!probe.ready&&<ul className="image-model-errors">{probe.missing.map(code=><li key={code}>{imageError(code,de)}</li>)}</ul>}

      {promptSearchOpen&&<div className="prompt-search" role="search"><Search size={15}/><input data-prompt-search value={promptSearch} onChange={e=>{setPromptSearch(e.target.value);setPromptMatch(0);}} onKeyDown={e=>{if(e.key==='Enter'){e.preventDefault();showPromptMatch(promptMatch+(e.shiftKey?-1:1));}if(e.key==='Escape'){e.preventDefault();setPromptSearchOpen(false);}}} placeholder={de?'In beiden Prompts suchen …':'Search both prompts …'}/><span>{promptMatches.length?`${Math.min(promptMatch+1,promptMatches.length)}/${promptMatches.length}`:'0/0'}</span><button type="button" className="icon-button" disabled={!promptMatches.length} aria-label={de?'Vorheriger Treffer':'Previous match'} onClick={()=>showPromptMatch(promptMatch-1)}><ArrowUp size={14}/></button><button type="button" className="icon-button" disabled={!promptMatches.length} aria-label={de?'Nächster Treffer':'Next match'} onClick={()=>showPromptMatch(promptMatch+1)}><ArrowDown size={14}/></button><button type="button" className="icon-button" aria-label={de?'Suche schließen':'Close search'} onClick={()=>setPromptSearchOpen(false)}><X size={14}/></button></div>}
      <div className="image-prompt-grid"><label className="field-label">Prompt<textarea ref={promptRef} aria-label="Image prompt" value={request.prompt} onChange={e=>setRequest(r=>({...r,prompt:e.target.value}))} rows={5} maxLength={4000} placeholder={editing?(de?'Was soll im markierten Bereich entstehen oder verändert werden?':'What should appear or change in the selected regions?'):(de?'Motiv, Stil, Licht und Details …':'Subject, style, lighting and details …')}/></label><details className="image-control-details" open><summary>{de?'Negativer Prompt':'Negative prompt'}{request.negativePrompt?' •':''}</summary><textarea ref={negativeRef} aria-label="Negative prompt" value={request.negativePrompt} onChange={e=>setRequest(r=>({...r,negativePrompt:e.target.value}))} rows={3} maxLength={4000} placeholder={de?'Was soll im Bild vermieden werden?':'What should the image avoid?'}/></details></div>
      <TagImporter de={de} removeCensor={removeCensorTags} onImport={tags=>setRequest(r=>({...r,prompt:[r.prompt.trim(),tags.join(', ')].filter(Boolean).join(', ').slice(0,4000)}))}/>
      <details className="image-control-details"><summary>{de?'Einstellungen aus Civitai übernehmen':'Import Civitai settings'}</summary><CivitaiImport language={language} folder={galleryFolder} onApply={patch=>setRequest(r=>({...r,...patch,reference:editing?r.reference:null}))}/></details>
      {!editing&&<ImageDimensions request={request} onChange={patch=>setRequest(r=>({...r,...patch}))} onValidityChange={setDimensionsEditingValid} de={de}/>}
      <details className="image-control-details"><summary>LoRA{request.loras?.length?` · ${request.loras.length}`:''}</summary>
        <p className="hub-hint">{de?'LoRAs werden modellbezogen gewählt. Vulkan kann jede lokale Safetensors-LoRA sicher für den Auftrag einbinden; ComfyUI verwendet LoRAs aus seinem Modellordner.':'LoRAs are selected per image setup. Vulkan can safely stage any local Safetensors LoRA for the job; ComfyUI uses LoRAs from its model folder.'}</p>
        <div className="image-model-path"><select aria-label={de?'LoRA hinzufügen':'Add LoRA'} value="" disabled={busy||(request.loras?.length??0)>=8} onChange={e=>{const candidate=loraCandidates.find(item=>item.path===e.target.value);if(candidate&&!request.loras?.some(item=>item.path===candidate.path))setRequest(r=>({...r,loras:[...(r.loras??[]),{path:candidate.path,strength:1}]}));}}><option value="">{de?'LoRA aus Modellbibliothek hinzufügen …':'Add LoRA from model library …'}</option>{loraCandidates.filter(candidate=>!request.loras?.some(item=>item.path===candidate.path)).map(candidate=><option key={candidate.id} value={candidate.path}>{modelDisplayName(candidate.name)}{candidate.profile?.baseFamily?` · ${candidate.profile.baseFamily}`:''}</option>)}</select><button type="button" className="button secondary" disabled={busy||(request.loras?.length??0)>=8} onClick={()=>void act(async()=>{const path=await open({multiple:false,filters:[{name:'LoRA · Safetensors',extensions:['safetensors']}]});if(typeof path==='string'&&!request.loras?.some(item=>item.path===path))setRequest(r=>({...r,loras:[...(r.loras??[]),{path,strength:1}]}));})}><FolderOpen size={16}/>{de?'Datei wählen':'Choose file'}</button></div>
        <div className="image-lora-list">{request.loras?.map((lora,index)=><div className="image-lora-row" key={`${lora.path}-${index}`}><span title={lora.path}>{modelDisplayName(displayPath(lora.path).split(/[\\/]/).pop()??lora.path)}</span><label>{de?'Stärke':'Strength'}<input type="number" min={-2} max={2} step={0.05} value={lora.strength} onChange={e=>setRequest(r=>({...r,loras:(r.loras??[]).map((item,i)=>i===index?{path:item.path,strength:Number(e.target.value)}:item)}))}/></label><button type="button" className="icon-button" aria-label={de?'LoRA entfernen':'Remove LoRA'} onClick={()=>setRequest(r=>({...r,loras:(r.loras??[]).filter((_,i)=>i!==index)}))}><X size={15}/></button></div>)}</div>
        {!request.loras?.length&&<p className="hub-hint">{de?'Keine LoRA aktiv.':'No LoRA active.'}</p>}
      </details>
      <details className="image-control-details"><summary>{de?'Engine und Modellinformationen':'Engine and model details'}</summary>
      <label className="field-label" htmlFor="image-engine">{de?'Bildengine':'Image engine'}</label>
      <select id="image-engine" className="image-model-select" value={request.engine??'auto'} disabled={busy} onChange={event=>setRequest(current=>({...current,engine:event.target.value as 'auto'|'vulkan'|'comfy'}))}>
        <option value="auto">{de?'Automatisch · für diese Hardware empfohlen':'Automatic · recommended for this hardware'}</option>
        <option value="vulkan">Vulkan · stable-diffusion.cpp</option>
        <option value="comfy">ComfyUI · API</option>
      </select>
      <p className="hub-hint">{de?'Automatisch verwendet Vulkan auf AMD-/Intel-Handhelds und bevorzugt ComfyUI auf NVIDIA. LoRAs funktionieren mit beiden Engines; Referenzbilder verwenden Vulkan.':'Automatic uses Vulkan on AMD/Intel handhelds and prefers ComfyUI on NVIDIA. LoRAs work with both engines; reference images use Vulkan.'}</p>
      <label className="field-label" htmlFor="image-model">{de ? 'SDXL-Modelldatei' : 'SDXL model file'}</label>
      {editing&&<><p className="hub-hint">{de?'KI-Bildbearbeitung verwendet lokale SDXL-Modelle mit Vulkan.':'AI image editing uses local SDXL models with Vulkan.'}</p><label className="privacy-check"><input type="checkbox" checked={request.vaeOnCpu??false} disabled={busy} onChange={event=>setRequest(r=>({...r,vaeOnCpu:event.target.checked}))}/>{de?'VAE auf der CPU ausführen (spart Grafikspeicher, kann länger dauern)':'Run VAE on CPU (saves GPU memory, may take longer)'}</label></>}
      <div className="image-model-path"><input id="image-model" value={displayPath(request.modelPath)} onChange={e => { probeGeneration.current++; setProbe(null); setRequest(r => ({ ...r, modelPath: e.target.value })); }} spellCheck={false} placeholder="D:\Modelle\modell.safetensors oder qwen-image-2.1-…gguf" />
        <button className="button secondary" disabled={busy} onClick={() => void act(async () => { const path = await open({ multiple: false, filters: [{ name: 'Safetensors', extensions: ['safetensors'] }] }); if (typeof path === 'string') modelPath(path); })}><FolderOpen size={16} />{de ? 'Modell wählen' : 'Choose model'}</button>
        <button className="button secondary" disabled={probeBusy || busy} onClick={() => void check()}><RefreshCw size={16} />{probeBusy ? (de ? 'Wird geprüft …' : 'Checking …') : (de ? 'Erneut prüfen' : 'Recheck')}</button></div>
      <p className="hub-hint">{de ? 'Dieser erste Adapter unterstützt einzelne SDXL-Safetensors-Dateien mit UNet, CLIP-L, CLIP-G und VAE. Diffusers-Ordner, SD 1.5, FLUX, Z-Image und Erweiterungen sind hier noch nicht ausführbar.' : 'This first adapter supports single SDXL Safetensors files with UNet, CLIP-L, CLIP-G and VAE. Diffusers folders, SD 1.5, FLUX, Z-Image and extensions cannot run here yet.'}</p>
      {probe && <div className={`image-readiness ${probe.ready ? 'ready' : ''}`} data-testid="image-readiness">
        <strong>{probe.ready ? (de ? 'Bereit für den Generierungsversuch' : 'Ready to attempt generation') : (de ? 'Noch nicht ausführbar' : 'Not ready')}</strong>
        {probe.missing.length > 0 && <ul>{probe.missing.map(code => <li key={code}>{imageError(code, de)}</li>)}</ul>}
        <p>{probe.family ?? (de ? 'SDXL nicht vollständig erkannt' : 'Complete SDXL not recognized')} · {formatGigabytes(probe.modelBytes, language)} · {probe.runtime}</p>
        <p>{probe.device?.split('\t').slice(1).join(' ') ?? (de ? 'Keine unterstützte GPU verfügbar' : 'No supported GPU available')} · VRAM: {probe.vramBytes === null ? (de ? 'unbekannt' : 'unknown') : formatBytes(probe.vramBytes, language)}</p>
        <p>{de ? `Runtime-Lizenz: ${probe.runtimeLicense}. Modelllizenz: unbekannt – Bedingungen der Modellquelle beachten. Die Vorprüfung ersetzt keinen erfolgreichen Modelllauf.` : `Runtime license: ${probe.runtimeLicense}. Model license: unknown — check the model source terms. Preflight does not replace a successful model run.`}</p>
      </div>}
      </details>
      <details className="image-control-details"><summary>{de?'Generierungseinstellungen':'Generation settings'} · {request.steps} {de?'Schritte':'steps'}</summary>
      <div className="image-parameters">
        <label className="field-label">{de ? 'Schritte' : 'Steps'}<input aria-label="Image steps" type="number" min={1} max={60} value={request.steps} onChange={e => setRequest(r => ({ ...r, steps: Number(e.target.value) }))} /></label>
        <label className="field-label">Guidance<input aria-label="Image guidance" type="number" min={1} max={20} step={0.5} value={request.guidance} onChange={e => setRequest(r => ({ ...r, guidance: Number(e.target.value) }))} /></label>
        <label className="field-label">Seed<input aria-label="Image seed" type="number" min={0} max={4294967295} value={request.seed} disabled={randomizeSeed} onChange={e => setRequest(r => ({ ...r, seed: Number(e.target.value) }))} /></label>
        <label className="field-label">Sampler<select aria-label="Image sampler" value={request.sampler} onChange={e => setRequest(r => ({ ...r, sampler: e.target.value }))}><option value="euler">Euler · Karras</option><option value="dpm++2m">DPM++ 2M · Karras</option></select></label>
      </div>
      <label className="privacy-check"><input type="checkbox" checked={randomizeSeed} disabled={busy} onChange={e=>setRandomizeSeed(e.target.checked)}/>{de?'Bei jeder Generierung einen neuen zufälligen Seed verwenden':'Use a new random seed for every generation'}</label>
      <div className="image-parameters"><label className="field-label">{de?'Bilder im Stapel':'Images in batch'}<input aria-label={de?'Bilder im Stapel':'Images in batch'} type="number" min={1} max={20} value={batchCount} disabled={busy} onChange={e=>setBatchCount(Number(e.target.value))}/></label><label className="field-label">{de?'Seeds im Stapel':'Batch seeds'}<select aria-label={de?'Seeds im Stapel':'Batch seeds'} value={incrementSeed?'increment':'same'} disabled={busy||batchCount===1} onChange={e=>setIncrementSeed(e.target.value==='increment')}><option value="increment">{de?'Startseed, danach jeweils +1':'Starting seed, then +1 per image'}</option><option value="same">{de?'Gleicher Seed für alle Bilder':'Same seed for all images'}</option></select></label></div>
      <p className="hub-hint">{de ? 'Ein Bild pro Auftrag; maximal 20 wartende Aufträge. Ausführung nacheinander. Beim Einreihen werden Modellbytes geprüft und bis zum Ende des Auftrags gegen Änderungen gesperrt. Der Worker gibt seinen Speicher nach jedem Auftrag frei.' : 'One image per job; up to 20 waiting jobs. Jobs run sequentially. Enqueuing verifies the model bytes and protects them from changes until the job ends. The worker releases its memory after each job.'}</p>
      </details>
      {!parametersValid&&<p className="notice warning" role="alert">{de?'Schritte, Guidance oder Seed liegen außerhalb des gültigen Bereichs.':'Steps, guidance or seed are outside the valid range.'}</p>}
      {!dimensionsValid&&<p className="notice warning" role="alert">{de?'Zum Generieren bitte die Bildgröße oben korrigieren oder den vorgeschlagenen Wert übernehmen.':'To generate, correct the image size above or apply the suggested dimensions.'}</p>}
      <div className="image-generate-bar"><button data-image-generate className="button primary" disabled={busy || probeBusy || (editing&&(!request.reference||!editValid)) || !dimensionsValid || !referenceValid || !parametersValid || !Number.isInteger(batchCount) || batchCount<1 || batchCount>20 || waiting.length+batchCount>20 || !probe?.ready || !request.prompt.trim()} onClick={() => void act(async () => { const seed=randomizeSeed?randomSeed():request.seed;let reference=editing?request.reference:null;if(editing&&reference){if(!editCanvas.current)throw new Error('image_mask');const data=editCanvas.current.mask();reference={...reference,mask:data?await invoke('image_edit_mask',{data,reference:{...reference,mask:null}}):null};}const next={...request,seed,reference};if(editing)setShowEditResult(true);if(randomizeSeed)setRequest(r=>({...r,seed}));if(batchCount===1){const job=await imageApi.generate(next);setSelected(job.id);}else{const result=await imageApi.generateBatch(next,batchCount,incrementSeed);if(result.jobs[0])setSelected(result.jobs[0].id);if(result.error)setError((de?`${result.jobs.length} Aufträge eingereiht. `:`${result.jobs.length} jobs queued. `)+imageError(result.error,de));} })}><ImagePlus size={17} />{busy ? (de ? 'Modell für Auftrag prüfen …' : 'Verifying model for job …') : (batchCount>1?(de?`${batchCount} Bilder einreihen`:`Queue ${batchCount} images`):active || waiting.length ? (de ? 'Bild einreihen' : 'Queue image') : (editing?(de?'Änderung anwenden':'Apply edit'):(de ? 'Bild generieren' : 'Generate image')))}</button>
      </div>
    </fieldset>}
    {!galleryOnly&&<div className="image-panel-resizer left" role="separator" aria-orientation="vertical" aria-label={de?'Breite der Einstellungen':'Settings width'} aria-valuemin={320} aria-valuemax={560} aria-valuenow={Math.round(panelWidths.left)} tabIndex={0} title={de?'Ziehen zum Anpassen · Doppelklick zum Zurücksetzen':'Drag to resize · double-click to reset'} onPointerDown={event=>startPanelDrag('left',event)} onPointerMove={movePanelDrag} onPointerUp={stopPanelDrag} onPointerCancel={stopPanelDrag} onLostPointerCapture={stopPanelDrag} onDoubleClick={()=>setPanelWidths(defaultPanelWidths)} onKeyDown={event=>{if(event.key==='Home'){event.preventDefault();setPanelWidths(current=>({...current,left:320}));}else if(event.key==='End'){event.preventDefault();setPanelWidths(current=>({...current,left:560}));}else if(event.key==='ArrowLeft'||event.key==='ArrowRight'){event.preventDefault();adjustPanel('left',event.key==='ArrowLeft'?-20:20);}}}/>} {/* panel divider */}
    <div className="image-workspace-output">
    {editing&&<div className="edit-view-switch"><button type="button" className="button secondary" aria-pressed={!showEditResult} onClick={()=>setShowEditResult(false)}>{de?'Bereiche markieren':'Mark regions'}</button><button type="button" className="button secondary" disabled={!current} aria-pressed={showEditResult} onClick={()=>setShowEditResult(true)}>{de?'Ergebnis':'Result'}</button>{active&&<button type="button" className="button secondary" onClick={()=>void act(()=>imageApi.cancel(active.id))}>{de?'Abbrechen':'Cancel'}</button>}</div>}
    {editing&&<div hidden={showEditResult}><ImageEditCanvas ref={editCanvas} reference={request.reference??null} de={de} disabled={disabled||busy} onValid={setEditValid}/></div>}
    <div hidden={editing&&!showEditResult}><ImageCanvas preview={preview?.id===current?.id&&!current?.discarded?preview?.data??'':''} job={current} width={request.width} height={request.height} de={de}/></div>
    {current && current.status !== 'running' && current.status !== 'queued' && <section className="panel image-result" data-testid="image-result">
      <div className="section-heading"><h2>{phases[current.phase]?.[de ? 0 : 1] ?? current.phase}</h2>{(activeImage(current) || current.status === 'paused') && <button className="button secondary" disabled={busy} onClick={() => void act(() => imageApi.cancel(current.id))}><X size={15} />{de ? 'Generierung abbrechen' : 'Cancel generation'}</button>}</div>
      {current.status === 'paused' && <button className="button secondary" disabled={busy || disabled} onClick={() => void act(() => imageApi.resume(current.id))}>{de ? 'Auftrag erneut einreihen' : 'Resume queued job'}</button>}
      {current.error && <p className="notice warning"><strong>{de?'Fehler dieses gespeicherten Auftrags: ':'Error from this saved job: '}</strong>{imageError(current.error, de)}</p>}
      {current.status === 'completed' && !current.discarded && <><p>{(current.elapsedMs / 1000).toFixed(1)} s · {current.request.width} × {current.request.height} · Seed {current.request.seed}</p><button className="button primary" disabled={busy || !!current.savedPath} onClick={() => void act(()=>saveToGallery(current.id))}><Save size={16} />{current.savedPath ? (de ? 'In Galerie gespeichert' : 'Saved to gallery') : galleryFolder?(de?`In „${galleryFolder.split('/').pop()}“ speichern`:`Save to “${galleryFolder.split('/').pop()}”`):(de ? 'In Galerie speichern' : 'Save to gallery')}</button><button className="button secondary" disabled={busy || projectDisabled} onClick={() => void act(async () => { await onAddToProject(current.id); })}>{de ? 'Ins Projekt übernehmen' : 'Add image to project'}</button>{current.savedPath && <p className="download-path">{displayPath(current.savedPath)}</p>}{!current.savedPath && <p className="hub-hint">{de ? 'Ungespeichertes Ergebnis: bleibt beim Ansichtswechsel erhalten. Beim Beenden kannst du es speichern oder verwerfen.' : 'Unsaved result: kept when switching views. When closing, you can save or discard it.'}</p>}</>}
      {current.status === 'completed' && !current.savedPath && !current.discarded && <button className="button secondary" disabled={busy} onClick={() => void act(async () => { if (await confirm(de ? 'Dieses ungespeicherte Bild endgültig verwerfen?' : 'Permanently discard this unsaved image?', { title: de ? 'Bild verwerfen' : 'Discard image', kind: 'warning' })) await imageApi.discard(current.id); })}>{de ? 'Bild verwerfen' : 'Discard image'}</button>}
      {current.discarded && <p>{de ? 'Ergebnis verworfen. Die Auftragsdaten bleiben im Verlauf.' : 'Result discarded. Job information remains in history.'}</p>}
      <button className="button secondary image-restore" disabled={busy || disabled} onClick={() => onRestore({ ...current.request })}>{de ? 'Einstellungen wiederherstellen' : 'Restore settings'}</button>
      <p className="hub-hint">{de ? 'Übernimmt Modellpfad, Prompts und Parameter in ein neues Formular. Es startet keine Generierung; die Modellprüfung läuft automatisch.' : 'Copies the model path, prompts and parameters into a new form. It does not start generation; model readiness is checked automatically.'}</p>
      <details><summary>{de ? 'Auftrag und technische Details' : 'Job and technical details'}</summary><dl><dt>Prompt</dt><dd>{current.request.prompt}</dd><dt>{de ? 'Modell' : 'Model'}</dt><dd>{displayPath(current.request.modelPath)}</dd><dt>SHA-256</dt><dd>{current.modelSha256 ?? '—'}</dd><dt>Runtime</dt><dd>{current.runtime}</dd><dt>GPU</dt><dd>{current.device}</dd><dt>{de ? 'Parameter' : 'Parameters'}</dt><dd>{current.request.steps} steps · CFG {current.request.guidance} · {current.request.sampler} · Karras</dd></dl><pre>{current.logTail}</pre></details>
    </section>}
    <section className="image-history"><h2>{de ? 'Letzte Aufträge' : 'Recent jobs'}</h2>{visibleJobs.length === 0 && <p className="hub-hint">{galleryOnly ? (de ? 'Noch keine generierten Bilder in der Galerie gespeichert.' : 'No generated images saved to the gallery yet.') : (de ? 'Noch keine Bildgenerierung gestartet.' : 'No image generation started yet.')}</p>}{visibleJobs.slice(0, historyLimit).map(job => <button key={job.id} aria-pressed={current?.id===job.id} className={`image-history-row ${current?.id === job.id ? 'selected' : ''}`} onClick={() => setSelected(job.id)}><span>{job.request.prompt.slice(0,100)}</span><small>{formatDate(job.createdAt, language)} · {phases[job.phase]?.[de ? 0 : 1] ?? job.phase}</small></button>)}{visibleJobs.length > historyLimit && <button className="button secondary" onClick={() => setHistoryLimit(n => n + 50)}>{de ? 'Weitere Aufträge anzeigen' : 'Show more jobs'}</button>}</section>
    </div>{!galleryOnly&&<><div className="image-panel-resizer right" role="separator" aria-orientation="vertical" aria-label={de?'Breite der Studio-Galerie':'Studio gallery width'} aria-valuemin={250} aria-valuemax={440} aria-valuenow={Math.round(panelWidths.right)} tabIndex={0} title={de?'Ziehen zum Anpassen · Doppelklick zum Zurücksetzen':'Drag to resize · double-click to reset'} onPointerDown={event=>startPanelDrag('right',event)} onPointerMove={movePanelDrag} onPointerUp={stopPanelDrag} onPointerCancel={stopPanelDrag} onLostPointerCapture={stopPanelDrag} onDoubleClick={()=>setPanelWidths(defaultPanelWidths)} onKeyDown={event=>{if(event.key==='Home'){event.preventDefault();setPanelWidths(current=>({...current,right:250}));}else if(event.key==='End'){event.preventDefault();setPanelWidths(current=>({...current,right:440}));}else if(event.key==='ArrowLeft'||event.key==='ArrowRight'){event.preventDefault();adjustPanel('right',event.key==='ArrowLeft'?20:-20);}}}/><StudioGallery onSelect={editing?async(entry,root)=>{if(entry.kind!=='image'||busy||disabled)return;try{const result=await invoke<{reference:import('./image-api').ImageReference}>('image_edit_source',{path:root.replace(/[\\/]$/,'')+'/'+entry.path});setRequest(r=>({...r,reference:result.reference,width:result.reference.width,height:result.reference.height}));}catch(e){setError(String(e));}}:undefined} language={language} folder={galleryFolder} refreshKey={galleryRefresh} onFolder={folder=>{setGalleryFolder(folder);try{sessionStorage.setItem('studio-gallery-folder',folder);}catch{/* session only */}}}/></>}</div>
  </div>;
}
