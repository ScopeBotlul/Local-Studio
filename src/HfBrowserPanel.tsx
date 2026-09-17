import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ArrowLeft, ArrowRight, ExternalLink, Home, LoaderCircle, RefreshCw, ShieldCheck, X } from 'lucide-react';
import type { Language } from './types';
import { hubError } from './hub-i18n';

type Bounds = { x: number; y: number; width: number; height: number };
type BrowserState = { url: string; title: string; loading: boolean; notice: string | null; modelRepo: string | null; visible: boolean };
type Action = { kind: 'home' | 'back' | 'forward' | 'reload' | 'openExternal' | 'dismissNotice' } | { kind: 'visit'; url: string };
// Serialize mount/layout/hide across React remounts, not just one component instance.
let nativeQueue: Promise<unknown> = Promise.resolve();
function mutate<T>(work: () => Promise<T>): Promise<T> {
  const result = nativeQueue.then(work); nativeQueue = result.catch(() => undefined); return result;
}

const copy = {
  de: { address: 'Hugging-Face-Adresse', go: 'Öffnen', back: 'Zurück', forward: 'Vorwärts', home: 'Hugging-Face-Startseite', reload: 'Website neu laden', external: 'Im Standardbrowser öffnen', model: 'In Local Studio öffnen', hint: 'Die echte Hugging-Face-Website. Ihre Anmeldung wird separat gespeichert; dein App-Konto bleibt unverändert.', loading: 'Website wird geladen …', retry: 'Erneut öffnen', blocked: 'Diese Adresse kann hier nicht geöffnet werden. Im Studio sind nur HTTPS-Seiten von Hugging Face erlaubt.', download: 'Dateidownloads im integrierten Browser sind noch deaktiviert. Öffne die Seite im Standardbrowser, wenn du eine Datei dort herunterladen möchtest.', opened: 'Externer Link im Standardbrowser geöffnet.', close: 'Hinweis schließen' },
  en: { address: 'Hugging Face address', go: 'Go', back: 'Back', forward: 'Forward', home: 'Hugging Face home', reload: 'Reload website', external: 'Open in default browser', model: 'Open in Local Studio', hint: 'The real Hugging Face website. Its sign-in is saved separately; your app account stays unchanged.', loading: 'Loading website …', retry: 'Open again', blocked: 'This address cannot be opened here. Only HTTPS Hugging Face pages are allowed inside Studio.', download: 'File downloads in the embedded browser are not enabled yet. Open the page in your default browser to download a file there.', opened: 'External link opened in your default browser.', close: 'Dismiss notice' },
};

