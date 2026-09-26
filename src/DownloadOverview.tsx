import {useEffect, useRef, useState} from 'react';
import {createPortal} from 'react-dom';
import {Download, X} from 'lucide-react';
import {activeDownload, downloads} from './download-api';
import {updateApi} from './update-api';
import {comfyApi} from './comfy-api';
import {formatBytes} from './helpers';
import {useWindowFrame} from './WindowFrame';
import type {Language} from './types';
import './download-overview.css';

export function downloadEta(remaining:number, speed:number, de:boolean):string {
  if (!Number.isFinite(speed) || speed <= 0 || !Number.isFinite(remaining) || remaining <= 0)
    return de ? 'Restzeit wird ermittelt …' : 'Estimating time remaining …';
  const seconds = Math.ceil(remaining / speed);
  if (seconds < 60) return de ? 'Noch ca. < 1 Min.' : 'About < 1 min left';
  const minutes = Math.ceil(seconds / 60);
  if (minutes < 60) return de ? `Noch ca. ${minutes} Min.` : `About ${minutes} min left`;
  const hours = Math.floor(minutes / 60), rest = minutes % 60;
  return de ? `Noch ca. ${hours} Std.${rest ? ` ${rest} Min.` : ''}` : `About ${hours} h${rest ? ` ${rest} min` : ''} left`;
}

interface Entry {id:string; name:string; status:string; total:number; received:number; speed:number}
const labels:Record<string,[string,string]> = {
  queued:['Wartet auf Start','Waiting to start'], downloading:['Wird heruntergeladen','Downloading'],
  resolving:['Offizielles Release wird ermittelt','Resolving official release'],
  verifying:['Dateien werden geprüft','Verifying files'], installing:['Dateien werden übernommen','Installing files'],
  pausing:['Wird pausiert','Pausing'], cancelling:['Wird abgebrochen','Cancelling'],
};

export default function DownloadOverview({language, disabled=false, onOpenDownloads}:{language:Language; disabled?:boolean; onOpenDownloads:()=>void}) {
  const {menuHost} = useWindowFrame(), de = language === 'de';
  const [shown,setShown] = useState(false), [entries,setEntries] = useState<Entry[]>([]);
  const [loaded,setLoaded] = useState(false), [error,setError] = useState(false);
  const dialog = useRef<HTMLDialogElement>(null), button = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (!shown) return;
    let live=true, timer:ReturnType<typeof setTimeout>;
    let previous:{version:string; bytes:number; at:number}|null=null;
    const refresh = async () => {
      const [models,update,comfy] = await Promise.allSettled([downloads.list(),updateApi.status(),comfyApi.status()]);
      if (!live) return;
      const rows:Entry[] = models.status === 'fulfilled' ? models.value.filter(activeDownload).filter(d=>!d.verifyOnly).map(d=>({
        id:d.id,name:d.repo,status:d.status,total:d.totalBytes,received:d.downloadedBytes,speed:d.bytesPerSecond,
      })) : [];
      if (update.status === 'fulfilled' && update.value.phase === 'downloading') {
        const s=update.value, at=performance.now(), version=s.latest?.version ?? '';
        const speed=previous && previous.version===version && at>previous.at ? Math.max(0,(s.received-previous.bytes)*1000/(at-previous.at)) : 0;
        previous={version,bytes:s.received,at};
        rows.push({id:'app-update',name:`Local Studio ${version}`,status:'downloading',total:s.total,received:s.received,speed});
      } else previous=null;
      if (comfy.status === 'fulfilled' && ['resolving','downloading','verifying','installing'].includes(comfy.value.install.phase)) {
        const s=comfy.value.install;
        rows.push({id:'comfyui-install',name:'ComfyUI Portable',status:s.phase,total:s.totalBytes,received:s.receivedBytes,speed:s.bytesPerSecond});
      }
      setEntries(rows); setLoaded(true); setError(models.status==='rejected'||update.status==='rejected'||comfy.status==='rejected');
      timer=setTimeout(()=>void refresh(),1000);
    };
    void refresh();
    return () => {live=false;clearTimeout(timer);};
  },[shown]);
  useEffect(() => {
    const element=dialog.current;
    if (shown) element?.showModal(); else element?.close();
    return () => {element?.close();};
  },[shown]);
  function close() {setShown(false);button.current?.focus();}
  function toggle() {
    if (shown) {close();return;}
    if (document.querySelector('dialog[open],[aria-modal="true"]')) return;
    setEntries([]);setLoaded(false);setError(false);setShown(true);
  }
  return <>
    {menuHost && createPortal(<button ref={button} type="button" className="window-download-button" title={de?'Laufende Downloads':'Active downloads'} aria-label={de?'Laufende Downloads':'Active downloads'} aria-haspopup="dialog" aria-expanded={shown} aria-controls="download-overview" disabled={disabled} onClick={toggle}><Download size={17}/></button>,menuHost)}
    <dialog id="download-overview" ref={dialog} className="download-overview" aria-labelledby="download-overview-title" onCancel={()=>setShown(false)} onClose={()=>{setShown(false);queueMicrotask(()=>button.current?.focus());}} onClick={e=>{if(e.target===e.currentTarget){const r=e.currentTarget.getBoundingClientRect();if(e.clientX<r.left||e.clientX>r.right||e.clientY<r.top||e.clientY>r.bottom)close();}}}>
      <div className="download-overview-heading"><h2 id="download-overview-title">{de?'Laufende Downloads':'Active downloads'}</h2><button autoFocus type="button" className="icon-button" aria-label={de?'Schließen':'Close'} onClick={close}><X size={18}/></button></div>
      <div className="download-overview-list">
        {error && <p role="status">{de?'Downloadstatus teilweise nicht erreichbar. Wird erneut versucht …':'Some download statuses are unavailable. Retrying …'}</p>}
        {!loaded && <p role="status">{de?'Downloads werden geladen …':'Loading downloads …'}</p>}
        {loaded && !error && !entries.length && <p>{de?'Keine laufenden Downloads.':'No active downloads.'}</p>}
        {entries.map(entry=>{
          const transferring=entry.status==='downloading', known=entry.total>0;
          const received=Math.max(0,Math.min(entry.received,entry.total));
          return <article className="download-overview-item" key={entry.id}>
            <strong title={entry.name}>{entry.name}</strong>
            <div className="download-overview-meta"><span>{labels[entry.status]?.[de?0:1]??entry.status}</span>{known && <span>{Math.floor(received/entry.total*100)} %</span>}</div>
            <progress max={known?entry.total:1} value={known?received:undefined} aria-label={entry.name}/>
            <div className="download-overview-meta"><span>{formatBytes(entry.received,language)}{known?` / ${formatBytes(entry.total,language)}`:''}</span>{transferring && entry.speed>0 && <span>{formatBytes(entry.speed,language)}/s</span>}</div>
            <small>{transferring ? downloadEta(known?entry.total-received:0,entry.speed,de) : de?'Restzeit für diesen Schritt nicht verfügbar':'Time remaining unavailable for this step'}</small>
          </article>;
        })}
      </div>
      <button type="button" className="button secondary download-overview-all" onClick={()=>{close();onOpenDownloads();}}>{de?'Alle Downloads':'All downloads'}</button>
    </dialog>
  </>;
}
