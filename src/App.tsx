import {PrivacyButton,PrivacyGate,usePrivacy} from "./Privacy";
import LearnedPreferences from "./LearnedPreferences";
import CoreFeatures,{CoreSettings} from "./CoreFeatures";
import {listen} from "@tauri-apps/api/event";
import AssistantPage from './AssistantPage';
import {ai} from './ai-api';
import CanvasStudio from './CanvasStudio';
import TimelineStudio from './TimelineStudio';
import VideoJobs from './VideoJobs';
import {mediaApi} from './media-tools';
import {useCreative} from './useCreative';
import {creativeApi} from './creative-state';
import {setMenuContext} from './menu-context';
import './creative.css';
import AppMenu from './AppMenu';
import HelpDialog from './HelpDialog';
import UpdateDialog from './UpdateDialog';
import {scheduleStartupUpdate} from './startup-update';
import {updateApi,updateError} from './update-api';
import {editorActivity} from './editor-state';
import Gallery from './Gallery';
import ProjectPanel from './ProjectPanel';
import HomeProjects from './HomeProjects';
import StorageMaintenance from './StorageMaintenance';
import { useFileDrop } from './useFileDrop';
import { invoke } from '@tauri-apps/api/core';
import { useProject } from './useProject';
import StorageSettings from './StorageSettings';
import ShortcutSettings from './ShortcutSettings';
import { shortcutFor } from './shortcuts';
import ImageStudio from './ImageStudio';
import { activeImage, imageApi, imageError, type ImageJob } from './image-api';
import ImageJobRow from './ImageJobRow';
import ExitDialog, { type ExitPrompt, type ExitChoice } from './ExitDialog';
import { useImageWorkspace } from './useImageWorkspace';
import DownloadsPage from './DownloadsPage';
import { downloads, activeDownload } from './download-api';
import HubPage from './HubPage';
import './hub.css';
import { useCallback, useEffect, useRef, useState, type FormEvent, type ReactNode } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open } from '@tauri-apps/plugin-dialog';
import { ArrowLeft, ArrowRight, Box, Check, CheckCircle2, ChevronDown, Circle, CircleHelp, Cpu, Download, FileCheck2, Film, Folder, HardDrive, House, Images, LoaderCircle, Monitor, Palette, RefreshCw, Settings2, ShieldCheck, Sparkles, Square, Terminal, TriangleAlert, Workflow, X, type LucideIcon } from 'lucide-react';
import { api, inDesktop } from './api';
import { clampScale, errorMessage, fileName, formatBytes, formatDate, initialLanguage, isActiveJob, ZOOM_STEP } from './helpers';
import { translations, type Translations } from './i18n';
import type { AppSnapshot, HardwareInfo, Job, Language, Settings, StoragePaths } from './types';

type Page = 'home' | 'studio' | 'models' | 'downloads' | 'jobs' | 'gallery' | 'hub' | 'assistant' | 'settings';
type JobFilter = 'all' | 'active' | 'finished';
const nav: { id: Page; icon: LucideIcon; group: number }[] = [
  { id: 'home', icon: House, group: 0 }, { id: 'studio', icon: Palette, group: 0 },
  { id: 'models', icon: Box, group: 1 }, { id: 'downloads', icon: Download, group: 1 },
  { id: 'jobs', icon: Workflow, group: 1 }, { id: 'gallery', icon: Images, group: 1 },
  { id: 'hub', icon: CircleHelp, group: 1 }, { id: 'assistant', icon: Sparkles, group: 1 },
];
const pathKeys: { key: keyof StoragePaths; label: keyof Translations }[] = [
  { key: 'models', label: 'pathsModels' }, { key: 'assistantModels', label: 'pathsAssistantModels' },
  { key: 'visionModels', label: 'pathsVisionModels' }, { key: 'downloads', label: 'pathsDownloads' },
  { key: 'gallery', label: 'pathsGallery' }, { key: 'projects', label: 'pathsProjects' },
  { key: 'temporary', label: 'pathsTemporary' }, { key: 'recovery', label: 'pathsRecovery' },
  { key: 'cache', label: 'pathsCache' }, { key: 'proxies', label: 'pathsProxies' },
];

function Logo({ small = false }: { small?: boolean }) {
  return <span aria-hidden="true" className={`logo ${small ? 'logo-small' : ''}`}><span /><span /><span /></span>;
}

function IconButton({ title, onClick, children, disabled = false }: { title: string; onClick: () => void; children: ReactNode; disabled?: boolean }) {
  return <button type="button" className="icon-button" title={title} aria-label={title} onClick={onClick} disabled={disabled}>{children}</button>;
}

function Hardware({ hardware, language, t, onRefresh, refreshing }: { hardware: HardwareInfo; language: Language; t: Translations; onRefresh: () => void; refreshing: boolean }) {
  return <section className="panel hardware-panel" aria-labelledby="hardware-heading">
    <div className="section-heading"><h2 id="hardware-heading">{t.device}</h2><IconButton title={t.refresh} onClick={onRefresh} disabled={refreshing}><RefreshCw size={16} className={refreshing ? 'spin' : ''} /></IconButton></div>
    <div className="hardware-row"><Cpu size={20} /><div><span className="eyebrow">{t.cpu}</span><strong>{hardware.cpu || t.unknown}</strong><small>{hardware.logicalCores} {t.cores}</small></div></div>
    <div className="hardware-row"><Square size={20} /><div><span className="eyebrow">{t.memory}</span><strong>{formatBytes(hardware.totalMemoryBytes, language)}</strong><small>{formatBytes(hardware.availableMemoryBytes, language)} {t.available}</small></div></div>
    <div className="hardware-row"><Monitor size={20} /><div><span className="eyebrow">{t.gpu}</span>{hardware.gpus.length ? hardware.gpus.map((gpu, index) => <div className="gpu-item" key={`${gpu.name}-${index}`}><strong>{gpu.name}</strong><small>{gpu.vramBytes === null ? t.unknown : `${formatBytes(gpu.vramBytes, language)} VRAM`}{gpu.driver ? ` · ${t.driver} ${gpu.driver}` : ''}</small></div>) : <strong>{t.noGpu}</strong>}</div></div>
    <details className="hardware-details"><summary>{t.system} & {t.storage}<ChevronDown size={14} /></summary><p>{hardware.os}</p>{hardware.disks.map((disk, index) => <div className="drive" key={`${disk.mountPoint}-${index}`}><span title={disk.mountPoint}>{disk.name || disk.mountPoint}</span><small>{formatBytes(disk.availableBytes, language)} {t.free} / {formatBytes(disk.totalBytes, language)}</small><meter min={0} max={Math.max(disk.totalBytes, 1)} value={Math.max(0, disk.totalBytes - disk.availableBytes)} aria-label={disk.mountPoint} /></div>)}</details>
    {hardware.warnings.map((warning, index) => <p className="inline-warning" key={index}><TriangleAlert size={15} /><span>{warning}</span></p>)}
  </section>;
}

