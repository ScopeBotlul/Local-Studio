import {useEffect,useRef,useState} from 'react';
import {Bug,ExternalLink,FolderOpen,RefreshCw,ShieldCheck} from 'lucide-react';
import {bugReportApi,bugReportError,type BugReportDraft,type BugReportInput} from './bug-report-api';

export default function BugReportDialog({de,context,onClose}:{de:boolean;context:string;onClose:()=>void}){
 const ref=useRef<HTMLDialogElement>(null);
 const [title,setTitle]=useState('');
 const [description,setDescription]=useState('');
 const [steps,setSteps]=useState('');
 const [draft,setDraft]=useState<BugReportDraft|null>(null);
 const [busy,setBusy]=useState(false);
 const [error,setError]=useState('');
 const [saved,setSaved]=useState('');
 const input:BugReportInput={title,description,steps,uiContext:context};
 const labels:Record<string,string>={
  'App version and installation mode':'App-Version und Installationsmodus',
  'Local Studio settings with redacted paths':'Local-Studio-Einstellungen mit bereinigten Pfaden',
  'Windows, CPU, RAM, GPU, drivers and disk capacity':'Windows, CPU, RAM, GPU, Treiber und Laufwerkskapazität',
  'Job counts without file names or contents':'Auftragsanzahlen ohne Dateinamen oder Inhalte',
  'ComfyUI and update status plus bounded local logs':'ComfyUI- und Update-Status sowie begrenzte lokale Protokolle',
  'Tokens, cookies and passwords':'Tokens, Cookies und Passwörter',
  'Prompts, media and project contents':'Prompts, Medien und Projektinhalte',
  'Model weights and database contents':'Modellgewichte und Datenbankinhalte',
  'Windows user, computer name and personal path segments':'Windows-Benutzer, Computername und persönliche Pfadbestandteile',
 };
 const label=(value:string)=>de?(labels[value]??value):value;
 useEffect(()=>{ref.current?.showModal();void refresh();return()=>ref.current?.close();},[]);
 async function refresh(){setBusy(true);setError('');try{setDraft(await bugReportApi.preview(input));}catch(e){setError(bugReportError(e,de));}finally{setBusy(false);}}
 async function submit(){if(!title.trim()||!description.trim())return;setBusy(true);setError('');try{const result=await bugReportApi.submit(input);setSaved(result.path);setDraft(current=>current?{...current,report:result.report}:current);}catch(e){setError(bugReportError(e,de));}finally{setBusy(false);}}
 return <dialog ref={ref} className="image-exit-dialog bug-report-dialog" aria-label={de?'Fehler melden':'Report a bug'} onCancel={event=>{event.preventDefault();onClose();}}>
  <div className="section-heading"><Bug size={20}/><div><h2>{de?'Fehler melden':'Report a bug'}</h2><p>{de?'Technische Informationen werden automatisch gesammelt.':'Technical information is collected automatically.'}</p></div></div>
  <p>{de?'Local Studio speichert einen vollständigen bereinigten Bericht lokal und öffnet ein vorausgefülltes GitHub-Issue. Prüfe den Bericht und klicke auf GitHub abschließend auf „Submit new issue“.':'Local Studio saves a complete sanitized report locally and opens a prefilled GitHub issue. Review it and finally click “Submit new issue” on GitHub.'}</p>
  <div className="bug-report-fields">
   <label className="field"><span>{de?'Kurzer Titel':'Short title'}</span><input autoFocus maxLength={120} value={title} onChange={event=>setTitle(event.target.value)} placeholder={de?'Was funktioniert nicht?':'What is not working?'}/></label>
   <label className="field"><span>{de?'Beschreibung':'Description'}</span><textarea maxLength={4000} rows={4} value={description} onChange={event=>setDescription(event.target.value)} placeholder={de?'Was ist passiert und was hast du erwartet?':'What happened and what did you expect?'}/></label>
   <label className="field"><span>{de?'Schritte zum Nachstellen':'Steps to reproduce'}</span><textarea maxLength={4000} rows={3} value={steps} onChange={event=>setSteps(event.target.value)} placeholder={'1. …\n2. …'}/></label>
  </div>
  {draft&&<div className="bug-report-scope"><div><h3><ShieldCheck size={16}/>{de?'Wird aufgenommen':'Included'}</h3><ul>{draft.included.map(item=><li key={item}>{label(item)}</li>)}</ul></div><div><h3>{de?'Wird ausgeschlossen':'Excluded'}</h3><ul>{draft.excluded.map(item=><li key={item}>{label(item)}</li>)}</ul></div></div>}
  <details className="bug-report-preview" open><summary>{de?'Berichtsvorschau':'Report preview'}</summary><pre>{draft?.report??(busy?(de?'Diagnosedaten werden gesammelt …':'Collecting diagnostics …'):'')}</pre></details>
  {error&&<p className="notice warning" role="alert">{error}</p>}
  {saved&&<div className="notice success" role="status"><strong>{de?'Bericht lokal gespeichert.':'Report saved locally.'}</strong><code>{saved}</code><p>{de?'GitHub wurde mit dem ausgefüllten Bericht geöffnet. Hänge dort bei Bedarf diese lokale Markdown-Datei an und sende das Issue ab.':'GitHub was opened with the completed report. Attach this local Markdown file there if needed and submit the issue.'}</p></div>}
  <div className="image-dialog-actions bug-report-actions">
   <button type="button" className="button secondary" disabled={busy} onClick={()=>void refresh()}><RefreshCw size={15}/>{de?'Vorschau aktualisieren':'Refresh preview'}</button>
   <button type="button" className="button secondary" disabled={busy} onClick={()=>void bugReportApi.openFolder().catch(e=>setError(bugReportError(e,de)))}><FolderOpen size={15}/>{de?'Berichtordner':'Reports folder'}</button>
   <span className="spacer"/>
   <button type="button" className="button secondary" onClick={onClose}>{de?'Schließen':'Close'}</button>
   <button type="button" className="button primary" disabled={busy||!title.trim()||!description.trim()} onClick={()=>void submit()}><ExternalLink size={15}/>{de?'Speichern und GitHub öffnen':'Save and open GitHub'}</button>
  </div>
 </dialog>;
}
