import {useEffect, useRef, useState, type CSSProperties, type PointerEvent as ReactPointerEvent} from 'react';
import {invoke} from '@tauri-apps/api/core';
import {open} from '@tauri-apps/plugin-dialog';
import {ArrowDown, ArrowUp, Film, FolderOpen, Images, LoaderCircle, Maximize, Minus, Plus, RefreshCw, Save, Sparkles, Trash2} from 'lucide-react';
import {displayPath, formatGigabytes} from './helpers';
import {galleryApi, galleryError, type GalleryEntry} from './gallery-api';
import StudioGallery from './StudioGallery';
import TagImporter from './TagImporter';
import type {Language} from './types';
import './image-studio.css';
import './gif-studio.css';

type Mode = 'ai' | 'frames';
type WanModel = {path:string; name:string; bytes:number};
type GifPending = {id:string; bytes:number; createdAt:number};
type Preview = {url:string; path:string; label:string; width?:number; height?:number};
type PanelWidths = {left:number; right:number};
const defaultWidths:PanelWidths = {left:410,right:300};
const imageExtensions = ['png','jpg','jpeg','webp','bmp'];
function fileName(path:string){return displayPath(path).split(/[\\/]/).pop() ?? path;}
function savedWidths():PanelWidths {
  try {
    const value=JSON.parse(localStorage.getItem('gif-studio-panel-widths')??'{}');
    return {
      left:Number.isFinite(value.left)?Math.max(320,Math.min(560,value.left)):defaultWidths.left,
      right:Number.isFinite(value.right)?Math.max(250,Math.min(440,value.right)):defaultWidths.right,
    };
  } catch {return defaultWidths;}
}
function message(error:unknown,de:boolean){
  const code=String(error);
  const labels:Record<string,[string,string]>={
    comfy_missing:['ComfyUI ist nicht eingerichtet. Richte es in den Einstellungen ein.','ComfyUI is not set up. Configure it in Settings.'],
    comfy_connection:['ComfyUI antwortet nicht. Starte die Engine in den Einstellungen.','ComfyUI is not responding. Start the engine in Settings.'],
    gif_ai_model_t2v:['Dieses Modell unterstützt nur Text-zu-Video. Für ein Startbild brauchst du ein Wan-I2V- oder TI2V-Modell.','This model is text-to-video only. A start image requires a Wan I2V or TI2V model.'],
    gif_ai_model_unavailable:['ComfyUI erkennt das gewählte Modell noch nicht. Starte die Engine nach dem Hinzufügen neu.','ComfyUI does not see this model yet. Restart the engine after adding it.'],
    gif_ai_model:['Das Wan-Modell muss im diffusion_models-Ordner der eingerichteten ComfyUI-Installation liegen.','The Wan model must be in the configured ComfyUI diffusion_models folder.'],
    gif_ai_encoder:['Ein Wan-UMT5-Textencoder fehlt in ComfyUI.','A Wan UMT5 text encoder is missing from ComfyUI.'],
    gif_ai_vae_21:['Für dieses Wan-2.1-Modell fehlt eine Wan-2.1-VAE in ComfyUI. Die installierte Wan-2.2-VAE ist nicht kompatibel.','This Wan 2.1 model needs a Wan 2.1 VAE in ComfyUI. The Wan 2.2 VAE is incompatible.'],
    gif_ai_vae_22:['Für dieses Wan-2.2-Modell fehlt eine Wan-2.2-VAE in ComfyUI.','This Wan 2.2 model needs a Wan 2.2 VAE in ComfyUI.'],
    gif_ai_vae_mismatch:['Die ausgewählte Wan-VAE passt nicht zum Modell. Für Wan 2.1 und Wan 2.2 sind unterschiedliche VAEs nötig.','The selected Wan VAE does not match the model. Wan 2.1 and Wan 2.2 require different VAEs.'],
    gif_ai_vae:['Eine passende Wan-VAE fehlt in ComfyUI.','A compatible Wan VAE is missing from ComfyUI.'],
    gif_ai_dimensions_22:['Wan 2.2 benötigt Breite und Höhe in 32er-Schritten.','Wan 2.2 requires width and height in steps of 32.'],
    gif_ai_workflow:['ComfyUI unterstützt den erforderlichen Wan-Workflow nicht.','ComfyUI does not support the required Wan workflow.'],
    gif_ai_memory:['ComfyUI hat beim Wan-Auftrag nicht genügend freien Grafikspeicher. Versuche weniger Frames oder eine kleinere Auflösung.','ComfyUI ran out of GPU memory. Try fewer frames or a smaller resolution.'],
    gif_ai_execution:['Die Wan-Generierung ist in ComfyUI fehlgeschlagen. Prüfe das ComfyUI-Protokoll.','Wan generation failed in ComfyUI. Check the ComfyUI log.'],
    gif_ai_timeout:['Die Wan-Generierung hat das Zeitlimit erreicht.','Wan generation timed out.'],
    gif_ai_source:['Das Startbild kann nicht gelesen werden.','The start image cannot be read.'],
    gif_source:['Dieses Bild kann nicht als Vorschau gelesen werden.','This image cannot be loaded as a preview.'],
    gif_dimensions:['Das GIF überschreitet die erlaubte Ausgabegröße.','The GIF exceeds the allowed output size.'],
    gif_missing:['Dieses temporäre GIF ist nicht mehr vorhanden.','This temporary GIF is no longer available.'],
    gif_preview_large:['Das GIF ist für die direkte Vorschau zu groß. Du kannst es trotzdem in der Galerie speichern.','This GIF is too large for direct preview. You can still save it to the gallery.'],
    gallery_exists:['Dieser Dateiname existiert im Galerieordner bereits.','A file with this name already exists in the gallery folder.'],
  };
  for(const [key,value] of Object.entries(labels))if(code.includes(key))return value[de?0:1];
  return galleryError(error,de);
}

