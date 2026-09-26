import {useEffect,useRef,useState} from 'react';
import {updateApi,updateError,type UpdateStatus} from './update-api';
import {comfyApi,comfyError,type ComfyStatus} from './comfy-api';

export default function UpdateDialog({de,version,automatic,onAutomatic,onInstall,onClose}:{de:boolean;version:string;automatic:boolean;onAutomatic:(value:boolean)=>void;onInstall:()=>void;onClose:()=>void}){
 const dialog=useRef<HTMLDialogElement>(null);
 const live=useRef(false),pending=useRef(false),installRequested=useRef(false);
 const [status,setStatus]=useState<UpdateStatus|null>(null),[comfy,setComfy]=useState<ComfyStatus|null>(null),[error,setError]=useState(''),[comfyErrorText,setComfyErrorText]=useState(''),[acting,setActing]=useState(false),[comfyActing,setComfyActing]=useState(false);
 const t=(a:string,b:string)=>de?a:b;
 useEffect(()=>{
  const el=dialog.current;el?.showModal();live.current=true;
  let timer:ReturnType<typeof setTimeout>;
  const poll=async()=>{
   const [local,nextComfy]=await Promise.allSettled([updateApi.status(),comfyApi.status()]);
   if(live.current){if(local.status==='fulfilled')setStatus(local.value);else setError(updateError(local.reason,de));if(nextComfy.status==='fulfilled')setComfy(nextComfy.value);else setComfyErrorText(comfyError(nextComfy.reason,de));}
   if(live.current)timer=setTimeout(()=>void poll(),350);
  };
  void poll();
  return()=>{live.current=false;installRequested.current=false;clearTimeout(timer);el?.close();};
 },[]);
 async function action(fn:()=>Promise<unknown>){
  if(pending.current)return;
  pending.current=true;setActing(true);setError('');
  try{await fn();const next=await updateApi.status();if(live.current)setStatus(next);}
  catch(e){if(live.current)setError(updateError(e,de));}
  finally{pending.current=false;if(live.current)setActing(false);}
 }
 async function comfyAction(fn:()=>Promise<ComfyStatus|void>){
  if(comfyActing)return;setComfyActing(true);setComfyErrorText('');
  try{const next=await fn();if(next)setComfy(next);else setComfy(await comfyApi.status());}
  catch(e){setComfyErrorText(comfyError(e,de));}
  finally{if(live.current)setComfyActing(false);}
 }
 function close(){
  if(comfy?.update.phase==='updating')return;
  installRequested.current=false;
  if(pending.current||status?.phase==='downloading')void updateApi.cancel().catch(()=>{});
  live.current=false;
  onClose();
 }
 async function install(){
  if(pending.current||!status||status.portable)return;
  if(status.phase==='ready'){onInstall();return;}
  if(status.phase!=='available')return;
  installRequested.current=true;
  await action(async()=>{
   const next=await updateApi.download();
   if(!live.current)return;
   setStatus(next);
   // Error statuses are returned too; only a verified package may close the app.
   if(installRequested.current&&next.phase==='ready'&&!next.portable){
    installRequested.current=false;
    onInstall();
   }
  });
  installRequested.current=false;
 }
 const busy=acting||!!status&&['checking','downloading','ready','installing'].includes(status.phase);
 const offered=status?.phase==='available'||status?.phase==='ready';
 const comfyOffered=comfy?.update.phase==='available';
 return <dialog ref={dialog} className="image-exit-dialog update-dialog" aria-label={t('Software-Updates','Software updates')} onCancel={e=>{e.preventDefault();close();}}>
  <h2>{offered||comfyOffered?t('Updates verfügbar','Updates available'):t('Update-Center','Update Center')}</h2>
  <label className="project-exit-option"><input type="checkbox" checked={automatic} onChange={e=>onAutomatic(e.target.checked)}/>{t('Beim Start automatisch nach Updates für Local Studio und ComfyUI suchen','Automatically check for Local Studio and ComfyUI updates at startup')}</label>
  <div className="update-columns">
   <section className="update-column">
    <h3>Local Studio</h3><p>{t('Installierte Version','Installed version')}: {version} · {status?.portable?'Portable':t('Installiert','Installed')}</p>
    {status?.phase==='current'&&<p role="status">{t('Aktuelle Version installiert.','Latest version installed.')}</p>}
    {status?.phase==='checking'&&<p role="status">{t('GitHub-Release wird geprüft …','Checking the GitHub release …')}</p>}
    {status?.latest&&<><p><strong>{t('Verfügbare Version','Available version')}: {status.latest.version}</strong></p><pre className="update-notes">{status.latest.notes}</pre></>}
    {(error||status?.error)&&<p className="notice warning" role="alert">{error||updateError(status?.error,de)}</p>}
    {status?.phase==='downloading'&&<><p role="status">{t('Update wird heruntergeladen und geprüft …','Downloading and verifying update …')}</p><progress max={status.total||1} value={status.received}/><p>{(status.received/1048576).toFixed(1)} / {(status.total/1048576).toFixed(1)} MiB</p></>}
    {offered&&!status?.portable&&<p>{t('Local Studio wird nach der geprüften Installation neu gestartet.','Local Studio restarts after the verified installation.')}</p>}
    {status?.portable&&<p>{t('Portable Updates werden weiterhin über das ZIP installiert. Local-Studio-Data behalten.','Portable updates are still installed using the ZIP. Keep Local-Studio-Data.')}</p>}
    <div className="update-column-actions">{!offered&&<button className="button secondary" disabled={busy} onClick={()=>void action(updateApi.check)}>{t('Erneut prüfen','Check again')}</button>}{offered&&(status?.portable?<button className="button primary" disabled={acting} onClick={()=>void action(updateApi.openDownload)}>{t('Download öffnen','Open download')}</button>:<button className="button primary" disabled={acting} onClick={()=>void install()}>{t('Update installieren','Install update')}</button>)}</div>
   </section>
   <section className="update-column">
    <h3>ComfyUI</h3>
    {!comfy?.installed?<p>{t('ComfyUI ist nicht eingerichtet.','ComfyUI is not configured.')}</p>:<><p>{t('Installierte Version','Installed version')}: {comfy.update.installedVersion??comfy.version??t('Unbekannt','Unknown')}</p>{comfy.update.phase==='current'&&<p role="status">{t('Aktuelle Version installiert.','Latest version installed.')}</p>}{comfy.update.phase==='checking'&&<p role="status">{t('Offizielles ComfyUI-Release wird geprüft …','Checking the official ComfyUI release …')}</p>}{comfy.update.latestVersion&&<p><strong>{t('Verfügbare Version','Available version')}: {comfy.update.latestVersion}</strong></p>}{comfyOffered&&<p>{t('Local Studio beendet die verwaltete Engine, führt den offiziellen stabilen Core-Updater aus und startet ComfyUI danach neu. Dabei können erforderliche ComfyUI-Kernabhängigkeiten angepasst werden; Custom Nodes werden nicht aktualisiert.','Local Studio stops the managed engine, runs the official stable core updater and restarts ComfyUI. Required ComfyUI core dependencies may be adjusted; custom nodes are not updated.')}</p>}{comfy.update.phase==='updating'&&<><p role="status">{t('ComfyUI wird direkt aktualisiert …','Updating ComfyUI directly …')}</p><progress aria-label={t('ComfyUI-Update läuft','ComfyUI update in progress')}/>{comfy.update.log&&<pre className="update-notes comfy-update-log">{comfy.update.log}</pre>}</>}</>}
    {(comfyErrorText||comfy?.update.error)&&<p className="notice warning" role="alert">{comfyErrorText||comfyError(comfy?.update.error,de)}</p>}
    <div className="update-column-actions">{comfy?.installed&&<button className="button secondary" disabled={comfyActing||['checking','updating'].includes(comfy.update.phase)} onClick={()=>void comfyAction(comfyApi.checkUpdate)}>{t('Erneut prüfen','Check again')}</button>}{(comfyOffered||comfy?.update.phase==='updating')&&<button className="button primary" disabled={comfyActing||comfy.update.phase==='updating'} onClick={()=>void comfyAction(comfyApi.update)}>{comfy.update.phase==='updating'?t('Update läuft …','Updating …'):t('Jetzt aktualisieren','Update now')}</button>}</div>
   </section>
  </div>
  <div className="image-dialog-actions"><button className="button secondary" disabled={comfy?.update.phase==='updating'} onClick={close}>{t('Schließen','Close')}</button></div>
  <p className="hub-hint">{t('Beide Prüfungen verbinden sich mit GitHub. Medien, Prompts und Modelldateien werden nicht übertragen. Updates werden nie allein durch die automatische Suche installiert.','Both checks connect to GitHub. Media, prompts and model files are not sent. Automatic checking never installs updates by itself.')}</p>
 </dialog>;
}
