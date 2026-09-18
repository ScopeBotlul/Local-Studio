import { useEffect, useRef, useState } from 'react';
import { Star, X } from 'lucide-react';
import { galleryApi, galleryError, type Annotation, type GalleryEntry } from './gallery-api';

export default function GalleryAnnotations({entry,rootId,de,onChanged}:{entry:GalleryEntry;rootId:string;de:boolean;onChanged:()=>void}) {
  const [saved,setSaved]=useState(entry.annotation); const [tag,setTag]=useState(''); const [pending,setPending]=useState(false); const [error,setError]=useState('');
  const writing=useRef(false); const alive=useRef(true);
  useEffect(()=>{alive.current=true;return()=>{alive.current=false;};},[]);
  useEffect(()=>{setSaved(previous=>entry.annotation.revision>=previous.revision?entry.annotation:previous);},[entry.annotation]);
  async function save(next:Pick<Annotation,'favorite'|'tags'>,clearInput=false) {
    if(writing.current || !entry.fileId)return; writing.current=true;setPending(true);setError('');
    try {
      const result=await galleryApi.annotate({rootId,path:entry.path,fileId:entry.fileId,revision:saved.revision,...next});
      if(alive.current){setSaved(result);if(clearInput)setTag('');}
    }catch(e){if(alive.current)setError(galleryError(e,de));}
    finally{writing.current=false;if(alive.current){setPending(false);onChanged();}}
  }
  return <section className="gallery-annotations" aria-label={de?'Favoriten und Tags':'Favorites and tags'}>
    <h3>{de?'Organisieren':'Organize'}</h3>
    {!entry.fileId && <p className="notice warning">{galleryError('gallery_identity',de)}</p>}
    {error && <p className="notice warning" role="alert">{error}</p>}
    <fieldset disabled={pending||!entry.fileId}>
      <button className="button secondary" aria-pressed={saved.favorite} onClick={()=>void save({favorite:!saved.favorite,tags:saved.tags})}><Star size={16} className={saved.favorite?'gallery-star':''}/>{saved.favorite?(de?'Aus Favoriten entfernen':'Remove from favorites'):(de?'Zu Favoriten hinzufügen':'Add to favorites')}</button>
      <div className="gallery-tag-chips">{saved.tags.map(t=><span key={t}>{t}<button className="icon-button" aria-label={de?`Tag entfernen: ${t}`:`Remove tag: ${t}`} onClick={()=>void save({favorite:saved.favorite,tags:saved.tags.filter(value=>value!==t)})}><X size={13}/></button></span>)}</div>
      <form className="gallery-actions" onSubmit={e=>{e.preventDefault();if(tag.trim())void save({favorite:saved.favorite,tags:[...saved.tags,tag]},true);}}>
        <input aria-label={de?'Neuer Tag':'New tag'} value={tag} onChange={e=>setTag(e.target.value)} maxLength={64} placeholder={de?'z. B. Urlaub':'e.g. Vacation'}/>
        <button className="button secondary" disabled={!tag.trim()||saved.tags.length>=32}>{de?'Tag hinzufügen':'Add tag'}</button>
      </form>
    </fieldset>
    <p className="hub-hint" role="status">{pending?(de?'Markierungen werden gespeichert …':'Saving annotations …'):(de?'Favoriten und hinzugefügte Tags werden lokal gespeichert. Die Mediendatei bleibt unverändert.':'Favorites and added tags are saved locally. The media file remains unchanged.')}</p>
  </section>;
}
