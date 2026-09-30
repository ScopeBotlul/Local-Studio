import {useEffect,useRef,useState} from 'react';
import {Trash2} from 'lucide-react';
import {galleryApi,galleryError,type GalleryEntry} from './gallery-api';
import GalleryDialog from './GalleryDialog';
import './gallery-context-menu.css';

export type MediaContext = {entry:GalleryEntry;rootId:string;x:number;y:number};

export default function GalleryContextMenu({target,de,onClose,onDeleted}:{target:MediaContext|null;de:boolean;onClose:()=>void;onDeleted:(entry:GalleryEntry)=>void}){
  const menu=useRef<HTMLDivElement>(null);
  const [confirm,setConfirm]=useState<MediaContext|null>(null),[busy,setBusy]=useState(false),[error,setError]=useState('');
  useEffect(()=>{
    if(!target)return;
    const close=(event:PointerEvent)=>{if(!menu.current?.contains(event.target as Node))onClose();};
    const key=(event:KeyboardEvent)=>{if(event.key==='Escape')onClose();};
    document.addEventListener('pointerdown',close);document.addEventListener('keydown',key);
    return()=>{document.removeEventListener('pointerdown',close);document.removeEventListener('keydown',key);};
  },[target,onClose]);
  async function remove(){
    if(!confirm?.entry.fileId||busy)return;
    setBusy(true);setError('');
    try{
      const report=await galleryApi.fileAction({rootId:confirm.rootId,targets:[{path:confirm.entry.path,fileId:confirm.entry.fileId,version:confirm.entry.thumbnailVersion}],action:{type:'trash',confirmed:true}});
      if(report.errors.length||report.completed.length!==1)throw new Error(report.errors[0]??'gallery_changed');
      onDeleted(confirm.entry);setConfirm(null);
    }catch(e){setError(galleryError(e,de));}
    finally{setBusy(false);}
  }
  return <>
    {target&&<div ref={menu} className="gallery-context-menu" role="menu" style={{left:Math.min(target.x,window.innerWidth-210),top:Math.min(target.y,window.innerHeight-58)}}>
      <button type="button" role="menuitem" disabled={!target.entry.fileId} onClick={()=>{setConfirm(target);setError('');onClose();}}><Trash2 size={15}/>{de?'Löschen …':'Delete …'}</button>
    </div>}
    {confirm&&<GalleryDialog title={de?'Medium löschen?':'Delete media?'} busy={busy} onClose={()=>{if(!busy)setConfirm(null);}}>
      <p>{de?'Das Medium wird in den lokalen Papierkorb verschoben und kann dort wiederhergestellt werden.':'The media file will move to the local trash and can be restored there.'}</p>
      <p className="gallery-operation-files">{confirm.entry.path}</p>
      {error&&<p className="notice warning" role="alert">{error}</p>}
      <div className="gallery-actions"><button type="button" className="button secondary" disabled={busy} onClick={()=>setConfirm(null)}>{de?'Abbrechen':'Cancel'}</button><button type="button" className="button primary" disabled={busy} onClick={()=>void remove()}><Trash2 size={15}/>{busy?(de?'Wird verschoben …':'Moving …'):(de?'In Papierkorb verschieben':'Move to trash')}</button></div>
    </GalleryDialog>}
  </>;
}
