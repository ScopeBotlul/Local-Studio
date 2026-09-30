import {useEffect,useRef,useState} from 'react';
import type {FormEvent} from 'react';
import {ArrowLeft,Folder,FolderPlus,Plus,RefreshCw,X} from 'lucide-react';
import {galleryApi,galleryError,type GalleryEntry,type GalleryListing} from './gallery-api';
import GalleryThumbnail from './GalleryThumbnail';
import GalleryContextMenu,{type MediaContext} from './GalleryContextMenu';
import type {Language} from './types';

export default function StudioGallery({language,folder,onFolder,refreshKey,onSelect,onDeleted,selectedPath}:{language:Language;folder:string;onFolder:(folder:string)=>void;refreshKey:number;onSelect?:(entry:GalleryEntry,root:string)=>void;onDeleted?:(entry:GalleryEntry)=>void;selectedPath?:string}){
 const de=language==='de';
 const [listing,setListing]=useState<GalleryListing|null>(null),[error,setError]=useState(''),[tick,setTick]=useState(0),[creating,setCreating]=useState(false),[name,setName]=useState(''),[busy,setBusy]=useState(false);
 const [mediaContext,setMediaContext]=useState<MediaContext|null>(null);
 const request=useRef(0);
 useEffect(()=>{
 const id=++request.current;
  let live=true;
  setError('');
  void galleryApi.list({folder,search:'',kind:'all',recursive:false,offset:0,favoritesOnly:false,tag:'',sort:'modifiedDesc'})
   .then(data=>{if(live&&request.current===id)setListing(data);})
   .catch(e=>{if(live&&request.current===id){
    // A saved session can still point at a folder that was removed outside the
    // app. Reset it immediately, so the next save targets the main gallery.
    if(folder&&['gallery_missing','gallery_path'].includes(String(e))){onFolder('');return;}
    setListing(null);setError(galleryError(e,de));
   }});
  return()=>{live=false;};
 },[folder,refreshKey,tick,de]);
 const parent=folder.includes('/')?folder.slice(0,folder.lastIndexOf('/')):'';
 async function createFolder(event:FormEvent){
  event.preventDefault();const next=name.trim();if(!next||busy)return;
  setBusy(true);setError('');
  try {await galleryApi.createFolder(folder,next);setName('');setCreating(false);setTick(value=>value+1);}
  catch(e){setError(galleryError(e,de));}
  finally{setBusy(false);}
 }
 const folders=listing?.folders??[];
 const items=folder?[{kind:'parent',path:parent,name:'..'},...folders]:folders;
 return <aside className="studio-gallery panel" aria-label={de?'Studio-Galerie':'Studio gallery'}>
  <GalleryContextMenu target={mediaContext} de={de} onClose={()=>setMediaContext(null)} onDeleted={entry=>{onDeleted?.(entry);setTick(value=>value+1);}}/>
  <div className="studio-gallery-heading"><div><strong>{de?'Galerie':'Gallery'}</strong><small title={folder}>{folder||(de?'Hauptordner':'Main folder')}</small></div><button className="icon-button" aria-label={de?'Galerie aktualisieren':'Refresh gallery'} title={de?'Galerie aktualisieren':'Refresh gallery'} onClick={()=>setTick(n=>n+1)}><RefreshCw size={15}/></button></div>
  <p className="hub-hint">{de?'Neue Bilder werden direkt in diesem Ordner gespeichert.':'New images are saved directly in this folder.'}</p>
  <div className="studio-gallery-actions"><button className="button secondary" disabled={busy} onClick={()=>setCreating(value=>!value)}><FolderPlus size={14}/>{de?'Ordner':'Folder'}</button></div>
  {creating&&<form className="studio-gallery-create" onSubmit={createFolder}><input aria-label={de?'Name des neuen Ordners':'New folder name'} value={name} onChange={event=>setName(event.target.value)} maxLength={100} placeholder={de?'Ordnername':'Folder name'} autoFocus disabled={busy}/><button className="icon-button" aria-label={de?'Ordner anlegen':'Create folder'} title={de?'Ordner anlegen':'Create folder'} disabled={busy||!name.trim()}><Plus size={15}/></button><button className="icon-button" type="button" aria-label={de?'Abbrechen':'Cancel'} title={de?'Abbrechen':'Cancel'} onClick={()=>{setCreating(false);setName('');}} disabled={busy}><X size={15}/></button></form>}
  <div className="studio-gallery-grid">
   {items.map(entry=><button className="studio-gallery-tile studio-gallery-folder-tile" key={`${entry.kind}:${entry.path}`} onClick={()=>onFolder(entry.path)} title={entry.kind==='parent'?(de?'Übergeordneter Ordner':'Parent folder'):entry.name}><span className="studio-gallery-folder-icon">{entry.kind==='parent'?<ArrowLeft size={27}/>:<Folder size={30}/>}</span><small>{entry.name}</small></button>)}
   {listing?.entries.map(entry=>onSelect?<button type="button" className={'studio-gallery-tile studio-gallery-media-tile'+(selectedPath===entry.path?' selected':'')} key={entry.path} title={entry.name} onClick={()=>onSelect(entry,listing.root)} onContextMenu={event=>{event.preventDefault();setMediaContext({entry,rootId:listing.rootId,x:event.clientX,y:event.clientY});}}><GalleryThumbnail entry={entry} rootId={listing.rootId} epoch={refreshKey+tick} de={de}/><small>{entry.name}</small></button>:<div className="studio-gallery-tile studio-gallery-media-tile" key={entry.path} title={entry.name} onContextMenu={event=>{event.preventDefault();setMediaContext({entry,rootId:listing.rootId,x:event.clientX,y:event.clientY});}}><GalleryThumbnail entry={entry} rootId={listing.rootId} epoch={refreshKey+tick} de={de}/><small>{entry.name}</small></div>)}
  </div>
  {listing&&!items.length&&!listing.entries.length&&<p className="hub-hint">{de?'Dieser Ordner ist leer.':'This folder is empty.'}</p>}
  {error&&<p className="notice warning" role="alert">{error}</p>}
 </aside>;
}
