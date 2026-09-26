import {useRef,useState} from 'react';
import {Box,CheckCircle2,Download,Globe2,LoaderCircle,LogIn,Search,ShieldCheck,UserRound,X} from 'lucide-react';
import CivitaiBrowser from './CivitaiBrowser';
import {civitaiApi,civitaiDirectDownloadEligible,type CivitaiDetail,type CivitaiModel} from './civitai-api';
import {downloads,type DownloadPlan} from './download-api';
import {formatBytes,formatGigabytes} from './helpers';
import type {Language} from './types';

const errors:Record<string,[string,string]>={
  civitai_query:['Ungültige Suchfilter.','Invalid search filters.'],
  civitai_network:['Civitai ist gerade nicht erreichbar.','Civitai is currently unavailable.'],
  civitai_http:['Civitai hat die Anfrage abgelehnt.','Civitai rejected the request.'],
  civitai_response:['Civitai hat eine unerwartete Antwort geliefert.','Civitai returned an unexpected response.'],
  civitai_file_unsafe:['Direktdownloads erfordern eine primäre Safetensors-Datei mit SHA-256 sowie erfolgreichen Virus- und Pickle-Scans.','Direct downloads require a primary Safetensors file with SHA-256 and successful virus and pickle scans.'],
  civitai_type_unsupported:['Dieser Ressourcentyp hat noch keinen sicheren Zielordner.','This resource type does not have a safe destination folder yet.'],
};
const message=(error:unknown,de:boolean)=>errors[String(error)]?.[de?0:1]??String(error);

