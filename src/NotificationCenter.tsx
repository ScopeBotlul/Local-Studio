import {useEffect, useRef, useState} from 'react';
import {createPortal} from 'react-dom';
import {Bell, CheckCircle2, Info, TriangleAlert, X} from 'lucide-react';
import {useWindowFrame} from './WindowFrame';
import type {Language} from './types';
import './notification-center.css';

export type NotificationKind = 'info' | 'success' | 'warning' | 'error';

export interface AppNotification {
  id: string;
  kind: NotificationKind;
  title: string;
  message: string;
  actionLabel?: string;
  onAction?: () => void | Promise<void>;
  dismissLabel?: string;
  onDismiss?: () => void | Promise<void>;
}

const icons: Record<NotificationKind, typeof Info> = {
  info: Info,
  success: CheckCircle2,
  warning: TriangleAlert,
  error: TriangleAlert,
};

export function notificationButtonLabel(count: number, de: boolean) {
  if (!count) return de ? 'Keine Benachrichtigungen' : 'No notifications';
  return de ? `${count} Benachrichtig${count === 1 ? 'ung' : 'ungen'}` : `${count} notification${count === 1 ? '' : 's'}`;
}

export default function NotificationCenter({language, disabled=false, entries, onError}:{language:Language; disabled?:boolean; entries:AppNotification[]; onError:(error:unknown)=>void}) {
  const {menuHost}=useWindowFrame(), de=language==='de';
  const [shown,setShown]=useState(false), [busy,setBusy]=useState<string|null>(null);
  const dialog=useRef<HTMLDialogElement>(null), button=useRef<HTMLButtonElement>(null);
  useEffect(()=>{
    const element=dialog.current;
    if(shown) element?.showModal(); else element?.close();
    return()=>element?.close();
  },[shown]);
  function close(){setShown(false);button.current?.focus();}
  function toggle(){
    if(shown){close();return;}
    if(document.querySelector('dialog[open],[aria-modal="true"]'))return;
    setShown(true);
  }
  async function run(id:string, action:()=>void|Promise<void>){
    if(busy)return;
    setBusy(id);
    try{await action();}catch(error){onError(error);}finally{setBusy(null);}
  }
  const label=notificationButtonLabel(entries.length,de);
  return <>
    {menuHost&&createPortal(<button ref={button} type="button" className="window-notification-button" title={label} aria-label={label} aria-haspopup="dialog" aria-expanded={shown} aria-controls="notification-center" disabled={disabled} onClick={toggle}>
      <Bell size={17}/>{entries.length>0&&<span className="window-notification-badge" aria-hidden="true">{entries.length>99?'99+':entries.length}</span>}
    </button>,menuHost)}
    <dialog id="notification-center" ref={dialog} className="notification-center" aria-labelledby="notification-center-title" onCancel={()=>setShown(false)} onClose={()=>{setShown(false);queueMicrotask(()=>button.current?.focus());}} onClick={event=>{if(event.target===event.currentTarget){const rect=event.currentTarget.getBoundingClientRect();if(event.clientX<rect.left||event.clientX>rect.right||event.clientY<rect.top||event.clientY>rect.bottom)close();}}}>
      <div className="notification-center-heading"><div><h2 id="notification-center-title">{de?'Benachrichtigungen':'Notifications'}</h2><small>{entries.length?label:(de?'Alles erledigt':'All caught up')}</small></div><button autoFocus type="button" className="icon-button" aria-label={de?'Schließen':'Close'} onClick={close}><X size={18}/></button></div>
      <div className="notification-center-list">
        {!entries.length&&<div className="notification-center-empty"><CheckCircle2 size={28}/><strong>{de?'Keine offenen Hinweise':'No pending notices'}</strong><p>{de?'Wichtige Hinweise und Wiederherstellungen erscheinen hier.':'Important notices and recovery actions appear here.'}</p></div>}
        {entries.map(entry=>{const Icon=icons[entry.kind];return <article className={`notification-center-item ${entry.kind}`} key={entry.id}>
          <Icon size={18}/><div><strong>{entry.title}</strong><p>{entry.message}</p>{(entry.onAction||entry.onDismiss)&&<div className="notification-center-actions">{entry.onAction&&<button type="button" className="button primary" disabled={!!busy} onClick={()=>void run(entry.id,entry.onAction!)}>{busy===entry.id?(de?'Wird ausgeführt …':'Working …'):entry.actionLabel}</button>}{entry.onDismiss&&<button type="button" className="button secondary" disabled={!!busy} onClick={()=>void run(`${entry.id}:dismiss`,entry.onDismiss!)}>{busy===`${entry.id}:dismiss`?(de?'Wird ausgeführt …':'Working …'):(entry.dismissLabel??(de?'Ausblenden':'Dismiss'))}</button>}</div>}</div>
        </article>;})}
      </div>
    </dialog>
  </>;
}
