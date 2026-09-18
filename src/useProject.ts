import {editorActivity} from './editor-state';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useRef, useState } from 'react';
import { open, save, confirm } from '@tauri-apps/plugin-dialog';
import { projectApi, projectError, type StudioProject, type RecoveryPoint, type RecentProject, type ProjectGallerySelection } from './project-api';
import { defaultImageRequest, type useImageWorkspace } from './useImageWorkspace';
import type {EditOperation,ProjectEditQuery} from './editor-state';
const filters = [{ name: 'Local Studio', extensions: ['localstudio'] }];

export function useProject(enabled: boolean, studio: ReturnType<typeof useImageWorkspace>, de: boolean, directory: string) {
  const [project, setProject] = useState<StudioProject | null>(null);
  const [ready, setReady] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const [recent, setRecent] = useState<RecentProject[]>([]);
  const [history, setHistory] = useState<RecoveryPoint[]>([]);
  const current = useRef(project);
  const context = useRef({ studio, de, directory }); context.current = { studio, de, directory };
  const working = useRef(false);
  const pending = useRef<Promise<unknown>>(Promise.resolve());
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const accept = (value: StudioProject | null, restore = false) => {
    current.current = value; setProject(value);
    if (restore && value) context.current.studio.restore(value.request ?? defaultImageRequest);
    return value;
  };
  useEffect(() => {
    if (!enabled) return;
    let live = true;
    void Promise.all([projectApi.snapshot(), projectApi.history(), projectApi.recent()]).then(([value, points, recent]) => { if (live) { accept(value); setHistory(points); setRecent(recent); setReady(true); } }).catch(e => { if (live) setError(projectError(e, context.current.de)); });
    return () => { live = false; if (timer.current) clearTimeout(timer.current); };
  }, [enabled]);
  async function sync() {
    const p = current.current; const workspace = context.current.studio;
    if (p && !p.locked && !p.recovery && workspace.ready && !workspace.recovery && JSON.stringify(p.request) !== JSON.stringify(workspace.request)) accept(await projectApi.update(p.id, workspace.request));
  }
  useEffect(() => {
    if (!ready || busy || !project || project.locked || project.recovery || studio.recovery) return;
    timer.current = setTimeout(() => {
      const task = pending.current.catch(() => {}).then(sync);
      pending.current = task;
      void task.catch(e => setError(projectError(e, context.current.de)));
    }, 450);
    return () => { if (timer.current) clearTimeout(timer.current); };
  }, [studio.request, ready, busy, project?.id, project?.recovery, studio.recovery]);
  async function run(action: () => Promise<boolean | void>) {
    if (working.current || !ready) return false;
    working.current = true; setBusy(true); setError(''); setNotice('');
    if (timer.current) clearTimeout(timer.current);
    const task = (async () => { await pending.current; await editorActivity.flushCreative?.(); await context.current.studio.flush(); await sync(); return (await action()) !== false; })();
    pending.current = task;
    try { return await task; }
    catch (e) { setError(projectError(e, context.current.de)); return false; }
    finally { working.current = false; setBusy(false); pending.current = Promise.resolve(); void projectApi.recent().then(setRecent).catch(()=>{}); void projectApi.history().then(setHistory).catch(e => setError(projectError(e, context.current.de))); }
  }
  async function replaceConfirmed() {
    if (current.current?.dirty) return confirm(de ? 'Ungespeicherte Projektänderungen verwerfen? Die letzte gespeicherte Datei bleibt erhalten.' : 'Discard unsaved project changes? The last saved file is kept.', { title: 'Local Studio', kind: 'warning' });
    if (!current.current && (context.current.studio.request.prompt || context.current.studio.request.negativePrompt)) return confirm(de ? 'Aktuelle Studio-Eingaben durch das Projekt ersetzen?' : 'Replace current Studio inputs with the project?', { title: 'Local Studio' });
    return true;
  }
  const saveProject = (as = false) => run(async () => {
    if (context.current.studio.recovery || current.current?.recovery) throw 'project_inactive';
    if (!current.current) accept(await projectApi.create(de ? 'Unbenannt' : 'Untitled', context.current.studio.request, false));
    const p = current.current!;
    const path = !as && p.path ? p.path : await save({ title: de ? 'Projekt speichern' : 'Save project', defaultPath: p.path ?? `${context.current.directory}/${p.name.replace(/[<>:"/\\|?*]/g, '_')}.localstudio`, filters });
    if (!path) return false;
    const sourceReference=current.current?.request?.reference;
    const saved=await projectApi.save(path);accept(saved);
    const storedReference=saved.request?.reference;
    if(sourceReference&&storedReference&&sourceReference.sha256===storedReference.sha256&&(sourceReference.path!==storedReference.path||sourceReference.mask?.path!==storedReference.mask?.path)){
      context.current.studio.setRequest(r=>r.reference?.sha256===sourceReference.sha256&&r.reference.path===sourceReference.path?{...r,reference:{...r.reference,path:storedReference.path,mask:r.reference.mask&&sourceReference.mask&&storedReference.mask&&r.reference.mask.sha256===sourceReference.mask.sha256?{...r.reference.mask,path:storedReference.mask.path}:r.reference.mask}}:r);
    }
    setNotice(de ? 'Projekt gespeichert.' : 'Project saved.');
  });
  async function ensureProject() {
    if (context.current.studio.recovery || current.current?.recovery) throw 'project_inactive';
    if (!current.current) accept(await projectApi.create(de ? 'Unbenannt' : 'Untitled', context.current.studio.request, false));
    return current.current!;
  }
  async function addFiles(sources: string[]) { await ensureProject(); accept(await projectApi.add(sources)); setNotice(de ? `${sources.length} Medien ins Projekt übernommen.` : `${sources.length} media files added to project.`); }
  const openPath = (path:string) => run(async()=>{if(!await replaceConfirmed())return false;const p=await projectApi.open(path,true);if(context.current.studio.recovery)await context.current.studio.recover();accept(p,true);});
  const openExternal = useRef(openPath);openExternal.current=openPath;
  useEffect(()=>{if(!ready||busy||!studio.ready)return;let live=true;let processing=false;let off:(()=>void)|undefined;
    const take=async()=>{if(processing||working.current||!live||document.querySelector('dialog[open], [aria-modal="true"]'))return;processing=true;try{const path=await projectApi.takeOpen();if(path&&live&&!working.current&&!document.querySelector('dialog[open], [aria-modal="true"]')){await openExternal.current(path);await projectApi.ackOpen(path);}}catch(e){setError(projectError(e,context.current.de));}finally{processing=false;}};
    void listen('project-open-requested',()=>void take()).then(fn=>{if(!live){fn();return;}off=fn;void take();}).catch(e=>setError(projectError(e,context.current.de)));
    const poll=setInterval(()=>void take(),1500);return()=>{live=false;off?.();clearInterval(poll);};
  },[ready,busy,studio.ready]);
  return {
    project, ready, busy, error, notice, history, recent, openPath,
    acceptCreative:accept, prepareCreative:()=>run(async()=>{await ensureProject();}),
    addEdit:(selection:ProjectGallerySelection,operations:EditOperation[])=>run(async()=>{const p=await ensureProject();accept(await projectApi.addEdit(p.id,selection,operations));setNotice(de?'Original und Bearbeitung im Projektarbeitsstand. Projektdatei mit Strg+S speichern.':'Original and edits added to project workspace. Save the project file with Ctrl+S.');}),
    saveEdit:(query:ProjectEditQuery,expected:EditOperation[],operations:EditOperation[])=>run(async()=>{accept(await projectApi.saveEdit(query,expected,operations));}),
    forgetRecent: (path:string)=>run(async()=>{await projectApi.forgetRecent(path);}), save: saveProject,
    rename: (name: string) => run(async () => { if (current.current) accept(await projectApi.rename(current.current.id, name)); }),
    restoreMedia: (id: string) => run(async () => { accept(await projectApi.restoreMedia(id)); }),
    checkpoint: () => run(async () => { await projectApi.checkpoint(); setNotice(de ? 'Wiederherstellungspunkt erstellt.' : 'Recovery point created.'); }),
    restorePoint: (id: number) => run(async () => { if (!await replaceConfirmed()) return false; const restored = await projectApi.restorePoint(id, true); if (context.current.studio.recovery) await context.current.studio.recover(); accept(restored, true); }),
    addFiles: (paths: string[]) => run(() => addFiles(paths)),
    addImage: (jobId: string) => run(async () => { const p = await ensureProject(); accept(await projectApi.addImage(p.id, jobId)); setNotice(de ? 'Bild ins Projekt übernommen.' : 'Image added to project.'); }),
    addGallery: (selection: ProjectGallerySelection) => run(async () => { const p = await ensureProject(); accept(await projectApi.addGallery(p.id, selection)); setNotice(de ? `${selection.targets.length} Medien ins Projekt übernommen.` : `${selection.targets.length} media files added to project.`); }),
    get: () => current.current,
    flush: async () => { await pending.current; if (!await run(async () => {})) throw new Error(de ? 'Projekt konnte nicht gesichert werden.' : 'Unable to preserve project workspace.'); },
    create: (name: string) => run(async () => { if (current.current && !await replaceConfirmed()) return false; accept(await projectApi.create(name, context.current.studio.request, true)); }),
    open: () => run(async () => { const path = await open({ multiple: false, filters }); if (!path || !await replaceConfirmed()) return false; accept(await projectApi.open(path, true), true); }),
    close: () => run(async () => { if (!await replaceConfirmed()) return false; await projectApi.close(true); accept(null); }),
    recover: () => run(async () => { const p = await projectApi.recover(); if (context.current.studio.recovery) await context.current.studio.recover(); accept(p, true); }),
    add: () => run(async () => { const sources = await open({ multiple: true, filters: [{ name: de ? 'Medien' : 'Media', extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif', 'bmp', 'avif', 'mp4', 'webm', 'mov', 'mkv', 'mp3', 'wav', 'flac', 'ogg', 'm4a'] }] }); if (sources?.length) accept(await projectApi.add(sources)); }),
    remove: (id: string) => run(async () => { accept(await projectApi.remove(id)); }),
    relink: () => run(async () => { const path = await open({ multiple: false, filters: [{ name: 'Safetensors', extensions: ['safetensors'] }] }); if (path) accept(await projectApi.relink(path), true); }),
    exportGallery: () => run(async () => { const report = await projectApi.exportGallery(); setNotice(de ? `${report.imported.length} Medien in Galerie kopiert. ${report.errors.length} Fehler.` : `${report.imported.length} media files copied to gallery. ${report.errors.length} errors.`); if (report.errors.length) setError(report.errors.join(' · ')); }),
  };
}
