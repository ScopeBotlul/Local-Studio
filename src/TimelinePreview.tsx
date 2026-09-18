import {useEffect,useState} from 'react';
import type {Timeline} from './creative-state';
import {creativeError} from './creative-state';
import {mediaApi,type Frame} from './media-tools';
export default function TimelinePreview({projectId,document,time,proxies,de,revision}:{projectId:string;document:Timeline;time:number;proxies:boolean;de:boolean;revision:number}){
 const [frame,setFrame]=useState<Frame|null>(null),[busy,setBusy]=useState(false),[error,setError]=useState('');
 useEffect(()=>{let live=true;setBusy(true);const timer=setTimeout(()=>{void mediaApi.frame(projectId,document,time,proxies).then(v=>{if(live){setFrame(v);setError('');}}).catch(e=>{if(live&&String(e)!=='video_cancelled')setError(creativeError(e,de));}).finally(()=>{if(live)setBusy(false);});},100);return()=>{live=false;clearTimeout(timer);};},[projectId,document,time,proxies,revision]);
 return <div className="scrub-preview"><div className="scrub-picture" aria-busy={busy}>{frame?<img src={frame.dataUrl} alt={de?'Vorschau am Abspielkopf':'Playhead preview'}/>:<p>{de?'Einzelbild wird berechnet …':'Computing frame …'}</p>}</div><p role="status">{busy?(de?'Vorschau wird aktualisiert …':'Updating preview …'):frame?`${frame.time.toFixed(3)} s · ${frame.proxyAssets.length?(de?'Proxy-Vorschau':'Proxy preview'):(de?'Originalvorschau':'Original preview')}`:''}</p>{error&&<p role="alert">{error}</p>}</div>;
}