export default function CivitaiModels({language,onQueued}:{language:Language;onQueued:()=>void}){
  const de=language==='de',busyRef=useRef(false);
  const [busy,setBusy]=useState(false),[error,setError]=useState('');
  const [query,setQuery]=useState(''),[kind,setKind]=useState(''),[base,setBase]=useState(''),[sort,setSort]=useState('Most Downloaded'),[period,setPeriod]=useState('AllTime'),[includeNsfw,setIncludeNsfw]=useState(false);
  const [models,setModels]=useState<CivitaiModel[]|null>(null),[cursor,setCursor]=useState(''),[detail,setDetail]=useState<CivitaiDetail|null>(null),[plan,setPlan]=useState<DownloadPlan|null>(null);
  const [website,setWebsite]=useState(false),[websiteUrl,setWebsiteUrl]=useState('https://civitai.com/login');

  async function run<T>(work:()=>Promise<T>){
    if(busyRef.current)throw new Error('busy');
    busyRef.current=true;setBusy(true);setError('');
    try{return await work();}catch(value){setError(message(value,de));throw value;}finally{busyRef.current=false;setBusy(false);}
  }
  async function search(more=false){
    try{
      const result=await run(()=>civitaiApi.search(query,kind,base,sort,period,more?cursor:'',includeNsfw));
      setModels(old=>more?[...(old??[]),...result.models.filter(model=>!old?.some(existing=>existing.id===model.id))]:result.models);
      setCursor(result.nextCursor??'');setDetail(null);setPlan(null);
    }catch{/* rendered */}
  }
  async function inspect(id:number){try{setDetail(await run(()=>civitaiApi.detail(id)));setPlan(null);}catch{/* rendered */}}
  async function prepare(versionId:number,fileId:number){try{setPlan(await run(()=>civitaiApi.plan(detail!.model.id,versionId,fileId)));}catch{/* rendered */}}
  async function start(){if(!plan)return;try{await run(()=>downloads.start(plan.id));onQueued();}catch{/* rendered */}}
  function openWebsite(url:string){setWebsiteUrl(url);setWebsite(true);}

  return <div className="civitai-models">
    {error&&<p className="notice warning" role="alert">{error}</p>}
    <section className="panel hub-account civitai-account" aria-label={de?'Civitai-Konto':'Civitai account'}>
      <div className="hub-account-title"><UserRound size={28}/><div><h2>Civitai</h2><p>{de?'Konto und Website im geschützten Studio-Browser':'Account and website in the protected Studio browser'}</p></div><ShieldCheck size={20}/></div>
      <p>{de?'Melde dich direkt auf Civitai an. Cookies und Sitzungsdaten bleiben im getrennten Civitai-Browserprofil von Local Studio und werden nicht an die Modellsuche oder andere Webseiten weitergegeben.':'Sign in directly on Civitai. Cookies and session data remain in Local Studio’s separate Civitai browser profile and are not shared with model search or other websites.'}</p>
      <div className="hub-actions">
        <button type="button" className="button primary" onClick={()=>openWebsite('https://civitai.com/login')}><LogIn size={16}/>{de?'Mit Civitai anmelden':'Sign in with Civitai'}</button>
        <button type="button" className="button secondary" onClick={()=>openWebsite('https://civitai.com/models')}><Globe2 size={16}/>{de?'Civitai-Website öffnen':'Open Civitai website'}</button>
        {website&&<button type="button" className="text-button" onClick={()=>setWebsite(false)}><X size={14}/>{de?'Browser schließen':'Close browser'}</button>}
      </div>
      <p className="hub-hint">{de?'Die Suche unten funktioniert auch ohne Anmeldung. Die Browser-Anmeldung bleibt für Civitai-Seiten erhalten; sichere App-Downloads werden weiterhin anhand von Dateiformat, Prüfsummen und Civitai-Scans geprüft.':'Search below also works without signing in. The browser sign-in persists for Civitai pages; secure app downloads continue to be checked using file format, checksums, and Civitai scans.'}</p>
    </section>
    {website&&<section className="civitai-website" aria-label={de?'Civitai-Website':'Civitai website'}><CivitaiBrowser key={websiteUrl} de={de} homeUrl={websiteUrl}/></section>}

    {!detail&&<form className="panel hub-search" onSubmit={event=>{event.preventDefault();void search(false);}}>
      <div className="hub-category-filters" role="group" aria-label={de?'Modelltyp':'Model type'}>
        {[['',de?'Alle':'All'],['Checkpoint','Checkpoints'],['LORA','LoRAs'],['VAE','VAEs'],['Controlnet','ControlNet'],['Upscaler','Upscaler'],['TextualInversion','Embeddings']].map(([value,label])=><button type="button" key={value} disabled={busy} aria-pressed={kind===value} onClick={()=>setKind(value)}>{label}</button>)}
      </div>
      <label className="field-label" htmlFor="civitai-query">{de?'Civitai durchsuchen':'Search Civitai'}</label>
      <div className="hub-search-input"><input id="civitai-query" value={query} maxLength={160} onChange={event=>setQuery(event.target.value)} placeholder="Illustrious, Pony, FLUX, LoRA …"/><button className="button primary" disabled={busy}>{busy?<LoaderCircle className="spin" size={16}/>:<Search size={16}/>} {de?'Suchen':'Search'}</button></div>
      <div className="hub-filters">
        <label>{de?'Basisfamilie':'Base model'}<input value={base} maxLength={100} onChange={event=>setBase(event.target.value)} placeholder="SDXL 1.0, Illustrious, Pony …"/></label>
        <label>{de?'Sortierung':'Sort'}<select value={sort} onChange={event=>setSort(event.target.value)}>{['Most Downloaded','Highest Rated','Newest'].map(value=><option key={value}>{value}</option>)}</select></label>
        <label>{de?'Zeitraum':'Period'}<select value={period} onChange={event=>setPeriod(event.target.value)}>{['AllTime','Year','Month','Week','Day'].map(value=><option key={value}>{value}</option>)}</select></label>
      </div>
      <label className="hub-check"><input type="checkbox" checked={includeNsfw} onChange={event=>setIncludeNsfw(event.target.checked)}/>{de?'18+-Treffer einbeziehen':'Include 18+ results'}</label>
      <p className="hub-filter-support"><ShieldCheck size={14}/>{de?'Direktdownloads erfordern Safetensors, SHA-256 und erfolgreiche Civitai-Scans. Modelle landen im passenden ComfyUI-Ordner oder im lokalen Modellordner.':'Direct downloads require Safetensors, SHA-256, and successful Civitai scans. Models are stored in the matching ComfyUI folder or the local model folder.'}</p>
    </form>}

    {detail?<section className="panel hub-detail">
      <div className="section-heading"><div><h2>{detail.model.name}</h2><small>Civitai #{detail.model.id} · {detail.model.kind} · {detail.model.creator||'—'}</small></div><button className="icon-button" aria-label={de?'Zurück':'Back'} onClick={()=>{setDetail(null);setPlan(null);}}><X size={18}/></button></div>
      <div className="hub-tags">{detail.model.nsfw&&<span className="pill">18+</span>}<span className="pill">{detail.model.downloads.toLocaleString(language)} Downloads</span>{detail.model.rating!==null&&<span className="pill">★ {detail.model.rating.toFixed(2)}</span>}</div>
      <p className="hub-hint">{de?`Nutzung laut Civitai: Namensnennung ${detail.model.creditRequired?'erforderlich':'nicht erforderlich'} · kommerziell ${detail.model.commercialUse||'nicht angegeben'} · Ableitungen ${detail.model.derivativesAllowed?'erlaubt':'nicht erlaubt'} · andere Lizenz ${detail.model.differentLicenseAllowed?'erlaubt':'nicht erlaubt'}`:`Use according to Civitai: credit ${detail.model.creditRequired?'required':'not required'} · commercial ${detail.model.commercialUse||'not specified'} · derivatives ${detail.model.derivativesAllowed?'allowed':'not allowed'} · different license ${detail.model.differentLicenseAllowed?'allowed':'not allowed'}`}</p>
      {detail.versions.map(version=><details className="civitai-version" key={version.id} open={detail.versions[0]?.id===version.id}><summary><strong>{version.name}</strong><span>{version.baseModel??(de?'Basis unbekannt':'Unknown base')}</span></summary>{version.trainedWords.length>0&&<p className="hub-hint">Trigger: {version.trainedWords.join(', ')}</p>}<div className="hub-files"><table><thead><tr><th>{de?'Datei':'File'}</th><th>{de?'Größe':'Size'}</th><th/></tr></thead><tbody>{version.files.map(file=><tr key={file.id}><td><strong>{file.name}</strong><small>{file.format??'—'} · SHA-256 {file.sha256?file.sha256.slice(0,16)+'…':'—'} · Virus {file.virusScan??'—'} · Pickle {file.pickleScan??'—'}</small></td><td>{formatGigabytes(file.sizeBytes,language)}</td><td><button className="button secondary" disabled={busy||!civitaiDirectDownloadEligible(file)} title={!civitaiDirectDownloadEligible(file)?(de?'Nicht als sicherer Direktdownload zugelassen':'Not eligible for safe direct download'):''} onClick={()=>void prepare(version.id,file.id)}><Download size={15}/>{de?'Prüfen':'Prepare'}</button></td></tr>)}</tbody></table></div></details>)}
      {plan&&<div className="panel download-plan"><h3>{de?'Download-Vorschau':'Download preview'}</h3><dl><dt>{de?'Datei':'File'}</dt><dd>{plan.download.files[0]?.path}</dd><dt>{de?'Größe':'Size'}</dt><dd>{formatBytes(plan.download.totalBytes,language)}</dd><dt>{de?'Zielordner':'Destination'}</dt><dd>{plan.download.destination}</dd></dl><button className="button primary" disabled={busy} onClick={()=>void start()}><Download size={16}/>{de?'Herunterladen':'Download'}</button></div>}
    </section>:models===null?<div className="panel hub-empty"><Box size={28}/><p>{de?'Durchsuche Civitai nach Checkpoints, LoRAs und Komponenten.':'Search Civitai for checkpoints, LoRAs, and components.'}</p></div>:<>
      <div className="hub-results">{models.map(model=><article className="panel hub-model" key={model.id}><div className="hub-model-icon"><Box size={21}/></div><div className="hub-model-content"><h2>{model.name}</h2><div className="hub-model-meta">{model.nsfw&&<span>18+</span>}<span>{model.kind}</span>{model.baseModel&&<span>{model.baseModel}</span>}<span>{model.creator||'—'}</span></div>{model.tags.length>0&&<div className="hub-tags compact-tags">{model.tags.slice(0,4).map(tag=><span className="pill" key={tag}>{tag}</span>)}</div>}<small>{model.downloads.toLocaleString(language)} Downloads{model.rating!==null?` · ★ ${model.rating.toFixed(2)}`:''}{model.latestVersion?` · ${model.latestVersion}`:''}</small></div><button className="button secondary" disabled={busy} onClick={()=>void inspect(model.id)}>{de?'Details':'Details'}</button></article>)}</div>
      {cursor&&<div className="hub-pagination"><span>{models.length} {de?'Treffer':'results'}</span><button className="button secondary" disabled={busy} onClick={()=>void search(true)}>{de?'Weitere Modelle':'More models'}</button></div>}
    </>}
    <p className="under-panel-note"><CheckCircle2 size={16}/>{de?'Civitai-Suche, Metadaten und Dateiprüfung werden erst auf deine Aktion ausgeführt.':'Civitai search, metadata, and file checks only run when you request them.'}</p>
  </div>;
}
