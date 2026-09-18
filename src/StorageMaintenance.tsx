import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { confirm } from '@tauri-apps/plugin-dialog';
import { formatBytes } from './helpers';
import type { Settings } from './types';
interface Preview { token: string; files: { category: string; owner: string; path: string; bytes: number }[]; bytes: number; protectedBytes: number; unavailable: number; retentionDays: number }
interface Report { deleted: number; bytes: number; skipped: number; errors: string[] }
export default function StorageMaintenance({ settings, de, disabled, dirty, onChange }: { settings: Settings; de: boolean; disabled: boolean; dirty: boolean; onChange: (patch: Partial<Settings>) => void }) {
  const [preview, setPreview] = useState<Preview | null>(null); const [report, setReport] = useState<Report | null>(null);
  const [busy, setBusy] = useState(false); const [error, setError] = useState('');
  async function inspect() {
    setBusy(true); setError(''); setReport(null);
    try { setPreview(await invoke<Preview>('storage_cleanup_preview')); }
    catch (e) { setError(String(e)); } finally { setBusy(false); }
  }
  async function clean() {
    if (!preview) return;
    setBusy(true); setError('');
    try {
      if (!await confirm(de ? `${preview.files.length} geprüfte temporäre Dateien (${formatBytes(preview.bytes, 'de')}) endgültig löschen?` : `Permanently delete ${preview.files.length} verified temporary files (${formatBytes(preview.bytes, 'en')})?`, { title: de ? 'Temporäre Dateien bereinigen' : 'Clean temporary files', kind: 'warning' })) return;
      setReport(await invoke<Report>('storage_cleanup_apply', { token: preview.token, confirmed: true })); setPreview(null);
    } catch (e) { setError(String(e)); setPreview(null); } finally { setBusy(false); }
  }
  return <section className="panel settings-section storage-cleanup">
    <h2>{de ? 'Speicher und Bereinigung' : 'Storage and cleanup'}</h2>
    <p className="hub-hint">{de ? 'Erfasst bekannte Projektarbeitskopien, Bild-/Video-Renderdateien, Proxies und Wellenformen. Aktive Projekte, die letzten drei Wiederherstellungsstände und ungespeicherte Bilder sowie Cache-Dateien des aktiven Projekts bleiben geschützt. Modelle, Galerieoriginale, Downloads und unbekannte Dateien werden nicht gelöscht.' : 'Covers registered project working copies, image/video render files, proxies and waveforms. Active projects, the latest three recovery points and unsaved images and the active project’s media cache stay protected. Models, gallery originals, downloads and unknown files are never deleted.'}</p>
    <label className="project-message"><input type="checkbox" checked={settings.autoCleanup} disabled={disabled || busy} onChange={e => onChange({ autoCleanup: e.target.checked })} />{de ? 'Alte, nicht mehr benötigte Arbeitsdateien automatisch bereinigen' : 'Automatically clean old, unneeded working files'}</label>
    <div className="project-message"><label htmlFor="temp-retention">{de ? 'Aufbewahrung in Tagen' : 'Retention in days'}</label><input id="temp-retention" type="number" min={1} max={365} value={settings.tempRetentionDays} disabled={disabled || busy} onChange={e => onChange({ tempRetentionDays: Number(e.target.value) })} /></div>
    <div className="project-message"><label htmlFor="project-undo-limit">{de ? 'Entfernte Projektmedien zum Zurückholen behalten' : 'Keep removed project media for restoration'}</label><input id="project-undo-limit" type="number" min={1} max={1000} value={settings.maxUndo} disabled={disabled || busy} onChange={e => onChange({ maxUndo: Number(e.target.value) })} /></div>
    <p className="hub-hint">{de ? 'Automatisch kurz nach dem Start und danach stündlich, solange die App läuft. Einstellungen erst mit „Änderungen speichern“ übernehmen.' : 'Automatically shortly after startup, then hourly while the app is running. Apply settings with “Save changes” first.'}</p>
    <button type="button" className="button secondary" disabled={disabled || dirty || busy} onClick={() => void inspect()}>{busy ? (de ? 'Speicher wird geprüft …' : 'Checking storage …') : (de ? 'Speicher prüfen' : 'Inspect storage')}</button>
    {error && <p role="alert">{de ? 'Bereinigung nicht ausgeführt. Bitte erneut prüfen.' : 'Cleanup was not completed. Inspect storage again.'} ({error})</p>}
    {preview && <><div className="storage-cleanup-stats"><span><strong>{formatBytes(preview.bytes, de ? 'de' : 'en')}</strong>{de ? `freigebbar · ${preview.files.length} Dateien` : `reclaimable · ${preview.files.length} files`}</span><span><strong>{formatBytes(preview.protectedBytes, de ? 'de' : 'en')}</strong>{de ? 'geschützt / noch benötigt' : 'protected / still needed'}</span></div><p className="hub-hint">{de ? `${preview.retentionDays} Tage Frist. ${preview.unavailable} geänderte oder nicht prüfbare Dateien übersprungen. Maximal 1.000 Dateien pro Durchlauf.` : `${preview.retentionDays}-day retention. ${preview.unavailable} changed or unavailable files skipped. Up to 1,000 files per batch.`}</p><details><summary>{de ? 'Betroffene Dateien ansehen' : 'Review affected files'}</summary><ul className="storage-cleanup-list">{preview.files.map(f => <li key={f.path}>{f.path} · {formatBytes(f.bytes, de ? 'de' : 'en')}</li>)}</ul></details><button type="button" className="button secondary" disabled={disabled || dirty || busy || !preview.files.length} onClick={() => void clean()}>{de ? 'Jetzt bereinigen …' : 'Clean now …'}</button></>}
    {report && <p role="status">{de ? `${report.deleted} Dateien gelöscht, ${formatBytes(report.bytes, 'de')} freigegeben. ${report.skipped} Dateien blieben erhalten.` : `${report.deleted} files deleted, ${formatBytes(report.bytes, 'en')} freed. ${report.skipped} files retained.`}</p>}
    {!!report?.errors.length && <details><summary>{de ? 'Details' : 'Details'}</summary><ul className="storage-cleanup-list">{report.errors.map((e, i) => <li key={i}>{e}</li>)}</ul></details>}
  </section>;
}
