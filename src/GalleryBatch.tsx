import { useEffect, useRef, useState } from 'react';
import { galleryApi, galleryError, type GalleryBatchAction, type GalleryEntry } from './gallery-api';

export default function GalleryBatch({ disabled, entries, rootId, de, onFinished, onBusy }: { disabled: boolean; entries: GalleryEntry[]; rootId: string; de: boolean; onFinished: () => void; onBusy: (busy: boolean) => void }) {
  const [tag,setTag]=useState(''); const [pending,setPending]=useState(false); const [message,setMessage]=useState(''); const [error,setError]=useState('');
  const alive=useRef(true); const writing=useRef(false);
  useEffect(()=>{alive.current=true;return()=>{alive.current=false;};},[]);
  const selectionKey=entries.map(e=>e.path).join('\0');
  useEffect(()=>{if(selectionKey){setMessage('');setError('');}},[selectionKey]);
  async function apply(action: GalleryBatchAction) {
    if(writing.current || !entries.length)return;
    writing.current=true;setPending(true);onBusy(true);setMessage('');setError('');
    try {
      const count=await galleryApi.annotateBatch({rootId,targets:entries.map(e=>({path:e.path,fileId:e.fileId!,revision:e.annotation.revision,version:e.thumbnailVersion})),action});
      if(alive.current){setMessage(de?`${count} Medien geändert. Auswahl aufgehoben.`:`Updated ${count} media files. Selection cleared.`);setTag('');}
    }catch(e){if(alive.current)setError(galleryError(e,de)+(de?' Kein Medium geändert. Bitte neu auswählen.':' No media were changed. Select the files again.'));}
    finally{writing.current=false;if(alive.current){setPending(false);onBusy(false);onFinished();}}
  }
  return <section className="gallery-batch" aria-label={de?'Sammelaktionen':'Batch actions'}>
    {!!entries.length && <>
      <p className="hub-hint">{de?`${entries.length} Medien ausgewählt. Die Aktion gilt für diese Auswahl.`:`${entries.length} media files selected. The action applies to this selection.`}</p>
      <fieldset disabled={pending||disabled}>
        <div className="gallery-actions"><button className="button secondary" onClick={()=>void apply({type:'favorite',value:true})}>{de?'Auswahl zu Favoriten':'Favorite selected'}</button><button className="button secondary" onClick={()=>void apply({type:'favorite',value:false})}>{de?'Favoriten der Auswahl aufheben':'Unfavorite selected'}</button></div>
        <form className="gallery-actions" onSubmit={e=>{e.preventDefault();if(tag.trim())void apply({type:'addTag',tag});}}>
          <input aria-label={de?'Tag für Auswahl':'Tag for selection'} value={tag} onChange={e=>setTag(e.target.value)} maxLength={64} placeholder={de?'Ein gemeinsamer Tag …':'One shared tag …'}/>
          <button className="button secondary" disabled={!tag.trim()}>{de?'Tag zur Auswahl hinzufügen':'Add tag to selected'}</button>
          <button type="button" className="button secondary" disabled={!tag.trim()} onClick={()=>void apply({type:'removeTag',tag})}>{de?'Tag aus Auswahl entfernen':'Remove tag from selected'}</button>
        </form>
      </fieldset>
    </>}
    {pending && <p className="notice" role="status">{de?'Auswahl wird gespeichert …':'Saving selection …'}</p>}
    {message && <p className="notice" role="status">{message}</p>}
    {error && <p className="notice warning" role="alert">{error}</p>}
  </section>;
}
