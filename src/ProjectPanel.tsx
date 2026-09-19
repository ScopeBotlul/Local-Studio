import {PrivacyGate} from "./Privacy";
import { useEffect, useState } from 'react';
import { FolderOpen, LoaderCircle, X } from 'lucide-react';
import type { useProject } from './useProject';
import { formatBytes } from './helpers';
import type { Language } from './types';
import './projects.css';
import ImageEditor from './ImageEditor';
import type {ProjectAsset} from './project-api';
import type {Shortcuts} from './shortcuts';

export default function ProjectPanel({ shortcuts,maxUndo,controller: c, language, disabled, changed, onClose }: { shortcuts:Shortcuts;maxUndo:number;controller: ReturnType<typeof useProject>; language: Language; disabled: boolean; changed: boolean; onClose:()=>void }) {
  const de = language === 'de'; const p = c.project;
  const [editing,setEditing]=useState<ProjectAsset|null>(null);
  const [rename, setRename] = useState('');
  useEffect(() => { setRename(p?.name ?? ''); }, [p?.id, p?.name]);
  const [selected, select] = useState<string | null>(null);
  const [failed, setFailed] = useState('');
  const asset = p?.assets.find(a => a.id === selected);
  const url = p && asset ? `http://project.localhost/${p.id}/${asset.id}` : '';
  const locked = disabled || c.busy || !c.ready;
  if(p?.locked)return <section className="project-details-page"><button className="button secondary" onClick={onClose}>{de?'Zurück zum Arbeitsbereich':'Back to workspace'}</button><PrivacyGate de={de}/></section>;
  return <section className="project-panel project-details-page" data-file-drop="project" aria-label={de ? 'Projekt' : 'Project'}>
    {editing&&p&&<ImageEditor key={editing.id} entry={{name:editing.name,path:editing.name,fileId:editing.id,thumbnailVersion:editing.sha256}} rootId={p.id} de={de} shortcuts={shortcuts} maxUndo={maxUndo} project={{query:{id:p.id,assetId:editing.id,sha256:editing.sha256},operations:editing.edit??[]}} onSaveProject={c.saveEdit} onClose={()=>setEditing(null)}/>}
    <div className="project-details-heading"><h2><FolderOpen size={17}/> {de?'Projektmedien und Details':'Project media and details'}{p?` · ${p.name}${!p.recovery&&(p.dirty||changed)?' *':''}`:''}</h2>{c.busy&&<LoaderCircle className="spin" size={17}/>}<button className="button secondary" onClick={onClose}>{de?'Zurück zum Arbeitsbereich':'Back to workspace'}</button></div>
    {p?.recovery && <div className="project-message"><span>{de ? 'Ein lokaler Projektarbeitsstand ist vorhanden. Du kannst ihn fortsetzen oder das Projekt schließen.' : 'A local project workspace is available. Resume it or close the project.'}</span><button className="button secondary" disabled={c.busy || !c.ready} onClick={() => void c.recover()}>{de ? 'Projekt fortsetzen' : 'Resume project'}</button></div>}
    {c.error && <p role="alert" className="inline-warning">{c.error}</p>}
    {c.notice && <p role="status" className="project-notice">{c.notice}</p>}
    {<div className="project-details">
      {c.recent.length>0 && <details className="project-recent"><summary>{de?'Zuletzt geöffnete Projekte':'Recent projects'}</summary>{c.recent.map(item=><div className="project-message" key={item.path}><button className="text-button" disabled={locked||!item.available} onClick={()=>void c.openPath(item.path)}>{item.name}<small className="project-path">{item.path}{!item.available&&(de?' · nicht verfügbar':' · unavailable')}</small></button><button className="text-button" disabled={locked} onClick={()=>void c.forgetRecent(item.path)}>{de?'Aus Liste entfernen':'Remove from list'}</button></div>)}</details>}
      {!p&&<p>{de?'Über Datei → Neues Projekt oder Projekt öffnen kannst du ein Projekt laden.':'Use File → New project or Open project to start.'}</p>}
      <p>{de ? 'Ein Projekt speichert die aktuellen Bild-Studio-Eingaben und die hier hinzugefügten Medien. Ergebnisse aus Studio und Galerie kannst du direkt übernehmen. Modellgewichte bleiben extern. Maximal 100 Medien / 2 GB.' : 'A project stores current image Studio inputs and media added here. Transfer Studio and gallery results directly. Model weights stay external. Up to 100 media files / 2 GB.'}</p>
      <p>{de ? 'Dateien hierher ziehen, um Kopien ins Projekt zu übernehmen. Originale bleiben erhalten.' : 'Drop files here to copy them into the project. Originals are retained.'}</p>
      {p && !p.recovery && <>
        <p className="project-path">{p.path ?? (de ? 'Noch keine .localstudio-Datei gespeichert.' : 'No .localstudio file saved yet.')}</p>
        <div className="project-create"><input aria-label={de ? 'Projekt umbenennen' : 'Rename project'} value={rename} maxLength={100} disabled={locked} onChange={e => setRename(e.target.value)} /><button className="button secondary" disabled={locked || !rename.trim() || rename.trim() === p.name} onClick={() => void c.rename(rename.trim())}>{de ? 'Namen ändern' : 'Change name'}</button></div>
        <div className="project-actions"><button className="button secondary" disabled={locked} onClick={() => void c.add()}>{de ? 'Medien hinzufügen' : 'Add media'}</button><button className="button secondary" disabled={locked || !p.assets.length} onClick={() => void c.exportGallery()}>{de ? 'Medien in Galerie kopieren' : 'Copy media to gallery'}</button></div>
        {p.model && <div className="project-model"><strong>{de ? 'Gespeicherte Modellreferenz' : 'Saved model reference'}: {p.model.name}</strong><code title="SHA-256">{p.model.sha256}</code><p>{p.request?.modelPath ? (de ? 'Lokaler Modellpfad zugeordnet. Das Studio prüft die Generierungsbereitschaft separat.' : 'Local model path assigned. Studio checks generation readiness separately.') : (de ? 'Modell noch nicht zugeordnet. Projektmedien bleiben verfügbar; zum Generieren ein lokales Modell auswählen.' : 'Model is not linked. Project media remain available; choose a local model to generate.')}</p><button className="button secondary" disabled={locked} onClick={() => void c.relink()}>{de ? 'Originalmodell zuordnen' : 'Link original model'}</button></div>}
        {!p.assets.length ? <p>{de ? 'Dieses Projekt enthält noch keine Medien.' : 'This project has no media yet.'}</p> : <div className="project-media"><ul>{p.assets.map(a => <li key={a.id}><button className="project-asset" aria-pressed={a.id === selected} onClick={() => { select(a.id); setFailed(''); }}><span>{a.name}</span><small>{a.kind} · {formatBytes(a.bytes, language)}{a.edit?.length?` · ${a.edit.length} ${de?'Bearbeitungsschritte':'edit steps'}`:''}</small></button><button className="icon-button" title={`${de ? 'Aus Projekt entfernen' : 'Remove from project'}: ${a.name}`} disabled={locked} onClick={() => void c.remove(a.id)}><X size={15} /></button></li>)}</ul>{asset && <div className="project-preview" key={url}>{asset.kind==='image'&&<div><button className="button primary" disabled={locked} onClick={()=>setEditing(asset)}>{de?'Projektbild bearbeiten':'Edit project image'}</button>{!!asset.edit?.length&&<p>{de?'Hier siehst du das Original. Im Editor werden die gespeicherten Schritte angewendet.':'This is the original. The editor applies the saved steps.'}</p>}</div>}{asset.kind === 'image' ? <img src={url} alt={asset.name} onError={() => setFailed(url)} /> : asset.kind === 'video' ? <video src={url} controls preload="metadata" onError={() => setFailed(url)} /> : <audio src={url} controls preload="metadata" onError={() => setFailed(url)} />}{failed === url && <p>{de ? 'Dieses Medium kann nicht als Vorschau angezeigt werden. Die eingebettete Datei bleibt erhalten.' : 'Preview is unavailable for this media. The embedded file is preserved.'}</p>}</div>}</div>}
        <small>{de ? 'Entfernen betrifft nur dieses Projekt. Originale und Galeriedateien bleiben erhalten; beim nächsten Speichern wird der Container ohne entfernte Medien geschrieben.' : 'Removal affects only this project. Originals and gallery files remain; the next save excludes removed media.'}</small>
        {p.removed.length > 0 && <details className="project-recovery"><summary>{de ? `Entfernte Medien zurückholen (${p.removed.length})` : `Restore removed media (${p.removed.length})`}</summary>{[...p.removed].reverse().map(a => <div className="project-message" key={a.id}><span>{a.name} · {formatBytes(a.bytes, language)}</span><button className="button secondary" disabled={locked} onClick={() => void c.restoreMedia(a.id)}>{de ? 'Zurückholen' : 'Restore media'}</button></div>)}</details>}
      </>}
      <details className="project-recovery"><summary>{de ? `Wiederherstellungsstände (${c.history.length})` : `Recovery points (${c.history.length})`}</summary>
        <p>{de ? 'Bis zu 20 lokale Stände. Die letzten drei bleiben vor Bereinigung geschützt. Wiederherstellen verändert die Projektdatei erst beim nächsten Speichern.' : 'Up to 20 local snapshots. The latest three are protected from cleanup. Restoring changes the project file only when you save again.'}</p>
        <button className="button secondary" disabled={locked || !p || p.recovery} onClick={() => void c.checkpoint()}>{de ? 'Wiederherstellungspunkt erstellen' : 'Create recovery point'}</button>
        {c.history.map(point => <div className="project-history-point" key={point.id}><div><strong>{point.name}</strong><small>{new Date(point.at * 1000).toLocaleString(language)} · {point.media} {de ? 'Medien' : 'media'}{point.protected ? (de ? ' · geschützt' : ' · protected') : ''}</small><p>{point.prompt}</p></div><button className="button secondary" disabled={c.busy || !c.ready} onClick={() => void c.restorePoint(point.id)}>{de ? 'Stand wiederherstellen' : 'Restore snapshot'}</button></div>)}
      </details>
    </div>}
  </section>;
}