function JobRow({ job, t, language, onCancel, pending }: { job: Job; t: Translations; language: Language; onCancel: (id: string) => void; pending: boolean }) {
  const active = isActiveJob(job);
  const percentage = job.progress === null ? null : Math.min(100, Math.max(0, job.progress * 100));
  return <article className="job-row">
    <div className={`job-icon status-${job.status}`}>{job.status === 'completed' ? <CheckCircle2 size={19} /> : active ? <LoaderCircle size={19} className={job.status === 'running' ? 'spin' : ''} /> : job.status === 'failed' || job.status === 'interrupted' ? <TriangleAlert size={19} /> : <FileCheck2 size={19} />}</div>
    <div className="job-content"><div className="job-heading"><strong title={job.inputPath}>{fileName(job.inputPath)}</strong><span className={`status status-${job.status}`}>{t[job.status]}</span></div>
      <div className="job-meta"><span>{t.hashJob}</span><span>·</span><time dateTime={job.createdAt}>{formatDate(job.createdAt, language)}</time></div>
      {active && <div className="progress-line"><progress max={100} value={percentage ?? undefined} aria-label={`${t[job.status]} ${fileName(job.inputPath)}`} />{percentage !== null && <small>{Math.floor(percentage)} %</small>}</div>}
      <details className="job-details"><summary>{t.details}<ChevronDown size={13} /></summary><dl><dt>{t.inputFile}</dt><dd>{job.inputPath}</dd><dt>{t.id}</dt><dd>{job.id}</dd>{job.startedAt && <><dt>{t.started}</dt><dd>{formatDate(job.startedAt, language)}</dd></>}{job.finishedAt && <><dt>{t.finished}</dt><dd>{formatDate(job.finishedAt, language)}</dd></>}</dl>{job.result && <div className="hash-result"><span>{t.result}</span><code tabIndex={0}>{job.result}</code></div>}{job.error && <p className="job-error">{job.error}</p>}</details>
    </div>
    {active && <IconButton title={t.cancelJob} onClick={() => onCancel(job.id)} disabled={pending}><X size={16} /></IconButton>}
  </article>;
}

function EmptyJobs({ t, filtered = false }: { t: Translations; filtered?: boolean }) {
  return <div className="empty-jobs"><div className="empty-icon"><Workflow size={25} strokeWidth={1.4} /></div><h3>{filtered ? t.noFilterJobs : t.emptyJobs}</h3>{!filtered && <p>{t.emptyJobsText}</p>}</div>;
}

function PlannedPage({ page, t, goHome }: { page: Exclude<Page, 'home' | 'settings' | 'jobs'>; t: Translations; goHome: () => void }) {
  const config = {
    studio: { icon: Palette, intro: t.studioIntro, items: t.studioItems, milestone: t.milestoneStudio },
    models: { icon: Box, intro: t.modelsIntro, items: t.modelsItems, milestone: t.milestoneModels },
    downloads: { icon: Download, intro: t.downloadsIntro, items: t.downloadsItems, milestone: t.milestoneModels },
    gallery: { icon: Images, intro: t.galleryIntro, items: t.galleryItems, milestone: t.milestoneGallery },
    hub: { icon: CircleHelp, intro: t.hubIntro, items: t.hubItems, milestone: t.milestoneModels },
    assistant: { icon: Sparkles, intro: t.assistantIntro, items: t.assistantItems, milestone: t.milestoneAssistant },
  }[page];
  const Icon = config.icon;
  return <div className="page planned-page"><header className="page-heading"><div><div className="eyebrow">{t.planned}</div><h1>{t[page]}</h1><p>{config.intro}</p></div></header><section className="panel planned-panel"><div className="planned-symbol"><Icon size={38} strokeWidth={1.2} /></div><span className="pill">{config.milestone}</span><h2>{t.foundation}</h2><p>{t.plannedIntro}</p><div className="planned-list"><h3>{t.plannedLabel}</h3>{config.items.map(item => <div key={item}><Circle size={9} /><span>{item}</span></div>)}</div><button className="button secondary" onClick={goHome}><ArrowLeft size={16} />{t.backHome}</button></section></div>;
}

function Setup({ snapshot, onSave, busy, t, language, onLanguage, canEdit, chooseFolder }: { snapshot: AppSnapshot; onSave: (settings: Settings) => Promise<void>; busy: boolean; t: Translations; language: Language; onLanguage: (language: Language) => void; canEdit: () => boolean; chooseFolder: (defaultPath: string, onChoose: (path: string) => void) => Promise<void> }) {
  const [root, setRoot] = useState(snapshot.settings.dataRoot);
  const heading = useRef<HTMLHeadingElement>(null);
  const container = useRef<HTMLDivElement>(null);
  useEffect(() => { heading.current?.focus(); }, []);
  async function choose() { await chooseFolder(root, setRoot); }
  return <div className="modal-backdrop"><div className="setup-dialog" role="dialog" aria-modal="true" aria-labelledby="setup-heading" ref={container} onKeyDown={event => {
    if (event.key !== 'Tab') return;
    const focusable = container.current?.querySelectorAll<HTMLElement>('button:not(:disabled), select:not(:disabled), input:not(:disabled), [tabindex="0"]');
    if (!focusable?.length) return;
    const first = focusable[0]; const last = focusable[focusable.length - 1];
    if (event.shiftKey && (document.activeElement === first || document.activeElement === heading.current)) { event.preventDefault(); last.focus(); }
    else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
  }}><div className="setup-top"><Logo /><select aria-label={t.language} value={language} disabled={busy} onChange={event => { if (canEdit()) onLanguage(event.target.value as Language); }}><option value="de">Deutsch</option><option value="en">English</option></select></div><div className="eyebrow">{t.setupEyebrow}</div><h1 id="setup-heading" ref={heading} tabIndex={-1}>{t.setupTitle}</h1><p className="setup-intro">{t.setupDescription}</p><label className="field-label" htmlFor="setup-root">{t.dataFolder}</label><div className="path-field"><input id="setup-root" value={root} disabled={busy} onChange={event => { if (canEdit()) setRoot(event.target.value); }} spellCheck={false} /><IconButton title={t.browse} onClick={() => void choose()} disabled={busy}><Folder size={18} /></IconButton></div><div className="setup-device"><Cpu size={21} /><div><strong>{t.setupHardware}</strong><span>{snapshot.hardware.cpu || t.unknown}</span><small>{formatBytes(snapshot.hardware.totalMemoryBytes, language)} RAM · {snapshot.hardware.gpus[0]?.name || t.noGpu}</small></div><Check size={18} /></div><p className="setup-privacy"><ShieldCheck size={19} /><span>{t.setupPrivacy}</span></p><button className="button primary setup-submit" disabled={busy || !root.trim()} onClick={() => void onSave({ ...snapshot.settings, language, dataRoot: root.trim(), setupComplete: true })}>{busy ? <LoaderCircle size={17} className="spin" /> : null}{busy ? t.saving : t.finishSetup}{!busy && <ArrowRight size={17} />}</button></div></div>;
}

