import {useEffect,useState} from 'react';
import {ArrowLeft,Folder,RefreshCw} from 'lucide-react';
import {galleryApi,galleryError,type GalleryListing} from './gallery-api';
import GalleryThumbnail from './GalleryThumbnail';
import type {Language} from './types';

export default function StudioGallery({language,folder,onFolder,refreshKey}:{language:Language;folder:string;onFolder:(folder:string)=>void;refreshKey:number}){
 const de=language==='de';const [listing,setListing]=useState<GalleryListing|null>(null),[error,setError]=useState(''),[tick,setTick]=useState(0);
 useEffect(()=>{let live=true;setError('');void galleryApi.list({folder,search:'',kind:'all',recursive:false,offset:0,favoritesOnly:false,tag:'',sort:'modifiedDesc'}).then(data=>{if(live)setListing(data);}).catch(e=>{if(live)setError(galleryError(e,de));});return()=>{live=false;};},[folder,refreshKey,tick,de]);
 const parent=folder.includes('/')?folder.slice(0,folder.lastIndexOf('/')):'';
 return <aside className="studio-gallery panel" aria-label={de?'Studio-Galerie':'Studio gallery'}>
  <div className="studio-gallery-heading"><div><strong>{de?'Galerie':'Gallery'}</strong><small>{folder||(de?'Hauptordner':'Main folder')}</small></div><button className="text-button" aria-label={de?'Galerie aktualisieren':'Refresh gallery'} onClick={()=>setTick(n=>n+1)}><RefreshCw size={15}/></button></div>
  <p className="hub-hint">{de?'Neue Bilder werden in diesen geöffneten Ordner gespeichert.':'New images are saved in this open folder.'}</p>
  {folder&&<button className="studio-gallery-folder" onClick={()=>onFolder(parent)}><ArrowLeft size={15}/><span>..</span></button>}
  {listing?.folders.map(entry=><button className="studio-gallery-folder" key={entry.path} onClick={()=>onFolder(entry.path)}><Folder size={16}/><span>{entry.name}</span></button>)}
  <div className="studio-gallery-media">{listing?.entries.slice(0,30).map(entry=><div key={entry.path} title={entry.name}><GalleryThumbnail entry={entry} rootId={listing.rootId} epoch={refreshKey} de={de}/><small>{entry.name}</small></div>)}</div>
  {listing&&!listing.folders.length&&!listing.entries.length&&<p className="hub-hint">{de?'Dieser Ordner ist leer.':'This folder is empty.'}</p>}
  {error&&<p className="notice warning" role="alert">{error}</p>}
 </aside>;
}
