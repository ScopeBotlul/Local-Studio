import {useEffect,useState} from 'react';
import {galleryApi,galleryError,type GalleryEntry,type LineageFamily,type LineageQuery} from './gallery-api';
export default function GalleryLineage({entry,rootId,de,disabled,onNavigate,onChange,refresh}:{entry:GalleryEntry;rootId:string;de:boolean;disabled:boolean;onNavigate:(path:string)=>void;onChange:()=>void;refresh:number}){
 const [family,setFamily]=useState<LineageFamily|null>(null);const [busy,setBusy]=useState(false);const [error,setError]=useState('');const [tick,setTick]=useState(0);
 const query:LineageQuery={rootId,target:{path:entry.path,fileId:entry.fileId??'',version:entry.thumbnailVersion}};
 useEffect(()=>{let live=true;setFamily(null);setError('');if(entry.fileId)void galleryApi.lineage(query).then(v=>{if(live)setFamily(v);}).catch(e=>{if(live)setError(galleryError(e,de));});return()=>{live=false;};},[rootId,entry.path,entry.fileId,entry.thumbnailVersion,tick,refresh,de]);
 async function action(fn:()=>Promise<void>){if(busy||disabled)return;setBusy(true);setError('');try{await fn();setTick(n=>n+1);onChange();}catch(e){setError(galleryError(e,de));}finally{setBusy(false);}}
 const labels={edit:de?'Bildbearbeitung':'Image edit',original:de?'Original / Import':'Original / import',textToImage:de?'Bildgenerierung':'Image generation',copy:de?'Dateikopie als Variante':'File copy as variant'};
 return <section className="gallery-lineage"><h3>{de?'Herkunft und Varianten':'Origin and variants'}</h3><p className="hub-hint">{de?'Eine Variante ist eine eigene Datei. Löschen betrifft nur die ausgewählte Datei; die Verknüpfung bleibt als Metadatum erhalten.':'A variant is a separate file. Deletion affects only the selected file; its relationship remains as metadata.'}</p>
  <button className="button secondary" disabled={disabled||busy||!family||family.versions.length>=100} onClick={()=>void action(async()=>{const path=await galleryApi.createVariant(query);onNavigate(path);})}>{de?'Kopie als Variante anlegen':'Create variant copy'}</button>
  {error&&<p role="alert" className="notice warning">{error}</p>}
  {family?.primary&&<p className="hub-hint"><button className="text-button" disabled={disabled||busy||family.primary===family.current} onClick={()=>{const main=family.versions.find(v=>v.node.id===family.primary);if(main?.available)onNavigate(main.node.path);}}>{de?"Hauptversion öffnen":"Open main version"}: {family.versions.find(v=>v.node.id===family.primary)?.node.name}</button></p>}
  {family&&<ol className="lineage-chain">{family.versions.map(({node,available,version})=><li key={node.id} data-lineage-id={node.id} data-lineage-available={available} className={node.id===family.current?'current':''}>
    <div><strong>{node.name}</strong><small>{labels[node.operation]} · {new Date(node.createdAt).toLocaleString(de?'de':'en')}{node.id===family.primary&&(de?' · Hauptversion':' · Main version')}</small>
    {node.parent&&<small>← {family.versions.find(v=>v.node.id===node.parent)?.node.name??(de?'Frühere Quelle':'Earlier source')}</small>}
    {node.origin&&<small>{node.origin.modelName} · {de?'Auftrag':'Job'} {node.origin.jobId}</small>}
    {node.origin&&<details><summary>{de?'Generierungsherkunft':'Generation origin'}</summary><p>{node.origin.request.prompt}</p><small>{node.origin.request.width} × {node.origin.request.height} · Seed {node.origin.request.seed} · {node.origin.request.steps} steps</small><small>SHA-256: {node.origin.modelSha256??'—'}</small></details>}
    {!available&&<small>{de?'Quelle fehlt, wurde verändert oder liegt im Papierkorb. Nur Metadaten aufbewahrt.':'Source missing, changed or in trash. Only metadata retained.'}</small>}</div>
    {available&&<div className="gallery-actions"><button className="text-button" disabled={disabled||busy||node.id===family.current} onClick={()=>onNavigate(node.path)}>{de?'Version öffnen':'Open version'}</button><button className="text-button" disabled={disabled||busy||!version||node.id===family.primary} onClick={()=>void action(()=>galleryApi.setPrimary({rootId,target:{path:node.path,fileId:node.fileId,version:version!}}))}>{de?'Als Hauptversion':'Set as main'}</button></div>}
  </li>)}</ol>}
 </section>;
}
