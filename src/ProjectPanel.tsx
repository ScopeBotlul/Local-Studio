import { useEffect, useState } from 'react';
import { ChevronDown, FolderOpen, LoaderCircle, Save, X } from 'lucide-react';
import type { useProject } from './useProject';
import { formatBytes } from './helpers';
import type { Language } from './types';
import './projects.css';

export default function ProjectPanel({ controller: c, language, disabled, changed }: { controller: ReturnType<typeof useProject>; language: Language; disabled: boolean; changed: boolean }) {
  const de = language === 'de'; const p = c.project;
  const [expanded, setExpanded] = useState(false);
  const [name, setName] = useState('');
  const [rename, setRename] = useState('');
  useEffect(() => { setRename(p?.name ?? ''); }, [p?.id, p?.name]);
  const [selected, select] = useState<string | null>(null);
  const [failed, setFailed] = useState('');
  const asset = p?.assets.find(a => a.id === selected);
  const url = p && asset ? `http://project.localhost/${p.id}/${asset.id}` : '';
  const locked = disabled || c.busy || !c.ready;
  return <section className="project-panel" data-file-drop="project" aria-label={de ? 'Projekt' : 'Project'}>
    <div className="project-toolbar">
      <button className="button secondary project-title" onClick={() => setExpanded(!expanded)} aria-expanded={expanded}><FolderOpen size={17} /><span>{p ? `${p.name}${!p.recovery && (p.dirty || changed) ? ' *' : ''}` : de ? 'Ohne Projekt' : 'No project'}</span><ChevronDown size={15} /></button>
      {c.busy && <LoaderCircle className="spin" size={17} aria-label={de ? 'Projekt wird verarbeitet' : 'Processing project'} />}
      {expanded && <><button className="button secondary" disabled={locked} onClick={() => void c.open()}>{de ? 'Projekt öffnen' : 'Open project'}</button>
      <button className="button secondary" disabled={locked || !!p?.recovery} onClick={() => void c.save()}><Save size={15} />{de ? 'Projekt speichern' : 'Save project'}</button>
      {p && <button className="icon-button" title={de ? 'Projekt schließen' : 'Close project'} disabled={locked} onClick={() => void c.close()}><X size={16} /></button>}</>}
    </div>
    {p?.recovery && <div className="project-message"><span>{de ? 'Ein lokaler Projektarbeitsstand ist vorhanden. Du kannst ihn fortsetzen oder das Projekt schließen.' : 'A local project workspace is available. Resume it or close the project.'}</span><button className="button secondary" disabled={c.busy || !c.ready} onClick={() => void c.recover()}>{de ? 'Projekt fortsetzen' : 'Resume project'}</button></div>}
    {c.error && <p role="alert" className="inline-warning">{c.error}</p>}
    {c.notice && <p role="status" className="project-notice">{c.notice}</p>}
    {expanded && <div className="project-details">
      {c.recent.length>0 && <details className="project-recent"><summary>{de?'Zuletzt geöffnete Projekte':'Recent projects'}</summary>{c.recent.map(item=><div className="project-message" key={item.path}><button className="text-button" disabled={locked||!item.available} onClick={()=>void c.openPath(item.path)}>{item.name}<small className="project-path">{item.path}{!item.available&&(de?' · nicht verfügbar':' · unavailable')}</small></button><button className="text-button" disabled={locked} onClick={()=>void c.forgetRecent(item.path)}>{de?'Aus Liste entfernen':'Remove from list'}</button></div>)}</details>}
      <div className="project-create"><input aria-label={de ? 'Projektname' : 'Project name'} maxLength={100} placeholder={de ? 'Neues Projekt' : 'New project'} value={name} disabled={locked} onChange={e => setName(e.target.value)} /><button className="button secondary" disabled={locked || !name.trim() || !!p?.recovery} onClick={() => void c.create(name.trim())}>{de ? 'Projekt anlegen' : 'Create project'}</button></div>
      <p>{de ? 'Ein Projekt speichert die aktuellen Bild-Studio-Eingaben und die hier hinzugefügten Medien. Ergebnisse aus Studio und Galerie kannst du direkt übernehmen. Modellgewichte bleiben extern. Maximal 100 Medien / 2 GB.' : 'A project stores current image Studio inputs and media added here. Transfer Studio and gallery results directly. Model weights stay external. Up to 100 media files / 2 GB.'}</p>
      <p>{de ? 'Dateien hierher ziehen, um Kopien ins Projekt zu übernehmen. Originale bleiben erhalten.' : 'Drop files here to copy them into the project. Originals are retained.'}</p>
      {p && !p.recovery && <>
        <p className="project-path">{p.path ?? (de ? 'Noch keine .localstudio-Datei gespeichert.' : 'No .localstudio file saved yet.')}</p>
        <div className="project-create"><input aria-label={de ? 'Projekt umbenennen' : 'Rename project'} value={rename} maxLength={100} disabled={locked} onChange={e => setRename(e.target.value)} /><button className="button secondary" disabled={locked || !rename.trim() || rename.trim() === p.name} onClick={() => void c.rename(rename.trim())}>{de ? 'Namen ändern' : 'Change name'}</button></div>
        <div className="project-actions"><button className="button secondary" disabled={locked} onClick={() => void c.save(true)}>{de ? 'Speichern unter …' : 'Save as …'}</button><button className="button secondary" disabled={locked} onClick={() => void c.add()}>{de ? 'Medien hinzufügen' : 'Add media'}</button><button className="button secondary" disabled={locked || !p.assets.length} onClick={() => void c.exportGallery()}>{de ? 'Medien in Galerie kopieren' : 'Copy media to gallery'}</button></div>
        {p.model && <div className="project-model"><strong>{de ? 'Gespeicherte Modellreferenz' : 'Saved model reference'}: {p.model.name}</strong><code title="SHA-256">{p.model.sha256}</code><p>{p.request?.modelPath ? (de ? 'Lokaler Modellpfad zugeordnet. Das Studio prüft die Generierungsbereitschaft separat.' : 'Local model path assigned. Studio checks generation readiness separately.') : (de ? 'Modell noch nicht zugeordnet. Projektmedien bleiben verfügbar; zum Generieren ein lokales Modell auswählen.' : 'Model is not linked. Project media remain available; choose a local model to generate.')}</p><button className="button secondary" disabled={locked} onClick={() => void c.relink()}>{de ? 'Originalmodell zuordnen' : 'Link original model'}</button></div>}
        {!p.assets.length ? <p>{de ? 'Dieses Projekt enthält noch keine Medien.' : 'This project has no media yet.'}</p> : <div className="project-media"><ul>{p.assets.map(a => <li key={a.id}><button className="project-asset" aria-pressed={a.id === selected} onClick={() => { select(a.id); setFailed(''); }}><span>{a.name}</span><small>{a.kind} · {formatBytes(a.bytes, language)}</small></button><button className="icon-button" title={`${de ? 'Aus Projekt entfernen' : 'Remove from project'}: ${a.name}`} disabled={locked} onClick={() => void c.remove(a.id)}><X size={15} /></button></li>)}</ul>{asset && <div className="project-preview" key={url}>{asset.kind === 'image' ? <img src={url} alt={asset.name} onError={() => setFailed(url)} /> : asset.kind === 'video' ? <video src={url} controls preload="metadata" onError={() => setFailed(url)} /> : <audio src={url} controls preload="metadata" onError={() => setFailed(url)} />}{failed === url && <p>{de ? 'Dieses Medium kann nicht als Vorschau angezeigt werden. Die eingebettete Datei bleibt erhalten.' : 'Preview is unavailable for this media. The embedded file is preserved.'}</p>}</div>}</div>}
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
