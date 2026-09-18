import { useEffect,useRef,useState,type ReactNode } from 'react';
import { flushSync } from 'react-dom';
import { galleryApi,galleryError,type GalleryEntry,type GalleryFileAction,type FileReport } from './gallery-api';
import GalleryDialog from './GalleryDialog';
export default function GalleryFileActions({extra,entries,rootId,de,disabled,onPrepare,onBusy,onFinished}:{extra?:ReactNode;entries:GalleryEntry[];rootId:string;de:boolean;disabled:boolean;onPrepare:()=>void;onBusy:(busy:boolean)=>void;onFinished:(report?:FileReport)=>void}){
 const [dialog,setDialog]=useState<{mode:'rename'|'move'|'trash';entries:GalleryEntry[];rootId:string}|null>(null);
 const [name,setName]=useState('');const [folder,setFolder]=useState('');const [folders,setFolders]=useState<GalleryEntry[]>([]);const [loading,setLoading]=useState(false);const [pending,setPending]=useState(false);const [error,setError]=useState('');const active=useRef(false);
 useEffect(()=>{if(dialog?.mode!=='move')return;let live=true;setLoading(true);setError('');void galleryApi.list({folder,search:'',kind:'all',recursive:false,offset:0,favoritesOnly:false,tag:''}).then(data=>{if(live){if(data.rootId!==dialog.rootId)throw new Error('gallery_changed');setFolders(data.folders);}}).catch(e=>{if(live)setError(galleryError(e,de));}).finally(()=>{if(live)setLoading(false);});return()=>{live=false;};},[folder,dialog,de]);
 function close(){if(active.current)return;setDialog(null);onBusy(false);}
 function show(mode:'rename'|'move'|'trash'){setError('');setName(entries[0]?.name??'');setFolder('');setDialog({mode,entries:[...entries],rootId});onBusy(true);}
 async function apply(action:GalleryFileAction){
   if(!dialog||active.current)return;active.current=true;setPending(true);setError('');flushSync(onPrepare);
   try{const report=await galleryApi.fileAction({rootId:dialog.rootId,targets:dialog.entries.map(e=>({path:e.path,fileId:e.fileId!,version:e.thumbnailVersion})),action});setDialog(null);onFinished(report);}
   catch(e){setError(galleryError(e,de));onFinished();}
   finally{active.current=false;setPending(false);onBusy(false);}
 }
 const title=dialog?.mode==='rename'?(de?'Datei umbenennen':'Rename file'):dialog?.mode==='move'?(de?'Medien verschieben':'Move media'):(de?'In den Papierkorb verschieben?':'Move to trash?');
 return <><div className="gallery-actions gallery-file-actions"><button className="button secondary" disabled={disabled||entries.length!==1||!entries[0]?.fileId} data-gallery-action="rename" onClick={()=>show('rename')}>{de?'Umbenennen':'Rename'}</button><button className="button secondary" disabled={disabled||!entries.length} onClick={()=>show('move')}>{de?'Verschieben …':'Move …'}</button><button className="button secondary" disabled={disabled||!entries.length} data-gallery-action="trash" onClick={()=>show('trash')}>{de?'In Papierkorb …':'Move to trash …'}</button>{extra}</div>
 {dialog&&<GalleryDialog title={title} busy={pending} onClose={close}>
   <p>{de?`${dialog.entries.length} ausgewählte Medien`:`${dialog.entries.length} selected media files`}</p><ul className="gallery-operation-files">{dialog.entries.map(e=><li key={e.path}>{e.path}</li>)}</ul>
   {dialog.mode==='rename'&&<label>{de?'Neuer Dateiname (Dateiendung beibehalten)':'New filename (keep extension)'}<input autoFocus aria-label={de?'Neuer Dateiname':'New filename'} value={name} disabled={pending} onChange={e=>setName(e.target.value)} maxLength={180}/></label>}
   {dialog.mode==='move'&&<><p>{de?'Zielordner':'Destination folder'}: <strong>{folder|| (de?'Galerieordner':'Gallery folder')}</strong></p><div className="gallery-actions"><button className="text-button" disabled={pending||!folder} onClick={()=>setFolder('')}>{de?'Galerieordner':'Gallery folder'}</button><button className="text-button" disabled={pending||!folder} onClick={()=>setFolder(folder.split('/').slice(0,-1).join('/'))}>{de?'Übergeordnet':'Parent'}</button></div><div className="gallery-folder-picker">{folders.map(f=><button className="button secondary" key={f.path} disabled={pending||loading} onClick={()=>setFolder(f.path)}>{f.name}</button>)}</div>{loading&&<p role="status">{de?'Ordner werden gelesen …':'Reading folders …'}</p>}</>}
   {dialog.mode==='trash'&&<p>{de?'Die Dateien werden in den lokalen App-Papierkorb verschoben. Dort lassen sie sich wiederherstellen.':'The files will move to the local app trash, where you can restore them.'}</p>}
   <p className="hub-hint">{de?'Vorhandene Dateien werden nicht überschrieben. Bei einem Fehler wird der Vorgang angehalten; bereits ausgeführte Schritte werden angezeigt.':'Existing files will not be overwritten. On error, the operation stops and reports completed steps.'}</p>
   {error&&<p role="alert" className="notice warning">{error}</p>}
   <div className="gallery-actions"><button className="button secondary" disabled={pending} onClick={close}>{de?'Abbrechen':'Cancel'}</button><button className="button primary" disabled={pending||loading||(dialog.mode==='rename'&&!name.trim())} onClick={()=>void apply(dialog.mode==='rename'?{type:'rename',name}:dialog.mode==='move'?{type:'move',folder}:{type:'trash',confirmed:true})}>{pending?(de?'Wird ausgeführt …':'Working …'):dialog.mode==='trash'?(de?'In Papierkorb verschieben':'Move to trash'):dialog.mode==='rename'?(de?'Umbenennen bestätigen':'Confirm rename'):(de?'Hierher verschieben':'Move here')}</button></div>
 </GalleryDialog>}
 </>;
}