export default function HfBrowserPanel({ language, onModel, signingIn = false }: { language: Language; signingIn?: boolean; onModel: (repo: string) => void }) {
  const t = copy[language];
  const host = useRef<HTMLDivElement>(null);
  const owner = useRef(crypto.randomUUID());
  const [state, setState] = useState<BrowserState | null>(null);
  const [address, setAddress] = useState('https://huggingface.co/models');
  const editing = useRef(false);
  const [error, setError] = useState<unknown>('');
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    let closed = false, mounted = false, frame = 0;
    let previous = '';
    const id = owner.current;
    const update = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        if (closed || !host.current) return;
        const initial = host.current.getBoundingClientRect();
        const zoom = Number(getComputedStyle(document.documentElement).getPropertyValue('--ui-scale')) || 1;
        const available = window.innerHeight - initial.top - 38;
        if (available < 32 || initial.width < 32) return;
        host.current.style.height = `${available / zoom}px`;
        const rect = host.current.getBoundingClientRect();
        const bounds: Bounds = { x: rect.left, y: rect.top, width: rect.width, height: Math.min(rect.height, available) };
        const serialized = JSON.stringify(bounds);
        if (serialized === previous) return;
        previous = serialized;
        void mutate(async () => {
          if (closed) return;
          if (!mounted) {
            const next = await invoke<BrowserState>('hf_browser_mount', { owner: id, bounds });
            mounted = true;
            if (!closed) { setState(next); setError(''); }
          } else { await invoke('hf_browser_layout', { owner: id, bounds }); }
        }).catch(value => { previous = ''; if (!closed) setError(value); });
      });
    };
    const observer = new ResizeObserver(update);
    if (host.current) { observer.observe(host.current); if (host.current.parentElement) observer.observe(host.current.parentElement); }
    const styles = new MutationObserver(update);
    styles.observe(document.documentElement, { attributes: true, attributeFilter: ['style'] });
    window.addEventListener('resize', update);
    window.addEventListener('scroll', update, true);
    update();
    let reading = false;
    const timer = setInterval(() => {
      if (!mounted || reading) return;
      reading = true;
      void invoke<BrowserState>('hf_browser_state').then(next => { if (!closed) { setState(next); if (!editing.current) setAddress(next.url); } })
        .catch(value => { if (!closed) setError(value); }).finally(() => { reading = false; });
    }, 700);
    return () => {
      closed = true; cancelAnimationFrame(frame); clearInterval(timer); observer.disconnect(); styles.disconnect();
      window.removeEventListener('resize', update); window.removeEventListener('scroll', update, true);
      void mutate(() => invoke('hf_browser_hide', { owner: id })).catch(() => undefined);
    };
  }, [attempt]);

  async function action(action: Action) {
    setError('');
    try { await mutate(() => invoke('hf_browser_action', { owner: owner.current, action })); }
    catch (value) { setError(value); }
  }
  const browserError = (value: unknown) => value === 'browser_hf_only' || value === 'browser_blocked' ? t.blocked : value === 'browser_download' ? t.download : value === 'browser_external' ? t.opened : hubError(value, language);
  return <section className="hf-browser-panel" aria-label={language === 'de' ? 'Integrierter Hugging-Face-Browser' : 'Embedded Hugging Face browser'}>
    <div className="hf-browser-toolbar">
      <button className="icon-button" title={t.back} aria-label={t.back} disabled={!state} onClick={() => void action({ kind: 'back' })}><ArrowLeft size={17} /></button>
      <button className="icon-button" title={t.forward} aria-label={t.forward} disabled={!state} onClick={() => void action({ kind: 'forward' })}><ArrowRight size={17} /></button>
      <button className="icon-button" title={t.reload} aria-label={t.reload} disabled={!state} onClick={() => void action({ kind: 'reload' })}><RefreshCw size={16} className={state?.loading ? 'spin' : ''} /></button>
      <button className="icon-button" title={t.home} aria-label={t.home} disabled={!state} onClick={() => void action({ kind: 'home' })}><Home size={16} /></button>
      <form onSubmit={event => { event.preventDefault(); editing.current = false; void action({ kind: 'visit', url: address }); }}><input aria-label={t.address} value={address} spellCheck={false} onFocus={() => { editing.current = true; }} onBlur={() => { editing.current = false; }} onChange={event => setAddress(event.target.value)} /><button className="button secondary" disabled={!state}>{t.go}</button></form>
      <button className="icon-button" title={t.external} aria-label={t.external} disabled={!state} onClick={() => void action({ kind: 'openExternal' })}><ExternalLink size={17} /></button>
      {state?.modelRepo && <button className="button secondary hf-open-model" onClick={() => onModel(state.modelRepo!)}>{t.model}<ArrowRight size={15} /></button>}
    </div>
    <p className="hf-browser-hint"><ShieldCheck size={14} />{signingIn ? (language === 'de' ? 'Melde dich bei Hugging Face an und erlaube Local Studio den Zugriff auf dein App-Konto.' : 'Sign in to Hugging Face and authorize Local Studio to connect your app account.') : t.hint}</p>
    {Boolean(error || state?.notice) && <div className="notice warning hf-browser-notice" role="status"><span>{browserError(error || state?.notice)}</span><button className="icon-button" aria-label={t.close} title={t.close} onClick={() => { setError(''); void action({ kind: 'dismissNotice' }); }}><X size={15} /></button>{Boolean(error) && <button className="text-button" onClick={() => { setError(''); setAttempt(value => value + 1); }}>{t.retry}</button>}</div>}
    <div ref={host} className="hf-browser-surface" data-testid="hf-browser-surface">{!state && <span><LoaderCircle size={18} className="spin" />{t.loading}</span>}</div>
  </section>;
}
