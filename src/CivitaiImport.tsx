import {useState} from 'react';
import {invoke} from '@tauri-apps/api/core';
import {Download, ExternalLink, ImageDown, LoaderCircle, Search} from 'lucide-react';
import {imageApi, type ImageRequest} from './image-api';
import {formatGigabytes} from './helpers';
import type {Language} from './types';
import CivitaiBrowser from './CivitaiBrowser';

interface Resource { id:number; modelId:number|null; name:string; version:string; kind:string; baseModel:string|null; downloadUrl:string|null; fileName:string|null; sizeBytes:number|null; sha256:string|null }
interface Info { id:number; prompt:string; negativePrompt:string; width:number; height:number; nsfwLevel:string; imageUrl:string; resources:Resource[] }

function Resources({items, language, onError}:{items:Resource[]; language:Language; onError:(error:string)=>void}) {
  const de=language==='de';
  return <div className="civitai-resources">{items.map(resource=><article key={resource.id}><div><strong>{resource.name}</strong><span>{resource.kind} · {resource.version}{resource.baseModel?` · ${resource.baseModel}`:''}{resource.sizeBytes?` · ${formatGigabytes(resource.sizeBytes,language)}`:''}</span>{resource.fileName&&<small>{resource.fileName}</small>}</div>{resource.downloadUrl&&<button type="button" className="button secondary" onClick={()=>void invoke('civitai_open_download',{url:resource.downloadUrl}).catch(error=>onError(String(error)))}><Download size={14}/>{de?'Download öffnen':'Open download'}</button>}</article>)}</div>;
}

export default function CivitaiImport({language,folder,onApply}:{language:Language;folder:string;onApply:(patch:Partial<ImageRequest>)=>void}){
  const de=language==='de';
  const [url,setUrl]=useState(''),[browser,setBrowser]=useState(false),[info,setInfo]=useState<Info|null>(null),[busy,setBusy]=useState(false),[error,setError]=useState(''),[saved,setSaved]=useState('');
  const [loraQuery,setLoraQuery]=useState(''),[baseModel,setBaseModel]=useState(''),[loraBusy,setLoraBusy]=useState(false),[loras,setLoras]=useState<Resource[]>([]);
  async function inspect(value=url){if(busy)return;setBusy(true);setError('');setInfo(null);try{const next=await invoke<Info>('civitai_image_info',{url:value});setInfo(next);setBaseModel(next.resources.find(resource=>resource.baseModel)?.baseModel??'');}catch(e){setError(String(e));}finally{setBusy(false);}}
  async function reference(){if(!info||busy)return;setBusy(true);setError('');try{const path=await invoke<string>('civitai_save_reference',{url,folder});const value=await imageApi.reference(path);setSaved(path);onApply({reference:value,width:value.width,height:value.height,...(info.prompt?{prompt:info.prompt}:{}),...(info.negativePrompt?{negativePrompt:info.negativePrompt}:{})});}catch(e){setError(String(e));}finally{setBusy(false);}}
  async function searchLoras(){if(!loraQuery.trim()||loraBusy)return;setLoraBusy(true);setError('');try{setLoras(await invoke<Resource[]>('civitai_lora_search',{query:loraQuery,baseModel}));}catch(e){setError(String(e));}finally{setLoraBusy(false);}}
  return <details className="image-control-details civitai-import"><summary>Civitai {de?'Vorlagen und LoRAs':'templates and LoRAs'}</summary>
    <p className="hub-hint">{de?'Suche in der integrierten Civitai-Ansicht nach einem Bild. Local Studio kann Prompt, Negativ-Prompt, Vorlage und die von Civitai zugeordneten Modelle übernehmen.':'Find an image in the embedded Civitai view. Local Studio can use its prompt, negative prompt, reference, and Civitai-linked models.'}</p>
    <button type="button" className="button secondary" onClick={()=>setBrowser(value=>!value)}>{browser?(de?'Integrierten Browser schließen':'Close embedded browser'):(de?'Im integrierten Browser suchen':'Browse inside the app')}</button>
    {browser&&<CivitaiBrowser de={de} onImage={value=>{setUrl(value);void inspect(value);}}/>}
    <form className="image-model-path" onSubmit={event=>{event.preventDefault();void inspect();}}><input value={url} onChange={event=>setUrl(event.target.value)} placeholder="https://civitai.red/images/…" spellCheck={false}/><button className="button secondary" disabled={busy||!url.trim()}>{busy?<LoaderCircle className="spin" size={16}/>:<ExternalLink size={16}/>} {de?'Prüfen':'Inspect'}</button></form>
    {error&&<p className="notice warning" role="alert">{error}</p>}
    {info&&<div className="civitai-result"><div><strong>Civitai #{info.id}</strong><span>{info.width} × {info.height} · {info.nsfwLevel}</span></div>{info.prompt&&<p>{info.prompt}</p>}<button type="button" className="button primary" disabled={busy} onClick={()=>void reference()}><ImageDown size={16}/>{de?'Als Vorlage speichern und übernehmen':'Save and use as template'}</button>{saved&&<p className="download-path">{saved}</p>}<h3>{de?'Verwendete Modelle und LoRAs':'Models and LoRAs used'} · {info.resources.length}</h3>{!info.resources.length&&<p className="hub-hint">{de?'Für dieses Bild meldet die öffentliche API keine zugeordneten Ressourcen.':'The public API reports no linked resources for this image.'}</p>}<Resources items={info.resources} language={language} onError={setError}/></div>}
    <section className="civitai-lora-search"><h3>{de?'Passende LoRAs suchen':'Find matching LoRAs'}</h3><p className="hub-hint">{de?'Beispiel: Wai-Anima oder der Name einer Figur. Die Suche zeigt ausschließlich als jugendfrei markierte LoRAs; die optionale Basisfamilie grenzt inkompatible Treffer ein.':'Example: Wai-Anima or a character name. Search only returns LoRAs marked safe; the optional base family narrows incompatible results.'}</p><form className="image-model-path" onSubmit={event=>{event.preventDefault();void searchLoras();}}><input value={loraQuery} maxLength={120} onChange={event=>setLoraQuery(event.target.value)} placeholder={de?'LoRA suchen …':'Search LoRAs …'}/><input value={baseModel} maxLength={100} onChange={event=>setBaseModel(event.target.value)} placeholder={de?'Basisfamilie, z. B. SDXL 1.0':'Base family, e.g. SDXL 1.0'}/><button className="button secondary" disabled={loraBusy||!loraQuery.trim()}>{loraBusy?<LoaderCircle className="spin" size={16}/>:<Search size={16}/>} {de?'Suchen':'Search'}</button></form>{loras.length>0&&<Resources items={loras} language={language} onError={setError}/>}</section>
  </details>;
}
