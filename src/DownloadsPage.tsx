import { useEffect, useRef, useState } from 'react';
import { confirm } from '@tauri-apps/plugin-dialog';
import { Download, Pause, Play, RefreshCw, ShieldCheck, X } from 'lucide-react';
import { activeDownload, downloads, type DownloadItem, type DownloadPlan } from './download-api';
import type { HfModelDetail } from './hub-types';
import type { Language } from './types';
import { formatBytes, formatGigabytes, totalFileBytes } from './helpers';
import { hubError } from './hub-i18n';
import './downloads.css';

const errors: Record<string, [string, string]> = {
  download_selection: ['Wähle mindestens eine Datei aus.', 'Select at least one file.'],
  download_unsafe_path: ['Ein Dateipfad ist für Windows unsicher oder verweist über eine Verknüpfung auf einen anderen Ort.', 'A Windows file path is unsafe or redirected through a link.'],
  download_unknown_size: ['Die Dateigröße ist unbekannt. Dieser Download kann noch nicht sicher geplant werden.', 'File size is unknown. This download cannot be planned safely yet.'],
  download_unknown_hash: ['Für eine Datei fehlt eine prüfbare Quell-Prüfsumme.', 'A file has no verifiable source checksum.'],
  download_space: ['Nicht genug freier Speicher für Download und geprüfte Zielkopie.', 'Not enough free space for the download and verified destination copy.'],
  download_storage: ['Dateien oder Download-Datenbank sind nicht zugänglich. Prüfe den Speicherort und freien Speicher.', 'Files or the download database are unavailable. Check the storage location and free space.'],
  download_redirect: ['Das Downloadziel gehört nicht zu den erlaubten Hugging-Face-Adressen.', 'The download target is outside the allowed Hugging Face addresses.'],
  download_http: ['Der Downloadserver hat die Datei nicht geliefert. Erneut versuchen.', 'The download server did not deliver the file. Retry the download.'],
  download_range: ['Der Server lieferte einen unerwarteten Dateibereich. Die Teil-Datei bleibt erhalten.', 'The server returned an unexpected file range. The partial file is preserved.'],
  download_size: ['Die Dateigröße stimmt nicht mit der gewählten Revision überein.', 'File size does not match the selected revision.'],
  download_hash: ['Prüfsumme stimmt nicht. Die Datei wird nicht als geprüft übernommen.', 'Checksum mismatch. The file will not be published as verified.'],
  download_resume_unsupported: ['Der Server unterstützt hier keine bytegenaue Fortsetzung. „Neu beginnen“ lädt ab Byte 0; die bisherigen Teile bleiben bis dahin erhalten.', 'The server does not support byte-accurate resume here. “Start over” downloads from byte 0; partial files are kept until then.'],
  download_recovered: ['Nach Neustart angehalten. Teil-Dateien bleiben erhalten; du kannst fortsetzen.', 'Paused after restart. Partial files are preserved; you can resume.'],
  download_duplicate: ['Diese Dateiauswahl ist bereits vorhanden oder in der Downloadliste.', 'This file selection already exists or is in the download list.'],
  download_plan_expired: ['Die Vorschau ist abgelaufen. Bitte erneut prüfen.', 'The preview expired. Prepare it again.'],
  download_destination_exists: ['Der Zielordner existiert bereits und wird nicht überschrieben.', 'The destination already exists and will not be overwritten.'],
  download_state: ['Diese Aktion ist im aktuellen Zustand nicht möglich. Liste aktualisieren.', 'This action is unavailable in the current state. Refresh the list.'],
};
function message(error: unknown, language: Language) { return errors[String(error)]?.[language === 'de' ? 0 : 1] ?? hubError(error, language); }
const statuses: Record<string, [string, string]> = {
  queued: ['Wartend', 'Queued'], downloading: ['Lädt herunter', 'Downloading'], verifying: ['Prüft Dateien', 'Verifying files'], installing: ['Übernimmt geprüfte Dateien', 'Publishing verified files'],
  completed: ['Lokal gespeichert', 'Stored locally'], paused: ['Pausiert', 'Paused'], cancelled: ['Abgebrochen · Teile behalten', 'Cancelled · partial files kept'],
  pausing: ['Wird pausiert', 'Pausing'], cancelling: ['Wird abgebrochen', 'Cancelling'], failed: ['Fehlgeschlagen', 'Failed'], invalid: ['Lokale Dateien fehlerhaft oder fehlen', 'Local files missing or invalid'],
};