export default function App() {
  const [helpDialog,setHelpDialog]=useState<'help'|'about'|null>(null);
  const [updatesOpen,setUpdatesOpen]=useState(false);
  const [projectDetailsOpen,setProjectDetailsOpen]=useState(false);
  const installing=useRef(false);
  const forceExit=useRef(false);
  useEffect(()=>{const exit=()=>{forceExit.current=true;void getCurrentWindow().close();};window.addEventListener("local-studio-exit",exit);let alive=true;let off:(()=>void)|undefined;void listen("app-exit-request",exit).then(fn=>{if(alive)off=fn;else fn();});return()=>{alive=false;off?.();window.removeEventListener("local-studio-exit",exit);};},[]);
  const autoChecked=useRef(false);
  const stopStartupUpdate=useRef<()=>void>(()=>{});
  const [snapshot, setSnapshot] = useState<AppSnapshot | null>(null);
  const [draft, setDraft] = useState<Settings | null>(null);
  const privacy=usePrivacy();
  const [page, setPage] = useState<Page>(()=>{const saved=sessionStorage.getItem('privacy-page') as Page|null;return saved&&['home','studio','models','downloads','jobs','gallery','hub','assistant','settings'].includes(saved)?saved:'home';});
  useEffect(()=>{sessionStorage.setItem('privacy-page',page);setProjectDetailsOpen(false);},[page]);
  const [selectedImageJob, setSelectedImageJob] = useState<string | null>(null);
  const [imageJobs, setImageJobs] = useState<ImageJob[]>([]);
  const [exitPrompt, setExitPrompt] = useState<ExitPrompt | null>(null);
  const [exitBusy, setExitBusy] = useState(false);
  const studio = useImageWorkspace(!!snapshot);
  const flushStudio = studio.flush;
  const [initialModelRepo, setInitialModelRepo] = useState<string | null>(null);
  const [language, setLanguage] = useState<Language>(initialLanguage);
  const projects = useProject(!!snapshot, studio, language === 'de', snapshot?.paths.projects ?? '');
  const creative=useCreative(projects,language==='de',snapshot?.settings.maxUndo??100);
  const [studioTab,setStudioTab]=useState<'generate'|'canvas'|'timeline'>(()=>{const s=sessionStorage.getItem('privacy-studio');return s==='canvas'||s==='timeline'?s:'generate';});
  useEffect(()=>{sessionStorage.setItem('privacy-studio',studioTab);},[studioTab]);
  useEffect(()=>{privacy.beforeLock.current=async()=>{if(studio.ready&&!studio.recovery){await studio.flush();if(projects.ready&&!projects.project?.recovery&&!projects.project?.locked)await projects.flush();}};return()=>{privacy.beforeLock.current=null;};},[studio,projects,privacy.beforeLock]);
  useEffect(()=>{if(projectDetailsOpen||page!=='studio'||studioTab==='generate')return;return setMenuContext('creative',{priority:1,actions:{...(creative.canUndo?{undo:creative.undo}:{}),...(creative.canRedo?{redo:creative.redo}:{})}});},[projectDetailsOpen,page,studioTab,creative.canUndo,creative.canRedo,creative.data]);
  useEffect(()=>{const handler=(e:KeyboardEvent)=>{if(!snapshot||projectDetailsOpen||page!=='studio'||studioTab==='generate'||document.querySelector('dialog[open]')||(e.target instanceof HTMLElement&&e.target.closest('input,textarea,select,[contenteditable=true]')))return;const action=shortcutFor(e,snapshot.settings.shortcuts);if(action==='undo'){e.preventDefault();creative.undo();}if(action==='redo'){e.preventDefault();creative.redo();}};window.addEventListener('keydown',handler);return()=>window.removeEventListener('keydown',handler);},[projectDetailsOpen,page,studioTab,creative.data,snapshot?.settings.shortcuts]);
  const projectRef = useRef(projects); projectRef.current = projects;
  const [loading, setLoading] = useState(inDesktop());
  const [startupError, setStartupError] = useState('');
  const [error, setError] = useState('');
  const [toast, setToast] = useState('');
  const [saving, setSaving] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [jobPending, setJobPending] = useState(false);
  const [jobLimit, setJobLimit] = useState(100);
  const [jobFilter, setJobFilter] = useState<JobFilter>('all');
  const [pollError, setPollError] = useState(false);
  const [logs, setLogs] = useState<string | null>(null);
  const [logsBusy, setLogsBusy] = useState(false);
  const snapshotRef = useRef(snapshot);
  const dirtyRef = useRef(false);
  const languageRef = useRef(language);
  const zoomTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const writeQueue = useRef<Promise<unknown>>(Promise.resolve());
  const closing = useRef(false);
  const closingDialog = useRef(false);
  // Ref guards take effect in the same event, before disabled controls render.
  const savingRef = useRef(false);
  const settingsSave = useRef<Promise<void> | null>(null);
  const settingsEditVersion = useRef(0);
  const canEditSettings = useCallback(() => !savingRef.current && !closingDialog.current, []);
  const t = translations(language);
  const dirty = !!(snapshot && draft && JSON.stringify(snapshot.settings) !== JSON.stringify(draft));
  snapshotRef.current = snapshot;
  dirtyRef.current = dirty;
  languageRef.current = language;

  const saveRaw = useCallback((settings: Settings): Promise<Settings> => {
    const result = writeQueue.current.then(() => api.saveSettings(settings));
    writeQueue.current = result.catch(() => undefined);
    return result;
  }, []);

  const reportError = useCallback((value: unknown) => { setError(errorMessage(value)); }, []);
  const dropTarget = useFileDrop(!snapshot?.settings.setupComplete || saving || exitBusy || !!exitPrompt || projects.busy, projects.addFiles, reportError);
  useEffect(() => {
    if (!snapshot?.settings.setupComplete) return;
    const clean = () => { if (canEditSettings()) void invoke<{ deleted: number }>('storage_cleanup_auto').then(result => { if (result.deleted) setToast(languageRef.current === 'de' ? `${result.deleted} alte Arbeitsdateien bereinigt.` : `${result.deleted} old working files cleaned.`); }).catch(reportError); };
    const start = setTimeout(clean, 30000); const interval = setInterval(clean, 3600000);
    return () => { clearTimeout(start); clearInterval(interval); };
  }, [snapshot?.settings.setupComplete, canEditSettings, reportError]);


  const bootstrap = useCallback(async () => {
    if (!inDesktop()) return;
    setLoading(true); setStartupError('');
    try { const state = await api.bootstrap(); setSnapshot(state); setDraft(state.settings); setLanguage(state.settings.language); }
    catch (value) { setStartupError(errorMessage(value)); }
    finally { setLoading(false); }
  }, []);

  useEffect(() => { void bootstrap(); }, [bootstrap]);

  useEffect(() => {
    const root = document.documentElement;
    const theme = snapshot?.settings.theme ?? 'system';
    root.dataset.theme = theme;
    root.lang = language;
    root.style.setProperty('--accent', snapshot?.settings.accentColor ?? '#4b9f91');
    root.style.setProperty('--ui-scale', String(snapshot?.settings.uiScale ?? 1));
    root.style.colorScheme = theme === 'system' ? 'light dark' : theme;
  }, [snapshot?.settings.theme, snapshot?.settings.accentColor, snapshot?.settings.uiScale, language]);

  useEffect(() => { if (!toast) return; const timer = setTimeout(() => setToast(''), 3200); return () => clearTimeout(timer); }, [toast]);

  useEffect(() => {
    if (!snapshot) return;
    let live = true;
    let timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      try { const [jobs, images] = await Promise.all([api.jobs(), imageApi.jobs()]); if (live) { setSnapshot(current => current ? { ...current, jobs } : current); setImageJobs(images); setPollError(false); } }
      catch { if (live) setPollError(true); }
      if (live) timer = setTimeout(() => void poll(), 800);
    };
    timer = setTimeout(() => void poll(), 800);
    return () => { live = false; clearTimeout(timer); };
  }, [!!snapshot]);

  useEffect(() => {
    if (!inDesktop()) return;
    const appWindow = getCurrentWindow();
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void appWindow.onCloseRequested(async event => {
      if (closing.current) return;
      event.preventDefault();
      if (closingDialog.current) return;
      if(!forceExit.current&&!installing.current&&snapshotRef.current?.settings.minimizeToTray){try{if(await invoke<boolean>("background_hide"))return;}catch(value){reportError(value);}}
      forceExit.current=false;
      closingDialog.current = true;
      try {
        // Include applying the saved settings and refreshed paths, not just the IPC write.
        editorActivity.cancelBatch?.();
        await editorActivity.pending?.catch(()=>{});
        await editorActivity.flushCreative?.();
        await settingsSave.current;
        const copy = translations(languageRef.current);
        const latest = await api.jobs();
        const transfers = await downloads.list();
        await flushStudio();
        await projectRef.current.flush();
        const activeProject = projectRef.current.get();
        const imageState = await imageApi.workspace();
        const imageJobs = await imageApi.jobs();
        const imagesRunning = imageJobs.some(activeImage);
        const unsaved = imageState.unsaved;
        const aiState=await ai.status();
        const aiRunning=['loading','running'].includes(aiState.phase)||(await ai.jobs()).some(j=>j.status==='running');
        const videoRunning=aiRunning || (await creativeApi.jobs()).some(j=>j.status==='running') || (await mediaApi.status())?.status==='running';
        const warnings = [videoRunning?(languageRef.current==='de'?'Beim Beenden werden laufende KI-Aufgaben, Videoexporte und Medienvorbereitungen abgebrochen. Die Timeline bleibt im lokalen Projektarbeitsstand.':'Closing cancels active AI tasks, video renders and media preparation. The timeline remains in the local project workspace.'):'',!snapshotRef.current?.settings.restoreSession && (imageState.workspace.request?.prompt || imageState.workspace.request?.negativePrompt) ? (languageRef.current === 'de' ? 'Die aktuellen Studio-Eingaben werden beim nächsten Start geleert. Prompts fertiger Aufträge bleiben in deren Metadaten erhalten.' : 'Current Studio inputs will be cleared on the next start. Prompts of completed jobs remain in their metadata.') : '', imagesRunning ? (languageRef.current === 'de' ? 'Beim Beenden wird die laufende Bildgenerierung abgebrochen; wartende Aufträge werden pausiert. Bereits fertig gewordene Bilder werden in deine Auswahl einbezogen.' : 'Closing cancels the running image generation and pauses waiting jobs. Images that have already completed are included in your choice.') : '', latest.some(isActiveJob) ? copy.exitActive : '', transfers.some(activeDownload) ? (languageRef.current === 'de' ? 'Downloads werden pausiert; Teil-Dateien bleiben zum Fortsetzen erhalten.' : 'Downloads will pause; partial files are kept for resuming.') : '', dirtyRef.current ? copy.exitUnsaved : ''].filter(Boolean);
        if ((await invoke<{status:string}|null>('model_move_status'))?.status==='running') warnings.push(languageRef.current==='de'?'Das Verschieben von Modelldateien wird sicher beendet oder abgebrochen. Bereits geprüfte Kopien bleiben bei einer unterbrochenen Umstellung erhalten.':'Moving model files will finish safely or be cancelled. Verified copies are retained if switching references is interrupted.');
        if (activeProject?.dirty) warnings.push(languageRef.current === 'de' ? 'Das Projekt enthält ungespeicherte Änderungen. Ohne Speichern bleibt die bisherige Projektdatei unverändert; die lokale Arbeitskopie kann später fortgesetzt werden.' : 'The project has unsaved changes. Without saving, the previous project file stays unchanged; the local working copy can be resumed later.');
        if(editorActivity.draft) warnings.push(languageRef.current === 'de' ? (editorActivity.persisted ? 'Bildbearbeitung noch nicht exportiert. Der lokale Entwurf bleibt zum Fortsetzen über dasselbe Bild in Galerie oder Projekt erhalten.' : 'Der Bildentwurf konnte nicht gespeichert werden. Abbrechen und im Editor exportieren, um die Änderungen zu behalten.') : (editorActivity.persisted ? 'Image edits have not been exported. The local draft can be resumed from the same image in the gallery or project.' : 'The image draft could not be saved. Cancel and export in the editor to retain changes.'));
        let saveProject = false;
        let choice: ExitChoice = 'close';
        if (warnings.length || unsaved) {
          window.dispatchEvent(new CustomEvent('studio-modal', { detail: true }));
          choice = await new Promise<ExitChoice>(resolve => setExitPrompt({ warnings, unsaved, imagesRunning, restoreSession: snapshotRef.current?.settings.restoreSession ?? false, project: !!activeProject && !activeProject.recovery && !activeProject.locked, projectDirty: !!activeProject?.dirty && !activeProject?.locked, resolve: (value, save) => { saveProject = !!save; resolve(value); } }));
        }
        if (choice === 'cancel') { installing.current=false; return; }
        if (saveProject && !await projectRef.current.save()) { installing.current=false; return; }
        setExitBusy(true);
        await flushStudio();
        if (zoomTimer.current) { clearTimeout(zoomTimer.current); zoomTimer.current = null; const current = snapshotRef.current; if (current) await saveRaw(current.settings); }
        await writeQueue.current;
        if(installing.current)await updateApi.arm();
        await api.cleanExit(choice === 'save' || choice === 'discard' || choice === 'keep' ? choice : undefined);
        closing.current = true;
        await appWindow.close();
      } catch (value) { if(installing.current){await updateApi.disarm().catch(()=>{});installing.current=false;} closing.current = false; reportError(String(value).startsWith('update_')?updateError(value,languageRef.current==='de'):imageError(value, languageRef.current === 'de')); }
      finally { closingDialog.current = false; setExitBusy(false); window.dispatchEvent(new CustomEvent('studio-modal', { detail: false })); }
    }).then(off => { if (disposed) off(); else unlisten = off; }).catch(reportError);
    return () => { disposed = true; unlisten?.(); };
  }, [reportError, saveRaw, flushStudio]);

  useEffect(() => {
    function zoom(next: number) {
      const current = snapshotRef.current;
      if (!current || !canEditSettings()) return;
      const uiScale = clampScale(next);
      if (uiScale === current.settings.uiScale) return;
      const settings = { ...current.settings, uiScale };
      snapshotRef.current = { ...current, settings };
      setSnapshot(snapshotRef.current);
      setDraft(value => value ? { ...value, uiScale } : value);
      if (zoomTimer.current) clearTimeout(zoomTimer.current);
      zoomTimer.current = setTimeout(() => {
        zoomTimer.current = null;
        const latest = snapshotRef.current;
        if (latest) void saveRaw(latest.settings).catch(reportError);
      }, 220);
    }
    const key = (event: KeyboardEvent) => {
      if (event.defaultPrevented || (event.target as HTMLElement).closest('[data-shortcut-recorder]')) return;
      if ((event.target as HTMLElement).closest('input,textarea,select,[contenteditable="true"]') && !event.ctrlKey && !event.altKey) return;
      const current = snapshotRef.current;
      if (!current) return;
      const command = shortcutFor(event, current.settings.shortcuts);
      if (command === 'projectSave' && canEditSettings() && !event.repeat && !document.querySelector('dialog[open]')) { event.preventDefault(); void projectRef.current.save(); return; }
      if (command && ['uiZoomIn', 'uiZoomOut', 'uiZoomReset'].includes(command)) {
        event.preventDefault();
        zoom(command === 'uiZoomReset' ? 1 : current.settings.uiScale + (command === 'uiZoomOut' ? -ZOOM_STEP : ZOOM_STEP));
      }
    };
    const wheel = (event: WheelEvent) => { if (event.ctrlKey && snapshotRef.current) { event.preventDefault(); zoom(snapshotRef.current.settings.uiScale + (event.deltaY < 0 ? ZOOM_STEP : -ZOOM_STEP)); } };
    window.addEventListener('keydown', key); window.addEventListener('wheel', wheel, { passive: false });
    return () => { window.removeEventListener('keydown', key); window.removeEventListener('wheel', wheel); };
  }, [canEditSettings, reportError, saveRaw]);

  useEffect(()=>{
    if(!snapshot?.settings.setupComplete||!snapshot.settings.autoUpdateCheck||autoChecked.current)return;
    const stop=scheduleStartupUpdate({
      status:updateApi.status, check:updateApi.check,
      onStarted:()=>{autoChecked.current=true;},
      canShow:()=>!closing.current&&!closingDialog.current&&!document.querySelector('dialog[open]'),
      onAvailable:()=>setUpdatesOpen(true),
    });
    stopStartupUpdate.current=stop;
    return stop;
  },[snapshot?.settings.setupComplete,snapshot?.settings.autoUpdateCheck]);

  function persist(settings: Settings): Promise<void> {
    if (!canEditSettings()) return settingsSave.current ?? Promise.resolve();
    savingRef.current = true;
    settingsEditVersion.current += 1;
    setSaving(true); setError('');
    if (zoomTimer.current) { clearTimeout(zoomTimer.current); zoomTimer.current = null; }
    const operation = (async () => {
      try {
        const previousStorage = JSON.stringify([snapshotRef.current?.settings.dataRoot, snapshotRef.current?.settings.storageOverrides]);
        const saved = await saveRaw(settings);
        if (snapshotRef.current) snapshotRef.current = { ...snapshotRef.current, settings: saved };
        dirtyRef.current = false;
        languageRef.current = saved.language;
        setSnapshot(current => current ? { ...current, settings: saved } : current);
        setDraft(saved); setLanguage(saved.language);
        if (previousStorage !== JSON.stringify([saved.dataRoot, saved.storageOverrides])) {
          const state = await api.bootstrap();
          snapshotRef.current = state;
          setSnapshot(state);
        }
        setToast(translations(saved.language).saved);
      } catch (value) { reportError(value); }
      finally { savingRef.current = false; setSaving(false); }
    })();
    settingsSave.current = operation;
    return operation;
  }

  async function chooseFile() {
    setJobPending(true); setError('');
    try { const path = await open({ multiple: false, directory: false, title: t.chooseFile }); if (typeof path === 'string') { await api.enqueueHash(path); const jobs = await api.jobs(); setSnapshot(current => current ? { ...current, jobs } : current); setPage('jobs'); setJobFilter('all'); } }
    catch (value) { reportError(value); }
    finally { setJobPending(false); }
  }
  async function cancelJob(id: string) {
    setJobPending(true);
    try { await api.cancelJob(id); const jobs = await api.jobs(); setSnapshot(current => current ? { ...current, jobs } : current); }
    catch (value) { reportError(value); }
    finally { setJobPending(false); }
  }
  async function refreshHardware() { setRefreshing(true); try { const hardware = await api.hardware(); setSnapshot(current => current ? { ...current, hardware } : current); } catch (value) { reportError(value); } finally { setRefreshing(false); } }
  async function chooseFolder(defaultPath: string, onChoose: (path: string) => void): Promise<void> {
    if (!canEditSettings()) return;
    const version = settingsEditVersion.current;
    try {
      const path = await open({ directory: true, multiple: false, title: t.chooseFolder, defaultPath });
      // A dialog opened before a save/discard must not restore a stale draft afterward.
      if (typeof path === 'string' && canEditSettings() && version === settingsEditVersion.current) onChoose(path);
    } catch (value) { reportError(value); }
  }
  async function chooseDataRoot() { if (draft) await chooseFolder(draft.dataRoot, path => updateDraft('dataRoot', path)); }
  async function dismissRecovery() { try { await api.dismissRecovery(); setSnapshot(current => current ? { ...current, recoveryAvailable: false } : current); } catch (value) { reportError(value); } }
  async function toggleLogs() { if (logs !== null) { setLogs(null); return; } setLogsBusy(true); try { setLogs(await api.logs()); } catch (value) { reportError(value); } finally { setLogsBusy(false); } }
  function updateDraft<K extends keyof Settings>(key: K, value: Settings[K]) {
    if (!canEditSettings()) return;
    settingsEditVersion.current += 1;
    setDraft(current => current ? { ...current, [key]: value } : current);
  }
  function discardSettings() {
    if (!canEditSettings() || !snapshotRef.current) return;
    settingsEditVersion.current += 1;
    setDraft(snapshotRef.current.settings);
  }

  if (!snapshot) return <div className="connection-screen"><div className="connection-content"><Logo /><span className="eyebrow">LOCAL STUDIO</span>{loading ? <><h1>{t.loading}</h1><LoaderCircle className="spin" size={24} /></> : <><h1>{startupError ? t.startupError : t.connectionTitle}</h1><p>{t.connectionText}</p>{startupError ? <pre className="startup-error">{startupError}</pre> : <><small>{t.launchCommand}</small><code className="launch-command">npm run desktop:dev</code></>}<button className="button secondary" onClick={() => { if (inDesktop()) void bootstrap(); else window.location.reload(); }}><RefreshCw size={16} />{t.retry}</button></>}</div><div className="connection-footer"><ShieldCheck size={15} />{t.local}</div></div>;

  const activeJobs = snapshot.jobs.filter(isActiveJob);
  const activeImages = imageJobs.filter(activeImage);
  const activeCount = activeJobs.length + activeImages.length;
  const combinedJobs = [...snapshot.jobs.map(job => ({ kind: 'file' as const, job })), ...imageJobs.map(job => ({ kind: 'image' as const, job }))].sort((a, b) => b.job.createdAt.localeCompare(a.job.createdAt));
  const visibleJobs = combinedJobs.filter(item => jobFilter === 'all' || (jobFilter === 'active' ? (item.kind === 'image' ? activeImage(item.job) : isActiveJob(item.job)) : (item.kind === 'image' ? !activeImage(item.job) : !isActiveJob(item.job))));
  const pageTitle = page === 'home' ? t.overview : t[page];
  const jobRows = (jobs: Job[]) => jobs.map(job => <JobRow key={job.id} job={job} t={t} language={language} onCancel={id => void cancelJob(id)} pending={jobPending} />);
  const hashButton = <button className="button primary" onClick={() => void chooseFile()} disabled={jobPending}><FileCheck2 size={17} />{t.hashAction}<ArrowRight size={16} /></button>;

  return <div className="app-shell">
    <AppMenu onProjectDetails={()=>setProjectDetailsOpen(v=>!v)} projectChanged={!!projects.project&&!projects.project.locked&&JSON.stringify(projects.project.request)!==JSON.stringify(studio.request)} workspaceRecovery={studio.recovery} projects={projects} settings={snapshot.settings} locked={exitBusy||saving||!studio.ready||!snapshot.settings.setupComplete} onSettings={()=>{setProjectDetailsOpen(false);setPage('settings');}} onHelp={()=>setHelpDialog('help')} onAbout={()=>setHelpDialog('about')} onUpdates={()=>{stopStartupUpdate.current();autoChecked.current=true;setUpdatesOpen(true);void updateApi.status().then(s=>{if(!['ready','downloading','checking','installing'].includes(s.phase))return updateApi.check();}).catch(e=>reportError(updateError(e,language==='de')));}} onZoom={scale=>{const current=snapshotRef.current;if(current)void persist({...current.settings,uiScale:clampScale(scale)});}} onError={reportError}/>
    {helpDialog&&<HelpDialog kind={helpDialog} de={language==='de'} version={snapshot.version} onClose={()=>setHelpDialog(null)}/>}
    {updatesOpen&&<UpdateDialog de={language==='de'} version={snapshot.version} automatic={snapshot.settings.autoUpdateCheck} onAutomatic={value=>{const current=snapshotRef.current;if(current)void persist({...current.settings,autoUpdateCheck:value});}} onClose={()=>setUpdatesOpen(false)} onInstall={()=>{installing.current=true;setUpdatesOpen(false);void getCurrentWindow().close();}}/>}
    <aside className="sidebar" inert={exitBusy}><div className="brand"><Logo small /><div>Local Studio<span>{t.core}</span></div></div><nav aria-label={t.workspace}>{[0, 1].map(group => <div className="nav-group" key={group}><span className="nav-label">{group === 0 ? t.workspace : t.library}</span>{nav.filter(item => item.group === group).map(item => { const Icon = item.icon; return <button key={item.id} className={`nav-item ${page === item.id ? 'selected' : ''}`} aria-current={page === item.id ? 'page' : undefined} onClick={() => { setInitialModelRepo(null); setProjectDetailsOpen(false); setPage(item.id); }}><Icon size={19} strokeWidth={1.7} /><span>{t[item.id]}</span>{item.id === 'studio' && activeImages.length > 0 && <span className="nav-count">{activeImages.length}</span>}{item.id === 'jobs' && activeCount > 0 && <span className="nav-count">{activeCount}</span>}</button>; })}</div>)}</nav><div className="sidebar-bottom"><button className={`nav-item ${page === 'settings' ? 'selected' : ''}`} aria-current={page === 'settings' ? 'page' : undefined} onClick={() => {setProjectDetailsOpen(false);setPage('settings');}}><Settings2 size={19} strokeWidth={1.7} /><span>{t.settings}</span>{dirty && <span className="unsaved-dot" title={t.unsaved} />}</button><div className="local-note"><span className={`connection-dot ${pollError ? 'warning' : ''}`} /><span>{t.local}</span></div></div></aside>
    <div className="main-shell" inert={exitBusy}><CoreFeatures settings={snapshot.settings} de={language==='de'} onJobs={()=>setPage('jobs')}/><header className="topbar"><div className="breadcrumb">Local Studio<span>/</span><strong>{projectDetailsOpen?(language==='de'?'Projektmedien und Details':'Project media and details'):pageTitle}</strong></div><button onClick={() => setPage('jobs')} className="topbar-status" title={pollError ? t.connectionLost : t.monitor}><span className={`connection-dot ${pollError ? 'warning' : ''}`} />{activeCount ? `${activeCount} ${t.active}${activeImages.length ? ' · Image' : ''}` : t.core}</button></header>
      <main id="main-content" className="main-content">
        <PrivacyButton de={language==='de'}/>{projectDetailsOpen&&<ProjectPanel onClose={()=>setProjectDetailsOpen(false)} shortcuts={snapshot.settings.shortcuts} maxUndo={snapshot.settings.maxUndo} controller={projects} language={language} disabled={!studio.ready || studio.recovery || saving} changed={!!projects.project && JSON.stringify(projects.project.request) !== JSON.stringify(studio.request)} />}
        {!projectDetailsOpen&&projects.error&&<p role="alert" className="notice warning">{projects.error}</p>}
        {!projectDetailsOpen&&projects.notice&&<p role="status" className="project-notice">{projects.notice}</p>}
        {!projectDetailsOpen&&projects.project?.recovery&&<p className="notice warning">{language==='de'?'Ein Projektarbeitsstand kann wiederhergestellt werden.':'A project workspace can be recovered.'} <button className="text-button" onClick={()=>setProjectDetailsOpen(true)}>{language==='de'?'Details öffnen':'Open details'}</button></p>}
        <div inert={projects.busy} hidden={projectDetailsOpen}>
        {(studio.error) && <p className="notice warning" role="alert">{imageError(studio.error, language === 'de')}</p>}
        {studio.recovery && <div className="recovery-banner" role="status"><TriangleAlert size={20} /><div><strong>{language === 'de' ? 'Bild-Arbeitsstand wiederherstellen' : 'Restore image workspace'}</strong><p>{language === 'de' ? 'Nach dem unerwarteten Beenden sind Eingaben und ungespeicherte Ergebnisse noch vorhanden. Unterbrochene Generierungen werden nicht fortgesetzt.' : 'Inputs and unsaved results are still available after the unexpected exit. Interrupted generations will not resume.'}</p><button className="button secondary" onClick={() => void studio.recover().then(() => { setSelectedImageJob(null); setStudioTab('generate'); setPage('studio'); }).catch(reportError)}>{language === 'de' ? 'Arbeitsstand wiederherstellen' : 'Restore workspace'}</button></div></div>}
        {snapshot.recoveryAvailable && <div className="recovery-banner" role="status"><TriangleAlert size={20} /><div><strong>{t.recovery}</strong><p>{t.recoveryText}</p><button className="text-button" onClick={() => setPage('jobs')}>{t.reviewJobs}<ArrowRight size={14} /></button></div><IconButton title={t.dismiss} onClick={() => void dismissRecovery()}><X size={16} /></IconButton></div>}
        {pollError && <div className="notice warning" role="status"><TriangleAlert size={17} />{t.connectionLost}</div>}
        {page === 'home' && <div className="page home-page"><header className="page-heading home-heading"><div><div className="eyebrow">LOCAL STUDIO</div><h1>{t.tagline}</h1><p>{t.subtitle}</p></div><span className="pill"><span className="connection-dot" />{t.core}</span></header><div className="home-grid"><div className="home-primary"><HomeProjects controller={projects} de={language==='de'} disabled={saving||exitBusy||!studio.ready||!snapshot.settings.setupComplete} workspaceRecovery={studio.recovery} onOpened={()=>{setProjectDetailsOpen(false);setPage('studio');}}/><section className="panel welcome-panel"><span className="panel-kicker"><CheckCircle2 size={17} />{t.ready}</span><h2>{t.hashTitle}</h2><p>{t.hashDescription}</p>{hashButton}<small className="hash-note">{t.hashNote}</small><div className="welcome-graphic" aria-hidden="true"><FileCheck2 size={70} strokeWidth={0.7} /><span className="graphic-line" /><code>SHA–256</code></div></section><section className="panel recent-panel"><div className="section-heading"><h2>{t.recentJobs}</h2><button className="text-button" onClick={() => setPage('jobs')}>{t.allJobs}<ArrowRight size={15} /></button></div>{snapshot.jobs.length ? jobRows(snapshot.jobs.slice(0, 4)) : <EmptyJobs t={t} />}</section><div className="privacy-note"><ShieldCheck size={23} strokeWidth={1.5} /><div><strong>{t.privacy}</strong><p>{t.privacyText}</p></div></div></div><Hardware hardware={snapshot.hardware} t={t} language={language} onRefresh={() => void refreshHardware()} refreshing={refreshing} /></div></div>}
        {page === 'jobs' && <VideoJobs de={language==='de'}/>}
        {page === 'jobs' && <div className="page"><header className="page-heading"><div><div className="eyebrow">{t.workspace}</div><h1>{t.jobs}</h1><p>{language === 'de' ? 'Lokale Bildgenerierungen und Dateiprüfungen mit Fortschritt, Ergebnis und Abbruch.' : 'Local image generations and file checks with progress, results and cancellation.'}</p></div>{hashButton}</header><section className="panel jobs-panel"><div className="jobs-toolbar"><div className="segmented" aria-label={t.jobs}>{(['all', 'active', 'finished'] as JobFilter[]).map(filter => <button key={filter} aria-pressed={jobFilter === filter} className={jobFilter === filter ? 'active' : ''} onClick={() => setJobFilter(filter)}>{filter === 'all' ? t.all : filter === 'active' ? t.activeFilter : t.finishedFilter}</button>)}</div><span className="subtle">{combinedJobs.length} {t.jobCount} · {activeCount} {t.active}</span></div>{visibleJobs.length ? visibleJobs.slice(0, jobLimit).map(item => item.kind === 'file' ? <JobRow key={item.job.id} job={item.job} t={t} language={language} onCancel={id => void cancelJob(id)} pending={jobPending} /> : <ImageJobRow key={item.job.id} job={item.job} language={language} pending={jobPending} onResume={() => { setJobPending(true); void imageApi.resume(item.job.id).catch(e => reportError(imageError(e, language === 'de'))).finally(() => setJobPending(false)); }} onOpen={() => { setSelectedImageJob(item.job.id); setStudioTab('generate'); setPage('studio'); }} onCancel={() => { setJobPending(true); void imageApi.cancel(item.job.id).catch(reportError).finally(() => setJobPending(false)); }} />) : <EmptyJobs t={t} filtered={combinedJobs.length > 0} />}{visibleJobs.length > jobLimit && <button className="button secondary" onClick={() => setJobLimit(n => n + 100)}>{language === 'de' ? 'Weitere Aufträge anzeigen' : 'Show more jobs'}</button>}</section><p className="under-panel-note"><ShieldCheck size={16} />{t.hashNote}</p></div>}
        {page === 'settings' && draft && <div className="page settings-page"><header className="page-heading"><div><div className="eyebrow">LOCAL STUDIO</div><h1>{t.settings}</h1><p>{t.settingsIntro}</p></div><span className="pill">{snapshot.portable ? t.portable : t.standard}</span></header><form onSubmit={(event: FormEvent) => { event.preventDefault(); void persist(draft); }}><section className="panel settings-section"><div className="settings-section-title"><Palette size={20} /><div><h2>{t.appearance}</h2><p>{t.appearanceHint}</p></div></div><div className="setting-row"><label htmlFor="language">{t.language}</label><select id="language" disabled={saving} value={draft.language} onChange={event => updateDraft('language', event.target.value as Language)}><option value="de">Deutsch</option><option value="en">English</option></select></div><div className="setting-row"><label htmlFor="restore-session">{language === 'de' ? 'Studio beim Start' : 'Studio on startup'}</label><select id="restore-session" disabled={saving} value={String(draft.restoreSession)} onChange={event => updateDraft('restoreSession', event.target.value === 'true')}><option value="false">{language === 'de' ? 'Leere Sitzung' : 'Empty session'}</option><option value="true">{language === 'de' ? 'Letzten Bild-Arbeitsstand wiederherstellen' : 'Restore last image workspace'}</option></select></div><div className="setting-row"><label htmlFor="theme">{t.theme}</label><select id="theme" disabled={saving} value={draft.theme} onChange={event => updateDraft('theme', event.target.value as Settings['theme'])}><option value="system">{t.themeSystem}</option><option value="light">{t.themeLight}</option><option value="dark">{t.themeDark}</option></select></div><div className="setting-row"><label htmlFor="accent">{t.accent}</label><div className="color-field"><span>{draft.accentColor.toUpperCase()}</span><input id="accent" disabled={saving} type="color" value={draft.accentColor} onChange={event => updateDraft('accentColor', event.target.value)} /></div></div><div className="setting-row scale-setting"><div><label htmlFor="scale">{t.scale}</label><small>{t.scaleHint}</small></div><div className="scale-control"><input id="scale" disabled={saving} type="range" min="0.75" max="1.5" step="0.05" value={draft.uiScale} onChange={event => updateDraft('uiScale', clampScale(Number(event.target.value)))} /><output htmlFor="scale">{Math.round(draft.uiScale * 100)} %</output><IconButton title={t.resetScale} disabled={saving} onClick={() => updateDraft('uiScale', 1)}><RefreshCw size={14} /></IconButton></div></div></section><section className="panel settings-section"><div className="settings-section-title"><Folder size={20} /><div><h2>{t.dataFolder}</h2><p>{t.dataHint}</p></div></div><div className="path-field"><input aria-label={t.dataFolder} disabled={saving} value={draft.dataRoot} onChange={event => updateDraft('dataRoot', event.target.value)} spellCheck={false} required /><button type="button" className="button secondary" disabled={saving} onClick={() => void chooseDataRoot()}><Folder size={16} />{t.browse}</button></div><StorageSettings settings={draft} de={language === 'de'} disabled={saving} onChange={value => updateDraft('storageOverrides', value)} chooseFolder={chooseFolder} /><details className="storage-paths"><summary>{t.dataPaths}<ChevronDown size={14} /></summary><dl>{pathKeys.map(({ key, label }) => <div key={key}><dt>{t[label]}</dt><dd>{snapshot.paths[key]}</dd></div>)}<div><dt>{t.database}</dt><dd>{snapshot.databasePath}</dd></div></dl></details></section><CoreSettings settings={draft} de={language==='de'} disabled={saving} onChange={patch=>{if(canEditSettings())setDraft(value=>value?{...value,...patch}:value);}}/><ShortcutSettings value={draft.shortcuts} de={language === 'de'} disabled={saving} onChange={value => updateDraft('shortcuts', value)} /><StorageMaintenance settings={draft} de={language === 'de'} disabled={saving} dirty={dirty} onChange={patch => { if (canEditSettings()) setDraft(value => value ? { ...value, ...patch } : value); }} /><div className="settings-actions"><span>{dirty ? t.unsaved : ''}</span><button type="button" className="button secondary" disabled={!dirty || saving} onClick={discardSettings}>{t.discard}</button><button className="button primary" type="submit" disabled={!dirty || saving || !draft.dataRoot.trim()}>{saving ? <LoaderCircle size={16} className="spin" /> : <Check size={16} />}{saving ? t.saving : t.save}</button></div></form><LearnedPreferences de={language==='de'} disabled={saving}/><section className="panel settings-section"><div className="settings-section-title"><Terminal size={20} /><div><h2>{t.diagnostics}</h2><p>{t.diagnosticsText}</p></div></div><button className="button secondary" disabled={logsBusy} onClick={() => void toggleLogs()}>{logsBusy ? <LoaderCircle size={16} className="spin" /> : <Terminal size={16} />}{logs === null ? t.showLogs : t.hideLogs}</button>{logs !== null && <pre className="log-output" tabIndex={0}>{logs || t.noLogs}</pre>}</section></div>}
        {(page === 'hub' || page === 'models') && <HubPage key={page} language={language} showImage={path => { studio.selectModel(path); setSelectedImageJob(null); setStudioTab('generate'); setPage('studio'); }} mode={page} showDownloads={() => setPage('downloads')} initialRepo={page === 'models' ? initialModelRepo : null} showModels={repo => { setInitialModelRepo(repo ?? null); setPage('models'); }} />}
        {page === 'assistant' && <AssistantPage de={language==='de'} onGallery={()=>setPage('gallery')} onApply={request=>{if(!studio.ready||studio.recovery){setError(language==='de'?'Zuerst den Bild-Arbeitsstand wiederherstellen oder verwerfen.':'First restore or discard the image workspace.');return;}studio.restore(request);setSelectedImageJob(null);setStudioTab('generate');setPage('studio');}}/>}
        {page === 'downloads' && <DownloadsPage language={language} />}
        {page === 'studio' && <><div className="studio-tabs" role="tablist" aria-label={language==='de'?'Studio-Bereich':'Studio workspace'}>{(['generate','canvas','timeline'] as const).map(tab=><button key={tab} role="tab" tabIndex={studioTab===tab?0:-1} className="button secondary" aria-selected={studioTab===tab} onKeyDown={event=>{const tabs=['generate','canvas','timeline'] as const;const index=tabs.indexOf(tab);const next=event.key==='ArrowRight'?tabs[(index+1)%3]:event.key==='ArrowLeft'?tabs[(index+2)%3]:event.key==='Home'?tabs[0]:event.key==='End'?tabs[2]:null;if(next){event.preventDefault();setStudioTab(next);const buttons=event.currentTarget.parentElement?.querySelectorAll<HTMLButtonElement>('[role="tab"]');buttons?.[tabs.indexOf(next)]?.focus();}}} onClick={()=>setStudioTab(tab)}>{tab==='generate'?<Sparkles size={16}/>:tab==='canvas'?<Palette size={16}/>:<Film size={16}/>}<span>{tab==='generate'?(language==='de'?'Bildgenerierung':'Image generation'):tab==='canvas'?(language==='de'?'Bildeditor':'Image editor'):(language==='de'?'Videoschnitt':'Video editor')}</span></button>)}</div>{studioTab === 'generate' && <ImageStudio workspaceLocked={studio.locked} projectDisabled={!projects.ready || projects.busy || !!projects.project?.recovery || studio.recovery} onAddToProject={projects.addImage} shortcuts={snapshot.settings.shortcuts} key={page} language={language} request={studio.request} setRequest={studio.setRequest} selectModel={studio.selectModel} onRestore={request => { studio.restore(request); setSelectedImageJob(null); setStudioTab('generate'); setPage('studio'); }} selectedJob={selectedImageJob} disabled={!studio.ready || studio.recovery}  />}{studioTab==='canvas'&&<PrivacyGate de={language==='de'} blocked={privacy.status.projectLocked}><CanvasStudio workspace={creative} projects={projects} de={language==='de'}/></PrivacyGate>} {studioTab==='timeline'&&<PrivacyGate de={language==='de'} blocked={privacy.status.projectLocked}><TimelineStudio workspace={creative} projects={projects} de={language==='de'}/></PrivacyGate>}</>}
        {page === 'gallery' && <Gallery onReference={reference=>{if(!studio.ready||studio.recovery)return;studio.setRequest(r=>({...r,reference,width:reference.width,height:reference.height}));setSelectedImageJob(null);setStudioTab('generate');setPage('studio');}} onAddEdit={async(selection,ops)=>{if(!await projects.addEdit(selection,ops))return null;const p=projects.get()!;const a=p.assets.at(-1)!;return {id:p.id,assetId:a.id,sha256:a.sha256};}} onSaveEdit={projects.saveEdit} maxUndo={snapshot.settings.maxUndo} projectDisabled={!projects.ready || projects.busy || !!projects.project?.recovery || studio.recovery} onAddToProject={projects.addGallery} shortcuts={snapshot.settings.shortcuts} language={language} restoreDisabled={!studio.ready || studio.recovery} onRestore={request => { studio.restore(request); setSelectedImageJob(null); setStudioTab('generate'); setPage('studio'); }} />}
        {!['home', 'jobs', 'settings', 'hub', 'models', 'downloads', 'studio', 'gallery', 'assistant'].includes(page) && <PlannedPage page={page as Exclude<Page, 'home' | 'jobs' | 'settings'>} t={t} goHome={() => setPage('home')} />}
        </div>
      </main><footer className="statusbar"><span><HardDrive size={12} />{snapshot.settings.dataRoot}</span><span>{Math.round(snapshot.settings.uiScale * 100)} %<i />v{snapshot.version}</span></footer>
    </div>
    {dropTarget && <div className="file-drop-notice" role="status">{language === 'de' ? `Dateien in ${dropTarget === 'project' ? 'Projekt' : 'Galerie'} kopieren` : `Copy files to ${dropTarget}`}</div>}
    {exitPrompt && <ExitDialog prompt={exitPrompt} language={language} onChoose={(choice, save) => { setExitPrompt(null); exitPrompt.resolve(choice, save); }} />}
    {exitBusy && <div className="modal-backdrop" role="status"><div className="setup-dialog">{language === 'de' ? 'Bilder sichern und Anwendung beenden …' : 'Finishing images and closing …'}</div></div>}
    {error && <div className="error-toast" role="alert"><TriangleAlert size={19} /><div><strong>{t.operationError}</strong><p>{error}</p></div><IconButton title={t.dismiss} onClick={() => setError('')}><X size={16} /></IconButton></div>}
    {toast && <div className="toast" role="status"><CheckCircle2 size={17} />{toast}</div>}
    {!snapshot.settings.setupComplete && <Setup snapshot={snapshot} t={t} language={language} onLanguage={setLanguage} onSave={persist} busy={saving} canEdit={canEditSettings} chooseFolder={chooseFolder} />}
  </div>;
}
