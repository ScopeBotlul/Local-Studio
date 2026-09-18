import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { formatGigabytes } from './helpers';
import type { LocalModel } from './LocalModels';
import type { Language } from './types';

interface MovePlan { id: string; model: LocalModel; destination: string; totalBytes: number; files: { source: string; destination: string; bytes: number }[]; }
interface MoveJob { id: string; name: string; source: string; destination: string; status: string; phase: string; totalBytes: number; copiedBytes: number; error: string | null; }
const messages: Record<string, [string, string]> = {
  move_recheck: ['Die Dateien zuerst erneut prüfen.', 'Recheck the files first.'],
  move_in_use: ['Das Modell wird verwendet. Zuerst entladen und laufende Aufträge beenden.', 'The model is in use. Unload it and finish active jobs first.'],
  move_space: ['Am Ziel ist nicht genug freier Speicher.', 'Not enough free space at the destination.'],
  move_changed: ['Eine Quelldatei wurde verändert. Erneut prüfen.', 'A source file changed. Recheck it.'],
  move_verify: ['Die Kopie stimmt nicht mit dem Original überein. Die Originale bleiben erhalten.', 'The copy differs from the original. Originals are retained.'],
  move_collision: ['Der Zielordner existiert bereits oder lässt sich nicht anlegen. Ein anderes Ziel wählen.', 'The destination exists or cannot be created. Choose another destination.'],
  move_expired: ['Die Vorschau ist abgelaufen. Das Ziel erneut auswählen.', 'The preview expired. Select the destination again.'],
  move_layout: ['Diese Dateianordnung lässt sich noch nicht gemeinsam verschieben.', 'Moving this file layout together is not yet supported.'],
  local_link: ['Verknüpfte Cache-Dateien zuerst in einen eigenen Modellordner kopieren.', 'Copy linked cache files into a separate model folder first.'],
  local_busy: ['Ein Suchlauf oder eine Modellverschiebung läuft bereits.', 'A scan or model move is already running.'],
  move_interrupted: ['Die Verschiebung wurde unterbrochen. Originale und vorhandene geprüfte Kopien am angegebenen Ziel kontrollieren, bevor du erneut verschiebst.', 'The move was interrupted. Check originals and any verified copies at the destination before trying again.'],
  move_original_retained: ['Die geprüfte Kopie ist eingebunden. Mindestens eine Originaldatei konnte nicht entfernt werden.', 'The verified copy is registered. At least one original could not be removed.'],
  move_ai_changed: ['Die KI-Registrierung stimmt nicht mehr mit dem Modell überein. Originale und geprüfte Kopien bleiben erhalten.', 'The AI registry no longer matches the model. Originals and verified copies are retained.'],
};
const phases: Record<string, [string, string]> = { checking: ['Quelle prüfen', 'Checking source'], copying: ['Kopieren', 'Copying'], verifying: ['Kopie prüfen', 'Verifying copy'], committing: ['Speicherort übernehmen', 'Updating location'], removing_originals: ['Originale entfernen', 'Removing originals'], completed: ['Verschoben', 'Moved'], cancelled: ['Abgebrochen', 'Cancelled'], failed: ['Fehlgeschlagen', 'Failed'], interrupted: ['Unterbrochen', 'Interrupted'] };