export function DownloadSelection({ detail, language, onQueued }: { detail: HfModelDetail; language: Language; onQueued: () => void }) {
  const de = language === 'de';
  const [selected, setSelected] = useState<string[]>([]);
  const [plan, setPlan] = useState<DownloadPlan | null>(null);
  const [error, setError] = useState<unknown>('');
  const [busy, setBusy] = useState(false); const busyRef = useRef(false);
  const selectedSize = selected.length ? totalFileBytes(detail.files.filter(f => selected.includes(f.path))) : 0;
  async function prepare(start: boolean) {
    if (busyRef.current) return; busyRef.current = true; setBusy(true); setError('');
    try { if (start && plan) { await downloads.start(plan.id); onQueued(); } else { setPlan(await downloads.plan(detail.model.id, detail.revision, selected)); } }
    catch (value) { setError(value); } finally { busyRef.current = false; setBusy(false); }
  }
  return <section className="download-selection">
    <h3>{de ? 'Dateien auswählen' : 'Select files'} ({detail.files.length})</h3>
    <p className="hub-hint">{de ? 'Wähle die gewünschte Variante und ihre benötigten Dateien selbst. Es wird nichts vorausgewählt oder ausgeführt.' : 'Choose your variant and its required files. Nothing is preselected or executed.'}</p>
    <div className="hub-files"><table><thead><tr><th>{de ? 'Datei' : 'File'}</th><th>{de ? 'Größe' : 'Size'}</th></tr></thead><tbody>{detail.files.map(file => <tr key={file.path}><td><label className="download-file-choice"><input type="checkbox" aria-label={file.path} checked={selected.includes(file.path)} disabled={busy} onChange={e => { setPlan(null); setSelected(value => e.target.checked ? [...value, file.path] : value.filter(v => v !== file.path)); }} /><span>{file.path}{file.sha256 && <small>SHA-256: {file.sha256}</small>}</span></label></td><td>{file.size === null ? (de ? 'Unbekannt' : 'Unknown') : `${formatGigabytes(file.size, language)} (${formatBytes(file.size, language)})`}</td></tr>)}</tbody></table></div>
    <div className="download-selection-actions"><span>{selected.length} {de ? 'gewählt' : 'selected'} · {formatGigabytes(selectedSize, language)}</span><button className="button secondary" disabled={busy || !selected.length} onClick={() => void prepare(false)}><Download size={16} />{de ? 'Download prüfen' : 'Prepare download'}</button></div>
    {Boolean(error) && <p className="notice warning" role="alert">{message(error, language)}</p>}
    {plan && <div className="panel download-plan" data-testid="download-plan"><h3>{de ? 'Download-Vorschau' : 'Download preview'}</h3><dl>
      <dt>{de ? 'Dateiauswahl / Variante' : 'File selection / variant'}</dt><dd>{plan.download.files.map(f => f.path).join(', ')}</dd>
      <dt>Revision</dt><dd>{plan.download.revision}</dd><dt>{de ? 'Downloadgröße' : 'Download size'}</dt><dd>{formatGigabytes(plan.download.totalBytes, language)} ({formatBytes(plan.download.totalBytes, language)})</dd>
      <dt>{de ? 'Zusätzlicher Speicher während Übernahme' : 'Additional space during publication'}</dt><dd>{formatBytes(plan.additionalBytes, language)}</dd>
      <dt>{de ? 'Zielordner' : 'Destination'}</dt><dd>{plan.download.destination}</dd><dt>{de ? 'Lizenz' : 'License'}</dt><dd>{plan.download.license ?? (de ? 'Unbekannt – Model Card prüfen' : 'Unknown – check model card')}</dd>
      <dt>{de ? 'Zugang' : 'Access'}</dt><dd>{detail.model.gated || detail.model.private ? (de ? 'HF-Konto mit Freigabe erforderlich' : 'Authorized HF account required') : (de ? 'Öffentlich' : 'Public')}</dd>
      <dt>Runtime</dt><dd>{de ? 'Download möglich – Ausführung in Local Studio derzeit nicht unterstützt.' : 'Download available – execution in Local Studio is not supported yet.'}</dd>
    </dl><button className="button primary" disabled={busy} onClick={() => void prepare(true)}><Download size={17} />{de ? 'Auswahl herunterladen' : 'Download selection'}</button></div>}
  </section>;
}

