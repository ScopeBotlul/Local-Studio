import { DownloadSelection } from './DownloadsPage';
import LocalModels from './LocalModels';
import HfBrowserPanel from './HfBrowserPanel';
import { useEffect, useRef, useState } from 'react';
import { ArrowRight, Box, CheckCircle2, ExternalLink, LoaderCircle, Search, ShieldCheck, UserRound, X } from 'lucide-react';
import { hubApi } from './hub-api';
import { hubError, hubText } from './hub-i18n';
import type { HfAuthStatus, HfModel, HfModelDetail, HfQuery } from './hub-types';
import { formatGigabytes, totalFileBytes } from './helpers';
import type { Language } from './types';

export default function HubPage({ language, mode, showModels, initialRepo, showDownloads, showImage }: { language: Language; mode: 'hub' | 'models'; showModels: (repo?: string) => void; initialRepo?: string | null; showDownloads: () => void; showImage: (path: string) => void }) {
  const t = hubText(language);
  const [website, setWebsite] = useState(false);
  const [localModels, setLocalModels] = useState(false);
  const [auth, setAuth] = useState<HfAuthStatus | null>(null);
  const [error, setError] = useState<unknown>('');
  const [busy, setBusy] = useState(false);
  const [token, setToken] = useState('');
  const [query, setQuery] = useState<HfQuery>({ search: '', task: '', sort: 'downloads', cursor: null });
  const [models, setModels] = useState<HfModel[] | null>(null);
  const [sizes, setSizes] = useState<Record<string, number | null>>({});
  const [cursor, setCursor] = useState<string | null>(null);
  const [onlyExecutable, setOnlyExecutable] = useState(false);
  const [detail, setDetail] = useState<HfModelDetail | null>(null);
  const [revision, setRevision] = useState('main');
  // Keep search responsive; fetch at most three metadata responses at a time.
  // Cancel queued work on navigation/query changes and discard late responses.
  useEffect(() => {
    if (mode !== 'models' || localModels || detail || !models) return;
    let cancelled = false;
    const queue = models.filter(model => sizes[`${model.id}@${model.revision}`] === undefined);
    async function worker() {
      while (!cancelled && queue.length) {
        const model = queue.shift()!;
        let bytes: number | null = null;
        try { bytes = await hubApi.size(model.id, model.revision ?? 'main'); } catch { /* Unknown is honest on access/network failures. */ }
        if (!cancelled) setSizes(previous => ({ ...previous, [`${model.id}@${model.revision}`]: bytes }));
      }
    }
    void Promise.all([worker(), worker(), worker()]);
    return () => { cancelled = true; };
  }, [models, mode, localModels, detail]);
  const alive = useRef(true);
  const busyRef = useRef(false);
  const loginWasPending = useRef(false);
  useEffect(() => {
    if (auth?.pending) { loginWasPending.current = true; }
    else if (auth && loginWasPending.current) { loginWasPending.current = false; setWebsite(false); }
  }, [auth?.pending]);

  useEffect(() => {
    alive.current = true;
    let cancelled = false;
    const refresh = () => hubApi.status().then(status => { if (!cancelled) setAuth(status); }).catch(value => { if (!cancelled) setError(value); });
    void refresh();
    // This reads local state only; it never calls Hugging Face.
    const timer = setInterval(() => { void refresh(); }, 1000);
    return () => { cancelled = true; alive.current = false; clearInterval(timer); };
  }, []);

  async function run(action: () => Promise<void>) {
    if (busyRef.current) return;
    busyRef.current = true; setBusy(true); setError('');
    try { await action(); }
    catch (value) { if (alive.current) setError(value); }
    finally { busyRef.current = false; if (alive.current) setBusy(false); }
  }
  async function account(action: () => Promise<HfAuthStatus>) {
    await run(async () => { const status = await action(); if (alive.current) { setAuth(status); setModels(null); setSizes({}); setDetail(null); setCursor(null); } });
  }
  function editQuery(key: 'search' | 'task' | 'sort', value: string) {
    setQuery(previous => ({ ...previous, [key]: value, cursor: null })); setModels(null); setCursor(null); setDetail(null);
  }
  async function search(more = false) {
    if (onlyExecutable) return;
    await run(async () => {
      const results = await hubApi.search({ ...query, cursor: more ? cursor : null });
      if (!alive.current) return;
      if (!more) setSizes({});
      setModels(previous => more ? Array.from(new Map([...(previous ?? []), ...results.models].map(model => [model.id, model])).values()) : results.models);
      setCursor(results.nextCursor); setDetail(null);
    });
  }
  async function inspect(repo: string, selectedRevision = 'main') {
    await run(async () => {
      const result = await hubApi.detail(repo, selectedRevision);
      if (alive.current) { setDetail(result); setRevision(selectedRevision); }
    });
  }
  useEffect(() => { if (initialRepo) void inspect(initialRepo); }, [initialRepo]);
  const open = (page: 'tokens' | 'applications' | 'home' | 'model', repo?: string) => void run(() => hubApi.open(page, repo));
  const errorCode = error || auth?.error;
  const working = <LoaderCircle size={16} className="spin" />;

  return <div className="page hub-page">
    <header className="page-heading"><div><div className="eyebrow">HUGGING FACE · HUB</div><h1>{mode === 'hub' ? (website ? 'Hugging Face' : t.account) : t.models}</h1><p>{mode === 'hub' ? (website ? (auth?.pending ? t.pending : t.websiteHint) : t.intro) : t.modelsIntro}</p></div><span className="pill"><ShieldCheck size={14} />{auth?.account ? `@${auth.account.username}` : t.anonymous}</span></header>
    {errorCode && <div className="notice warning" role="alert">{hubError(errorCode, language)}</div>}
    {mode === 'models' && <div className="segmented local-model-tabs"><button className={!localModels ? 'active' : ''} onClick={() => setLocalModels(false)}>Hugging Face</button><button className={localModels ? 'active' : ''} onClick={() => setLocalModels(true)}>{language === 'de' ? 'Lokal gespeichert' : 'Stored locally'}</button></div>}
    {mode === 'hub' && <div className="segmented hf-hub-tabs"><button aria-pressed={!website} className={!website ? 'active' : ''} onClick={() => setWebsite(false)}>{language === 'de' ? 'App-Konto' : 'App account'}</button><button aria-pressed={website} className={website ? 'active' : ''} onClick={() => setWebsite(true)}>{language === 'de' ? 'Website im Studio' : 'Website in Studio'}</button></div>}
    {mode === 'hub' && website && auth?.pending && <div className="hub-login-pending" role="status">{working}<span>{t.pending}</span><button className="button secondary" onClick={() => void hubApi.cancelLogin().then(setAuth).catch(setError)}>{t.cancel}</button></div>}
    {mode === 'hub' ? website ? <HfBrowserPanel language={language} signingIn={!!auth?.pending} onModel={repo => showModels(repo)} /> : <>
      <section className="panel hub-account" aria-label={t.account}>
        <div className="hub-account-title"><UserRound size={28} /><div><h2>{auth?.account ? (auth.account.displayName || auth.account.username) : t.anonymous}</h2>{auth?.account && <p>@{auth.account.username} · {auth.expired ? t.expired : auth.verified ? t.verified : t.cached}</p>}</div>{auth?.verified && <CheckCircle2 size={20} />}</div>
        <p>{t.secret}</p>
        {auth?.account ? <div className="hub-actions"><button className="button secondary" disabled={busy} onClick={() => void account(hubApi.verify)}>{busy ? working : <ShieldCheck size={16} />}{t.verify}</button><button className="button secondary" disabled={busy} onClick={() => void account(hubApi.logout)}>{t.logout}</button><button className="text-button" disabled={busy} onClick={() => open('applications')}>{t.manageApps}<ExternalLink size={14} /></button></div> : <>
          <button className="button primary" disabled={busy || !auth?.oauthConfigured || auth.pending} onClick={() => void account(async () => { const next = await hubApi.startLogin(); if (next.pending) setWebsite(true); return next; })}>{busy ? working : <UserRound size={16} />}{t.connect}</button>
          {auth && !auth.oauthConfigured && <p className="hub-hint">{t.missingClient}</p>}
        </>}
        {auth?.pending && <div className="hub-login-pending" role="status">{working}<span>{t.pending}</span><button className="button secondary" onClick={() => void hubApi.cancelLogin().then(setAuth).catch(setError)}>{t.cancel}</button></div>}
        {!auth?.account && <details className="hub-advanced"><summary>{t.advanced}</summary><p>{t.tokenHint}</p><form onSubmit={event => {
          event.preventDefault(); if (busyRef.current || auth?.pending) return;
          const submitted = token; setToken(''); void account(() => hubApi.connectToken(submitted));
        }}><label className="field-label" htmlFor="hf-token">{t.token}</label><div className="hub-token-field"><input id="hf-token" type="password" autoComplete="off" spellCheck={false} value={token} maxLength={2048} disabled={busy || !!auth?.pending} onChange={event => setToken(event.target.value)} /><button className="button secondary" disabled={busy || !!auth?.pending || !token.trim()}>{busy ? working : <ShieldCheck size={16} />}{t.tokenSubmit}</button></div></form><button className="text-button" disabled={busy} onClick={() => open('tokens')}>{t.manageTokens}<ExternalLink size={14} /></button></details>}
      </section>
      <section className="panel hub-website"><h2>{t.models}</h2><p>{t.emptyStart}</p><button className="button primary" onClick={() => showModels()}><Search size={16} />{t.search}<ArrowRight size={16} /></button><hr /><p>{t.websiteHint}</p><button className="button secondary" disabled={busy} onClick={() => setWebsite(true)}>{t.website}<ExternalLink size={15} /></button></section>
    </> : localModels ? <LocalModels language={language} showImage={showImage} /> : <>
      <form className="panel hub-search" onSubmit={event => { event.preventDefault(); void search(); }}>
        <label className="field-label" htmlFor="hf-query">{t.searchLabel}</label><div className="hub-search-input"><input id="hf-query" value={query.search} maxLength={200} disabled={busy} onChange={event => editQuery('search', event.target.value)} placeholder="Qwen, FLUX, Whisper …" /><button className="button primary" disabled={busy || onlyExecutable}>{busy ? working : <Search size={16} />}{t.search}</button></div>
        <div className="hub-filters"><label>{t.task}<select value={query.task} disabled={busy} onChange={event => editQuery('task', event.target.value)}>{[['', t.allTasks], ['text-to-image', t.image], ['image-to-image', t.imageEdit], ['text-to-video', t.video], ['image-to-video', t.imageVideo], ['text-generation', t.chat], ['image-text-to-text', t.vision], ['text-to-audio', t.audio], ['automatic-speech-recognition', t.speech]].map(([value, label]) => <option key={value} value={value}>{label}</option>)}</select></label><label>{t.sort}<select value={query.sort} disabled={busy} onChange={event => editQuery('sort', event.target.value)}>{[['downloads', t.downloads], ['likes', t.likes], ['lastModified', t.recent], ['trendingScore', t.trending]].map(([value, label]) => <option key={value} value={value}>{label}</option>)}</select></label></div>
        <label className="hub-check"><input type="checkbox" checked={onlyExecutable} disabled={busy} onChange={event => { setOnlyExecutable(event.target.checked); setModels(null); setSizes({}); setDetail(null); setCursor(null); }} />{t.executableOnly}</label>
      </form>
      <p className="hub-hint">{t.unsupported} {t.downloadPlanned}</p>
      {detail ? <section className="panel hub-detail" aria-label={t.details}>
        <div className="section-heading"><h2>{detail.model.id}</h2><button className="icon-button" title={t.back} aria-label={t.back} onClick={() => setDetail(null)} disabled={busy}><X size={18} /></button></div>
        <p className="model-size" data-testid="model-total-size">{t.repoSize}: <strong>{formatGigabytes(totalFileBytes(detail.files), language)}</strong><small>{t.repoSizeHint}</small></p>
        <div className="hub-tags">{detail.model.restricted&&<span className="pill">18+</span>}<span className="pill">{detail.model.gated ? t.gated : detail.model.private ? t.private : t.public}</span><span className="pill">{t.license}: {detail.model.license ?? t.unknown}</span>{detail.model.library && <span className="pill">{detail.model.library}</span>}</div>
        <form className="hub-revision" onSubmit={event => { event.preventDefault(); void inspect(detail.model.id, revision); }}><label>{t.revision}<input value={revision} maxLength={200} disabled={busy} onChange={event => setRevision(event.target.value)} /></label><button className="button secondary" disabled={busy || !revision.trim()}>{busy ? working : null}{t.loadRevision}</button></form>
        <p className="hub-commit">{t.resolved}: <code>{detail.revision}</code></p><button className="text-button" disabled={busy} onClick={() => open('model', detail.model.id)}>{t.modelPage}<ExternalLink size={14} /></button>
        <DownloadSelection key={`${detail.model.id}@${detail.revision}`} detail={detail} language={language} onQueued={showDownloads} />
        <details className="hub-card"><summary>{t.card}</summary>{detail.card === null ? <p>{t.cardMissing} {hubError(detail.cardError, language)}</p> : <pre tabIndex={0}>{detail.card}</pre>}</details>
      </section> : onlyExecutable ? <div className="panel hub-empty"><Box size={28} /><p>{t.noRuntime}</p></div> : models === null ? <div className="panel hub-empty"><Search size={28} /><p>{busy ? t.working : t.emptyStart}</p></div> : <>
        <div className="hub-results" aria-live="polite">{models.length === 0 ? <p>{t.empty}</p> : models.map(model => <article className="panel hub-model" key={model.id}><div className="hub-model-icon"><Box size={21} /></div><div className="hub-model-content"><h2>{model.id}</h2><p className="model-size" data-testid="model-size" title={t.repoSizeHint}>{t.repoSize}: <strong>{sizes[`${model.id}@${model.revision}`] === undefined ? t.working : formatGigabytes(sizes[`${model.id}@${model.revision}`], language)}</strong></p><div className="hub-model-meta">{model.restricted&&<span>18+</span>}{model.task && <span>{model.task}</span>}<span>{t.license}: {model.license ?? t.unknown}</span>{model.gated && <span>{t.gated}</span>}{model.private && <span>{t.private}</span>}</div><small>{new Intl.NumberFormat(language).format(model.downloads)} {t.downloads} · {new Intl.NumberFormat(language).format(model.likes)} {t.likes}</small></div><button className="button secondary" disabled={busy} onClick={() => void inspect(model.id)}>{t.details}<ArrowRight size={14} /></button></article>)}</div>
        <div className="hub-pagination"><span>{models.length} {t.resultCount}</span>{cursor && <button className="button secondary" disabled={busy} onClick={() => void search(true)}>{busy ? working : null}{t.more}</button>}</div>
      </>}
    </>}
    {!website && <p className="under-panel-note"><ShieldCheck size={16} />{t.privacy}</p>}
  </div>;
}
