import {useEffect,useState} from 'react';
import {Cpu,RefreshCw} from 'lucide-react';
import {ai,type AiModel} from './ai-api';
import {imageApi,type ImageJob} from './image-api';
import {formatGigabytes} from './helpers';
import type {Language} from './types';

interface Loaded {id:string;name:string;kind:string;bytes:number|null;state:string}
export default function LoadedModels({language}:{language:Language}){
 const de=language==='de';const [items,setItems]=useState<Loaded[]>([]),[error,setError]=useState(false),[tick,setTick]=useState(0);
 useEffect(()=>{let live=true,timer:ReturnType<typeof setTimeout>;const poll=async()=>{const [chat,models,images,speech]=await Promise.allSettled([ai.status(),ai.models(),imageApi.jobs(),ai.jobs()]);if(!live)return;const list:Loaded[]=[];
  if(chat.status==='fulfilled'&&models.status==='fulfilled'&&chat.value.modelId&&['loading','ready','running'].includes(chat.value.phase)){const model=models.value.find((m:AiModel)=>m.id===chat.value.modelId);list.push({id:'chat',name:model?.name??chat.value.modelId,kind:de?'Assistent':'Assistant',bytes:model?.bytes??null,state:chat.value.phase});}
  if(images.status==='fulfilled')for(const job of images.value.filter((j:ImageJob)=>j.status==='running'&&['loading','processing','sampling','decoding'].includes(j.phase)))list.push({id:job.id,name:job.request.modelPath.split(/[\\/]/).pop()??job.request.modelPath,kind:de?'Bildgenerierung':'Image generation',bytes:job.modelBytes,state:job.phase});
  if(speech.status==='fulfilled'&&models.status==='fulfilled')for(const job of speech.value.filter(j=>j.status==='running'&&['checking','decoding','transcribing'].includes(j.phase))){const model=models.value.find(m=>m.id===job.modelId);list.push({id:job.id,name:model?.name??job.modelId,kind:de?'Spracherkennung':'Speech recognition',bytes:model?.bytes??null,state:job.phase});}
  setItems(list);setError([chat,models,images,speech].some(v=>v.status==='rejected'));timer=setTimeout(()=>void poll(),1500);};void poll();return()=>{live=false;clearTimeout(timer);};},[de,tick]);
 return <section className="panel loaded-models"><div className="section-heading"><div><h2><Cpu size={17}/>{de?'Im Speicher geladene Modelle':'Models loaded in memory'}</h2><p className="hub-hint">{de?'Bild- und Sprachmodelle werden nach dem Auftrag automatisch entladen.':'Image and speech models are unloaded automatically after each job.'}</p></div><button className="text-button" onClick={()=>setTick(n=>n+1)} aria-label={de?'Status aktualisieren':'Refresh status'}><RefreshCw size={14}/></button></div>{items.map(item=><div className="loaded-model-row" key={item.id}><div><strong>{item.name.replace(/\.safetensors$/i,'')}</strong><small>{item.kind} · {item.state}</small></div><span>{formatGigabytes(item.bytes,language)}</span></div>)}{!items.length&&!error&&<p className="hub-hint">{de?'Derzeit ist kein Modell im RAM oder VRAM geladen.':'No model is currently loaded in RAM or VRAM.'}</p>}{error&&<p className="notice warning">{de?'Der Speicherstatus ist teilweise nicht erreichbar.':'Some memory status information is unavailable.'}</p>}</section>;
}