export default function DownloadsPage({ language, localOnly = false }: { language: Language; localOnly?: boolean }) {
  const de = language === 'de'; const [items, setItems] = useState<DownloadItem[]>([]); const [error, setError] = useState<unknown>(''); const [busy, setBusy] = useState<string | null>(null);
  useEffect(() => {
    let closed = false, loading = false;
    const refresh = async () => { if (loading) return; loading = true; try { const next = await downloads.list(); if (!closed) setItems(next); } catch (value) { if (!closed) setError(value); } finally { loading = false; } };
    void refresh(); const timer = setInterval(() => void refresh(), 500); return () => { closed = true; clearInterval(timer); };
  }, []);
  async function act(item: DownloadItem, action: string, priority: number | null = null) {
    setBusy(item.id); setError('');
    try {
      if (action === 'restart' && !(await confirm(de ? 'Teil-Dateien dieses Downloads verwerfen und ab Byte 0 neu beginnen?' : 'Discard partial files for this download and start from byte 0?', { title: 'Local Studio', kind: 'warning' }))) return;
      await downloads.action(item.id, action, priority); setItems(await downloads.list());
    } catch (value) { setError(value); } finally { setBusy(null); }
  }
  const visible = localOnly ? items.filter(item => ['completed', 'invalid'].includes(item.status) || item.verifyOnly) : items;
  return <div className={localOnly ? 'local-downloads' : 'page downloads-page'}>
    {!localOnly && <header className="page-heading"><div><div className="eyebrow">LOCAL STUDIO · HUB</div><h1>Downloads</h1><p>{de ? 'Übertragungen, Teil-Dateien und geprüfte lokale Dateiauswahlen.' : 'Transfers, partial files and verified local file selections.'}</p></div></header>}
    {Boolean(error) && <div className="notice warning" role="alert">{message(error, language)}</div>}
    {visible.length === 0 && <div className="panel hub-empty"><Download size={27} /><p>{de ? (localOnly ? 'Noch keine lokal gespeicherten Modell-Dateien. Wähle Dateien in den HF-Modell-Details aus.' : 'Noch keine Downloads. Öffne unter Modelle eine Modell-Detailseite und wähle Dateien aus.') : (localOnly ? 'No model files stored locally yet. Select files in HF model details.' : 'No downloads yet. Open a model detail page and select files.')}</p></div>}
    <div className="download-list">{visible.map(item => {
      const remaining = Math.max(0, item.totalBytes - item.downloadedBytes); const active = activeDownload(item); const speed = item.status === 'downloading' ? item.bytesPerSecond : 0;
      return <article className="panel download-card" key={item.id} data-download-id={item.id}>
        <div className="section-heading"><div><h2>{item.repo}</h2><p className="model-size" data-testid="local-model-size">{de ? 'Dateiauswahl' : 'File selection'}: <strong>{formatGigabytes(item.totalBytes, language)}</strong></p><small>{item.task ?? (de ? 'Modell-Dateien' : 'Model files')} · {item.files.length} {de ? 'Dateien' : 'files'} · {item.license ?? (de ? 'Lizenz unbekannt' : 'Unknown license')}</small></div><span className="pill" data-testid="download-status">{statuses[item.status]?.[de ? 0 : 1] ?? item.status}</span></div>
        <progress max={Math.max(1, item.totalBytes)} value={item.downloadedBytes} aria-label={de ? 'Downloadfortschritt' : 'Download progress'} />
        <div className="download-metrics"><span>{formatBytes(item.downloadedBytes, language)} / {formatBytes(item.totalBytes, language)}</span><span>{de ? 'Rest' : 'Remaining'}: {formatBytes(remaining, language)}</span>{speed > 0 && <><span>{formatBytes(speed, language)}/s</span><span>≈ {Math.ceil(remaining / speed)} s</span></>}</div>
        <p className="download-path">{item.destination}</p><p className="hub-hint">Revision: {item.revision}</p>
        {item.error && <p className="notice warning">{message(item.error, language)}</p>}
        <div className="hub-actions">
          {active && !['pausing', 'cancelling'].includes(item.status) && <button className="button secondary" disabled={busy === item.id} onClick={() => void act(item, 'pause')}><Pause size={15} />{de ? 'Pausieren' : 'Pause'}</button>}
          {['paused', 'failed', 'cancelled'].includes(item.status) && <button className="button secondary" disabled={busy === item.id} onClick={() => void act(item, item.status === 'failed' ? 'retry' : 'resume')}><Play size={15} />{item.status === 'failed' ? (de ? 'Erneut versuchen' : 'Retry') : (de ? 'Fortsetzen' : 'Resume')}</button>}
          {active && !['pausing', 'cancelling'].includes(item.status) && <button className="button secondary" disabled={busy === item.id} onClick={() => void act(item, 'cancel')}><X size={15} />{de ? 'Abbrechen' : 'Cancel'}</button>}
          {!item.verifyOnly && ['failed', 'paused', 'cancelled'].includes(item.status) && <button className="text-button" disabled={busy === item.id} onClick={() => void act(item, 'restart')}><RefreshCw size={14} />{de ? 'Neu beginnen' : 'Start over'}</button>}
          {['completed', 'invalid'].includes(item.status) && <button className="button secondary" disabled={busy === item.id} onClick={() => void act(item, 'verify')}><ShieldCheck size={15} />{de ? 'Lokale Dateien prüfen' : 'Verify local files'}</button>}
          {!localOnly && item.status !== 'completed' && <label className="download-priority">{de ? 'Priorität' : 'Priority'}<select value={item.priority} disabled={busy === item.id} onChange={event => void act(item, 'priority', Number(event.target.value))}><option value={0}>{de ? 'Normal' : 'Normal'}</option><option value={1}>{de ? 'Hoch' : 'High'}</option><option value={2}>{de ? 'Sehr hoch' : 'Highest'}</option></select></label>}
        </div>
        {localOnly && <><p className="hub-hint">{de ? 'Dateiauswahl gespeichert. Vollständigkeit für eine Runtime und Ausführbarkeit sind noch nicht getestet.' : 'File selection stored. Runtime completeness and execution have not been tested.'}</p><details><summary>{de ? 'Dateien und Prüfsummen' : 'Files and checksums'}</summary><ul className="download-file-list">{item.files.map(file => <li key={file.path}><strong>{file.path}</strong><span>{formatGigabytes(file.size, language)} ({formatBytes(file.size, language)})</span><code>SHA-256: {file.actualSha256 ?? '—'}</code></li>)}</ul></details></>}
      </article>;
    })}</div>
  </div>;
}
