import { Image, X } from 'lucide-react';
import { activeImage, imageError, type ImageJob } from './image-api';
import { fileName, formatDate } from './helpers';
import type { Language } from './types';
export const imagePhases: Record<string, [string, string]> = {
  queued: ['Wartet', 'Queued'], paused: ['Pausiert', 'Paused'],
  hashing: ['Modelldatei prüfen', 'Verifying model file'], loading: ['Modell laden', 'Loading model'], sampling: ['Bild generieren', 'Generating image'], decoding: ['Bild dekodieren', 'Decoding image'],
  completed: ['Fertig · Modell entladen', 'Completed · model unloaded'], failed: ['Fehlgeschlagen', 'Failed'], cancelled: ['Abgebrochen', 'Cancelled'], interrupted: ['Unterbrochen', 'Interrupted'],
};
export default function ImageJobRow({ job, language, onOpen, onCancel, onResume, pending }: { job: ImageJob; language: Language; onOpen: () => void; onCancel: () => void; onResume: () => void; pending: boolean }) {
  const de = language === 'de'; const running = job.status === 'running';
  return <article className="job-row" data-image-job-id={job.id}>
    <div className={`job-icon status-${job.status}`}><Image size={19} /></div>
    <div className="job-content"><div className="job-heading"><strong title={job.request.modelPath}>{fileName(job.request.modelPath)}</strong><span className={`status status-${job.status}`}>{imagePhases[job.phase]?.[de ? 0 : 1] ?? job.phase}</span></div>
      <div className="job-meta">Image · {formatDate(job.createdAt, language)} · {job.request.width} × {job.request.height} · {(job.elapsedMs / 1000).toFixed(1)} s</div>
      {job.status === 'queued' && <p className="hub-hint">{de ? `Warteschlange · Position ${job.queuePosition ?? '…'}` : `Queue · position ${job.queuePosition ?? '…'}`}</p>}
      {running && <div className="progress-line"><progress aria-label="Image job progress" max={job.phase === 'hashing' ? job.modelBytes : job.request.steps} value={job.phase === 'hashing' ? job.hashedBytes : job.phase === 'sampling' ? job.step : undefined} />{job.phase === 'sampling' && <small>{job.step} / {job.request.steps}</small>}</div>}
      {job.error && <p className="job-error">{imageError(job.error, de)}</p>}
      {job.discarded && <p className="hub-hint">{de ? 'Ergebnis verworfen' : 'Result discarded'}</p>}
      <button className="text-button" onClick={onOpen}>{de ? 'Im Studio öffnen' : 'Open in Studio'}</button>
      <details className="job-details"><summary>{de ? 'Auftragsdetails' : 'Job details'}</summary><p>{job.request.prompt}</p><p>{job.request.steps} steps · CFG {job.request.guidance} · Seed {job.request.seed} · {job.request.sampler}</p><code>{job.modelSha256}</code></details>
    </div>
    {job.status === 'paused' && <button className="button secondary" aria-label={de ? 'Bildauftrag erneut einreihen' : 'Resume image job'} disabled={pending} onClick={onResume}>{de ? 'Erneut einreihen' : 'Resume'}</button>}
    {(activeImage(job) || job.status === 'paused') && <button className="icon-button" aria-label={de ? 'Bildauftrag abbrechen' : 'Cancel image job'} disabled={pending} onClick={onCancel}><X size={16} /></button>}
  </article>;
}