export default function ModelTransfer({ language, model, dismiss, onBusy }: { language: Language; model: LocalModel | null; dismiss: () => void; onBusy: (value: boolean) => void }) {
  const de = language === 'de';
  const [plan, setPlan] = useState<MovePlan | null>(null), [job, setJob] = useState<MoveJob | null>(null);
  const [busy, setBusy] = useState(false), [error, setError] = useState('');
  const text = (value: string) => messages[value]?.[de ? 0 : 1] ?? value;
  useEffect(() => { let alive = true, loading = false; const poll = async () => { if (loading) return; loading = true; try { const value = await invoke<MoveJob | null>('model_move_status'); if (alive) { setJob(value); onBusy(value?.status === 'running'); } } catch (e) { if (alive) setError(String(e)); } finally { loading = false; } }; void poll(); const timer = setInterval(() => void poll(), 700); return () => { alive = false; clearInterval(timer); }; }, [onBusy]);
  useEffect(() => { setPlan(null); setError(''); }, [model?.id]);
  async function choose() { setBusy(true); setError(''); try { const destination = await open({ directory: true, title: de ? 'Neuen Modellordner wählen' : 'Choose new model folder' }); if (typeof destination === 'string' && model) setPlan(await invoke<MovePlan>('model_move_plan', { id: model.id, destination })); } catch (e) { setError(String(e)); } finally { setBusy(false); } }
  async function start() { if (!plan) return; setBusy(true); setError(''); try { setJob(await invoke<MoveJob>('model_move_start', { planId: plan.id, confirmed: true })); onBusy(true); setPlan(null); dismiss(); } catch (e) { setError(String(e)); } finally { setBusy(false); } }
  return <>
    {job && <section className="panel" aria-label={de ? 'Modellverschiebung' : 'Model move'} data-testid="model-move-status"><strong>{job.name} · {phases[job.status === 'running' ? job.phase : job.status]?.[de ? 0 : 1] ?? job.status}</strong><p className="download-path">{job.destination}</p><p>{formatGigabytes(job.copiedBytes, language)} / {formatGigabytes(job.totalBytes, language)}</p>{job.status === 'running' && <><progress max={job.totalBytes || 1} value={job.copiedBytes} /><button className="button secondary" disabled={['committing', 'removing_originals'].includes(job.phase)} onClick={() => void invoke('model_move_cancel').catch(e => setError(String(e)))}>{de ? 'Abbrechen' : 'Cancel'}</button></>}{job.error && job.error !== 'move_cancelled' && <p className="notice warning">{text(job.error)}</p>}</section>}
    {error && !model && <p role="alert" className="notice warning">{text(error)}</p>}
    {model && <section className="panel" aria-label={de ? 'Verschieben vorbereiten' : 'Prepare move'}>
      <h3>{de ? 'Modelldateien verschieben' : 'Move model files'} · {model.name}</h3>
      <p>{de ? 'Erfasste Dateien werden kopiert und mit SHA-256 geprüft. Erst danach wird die Modellliste umgestellt und das Original entfernt. Nicht erfasste Dateien bleiben am bisherigen Ort. Bestehende Studio- und Projektparameter behalten ihren bisherigen Modellpfad; wähle das Modell dort anschließend am neuen Ort aus.' : 'Recorded files are copied and verified with SHA-256 before the model list is updated and originals are removed. Unrecorded files stay in place. Existing Studio and project parameters retain the previous model path; select the new location there afterwards.'}</p>
      <p className="download-path">{model.path}</p>
      <button className="button secondary" disabled={busy || job?.status === 'running'} onClick={() => void choose()}>{de ? 'Zielordner wählen' : 'Choose destination'}</button>
      {plan && <><p className="download-path">{plan.destination}</p><p>{plan.files.length} {de ? 'Dateien' : 'files'} · {formatGigabytes(plan.totalBytes, language)}</p><details><summary>{de ? 'Dateien und Zielpfade prüfen' : 'Review files and destinations'}</summary><ul>{plan.files.map(f => <li key={f.source}><p className="download-path">{f.source} → {f.destination}</p>{formatGigabytes(f.bytes, language)}</li>)}</ul></details><button className="button primary" disabled={busy} onClick={() => void start()}>{de ? 'Geprüft verschieben' : 'Move with verification'}</button></>}
      {error && <p className="notice warning" role="alert">{text(error)}</p>}
      <button className="text-button" disabled={busy} onClick={dismiss}>{de ? 'Schließen' : 'Close'}</button>
    </section>}
  </>;
}
