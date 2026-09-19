import {useEffect,useRef,useState} from 'react';
import {updateApi,updateError,type UpdateStatus} from './update-api';

export default function UpdateDialog({de,version,automatic,onAutomatic,onInstall,onClose}:{de:boolean;version:string;automatic:boolean;onAutomatic:(value:boolean)=>void;onInstall:()=>void;onClose:()=>void}){
 const dialog=useRef<HTMLDialogElement>(null);
 const live=useRef(false),pending=useRef(false),installRequested=useRef(false);
 const [status,setStatus]=useState<UpdateStatus|null>(null),[error,setError]=useState(''),[acting,setActing]=useState(false);
 const t=(a:string,b:string)=>de?a:b;
 useEffect(()=>{
  const el=dialog.current;el?.showModal();live.current=true;
  let timer:ReturnType<typeof setTimeout>;
  const poll=async()=>{
   try{const next=await updateApi.status();if(live.current)setStatus(next);}
   catch(e){if(live.current)setError(updateError(e,de));}
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
 function close(){
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
 return <dialog ref={dialog} className="image-exit-dialog update-dialog" aria-label={t('Software-Updates','Software updates')} onCancel={e=>{e.preventDefault();close();}}>
  <h2>{offered?t('Update verfügbar','Update available'):t('Local Studio aktualisieren','Update Local Studio')}</h2>
  <p>{t('Installierte Version','Installed version')}: {version} · {status?.portable?'Portable':t('Installiert','Installed')}</p>
  <label className="project-exit-option"><input type="checkbox" checked={automatic} onChange={e=>onAutomatic(e.target.checked)}/>{t('Beim Start automatisch nach Updates suchen','Automatically check for updates at startup')}</label>
  {status?.phase==='current'&&<p role="status">{t('Du verwendest die aktuelle Version.','You are using the latest version.')}</p>}
  {status?.phase==='checking'&&<p role="status">{t('GitHub-Release wird geprüft …','Checking the GitHub release …')}</p>}
  {status?.latest&&<><h3>{t('GitHub-Version','GitHub version')}: {status.latest.version}</h3><pre className="update-notes">{status.latest.notes}</pre></>}
  {(error||status?.error)&&<p className="notice warning" role="alert">{error||updateError(status?.error,de)}</p>}
  {status?.phase==='downloading'&&<><p role="status">{t('Update wird heruntergeladen und geprüft …','Downloading and verifying update …')}</p><progress max={status.total||1} value={status.received}/><p>{(status.received/1048576).toFixed(1)} / {(status.total/1048576).toFixed(1)} MiB</p></>}
  {offered&&!status?.portable&&<p>{t('Das Update wird geprüft und installiert. Local Studio wird danach neu gestartet. Ungespeicherte Arbeit wird vor dem Beenden berücksichtigt.','The update will be verified and installed, then Local Studio will restart. Unsaved work will be handled before closing.')}</p>}
  {status?.portable&&<p>{t('ZIP herunterladen, App schließen und Programmdateien ersetzen. Local-Studio-Data behalten. Die portable Version installiert nichts automatisch.','Download the ZIP, close the app and replace the program files. Keep Local-Studio-Data. Portable builds never install updates automatically.')}</p>}
  <div className="image-dialog-actions">
   <button className="button secondary" onClick={close}>{t('Abbrechen','Cancel')}</button>
   {!offered&&<button className="button secondary" disabled={busy} onClick={()=>void action(updateApi.check)}>{t('Erneut prüfen','Check again')}</button>}
   {offered&&(status?.portable?<button className="button primary" disabled={acting} onClick={()=>void action(updateApi.openDownload)}>{t('Download auf GitHub öffnen','Open download on GitHub')}</button>:<button className="button primary" disabled={acting} onClick={()=>void install()}>{t('Update installieren','Install update')}</button>)}
  </div>
  <p className="hub-hint">{t('Die Prüfung verbindet sich mit GitHub. Medien, Prompts und Modelldateien werden nicht übertragen.','Checking connects to GitHub. Media, prompts and model files are not sent.')}</p>
 </dialog>;
}