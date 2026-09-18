import {setMenuContext} from './menu-context';
import ImageEditor from './ImageEditor';
import BatchCorrections from './BatchCorrections';
import type {EditOperation,ProjectEditQuery} from './editor-state';
import GalleryLineage from './GalleryLineage';
import { useGalleryWatch } from './useGalleryWatch';
import GalleryImage from './GalleryImage';
import type { ProjectGallerySelection } from './project-api';
import GalleryCompare from './GalleryCompare';
import type { GallerySort } from './gallery-api';
import GalleryFileActions from './GalleryFileActions';
import GalleryTrash from './GalleryTrash';
import { selectGesture } from './gallery-selection';
import type { MouseEvent } from 'react';
import GalleryBatch from './GalleryBatch';
import GalleryThumbnail from './GalleryThumbnail';
import GalleryAnnotations from './GalleryAnnotations';
import { useEffect, useRef, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { ArrowLeft, ArrowRight, Folder, FolderOpen, Image, Music, Video, RefreshCw, Upload, Plus, Star, Grid2X2, List } from 'lucide-react';
import { galleryApi, galleryError, type GalleryEntry, type GalleryListing, type GalleryDetail, type MediaKind } from './gallery-api';
import type { Language } from './types';
import {imageError,type ImageRequest,type ImageReference} from './image-api';
import {invoke} from '@tauri-apps/api/core';
import { formatBytes } from './helpers';
import './gallery.css';
import { shortcutFor, type Shortcuts } from './shortcuts';

export default function Gallery({ onReference,onAddEdit,onSaveEdit,maxUndo, onAddToProject, projectDisabled, shortcuts, language, onRestore, restoreDisabled }: { onReference:(reference:ImageReference)=>void;onAddEdit:(selection:ProjectGallerySelection,ops:EditOperation[])=>Promise<ProjectEditQuery|null>;onSaveEdit:(query:ProjectEditQuery,expected:EditOperation[],ops:EditOperation[])=>Promise<boolean>;maxUndo:number; onAddToProject: (selection: ProjectGallerySelection) => Promise<boolean>; projectDisabled: boolean; shortcuts: Shortcuts; language: Language; onRestore: (request: ImageRequest) => void; restoreDisabled: boolean }) {
  const [editor,setEditor]=useState<{entry:GalleryEntry;rootId:string}|null>(null);
  const [comparison,setComparison]=useState<{entries:GalleryEntry[];rootId:string}|null>(null);
  const [sort,setSort]=useState<GallerySort>(()=>{try{const value=localStorage.getItem('gallery-sort');return ['modifiedDesc','modifiedAsc','nameAsc','nameDesc','sizeAsc','sizeDesc'].includes(value??'')?value as GallerySort:'modifiedDesc';}catch{return 'modifiedDesc';}});
  const [trash,setTrash]=useState(false);const anchor=useRef<string|null>(null);
  const de = language === 'de'; const [folder,setFolder] = useState(''); const [search,setSearch] = useState(''); const [kind,setKind] = useState<'all'|MediaKind>('all');
  const [view,setView] = useState<'grid'|'list'>(()=>{try{return localStorage.getItem('gallery-view')==='list'?'list':'grid';}catch{return 'grid';}});
  const [thumbnailEpoch,setThumbnailEpoch] = useState(0);
  const [correctionBatch,setCorrectionBatch]=useState<{entries:GalleryEntry[];rootId:string}|null>(null);
  const [chosen,setChosen] = useState<Record<string,GalleryEntry>>({}); const [batchBusy,setBatchBusy] = useState(false); const [selectionReady,setSelectionReady] = useState(false);
  const [favoritesOnly,setFavoritesOnly] = useState(false); const [tag,setTag] = useState('');
  const [recursive,setRecursive] = useState(true); const [offset,setOffset] = useState(0); const [listing,setListing] = useState<GalleryListing|null>(null);
  const [selected,setSelected] = useState<string|null>(null); const [detail,setDetail] = useState<GalleryDetail|null>(null); const [error,setError] = useState('');
  const [busy,setBusy] = useState(false); const [note,setNote] = useState(''); const [newFolder,setNewFolder] = useState(''); const [creating,setCreating] = useState(false);
  const [actualSize,setActualSize] = useState(false); const [viewReset,setViewReset] = useState(0);
  function fitImage() { setZoom(1); setActualSize(false); setViewReset(n=>n+1); }
  const [tick,setTick] = useState(0); const [zoom,setZoom] = useState(1); const [previewError,setPreviewError] = useState<string|null>(null); const viewer = useRef<HTMLDivElement>(null);
  const watching = useGalleryWatch(listing?.rootId,()=>setTick(n=>n+1));
  const generation = useRef(0); const operation = useRef(false); const mounted = useRef(true);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  useEffect(() => {
    setSelectionReady(false);
    let live = true; let loading = false; const load = async () => {
      if (loading || document.hidden || trash || batchBusy) return; loading = true;
      try { const data = await galleryApi.list({ folder,search,kind,recursive,offset,favoritesOnly,tag,sort }); if (live) { setListing(data); setSelectionReady(true); if (offset && offset >= data.total) setOffset(0); } }
      catch (e) { if (live) setError(galleryError(e,de)); } finally { loading = false; }
    };
    const timer = setTimeout(() => void load(),180); const poll = setInterval(() => void load(),watching?30000:3000);
    return () => { live = false; clearTimeout(timer); clearInterval(poll); };
  },[folder,search,kind,recursive,offset,favoritesOnly,tag,sort,tick,de,trash,batchBusy,watching]);
  const entries = listing?.entries ?? []; const current = entries.find(e => e.path === selected) ?? entries[0];
  useEffect(()=>{setChosen({});anchor.current=null;},[folder,search,kind,recursive,offset,favoritesOnly,tag,sort,listing?.rootId]);
  useEffect(()=>{
    if(!listing)return;
    setChosen(previous=>{
      const keep=Object.fromEntries(Object.entries(previous).filter(([path,old])=>listing.entries.some(e=>e.path===path && e.fileId===old.fileId && e.thumbnailVersion===old.thumbnailVersion && e.annotation.revision===old.annotation.revision)));
      return Object.keys(keep).length===Object.keys(previous).length?previous:keep;
    });
  },[listing]);
  function toggleSelection(entry: GalleryEntry,checked: boolean) {
    anchor.current=entry.path;setChosen(previous=>{const next={...previous};if(checked && entry.fileId)next[entry.path]=entry;else delete next[entry.path];return next;});
  }
  function selectPage() {
    const identities=new Set<string>();const next:Record<string,GalleryEntry>={};
    for(const entry of entries)if(entry.fileId && !identities.has(entry.fileId)){identities.add(entry.fileId);next[entry.path]=entry;}
    setChosen(next);
  }

  useEffect(() => {
    const id = ++generation.current; setDetail(null); setPreviewError(null); setZoom(1); setActualSize(false);
    if (!current || trash || batchBusy) return;
    void galleryApi.detail(current.path).then(data => { if (mounted.current && id === generation.current) setDetail(data); }).catch(e => { if (mounted.current && id === generation.current) setError(galleryError(e,de)); });
    return () => { generation.current++; };
  },[current?.path,current?.modified,current?.bytes,current?.thumbnailVersion,listing?.rootId,de,trash,batchBusy]);
  function changeSort(value:GallerySort){setSort(value);setOffset(0);setSelected(null);try{localStorage.setItem('gallery-sort',value);}catch{/* Session sorting remains available. */}}
  function galleryKey(event:React.KeyboardEvent<HTMLDivElement>){
    const target=event.target as HTMLElement;
    if(event.defaultPrevented||batchBusy||busy||!selectionReady||target.closest('input,textarea,select,[contenteditable="true"],dialog,audio,video') )return;
    const command = shortcutFor(event.nativeEvent, shortcuts);
    if(command === 'selectAll'){event.preventDefault();selectPage();return;}
    if(command === 'compare'){const button = event.currentTarget.querySelector<HTMLButtonElement>('[data-gallery-compare]'); if(button && !button.disabled && !event.repeat){event.preventDefault();button.click();}return;}
    if(event.key==='Escape'){event.preventDefault();setChosen({});return;}
    if(command === 'rename' || command === 'delete'){
      const button=event.currentTarget.querySelector<HTMLButtonElement>(`[data-gallery-action="${command==='rename'?'rename':'trash'}"]`);
      if(button&&!button.disabled&&!event.repeat){event.preventDefault();button.click();}return;
    }
    if(event.ctrlKey||event.metaKey||event.altKey)return;
    if((event.key==='ArrowLeft'||event.key==='ArrowRight')&&target.closest('.gallery-file')){
      const path=target.closest<HTMLElement>('.gallery-file')?.dataset.galleryPath;const index=entries.findIndex(e=>e.path===path);const next=entries[index+(event.key==='ArrowLeft'?-1:1)];if(!next)return;
      event.preventDefault();const start=anchor.current??path??null;
      if(event.shiftKey){anchor.current=start;setChosen(old=>selectGesture(entries,old,next.path,start,false,true));}else{anchor.current=next.path;setChosen({});}
      choose(next);const button=Array.from(event.currentTarget.querySelectorAll<HTMLButtonElement>('.gallery-file')).find(b=>b.dataset.galleryPath===next.path);button?.focus();
    }
  }
  function changeView(mode: 'grid'|'list') { setView(mode); try { localStorage.setItem('gallery-view',mode); } catch { /* Session preference still works. */ } }
  function navigate(next: string) { setFolder(next); setOffset(0); setSelected(null); setListing(null); setError(''); }
  function choose(next: GalleryEntry) { setSelected(next.path); }
  function clickEntry(entry:GalleryEntry,event:MouseEvent){
    if(batchBusy||!selectionReady)return;
    setChosen(old=>selectGesture(entries,old,entry.path,anchor.current,event.ctrlKey||event.metaKey,event.shiftKey));
    if(!event.shiftKey)anchor.current=entry.path;
    if(!event.ctrlKey&&!event.metaKey&&!event.shiftKey)choose(entry);
  }
  function step(delta: number) { const index=entries.findIndex(e=>e.path===current?.path); const next=entries[index+delta]; if(next) choose(next); }
  async function act(action: () => Promise<void>) {
    if(operation.current) return; operation.current=true; setBusy(true); setError(''); setNote('');
    try { await action(); if(mounted.current) setTick(n=>n+1); } catch(e) { if(mounted.current) setError(galleryError(e,de)); }
    finally { operation.current=false; if(mounted.current) setBusy(false); }
  }

  useEffect(() => {
    const drop = (event: Event) => {
      if (busy || batchBusy || trash || creating || operation.current) return;
      const paths = (event as CustomEvent<{ paths: string[] }>).detail?.paths;
      if (!Array.isArray(paths) || !paths.length || paths.some(p => typeof p !== 'string')) return;
      void act(async () => {
        const copied = await galleryApi.importFiles(folder, paths);
        if (mounted.current) { setNote(de ? `${copied.imported.length} Dateien kopiert. Originale bleiben erhalten.` : `Copied ${copied.imported.length} files. Originals are retained.`); if (copied.errors.length) setError(copied.errors.map(e => galleryError(e, de)).join(' · ')); }
      });
    };
    window.addEventListener('studio-file-drop', drop);
    return () => window.removeEventListener('studio-file-drop', drop);
  }, [busy, batchBusy, trash, creating, folder, de]);
  useEffect(()=>setMenuContext('gallery',{priority:1,actions:detail?.kind==='image'&&!batchBusy?{imageFit:fitImage,imageReset:fitImage,imageActual:()=>{setZoom(1);setActualSize(true);setViewReset(n=>n+1);}}:{}}),[detail?.kind,batchBusy]);
  const previewKey = detail ? detail.url+':'+current?.thumbnailVersion : '';
  const label = (kind: string) => kind==='image' ? (de?'Bilder':'Images') : kind==='video' ? 'Video' : (de?'Audio / Musik':'Audio / Music');
  if(trash)return <GalleryTrash de={de} onBack={()=>{setTrash(false);setTick(n=>n+1);}}/>;
  return <div className="page gallery-page" data-file-drop="gallery" tabIndex={-1} onKeyDown={galleryKey}>
    {correctionBatch&&<BatchCorrections {...correctionBatch} de={de} onClose={()=>{setCorrectionBatch(null);setBatchBusy(false);setChosen({});setSelectionReady(false);setTick(n=>n+1);}}/>}
    {editor&&<ImageEditor onSaveProject={onSaveEdit} onAddToProject={projectDisabled?undefined:ops=>onAddEdit({rootId:editor.rootId,targets:[{path:editor.entry.path,fileId:editor.entry.fileId!,version:editor.entry.thumbnailVersion}]},ops)} entry={editor.entry} rootId={editor.rootId} de={de} shortcuts={shortcuts} maxUndo={maxUndo} onClose={()=>{setEditor(null);setBatchBusy(false);setTick(n=>n+1);}}/>}
    {comparison&&<GalleryCompare shortcuts={shortcuts} entries={comparison.entries} rootId={comparison.rootId} de={de} onClose={()=>{setComparison(null);setBatchBusy(false);setTick(n=>n+1);}}/>}
    <header className="page-heading"><div><div className="eyebrow">{de?'LOKALE MEDIEN':'LOCAL MEDIA'}</div><h1>{de?'Galerie':'Gallery'}</h1><p>{de?'Bilder, Videos, Audio und Musik in echten Ordnern. Neue Dateien werden automatisch erkannt.':'Images, videos, audio and music in real folders. New files are detected automatically.'}</p></div></header>
    {error && <p className="notice warning" role="alert">{error}<button className="text-button" onClick={()=>setError('')}>{de?'Schließen':'Dismiss'}</button></p>}
    {note && <p className="notice" role="status">{note}</p>}
    <p className="hub-hint" data-gallery-watch={watching ? "active" : "fallback"}>{watching ? (de ? "Ordnerüberwachung aktiv · Änderungen erscheinen automatisch." : "Folder monitoring active · Changes appear automatically.") : (de ? "Regelmäßige Aktualisierung · Ordnerüberwachung wird verbunden." : "Periodic refresh · Connecting folder monitoring.")}</p>
      <section className="panel gallery-toolbar">
      <div className="gallery-actions">
        <button className="button secondary" disabled={busy||batchBusy} onClick={()=>{setDetail(null);setTrash(true);}}>{de?'Papierkorb':'Trash'}</button>
        <button className="button primary" disabled={busy} onClick={()=>void act(async()=>{
          const result=await open({multiple:true,filters:[{name:de?'Medien':'Media',extensions:['png','jpg','jpeg','webp','gif','bmp','avif','mp4','m4v','webm','mov','mkv','avi','mp3','wav','ogg','opus','flac','m4a']}]});
          if(!result) return; const copied=await galleryApi.importFiles(folder,typeof result==='string'?[result]:result);
          if(mounted.current) { setNote(de?`${copied.imported.length} Dateien kopiert. Originale bleiben erhalten.`:`Copied ${copied.imported.length} files. Originals are retained.`); if(copied.errors.length) setError(copied.errors.map(e=>galleryError(e,de)).join('\n')); }
        })}><Upload size={16}/>{busy?(de?'Dateiaktion läuft …':'File operation in progress …'):(de?'Dateien kopieren …':'Copy files …')}</button>
        <button className="button secondary" disabled={busy} onClick={()=>setCreating(v=>!v)}><Plus size={16}/>{de?'Neuer Ordner':'New folder'}</button>
        <button className="button secondary" onClick={()=>void act(()=>galleryApi.openFolder(folder))} disabled={busy}><FolderOpen size={16}/>{de?'Ordner im Explorer':'Folder in Explorer'}</button>
        <button className="text-button" disabled={busy} onClick={()=>void act(async()=>{await galleryApi.clearThumbnails();if(mounted.current){setThumbnailEpoch(n=>n+1);setNote(de?'Vorschau-Cache geleert. Sichtbare Miniaturen werden neu erzeugt.':'Preview cache cleared. Visible thumbnails are being regenerated.');}})}>{de?'Vorschaubilder neu aufbauen':'Rebuild thumbnails'}</button>
        <button className="icon-button" aria-label={de?'Galerie aktualisieren':'Refresh gallery'} onClick={()=>{setError('');setTick(n=>n+1);}}><RefreshCw size={17}/></button>
      </div>
      {creating && <form className="gallery-actions" onSubmit={e=>{e.preventDefault();void act(async()=>{await galleryApi.createFolder(folder,newFolder);setNewFolder('');setCreating(false);});}}><input aria-label={de?'Ordnername':'Folder name'} value={newFolder} onChange={e=>setNewFolder(e.target.value)} maxLength={100} autoFocus/><button className="button secondary" disabled={busy||!newFolder.trim()}>{de?'Ordner anlegen':'Create folder'}</button></form>}
      <p className="gallery-root">{listing?.root ?? (de?'Galerie wird gelesen …':'Reading gallery …')}</p>
      <div className="gallery-actions"><button className="text-button" disabled={!folder} onClick={()=>navigate('')}>{de?'Galerieordner':'Gallery folder'}</button>{folder && <><span>/ {folder}</span><button className="text-button" onClick={()=>navigate(folder.split('/').slice(0,-1).join('/'))}><ArrowLeft size={14}/>{de?'Übergeordneter Ordner':'Parent folder'}</button></>}</div>
      <div className="gallery-filters"><input aria-label={de?'Dateien suchen':'Search files'} placeholder={de?'Dateiname, Modell, Prompt oder Tag suchen …':'Search filename, model, prompt or tag …'} value={search} onChange={e=>{setSearch(e.target.value);setOffset(0);}} maxLength={256}/><select aria-label={de?'Medientyp':'Media type'} value={kind} onChange={e=>{setKind(e.target.value as typeof kind);setOffset(0);}}><option value="all">{de?'Alle Medien':'All media'}</option>{(['image','video','audio'] as const).map(k=><option key={k} value={k}>{label(k)}</option>)}</select><label><input type="checkbox" checked={recursive} onChange={e=>{setRecursive(e.target.checked);setOffset(0);}}/>{de?'Unterordner einbeziehen':'Include subfolders'}</label></div>
      <div className="gallery-filters"><label>{de?'Sortieren':'Sort'}<select aria-label={de?'Galerie sortieren':'Gallery sort'} value={sort} onChange={e=>changeSort(e.target.value as GallerySort)}>{([['modifiedDesc','Zuletzt geändert zuerst','Recently modified first'],['modifiedAsc','Älteste Änderung zuerst','Oldest modified first'],['nameAsc','Name A–Z','Name A–Z'],['nameDesc','Name Z–A','Name Z–A'],['sizeDesc','Größte zuerst','Largest first'],['sizeAsc','Kleinste zuerst','Smallest first']] as const).map(([value,german,english])=><option key={value} value={value}>{de?german:english}</option>)}</select></label><label><input type="checkbox" checked={favoritesOnly} onChange={e=>{setFavoritesOnly(e.target.checked);setOffset(0);}}/>{de?'Nur Favoriten':'Favorites only'}</label><select aria-label={de?'Tagfilter':'Tag filter'} value={tag} onChange={e=>{setTag(e.target.value);setOffset(0);}}><option value="">{de?'Alle Tags':'All tags'}</option>{[...new Set([...(listing?.tags??[]),...(tag?[tag]:[])])].map(t=><option key={t} value={t}>{t}</option>)}</select></div>
      {!!listing?.folders.length && <div className="gallery-folders">{listing.folders.map(f=><button key={f.path} className="button secondary" onClick={()=>navigate(f.path)}><Folder size={15}/>{f.name}</button>)}</div>}
      {(listing?.limited || !!listing?.skipped) && <p className="notice warning">{de?`Ansicht begrenzt oder Pfade übersprungen (${listing?.skipped}). Einen Unterordner öffnen, um gezielter zu suchen.`:`View limited or paths skipped (${listing?.skipped}). Open a subfolder to narrow the search.`}</p>}
    </section>
    <div className={`gallery-layout gallery-layout-${view}`}><section className="panel gallery-files"><div className="section-heading"><h2>{de?'Medien':'Media'} <small>({listing?.total ?? '…'})</small></h2><div className="gallery-actions" role="group" aria-label={de?'Galerieansicht':'Gallery view'}>{(['grid','list'] as const).map(mode=><button key={mode} className="icon-button" aria-label={mode==='grid'?(de?'Rasteransicht':'Grid view'):(de?'Listenansicht':'List view')} aria-pressed={view===mode} onClick={()=>changeView(mode)}>{mode==='grid'?<Grid2X2 size={18}/>:<List size={18}/>}</button>)}</div></div>
      <div className="gallery-actions gallery-selection-controls"><button className="text-button" disabled={batchBusy||!selectionReady||!entries.some(e=>e.fileId)} onClick={selectPage}>{de?'Alle auf dieser Seite auswählen':'Select all on this page'}</button><button className="text-button" disabled={batchBusy||!Object.keys(chosen).length} onClick={()=>setChosen({})}>{de?'Auswahl aufheben':'Clear selection'}</button><span className="hub-hint" role="status">{de?`${Object.keys(chosen).length} ausgewählt`:`${Object.keys(chosen).length} selected`}</span></div>
      <details className="gallery-shortcuts"><summary>{de?'Auswahl & Tastatur':'Selection & keyboard'}</summary><p className="hub-hint">{de?'Strg + Klick: einzeln auswählen · Umschalt + Klick: Bereich auswählen · Klick: Vorschau öffnen. Esc: Auswahl aufheben. Weitere Tastenkürzel sind in den Einstellungen frei belegbar. Links/Rechts wechselt Medien; mit Umschalt einen Bereich auswählen.':'Ctrl + click: toggle selection · Shift + click: select a range · Click: open preview. Esc: clear selection. Other shortcuts can be customized in Settings. Left/right switch media; hold Shift to select a range.'}</p></details>
      <button className="button secondary" disabled={busy||batchBusy||!selectionReady||!Object.values(chosen).length||Object.values(chosen).some(e=>e.kind!=='image'||!e.fileId)} onClick={()=>{setCorrectionBatch({entries:Object.values(chosen),rootId:listing!.rootId});setBatchBusy(true);}}>{de?'Bilder korrigieren …':'Batch image corrections …'}</button>
      <GalleryFileActions extra={<button className="button secondary" disabled={busy||batchBusy||!selectionReady||Object.values(chosen).length!==2||Object.values(chosen).some(e=>e.kind!=='image'||!e.fileId)} data-gallery-compare onClick={()=>{setComparison({entries:Object.values(chosen),rootId:listing!.rootId});setBatchBusy(true);}}>{de?'Zwei Bilder vergleichen':'Compare two images'}</button>} entries={Object.keys(chosen).length?Object.values(chosen):(current?.fileId?[current]:[])} rootId={listing?.rootId??''} de={de} disabled={busy||batchBusy||!selectionReady} onBusy={setBatchBusy} onPrepare={()=>{generation.current++;setDetail(null);}} onFinished={report=>{setChosen({});setSelectionReady(false);setTick(n=>n+1);if(report){setNote(de?`${report.completed.length} Dateiaktionen abgeschlossen.`:`Completed ${report.completed.length} file operations.`);setError(report.errors.map(e=>galleryError(e,de)).join('\n'));}}}/>
      <button className="button secondary" disabled={busy || batchBusy || projectDisabled || !selectionReady || !(Object.keys(chosen).length || current?.fileId)} onClick={() => void act(async () => {
        const selected = Object.keys(chosen).length ? Object.values(chosen) : current ? [current] : [];
        if (listing && selected.length && selected.every(e => e.fileId)) await onAddToProject({ rootId: listing.rootId, targets: selected.map(e => ({ path: e.path, fileId: e.fileId!, version: e.thumbnailVersion })) });
      })}>{de ? 'Auswahl ins Projekt übernehmen' : 'Add selection to project'}</button>
      <GalleryBatch disabled={batchBusy} entries={Object.values(chosen)} rootId={listing?.rootId??''} de={de} onBusy={setBatchBusy} onFinished={()=>{setSelectionReady(false);setChosen({});setTick(n=>n+1);}}/>
      <div className={`gallery-items gallery-items-${view}`}>
      {listing && !entries.length && <p className="hub-hint">{de?'Keine passenden Medien. Dateien kopieren, im Explorer ablegen oder Suchfilter ändern.':'No matching media. Copy files, add them through Explorer or change the filters.'}</p>}
      {entries.map(e=>{const Icon=e.kind==='image'?Image:e.kind==='video'?Video:Music;return <div className="gallery-card" key={e.path}><label className="gallery-select" title={!e.fileId?galleryError('gallery_identity',de):(de?'Für Sammelaktionen auswählen':'Select for batch actions')}><input type="checkbox" aria-label={de?`Auswählen: ${e.path}`:`Select: ${e.path}`} disabled={batchBusy||!selectionReady||!e.fileId} checked={!!chosen[e.path]} onChange={event=>toggleSelection(e,event.target.checked)}/></label><button className={`gallery-file ${current?.path===e.path?'selected':''}`} aria-pressed={current?.path===e.path} disabled={batchBusy||!selectionReady} data-gallery-path={e.path} onClick={event=>clickEntry(e,event)}><>{view==='grid' && listing ? <GalleryThumbnail entry={e} rootId={listing.rootId} epoch={thumbnailEpoch} de={de}/> : <Icon size={24}/>}</><span className="gallery-file-info"><strong>{e.annotation.favorite && <Star size={13} className="gallery-star" aria-label={de?'Favorit':'Favorite'}/>} {e.name}</strong><small>{e.path}</small><small>{label(e.kind)} · {formatBytes(e.bytes,language)}</small>{e.origin && <small>{e.origin.modelName} · {e.origin.request.width} × {e.origin.request.height}</small>}<small>{new Date(e.modified).toLocaleDateString(language)}</small>{e.annotation.tags.length>0 && <small className="gallery-row-tags">{e.annotation.tags.join(' · ')}</small>}</span></button></div>;})}
      </div>
      {(listing?.total ?? 0)>50 && <div className="gallery-pagination"><button className="button secondary" disabled={!offset} onClick={()=>setOffset(n=>Math.max(0,n-50))}><ArrowLeft size={15}/></button><span>{offset+1}–{Math.min(offset+50,listing?.total??0)} / {listing?.total}</span><button className="button secondary" disabled={offset+50>=(listing?.total??0)} onClick={()=>setOffset(n=>n+50)}><ArrowRight size={15}/></button></div>}
    </section><section className="panel gallery-viewer" ref={viewer} tabIndex={0} aria-label={de?'Medienvorschau':'Media preview'} onKeyDown={e=>{if(e.target!==e.currentTarget)return;const command=shortcutFor(e.nativeEvent,shortcuts);if(detail?.kind==='image' && ['imageFit','imageActual','imageReset'].includes(command??'')){e.preventDefault();e.stopPropagation();fitImage();if(command==='imageActual')setActualSize(true);return;}if(e.key==='ArrowLeft'){e.preventDefault();step(-1);}if(e.key==='ArrowRight'){e.preventDefault();step(1);}}}>
      {current && <><div className="section-heading"><h2>{current.name}</h2><div className="gallery-actions"><button className="icon-button" aria-label={de?'Vorheriges Medium':'Previous media'} disabled={entries[0]?.path===current.path} onClick={()=>step(-1)}><ArrowLeft size={16}/></button><button className="icon-button" aria-label={de?'Nächstes Medium':'Next media'} disabled={entries.at(-1)?.path===current.path} onClick={()=>step(1)}><ArrowRight size={16}/></button></div></div>
      {current.kind==='image' && <button className="button primary" disabled={busy||batchBusy||!selectionReady||!current.fileId} onClick={()=>{setEditor({entry:current,rootId:listing!.rootId});setBatchBusy(true);}}>{de?'Bild bearbeiten':'Edit image'}</button>}
      {current.kind==='image'&&current.name.toLowerCase().endsWith('.png')&&<button className="button secondary" disabled={busy||batchBusy||!selectionReady||restoreDisabled} onClick={()=>void (async()=>{if(!listing)return;setBusy(true);setError('');try{const result=await invoke<{reference:ImageReference}>('image_reference',{path:listing.root+'\\'+current.path.replaceAll('/','\\')});onReference(result.reference);}catch(e){setError(imageError(e,de));}finally{setBusy(false);}})()}>{de?'Als Bildreferenz verwenden':'Use as image reference'}</button>}
      {detail?.kind==='image' && <><div className="gallery-actions"><label>Zoom <input aria-label="Image zoom" type="range" min={1} max={4} step={0.25} value={zoom} onChange={e=>setZoom(Number(e.target.value))}/></label><button className="text-button" onClick={fitImage}>{de?'Einpassen':'Fit'}</button><button className="text-button" onClick={()=>{setZoom(1);setActualSize(true);setViewReset(n=>n+1);}}>100 %</button><button className="text-button" onClick={()=>void viewer.current?.requestFullscreen().catch(()=>setError(de?'Vollbild nicht verfügbar.':'Fullscreen unavailable.'))}>{de?'Vollbild':'Fullscreen'}</button></div><GalleryImage key={previewKey} url={detail.url} zoom={zoom} actualSize={actualSize} resetKey={viewReset} alt={de?'Galeriebild':'Gallery image'} onError={()=>setPreviewError(previewKey)}/></>}
      {detail?.kind==='video' && <video key={detail.url+current.modified} src={detail.url} controls preload="metadata" onError={()=>setPreviewError(previewKey)}/>}
      {detail?.kind==='audio' && <div className="gallery-audio"><Music size={64}/><audio key={detail.url+current.modified} src={detail.url} controls preload="metadata" onError={()=>setPreviewError(previewKey)}/></div>}
      {previewError && previewError===previewKey && <p className="notice warning">{de?'Keine Vorschau möglich. Das Format/der Codec wird nicht unterstützt, die Datei ist beschädigt oder das Bild überschreitet 32 MiB. Die Originaldatei bleibt erhalten.':'Preview unavailable. The format/codec is unsupported, the file is damaged or the image exceeds 32 MiB. The original file is retained.'}</p>}
      <p className="gallery-root">{current.path}</p><p className="hub-hint">{formatBytes(current.bytes,language)} · {new Date(current.modified).toLocaleString(language)}</p>
      {detail?.dimensions&&<p className="hub-hint">{detail.dimensions[0]} × {detail.dimensions[1]} px</p>}
      {detail?.origin&&<dl className="gallery-metadata"><dt>{de?'Modell':'Model'}</dt><dd>{detail.origin.modelName}</dd><dt>SHA-256</dt><dd>{detail.origin.modelSha256??'—'}</dd><dt>Runtime</dt><dd>{detail.origin.runtime}</dd><dt>{de?'Erstellt':'Created'}</dt><dd>{new Date(detail.origin.createdAt).toLocaleString(language)}</dd><dt>{de?'Auftrag':'Job'}</dt><dd>{detail.origin.jobId}</dd></dl>}
      <button className="text-button" onClick={()=>void act(()=>galleryApi.openFolder(current.path.split('/').slice(0,-1).join('/')))}>{de?'Speicherordner öffnen':'Open containing folder'}</button>
      {listing && <GalleryAnnotations key={listing.rootId+':'+(current.fileId??current.path)} entry={current} rootId={listing.rootId} de={de} onChanged={()=>setTick(n=>n+1)}/>}
      {detail?.request && <><h3>{de?'Generierungseinstellungen':'Generation settings'}</h3><p className="gallery-prompt">{detail.request.prompt}</p>{detail.request.negativePrompt&&<><h4>{de?'Negativer Prompt':'Negative prompt'}</h4><p className="gallery-prompt">{detail.request.negativePrompt}</p></>}<p className="hub-hint">{detail.request.width} × {detail.request.height} · {detail.request.steps} steps · CFG {detail.request.guidance} · Seed {detail.request.seed} · {detail.request.sampler}</p><button className="button secondary" disabled={restoreDisabled} onClick={()=>detail.request&&onRestore(detail.request)}>{de?'Einstellungen wiederherstellen':'Restore settings'}</button><p className="hub-hint">{de?'Öffnet die Eingaben im Studio, ohne eine Generierung zu starten.':'Opens the inputs in Studio without starting generation.'}</p></>}
{current?.fileId && listing && <GalleryLineage key={`${listing.rootId}:${current.fileId}:${current.thumbnailVersion}`} entry={current} rootId={listing.rootId} refresh={tick} de={de} disabled={busy||batchBusy||!selectionReady} onChange={()=>setTick(n=>n+1)} onNavigate={path=>{setFolder(path.includes('/')?path.slice(0,path.lastIndexOf('/')):'');setSearch('');setKind('all');setTag('');setFavoritesOnly(false);setOffset(0);setSelected(path);setTick(n=>n+1);}}/>}
      </>}
    </section></div>
    <p className="hub-hint">{de?'Audio und Musik werden gemeinsam geführt. Wiedergabe hängt vom Format und den verfügbaren Windows/WebView-Codecs ab. Das Raster lädt kleine Bild- und Videominiaturen; die große Vorschau nur das ausgewählte Medium. Die Ordnerüberwachung wird durch regelmäßige Prüfungen ergänzt.':'Audio and music share one category. Playback depends on the format and available Windows/WebView codecs. The grid loads small image and video thumbnails; the full preview loads only the selected media. Folder monitoring is supplemented by periodic checks.'}</p>
  </div>;
}
