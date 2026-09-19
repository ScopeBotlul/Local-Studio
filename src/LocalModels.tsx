import {ModelPrivacy} from "./Privacy";
import { useEffect, useRef, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { FolderOpen, HardDrive, RefreshCw, Search, X } from 'lucide-react';
import DownloadsPage from './DownloadsPage';
import ModelTransfer from './ModelTransfer';
import ModelUpdates from './ModelUpdates';
import Benchmarks from './Benchmarks';
import { displayPath, formatBytes, formatDate, formatGigabytes } from './helpers';
import type { Language } from './types';
import './local-models.css';
import {ModelClassification,modelCategories,categoryLabel,inModelCategory,type ModelCategory,type ModelProfile} from './ModelClassification';

interface LocalFile { path: string; size: number | null; modified: number | null; }
export interface LocalModel {
  profile?:ModelProfile|null;
  id: string; name: string; path: string; sourceRoot: string; format: string; kind: string;
  discovery: 'model' | 'candidate' | 'excluded'; discoveryReason: string;
  family: string | null; totalBytes: number; status: string; completeness: string; files: LocalFile[]; checkedAt: string;
}
interface Scan { mode: 'folder' | 'quick' | 'full'; roots: string[]; rootsFinished: number; status: string; root: string; visited: number; found: number; imported: number; skipped: number; truncated: boolean; notes: { path: string; code: string }[]; }
export interface Snapshot { entries: LocalModel[]; scan: Scan; }
const labels: Record<string, [string, string]> = {
  checked: ['Dateistruktur geprüft', 'File structure checked'], recognized: ['Format erkannt', 'Format recognized'], unverified: ['Ungeprüfter Kandidat', 'Unverified candidate'],
  incomplete: ['Index-Dateien fehlen', 'Indexed files missing'], invalid: ['Dateistruktur fehlerhaft', 'Invalid file structure'], missing: ['Nicht gefunden', 'Not found'],
  changed: ['Dateien verändert · erneut prüfen', 'Files changed · recheck'], unavailable: ['Speicherort nicht verfügbar', 'Storage unavailable'],
  idle: ['Noch kein Suchlauf', 'No scan yet'], running: ['Suche läuft', 'Scan running'], completed: ['Suche abgeschlossen', 'Scan completed'],
  cancelled: ['Suche abgebrochen · Ergebnisse nicht übernommen', 'Scan cancelled · results not imported'], interrupted: ['Suche durch Neustart unterbrochen', 'Scan interrupted by restart'], failed: ['Suche fehlgeschlagen', 'Scan failed'],
  local_system_files: ['Papierkorb oder Systemdateien', 'Recycle bin or system files'],
  local_application_files: ['Bestandteil einer installierten Anwendung', 'Installed application files'],
  local_dependency_files: ['Bibliotheks-, Entwicklungs- oder Testdateien', 'Dependency, development or test files'],
  local_plain_text: ['Textdatei; kein binärer Modell-Checkpoint', 'Text file; not a binary model checkpoint'],
  local_tensor_data: ['Tensorcontainer ohne erkannte Modellgewichte, z. B. Trainings-Zwischendaten', 'Tensor container without identified model weights, e.g. cached training data'],
  local_identified_format: ['Modellformat erkannt', 'Model format identified'],
  local_unconfirmed_format: ['Modell nicht bestätigt', 'Model not confirmed'],
  local_path: ['Bitte einen absoluten lokalen Ordnerpfad wählen. Netzwerkpfade werden noch nicht unterstützt.', 'Select an absolute local folder path. Network paths are not supported yet.'],
  local_quick_empty: ['Keine typischen Modellordner gefunden. Bitte einen Ordner wählen.', 'No typical model folders found. Please choose a folder.'],
  local_drives: ['Keine lokalen Laufwerke für die Suche verfügbar.', 'No local drives available for scanning.'],
  local_folder: ['Bitte einen Ordner auswählen.', 'Select a folder.'], local_busy: ['Bitte den laufenden Suchlauf abwarten oder abbrechen.', 'Wait for the scan or cancel it first.'],
  local_unavailable: ['Speicherort nicht lesbar oder nicht verfügbar.', 'Storage is unreadable or unavailable.'], local_link: ['Verknüpfung oder umgeleiteter Pfad übersprungen.', 'Link or redirected path skipped.'],
  local_limit: ['Such- oder Dateigrenze erreicht.', 'Scan or file limit reached.'], local_missing: ['Eintrag nicht mehr vorhanden.', 'Entry no longer exists.'], local_storage: ['Lokale Modellliste konnte nicht gelesen oder gespeichert werden.', 'Unable to read or save the local model list.'],
};
const formatNames: Record<string, string> = { safetensors: 'Safetensors', gguf: 'GGUF', onnx: 'ONNX', pytorch: 'PyTorch / Checkpoint', 'safetensors-index': 'Safetensors · Index', 'pytorch-index': 'PyTorch · Index' };

export default function LocalModels({ language, showImage }: { language: Language; showImage: (path: string) => void }) {
  const de = language === 'de'; const text = (code: string) => labels[code]?.[de ? 0 : 1] ?? code;
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null); const [path, setPath] = useState('');
  const [error, setError] = useState(''); const [busy, setBusy] = useState(false); const locked = useRef(false);
  const alive = useRef(true); const generation = useRef(0);
  const [moving, setMoving] = useState(false), [moveModel, setMoveModel] = useState<LocalModel | null>(null);
  const running = snapshot?.scan.status === 'running' || moving;
  const [category, setCategory] = useState<ModelCategory>('models');
  const [query,setQuery]=useState('');
  const [page, setPage] = useState(0);
  const filtered = snapshot?.entries.filter(entry => inModelCategory(entry,category)&&(!query.trim()||[entry.name,entry.profile?.family??'',entry.format].join(' ').toLocaleLowerCase().includes(query.trim().toLocaleLowerCase()))) ?? [];
  const pages = Math.max(1, Math.ceil(filtered.length / 50));
  const currentPage = Math.min(page, pages - 1);
  const visible = filtered.slice(currentPage * 50, (currentPage + 1) * 50);
  async function refresh() {
    const current = ++generation.current;
    const next = await invoke<Snapshot>('model_library_list');
    if (alive.current && current === generation.current) { setSnapshot(next); }
  }
  useEffect(() => {
    alive.current = true; let loading = false;
    const poll = async () => { if (loading || locked.current) return; loading = true; try { await refresh(); } catch (e) { if (alive.current) setError(String(e)); } finally { loading = false; } };
    void poll(); const timer = setInterval(() => void poll(), 2000);
    return () => { alive.current = false; generation.current++; clearInterval(timer); };
  }, []);
  async function act(action: () => Promise<unknown>) {
    if (locked.current) return; locked.current = true; setBusy(true); setError(''); generation.current++;
    try { await action(); await refresh(); } catch (e) { if (alive.current) setError(String(e)); }
    finally { locked.current = false; if (alive.current) setBusy(false); }
  }
  async function choose() {
    const selected = await open({ directory: true, multiple: false, title: de ? 'Modellordner auswählen' : 'Choose model folder' });
    if (typeof selected === 'string' && alive.current) setPath(selected);
  }
  return <div className="local-model-library">
    <details className="model-maintenance"><summary>{de?'Wartung, Modellupdates und Benchmarks':'Maintenance, model updates and benchmarks'}</summary><Benchmarks language={language} /><ModelUpdates language={language} /></details>
    <ModelTransfer language={language} model={moveModel} dismiss={() => setMoveModel(null)} onBusy={setMoving} />
    <section className="panel local-import" aria-label={de ? 'Modelle vom PC einbinden' : 'Import models from PC'}>
      <h2>{de ? 'Modelle vom PC einbinden' : 'Import models from PC'}</h2>
      <p className="hub-hint">{de ? 'Durchsucht den gewählten Ordner mit Unterordnern. Dateien bleiben am Originalort; Modellcode wird nicht ausgeführt. Die Erkennung bestätigt noch keine Ausführbarkeit.' : 'Scans the selected folder and its subfolders. Files stay in place; model code is never executed. Detection does not confirm that a model can run.'}</p>
      <div className="local-full-scan">
        <button type="button" className="button secondary" disabled={busy || running} onClick={() => void act(() => invoke('model_scan_quick'))}><Search size={16} />{de ? 'Schnellsuche' : 'Quick scan'}</button>
        <button type="button" className="button primary" disabled={busy || running} onClick={() => void act(() => invoke('model_scan_full'))}><HardDrive size={16} />{de ? 'Vollständige Suche' : 'Full scan'}</button>
        <p className="hub-hint">{de ? 'Durchsucht alle lokalen Laufwerke mit Laufwerksbuchstaben nach Modellen. Kann länger dauern und jederzeit abgebrochen werden. Nicht lesbare Ordner werden übersprungen und aufgeführt.' : 'Searches all local drives with drive letters for models. May take a while; you can cancel at any time. Unreadable folders are skipped and reported.'}</p>
      </div>
      <form onSubmit={e => { e.preventDefault(); void act(() => invoke('model_scan_start', { path })); }}>
        <label className="field-label" htmlFor="local-model-path">{de ? 'Modellordner' : 'Model folder'}</label>
        <div className="local-import-controls"><input id="local-model-path" value={path} onChange={e => setPath(e.target.value)} disabled={busy || running} placeholder="D:\Modelle" /><button type="button" className="button secondary" onClick={() => void act(choose)} disabled={busy || running}><FolderOpen size={16} />{de ? 'Ordner wählen' : 'Choose folder'}</button><button className="button primary" disabled={busy || running || !path.trim()}><Search size={16} />{de ? 'Ordner durchsuchen' : 'Scan folder'}</button></div>
      </form>
      {error && <p className="notice warning" role="alert">{text(error)}</p>}
      {snapshot && snapshot.scan.status !== 'idle' && <div className="local-scan" role="status" data-testid="model-scan-status">
        <strong>{snapshot.scan.truncated && snapshot.scan.status === 'completed' ? (de ? 'Suche unvollständig · Grenze erreicht' : 'Scan incomplete · limit reached') : text(snapshot.scan.status)}</strong>
        {snapshot.scan.mode === 'quick' && <span>{de ? 'Schnellsuche' : 'Quick scan'} · {snapshot.scan.rootsFinished}/{snapshot.scan.roots.length} {de ? 'Orte bearbeitet' : 'locations processed'}</span>}
        {snapshot.scan.mode === 'full' && <span>{de ? 'Vollständige Suche' : 'Full scan'} · {snapshot.scan.rootsFinished}/{snapshot.scan.roots.length} {de ? 'Laufwerke bearbeitet' : 'drives processed'}: {snapshot.scan.roots.join(', ')}</span>}
        <span>{snapshot.scan.visited} {de ? 'Pfade geprüft' : 'paths checked'} · {snapshot.scan.found} {de ? 'Funde' : 'findings'} · {snapshot.scan.imported} {de ? 'Einträge übernommen' : 'entries imported'} · {snapshot.scan.skipped} {de ? 'übersprungen' : 'skipped'}</span>
        <p className="download-path">{snapshot.scan.root}</p>
        {running && <button className="button secondary" disabled={busy} onClick={() => void act(() => invoke('model_scan_cancel'))}><X size={15} />{de ? 'Suche abbrechen' : 'Cancel scan'}</button>}
        {snapshot.scan.truncated && <p className="notice warning">{de ? 'Suchgrenze erreicht. Die Ergebnisse können unvollständig sein. Wähle einen kleineren Unterordner.' : 'Scan limit reached. Results may be incomplete. Choose a smaller subfolder.'}</p>}
        {snapshot.scan.notes.length > 0 && <details><summary>{de ? 'Hinweise zu übersprungenen Pfaden (max. 50)' : 'Skipped path details (up to 50)'}</summary><ul>{snapshot.scan.notes.map((note, i) => <li key={i}><span>{text(note.code)}</span><p className="download-path">{note.path}</p></li>)}</ul></details>}
      </div>}
    </section>
    <div className="section-heading"><h2>{de ? 'Modelldateien auf diesem PC' : 'Model files on this PC'}</h2><button className="text-button" disabled={busy} onClick={() => void act(refresh)}><RefreshCw size={14} />{de ? 'Liste aktualisieren' : 'Refresh list'}</button></div>
    <nav className="model-categories" aria-label={de?'Modellbereiche':'Model categories'}>{modelCategories.map(group=><button type="button" key={group} aria-pressed={category===group} onClick={()=>{setCategory(group);setPage(0);}}><span>{categoryLabel(group,de)}</span><small>{snapshot?.entries.filter(entry=>inModelCategory(entry,group)).length??0}</small></button>)}</nav>
    <div className="model-library-search"><label className="field-label" htmlFor="model-library-query">{de?'In diesem Bereich suchen':'Search this category'}</label><input type="search" id="model-library-query" value={query} placeholder={de?'Name, Familie oder Dateiformat …':'Name, family or file format …'} onChange={e=>{setQuery(e.target.value);setPage(0);}}/><span>{filtered.length} {de?'Einträge':'entries'}</span></div>
    <p className="hub-hint">{de?'Modelle sind nach Einsatzzweck geordnet. LoRAs und ControlNet findest du unter Erweiterungen, Encoder und VAEs unter Technische Komponenten. Nicht eindeutig erkannte Dateien bleiben separat sichtbar.':'Models are grouped by purpose. LoRAs and ControlNet are under Extensions; encoders and VAEs are under Technical components. Unidentified files remain visible separately.'}</p>
    {category === 'excluded' && <p className="hub-hint">{de ? 'Treffer aus früheren Versionen bleiben hier zur Kontrolle erhalten. Originaldateien werden nicht verändert.' : 'Findings from previous versions are retained here for review. Original files are not modified.'}</p>}
    {filtered.length === 0 && (snapshot?.entries.length ?? 0) > 0 && <p className="hub-hint">{de ? 'Keine Treffer in dieser Kategorie.' : 'No findings in this category.'}</p>}
    {snapshot?.entries.length === 0 && <p className="hub-hint">{de ? 'Noch keine externen Modelldateien eingebunden. Wähle oben ihren Ordner.' : 'No external model files imported yet. Choose their folder above.'}</p>}
    <div className="local-model-entries">{visible.map(entry => <article className="panel local-model-entry" key={entry.id} data-model-id={entry.id}>
      <div className="section-heading"><div><h3>{entry.name}</h3><p className="model-size">{de ? 'Vorhandene Dateien' : 'Present files'}: <strong>{formatGigabytes(entry.totalBytes, language)}</strong> ({formatBytes(entry.totalBytes, language)})</p></div><span className="pill" data-testid="local-model-status">{text(entry.status)}</span></div>
      <div className="hub-tags"><span>{formatNames[entry.format]??entry.format}</span></div>
      <ModelClassification model={entry} de={de}/>
      <details className="model-file-details"><summary>{de?'Dateien und Erkennung':'Files and identification'}</summary>
      <p className="download-path">{displayPath(entry.path)}</p>
      <p className="hub-hint">{de ? (entry.completeness === 'index' ? 'Prüfumfang: Dateien aus dem Gewichtsindex. Weitere Runtime-Dateien können fehlen.' : entry.completeness === 'container' ? 'Geprüft: Safetensors-Header und Bytebereiche. Die Familien- und Bestandteilerkennung steht oben; Ausführbarkeit wird getrennt geprüft.' : 'Vollständigkeit unbekannt. Format-/Dateiname ist kein Nachweis eines vollständigen Modells.') : (entry.completeness === 'index' ? 'Scope: files listed in the weight index. Other runtime files may be missing.' : entry.completeness === 'container' ? 'Inspected: Safetensors header and byte ranges. Family and component identification is shown above; execution readiness is checked separately.' : 'Completeness unknown. The format or file name does not prove a complete model.')}</p>
      <p className="hub-hint">{de ? 'Dateierkennung bestätigt keine Ausführbarkeit. SDXL-Checkpoints werden bei Auswahl im Studio automatisch geprüft. Quelle: lokaler PC. Lizenz nicht geprüft.' : 'Detection does not confirm execution. SDXL checkpoints are checked automatically when selected in Studio. Source: local PC. License not verified.'}</p>
      <p className="hub-hint">{text(entry.discoveryReason)}</p>
      <small>{de ? 'Zuletzt geprüft' : 'Last inspected'}: {formatDate(entry.checkedAt, language)}</small>
      </details>
      <div className="hub-actions">{entry.format === 'safetensors' && entry.profile?.support === 'preflight' && entry.status === 'checked' && <button className="button primary" onClick={() => showImage(entry.path)}>{de ? 'Im Studio prüfen' : 'Check in Studio'}</button>}<button className="button secondary" disabled={busy || running} onClick={() => void act(() => invoke('model_library_recheck', { id: entry.id }))}><RefreshCw size={14} />{de ? 'Erneut prüfen' : 'Recheck'}</button><button className="text-button" disabled={busy || running} title={de ? 'Entfernt nur den Listeneintrag; die Dateien bleiben erhalten.' : 'Removes only the list entry; files are kept.'} onClick={() => void act(() => invoke('model_library_forget', { id: entry.id }))}>{de ? 'Aus Liste entfernen' : 'Remove from list'}</button></div>
      {entry.discovery === 'model' && <button className="button secondary" disabled={busy || running || !['checked', 'recognized'].includes(entry.status)} onClick={() => setMoveModel(entry)}>{de ? 'Dateien verschieben' : 'Move files'}</button>}
      {entry.discovery==='model'&&<ModelPrivacy path={entry.path} de={de}/>}<details><summary>{de ? 'Erfasste Dateien' : 'Recorded files'} ({entry.files.length})</summary><ul className="download-file-list">{entry.files.map(file => <li key={file.path}><span className="download-path">{displayPath(file.path)}</span><span>{formatGigabytes(file.size, language)}</span></li>)}</ul></details>
    </article>)}</div>
    {pages > 1 && <div className="hub-actions"><button className="button secondary" disabled={currentPage === 0} onClick={() => setPage(currentPage - 1)}>{de ? 'Zurück' : 'Previous'}</button><span>{de ? 'Seite' : 'Page'} {currentPage + 1} / {pages}</span><button className="button secondary" disabled={currentPage + 1 >= pages} onClick={() => setPage(currentPage + 1)}>{de ? 'Weiter' : 'Next'}</button></div>}
    <h2 className="local-download-heading">{de ? 'Über Local Studio heruntergeladen' : 'Downloaded through Local Studio'}</h2>
    <DownloadsPage language={language} localOnly />
  </div>;
}