export default function GifStudio({de}:{de:boolean}){
  const language:Language=de?'de':'en';
  const [mode,setMode]=useState<Mode>('ai');
  const [panelWidths,setPanelWidths]=useState<PanelWidths>(savedWidths);
  const panelDrag=useRef<{side:keyof PanelWidths;startX:number;startWidth:number}|null>(null);
  const [galleryFolder,setGalleryFolder]=useState(()=>{try{return sessionStorage.getItem('studio-gallery-folder')??'';}catch{return '';}});
  const [galleryRefresh,setGalleryRefresh]=useState(0);
  const [selectedPath,setSelectedPath]=useState('');
  const [models,setModels]=useState<WanModel[]>([]);
  const [modelsBusy,setModelsBusy]=useState(true);
  const [model,setModel]=useState('');
  const [source,setSource]=useState('');
  const [frames,setFrames]=useState<string[]>([]);
  const [prompt,setPrompt]=useState('');
  const [negativePrompt,setNegativePrompt]=useState('');
  const [width,setWidth]=useState(832),[height,setHeight]=useState(480),[length,setLength]=useState(21);
  const [steps,setSteps]=useState(20),[guidance,setGuidance]=useState(6);
  const [delay,setDelay]=useState(120),[looped,setLooped]=useState(true);
  const [busy,setBusy]=useState(false),[error,setError]=useState(''),[preview,setPreview]=useState<Preview|null>(null);
  const [pending,setPending]=useState<GifPending[]>([]),[activePending,setActivePending]=useState<string|null>(null);
  const [zoom,setZoom]=useState(1);
  const previewRequest=useRef(0);

  async function refreshModels(){
    setModelsBusy(true);
    try {setModels(await invoke<WanModel[]>('gif_model_catalog'));}
    catch(e){setModels([]);setError(message(e,de));}
    finally {setModelsBusy(false);}
  }
  useEffect(()=>{void refreshModels();},[]);
  useEffect(()=>{let live=true;void invoke<GifPending[]>('gif_pending_list').then(items=>{if(live){setPending(items);if(items[0])void showPending(items[0].id);}}).catch(e=>{if(live)setError(message(e,de));});return()=>{live=false;};},[]);
  useEffect(()=>{try{localStorage.setItem('gif-studio-panel-widths',JSON.stringify(panelWidths));}catch{/* local preference */}},[panelWidths]);
  function onFolder(folder:string){setGalleryFolder(folder);setSelectedPath('');try{sessionStorage.setItem('studio-gallery-folder',folder);}catch{/* session only */}}
  function showLocal(path:string){
    const id=++previewRequest.current;
    setActivePending(null);
    setPreview(null);
    void invoke<string>('gif_source_preview',{path}).then(url=>{if(previewRequest.current===id){setPreview({url,path,label:fileName(path)});setZoom(1);}})
      .catch(e=>{if(previewRequest.current===id)setError(message(e,de));});
  }
  async function chooseSource(){
    const picked=await open({multiple:false,title:de?'Startbild wählen':'Choose start image',filters:[{name:de?'Bilder':'Images',extensions:imageExtensions}]});
    if(typeof picked==='string'){setSource(picked);setSelectedPath('');showLocal(picked);}
  }
  async function chooseFrames(){
    const picked=await open({multiple:true,title:de?'GIF-Frames wählen':'Choose GIF frames',filters:[{name:de?'Bilder':'Images',extensions:imageExtensions}]});
    if(Array.isArray(picked)&&picked.length){setFrames(current=>[...current,...picked].slice(0,200));showLocal(picked[0]);}
  }
  async function chooseModel(){
    const picked=await open({multiple:false,title:de?'Wan-Modell wählen':'Choose Wan model',filters:[{name:'Wan · Safetensors / GGUF',extensions:['safetensors','gguf']}]});
    if(typeof picked==='string')setModel(picked);
  }
  async function selectGallery(entry:GalleryEntry,root:string){
    if(entry.kind!=='image')return;
    ++previewRequest.current;
    setActivePending(null);
    setError('');
    setSelectedPath(entry.path);
    try {
      const detail=await galleryApi.detail(entry.path);
      setPreview({url:detail.url,path:entry.path,label:entry.name,width:detail.dimensions?.[0],height:detail.dimensions?.[1]});
      setZoom(1);
      const fullPath=root.replace(/[\\/]$/,'')+'\\'+entry.path.replace(/\//g,'\\');
      if(mode==='ai'&& !entry.name.toLowerCase().endsWith('.gif'))setSource(fullPath);
      if(mode==='frames'&&frames.length<200)setFrames(current=>[...current,fullPath]);
    }catch(e){setError(message(e,de));}
  }
  function moveFrame(index:number,delta:number){setFrames(current=>{const next=[...current];const target=index+delta;if(target<0||target>=next.length)return current;[next[index],next[target]]=[next[target],next[index]];return next;});}
  async function showPending(id:string){
    const request=++previewRequest.current;
    setActivePending(id);setSelectedPath('');setError('');
    try {const url=await invoke<string>('gif_pending_preview',{id});if(previewRequest.current===request){setPreview({url,path:'',label:`Local-Studio-${id}.gif`});setZoom(1);}}
    catch(e){if(previewRequest.current===request){setPreview(null);setError(message(e,de));}}
  }
  async function create(){
    if(busy)return;
    setBusy(true);setError('');
    try {
      const result=mode==='ai'
        ? await invoke<GifPending>('gif_ai_create',{request:{sourcePath:source,modelPath:model,prompt,negativePrompt,width,height,frames:length,steps,guidance,seed:crypto.getRandomValues(new Uint32Array(1))[0],delayMs:delay,looped}})
        : await invoke<GifPending>('gif_create',{paths:frames,delayMs:delay,looped});
      setPending(current=>[result,...current]);
      await showPending(result.id);
    }catch(e){setError(message(e,de));}
    finally {setBusy(false);}
  }
  async function savePending(){
    if(!activePending||busy)return;
    const id=activePending;setBusy(true);setError('');
    try {
      let folder=galleryFolder;
      let path:string;
      try {path=await invoke<string>('gif_pending_save',{id,folder});}
      catch(e){if(!folder||!['gallery_missing','gallery_path'].some(code=>String(e).includes(code)))throw e;folder='';path=await invoke<string>('gif_pending_save',{id,folder});onFolder('');}
      setPending(current=>current.filter(item=>item.id!==id));setActivePending(null);
      const relative=[folder,fileName(path)].filter(Boolean).join('/');
      setGalleryRefresh(value=>value+1);setSelectedPath(relative);
      const detail=await galleryApi.detail(relative);
      setPreview({url:detail.url,path:relative,label:fileName(path),width:detail.dimensions?.[0],height:detail.dimensions?.[1]});
    }catch(e){setError(message(e,de));}
    finally{setBusy(false);}
  }
  async function discardPending(){
    if(!activePending||busy)return;
    const id=activePending;setBusy(true);setError('');
    try {await invoke('gif_pending_discard',{id});setPending(current=>current.filter(item=>item.id!==id));setActivePending(null);setPreview(null);}
    catch(e){setError(message(e,de));}finally{setBusy(false);}
  }
  function startDrag(side:keyof PanelWidths,event:ReactPointerEvent<HTMLDivElement>){
    event.preventDefault();panelDrag.current={side,startX:event.clientX,startWidth:panelWidths[side]};event.currentTarget.setPointerCapture(event.pointerId);
  }
  function dragPanel(event:ReactPointerEvent<HTMLDivElement>){
    const drag=panelDrag.current;if(!drag)return;
    const scale=Number(getComputedStyle(document.documentElement).getPropertyValue('--ui-scale'))||1;
    const direction=drag.side==='left'?1:-1;
    const value=drag.startWidth+(event.clientX-drag.startX)/scale*direction;
    setPanelWidths(current=>({...current,[drag.side]:Math.max(drag.side==='left'?320:250,Math.min(drag.side==='left'?560:440,value))}));
  }
  const canGenerate=mode==='ai'
    ? !!source&&!!model&&Number.isInteger(width)&&Number.isInteger(height)&&width>=128&&height>=128&&width<=2048&&height<=2048&&width%16===0&&height%16===0&&Number.isInteger(length)&&length>=5&&length<=81&&(length-1)%4===0&&Number.isInteger(steps)&&steps>=1&&steps<=50&&Number.isFinite(guidance)&&guidance>=0&&guidance<=20
    : frames.length>0&&frames.length<=200;
  const validOutput=Number.isInteger(delay)&&delay>=20&&delay<=10000;

  return <div className="page image-studio gif-studio">
    {error&&<p className="notice warning" role="alert">{error}</p>}
    <div className="image-workspace gif-workspace" style={{'--image-left-panel':`${panelWidths.left}px`,'--image-right-panel':`${panelWidths.right}px`} as CSSProperties}>
      <fieldset className="panel image-config gif-config" disabled={busy}>
        <div className="gif-mode-switch" role="group" aria-label={de?'GIF-Methode':'GIF method'}>
          <button type="button" className="button secondary" aria-pressed={mode==='ai'} onClick={()=>setMode('ai')}><Sparkles size={15}/>{de?'Mit KI':'With AI'}</button>
          <button type="button" className="button secondary" aria-pressed={mode==='frames'} onClick={()=>setMode('frames')}><Images size={15}/>{de?'Aus Bildern':'From images'}</button>
        </div>
        {mode==='ai'?<>
          <div className="section-heading"><h2>{de?'Modell':'Model'}</h2><button type="button" className="text-button" title={de?'Modelle aktualisieren':'Refresh models'} aria-label={de?'Modelle aktualisieren':'Refresh models'} onClick={()=>void refreshModels()}><RefreshCw size={15}/></button></div>
          <label className="field-label" htmlFor="gif-model">{de?'Wan-Videomodell':'Wan video model'}</label>
          <select id="gif-model" className="image-model-select" value={model} onChange={event=>setModel(event.target.value)} disabled={modelsBusy}>
            <option value="">{modelsBusy?(de?'Modelle werden gelesen …':'Loading models …'):(de?'Modell auswählen …':'Select a model …')}</option>
            {model&&!models.some(item=>item.path===model)&&<option value={model}>{fileName(model)}</option>}
            {models.map(item=><option key={item.path} value={item.path}>{item.name} · {formatGigabytes(item.bytes,language)}</option>)}
          </select>
          <button type="button" className="button secondary gif-file-button" onClick={()=>void chooseModel()}><FolderOpen size={15}/>{de?'Modelldatei wählen':'Choose model file'}</button>
          <p className="hub-hint">{de?'Für ein Startbild brauchst du ein Wan-I2V- oder TI2V-Modell aus ComfyUI. GGUF benötigt den installierten GGUF-Knoten.':'A start image needs a Wan I2V or TI2V model in ComfyUI. GGUF requires the installed GGUF node.'}</p>
          <div className="gif-control-group">
            <div className="section-heading"><h2>{de?'Startbild':'Start image'}</h2></div>
            <button type="button" className="button secondary gif-file-button" onClick={()=>void chooseSource()}><FolderOpen size={15}/>{source?fileName(source):(de?'Bild wählen':'Choose image')}</button>
            <p className="hub-hint">{de?'Du kannst auch ein Bild rechts in der Galerie anklicken.':'You can also click an image in the gallery on the right.'}</p>
          </div>
          <div className="gif-control-group">
            <label className="field-label">{de?'Prompt / Bewegung / Tags (optional)':'Prompt / motion / tags (optional)'}<textarea rows={5} maxLength={8000} value={prompt} onChange={event=>setPrompt(event.target.value)} placeholder={de?'Optional: gewünschte Bewegung beschreiben …':'Optional: describe the motion …'}/></label>
            <label className="field-label">{de?'Negativer Prompt':'Negative prompt'}<textarea rows={3} maxLength={8000} value={negativePrompt} onChange={event=>setNegativePrompt(event.target.value)}/></label>
            <TagImporter de={de} removeCensor={false} onImport={tags=>setPrompt(current=>[current.trim(),tags.join(', ')].filter(Boolean).join(', ').slice(0,8000))}/>
          </div>
          <details className="image-control-details" open><summary>{de?'Größe und Bewegung':'Size and motion'}</summary>
            <div className="image-parameters"><label className="field-label">{de?'Breite':'Width'}<input type="number" min={128} max={2048} step={16} value={width} onChange={event=>setWidth(Number(event.target.value))}/></label><label className="field-label">{de?'Höhe':'Height'}<input type="number" min={128} max={2048} step={16} value={height} onChange={event=>setHeight(Number(event.target.value))}/></label><label className="field-label">Frames<input type="number" min={5} max={81} step={4} value={length} onChange={event=>setLength(Number(event.target.value))}/></label><label className="field-label">{de?'Schritte':'Steps'}<input type="number" min={1} max={50} value={steps} onChange={event=>setSteps(Number(event.target.value))}/></label><label className="field-label">Guidance<input type="number" min={0} max={20} step={0.5} value={guidance} onChange={event=>setGuidance(Number(event.target.value))}/></label></div>
            <p className="hub-hint">{de?'Maße in 16er-Schritten. Framezahl: 5, 9, 13 … 81.':'Dimensions in steps of 16. Frame count: 5, 9, 13 … 81.'}</p>
          </details>
        </>:<>
          <div className="section-heading"><h2>{de?'Bilder in Reihenfolge':'Images in order'} · {frames.length}</h2></div>
          <button type="button" className="button secondary gif-file-button" onClick={()=>void chooseFrames()} disabled={frames.length>=200}><FolderOpen size={15}/>{de?'Bilder hinzufügen':'Add images'}</button>
          <p className="hub-hint">{de?'Bilder rechts in der Galerie anklicken oder Dateien wählen. Bis zu 200 Frames.':'Click images in the gallery or choose files. Up to 200 frames.'}</p>
          <ol className="gif-frame-list">{frames.map((path,index)=><li key={`${path}-${index}`}><span title={displayPath(path)}>{index+1}. {fileName(path)}</span><button type="button" className="icon-button" disabled={index===0} aria-label={de?'Nach oben':'Move up'} onClick={()=>moveFrame(index,-1)}><ArrowUp size={14}/></button><button type="button" className="icon-button" disabled={index===frames.length-1} aria-label={de?'Nach unten':'Move down'} onClick={()=>moveFrame(index,1)}><ArrowDown size={14}/></button><button type="button" className="icon-button" aria-label={de?'Entfernen':'Remove'} onClick={()=>setFrames(current=>current.filter((_,i)=>i!==index))}><Trash2 size={14}/></button></li>)}</ol>
        </>}
        <details className="image-control-details" open><summary>{de?'GIF-Ausgabe':'GIF output'}</summary>
          <div className="image-parameters"><label className="field-label">{de?'Zeit je Frame (ms)':'Time per frame (ms)'}<input type="number" min={20} max={10000} value={delay} onChange={event=>setDelay(Number(event.target.value))}/></label></div>
          <label className="privacy-check"><input type="checkbox" checked={looped} onChange={event=>setLooped(event.target.checked)}/>{de?'Endlosschleife':'Loop forever'}</label>
          <p className="hub-hint">{de?'Das Ergebnis bleibt zunächst ungespeichert. Erst „In Galerie speichern“ legt es unter einer eindeutigen ID im gewählten Ordner ab.':'The result stays unsaved until you choose Save to gallery. It then receives a unique ID in the selected folder.'}</p>
        </details>
        <div className="image-generate-bar"><button type="button" className="button primary" disabled={!canGenerate||!validOutput||busy} onClick={()=>void create()}>{busy?<LoaderCircle className="spin" size={17}/>:<Film size={17}/>} {busy?(de?'GIF wird erstellt …':'Creating GIF …'):mode==='ai'?(de?'GIF generieren':'Generate GIF'):(de?'GIF aus Bildern erstellen':'Create GIF from images')}</button></div>
      </fieldset>
      <div className="image-panel-resizer left" role="separator" aria-orientation="vertical" aria-label={de?'Breite der Einstellungen':'Settings width'} aria-valuemin={320} aria-valuemax={560} aria-valuenow={Math.round(panelWidths.left)} tabIndex={0} onPointerDown={event=>startDrag('left',event)} onPointerMove={dragPanel} onPointerUp={()=>panelDrag.current=null} onPointerCancel={()=>panelDrag.current=null} onDoubleClick={()=>setPanelWidths(defaultWidths)} onKeyDown={event=>{if(event.key==='ArrowLeft'||event.key==='ArrowRight'){event.preventDefault();setPanelWidths(current=>({...current,left:Math.max(320,Math.min(560,current.left+(event.key==='ArrowRight'?20:-20)))}));}}}/>
      <main className="image-workspace-output">
        <section className="image-canvas gif-canvas" aria-label={de?'GIF-Canvas':'GIF canvas'}>
          <div className="image-canvas-toolbar"><div><strong>Canvas</strong><span>{preview?.label??(de?'Vorschau':'Preview')}</span></div><div role="group" aria-label={de?'Vorschaugröße':'Preview zoom'}><button type="button" className="text-button" disabled={!preview} aria-label={de?'Verkleinern':'Zoom out'} onClick={()=>setZoom(value=>Math.max(.25,value-.25))}><Minus size={15}/></button><button type="button" className="text-button" disabled={!preview} onClick={()=>setZoom(1)}>{Math.round(zoom*100)}%</button><button type="button" className="text-button" disabled={!preview} aria-label={de?'Vergrößern':'Zoom in'} onClick={()=>setZoom(value=>Math.min(4,value+.25))}><Plus size={15}/></button><button type="button" className="text-button" disabled={!preview} aria-label={de?'Einpassen':'Fit'} onClick={()=>setZoom(1)}><Maximize size={15}/></button></div></div>
          <div className="image-canvas-viewport gif-canvas-viewport">{preview?<img src={preview.url} alt={preview.label} style={{transform:`scale(${zoom})`}}/>:<div className="image-canvas-empty"><div className="image-canvas-symbol"><Film size={32}/></div><h2>{busy?(de?'GIF wird erzeugt …':'Generating GIF …'):(de?'Dein GIF erscheint hier':'Your GIF appears here')}</h2><p>{mode==='ai'?(de?'Wan-I2V-Modell und Startbild wählen; der Bewegungs-Prompt ist optional.':'Choose a Wan I2V model and start image; the motion prompt is optional.'):(de?'Bilder hinzufügen und als GIF erzeugen.':'Add images and create a GIF.')}</p>{busy&&<progress aria-label={de?'GIF wird erstellt':'Creating GIF'}/>}</div>}{busy&&preview&&<div className="gif-canvas-progress"><LoaderCircle className="spin" size={16}/><span>{de?'GIF wird erstellt …':'Creating GIF …'}</span><progress aria-label={de?'GIF wird erstellt':'Creating GIF'}/></div>}</div>
          <div className="image-canvas-status"><span>{busy?(de?'Lokale Verarbeitung läuft':'Local processing in progress'):(preview?.path??(de?'Bereit':'Ready'))}</span><span>{preview?.width&&preview.height?`${preview.width} × ${preview.height} px`:(de?'Lokal auf deinem PC':'Local on your PC')}</span></div>
        </section>
        {activePending&&<div className="gif-result-actions"><strong>{de?'Ungespeichertes GIF':'Unsaved GIF'}</strong><span>{de?'Ziel:':'Destination:'} {galleryFolder||(de?'Galerie-Hauptordner':'Gallery root')}</span><button type="button" className="button primary" disabled={busy} onClick={()=>void savePending()}><Save size={15}/>{de?'In Galerie speichern':'Save to gallery'}</button><button type="button" className="button secondary" disabled={busy} onClick={()=>void discardPending()}><Trash2 size={15}/>{de?'Verwerfen':'Discard'}</button></div>}
        {pending.length>0&&<section className="gif-pending-history" aria-label={de?'Ungespeicherte GIFs':'Unsaved GIFs'}><h2>{de?'Ungespeicherte GIFs':'Unsaved GIFs'} · {pending.length}</h2><div>{pending.map(item=><button type="button" key={item.id} className="button secondary" aria-pressed={activePending===item.id} onClick={()=>void showPending(item.id)}>GIF {item.id.slice(0,8)}</button>)}</div></section>}
      </main>
      <div className="image-panel-resizer right" role="separator" aria-orientation="vertical" aria-label={de?'Breite der Galerie':'Gallery width'} aria-valuemin={250} aria-valuemax={440} aria-valuenow={Math.round(panelWidths.right)} tabIndex={0} onPointerDown={event=>startDrag('right',event)} onPointerMove={dragPanel} onPointerUp={()=>panelDrag.current=null} onPointerCancel={()=>panelDrag.current=null} onDoubleClick={()=>setPanelWidths(defaultWidths)} onKeyDown={event=>{if(event.key==='ArrowLeft'||event.key==='ArrowRight'){event.preventDefault();setPanelWidths(current=>({...current,right:Math.max(250,Math.min(440,current.right+(event.key==='ArrowLeft'?20:-20)))}));}}}/>
      <StudioGallery language={language} folder={galleryFolder} onFolder={onFolder} refreshKey={galleryRefresh} onSelect={(entry,root)=>void selectGallery(entry,root)} onDeleted={entry=>{if(entry.path===selectedPath){setSelectedPath('');setPreview(null);}}} selectedPath={selectedPath}/>
    </div>
  </div>;
}
