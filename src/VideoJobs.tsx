import {useEffect,useState} from 'react';
import {creativeApi,creativeError,type VideoJob} from './creative-state';
import {mediaApi,type MediaTask} from './media-tools';
export default function VideoJobs({de}:{de:boolean}) {
 const [jobs,setJobs]=useState<VideoJob[]>([]),[task,setTask]=useState<MediaTask|null>(null);
 useEffect(()=>{let live=true;const poll=()=>{void creativeApi.jobs().then(j=>{if(live)setJobs(j);}).catch(()=>{});void mediaApi.status().then(t=>{if(live)setTask(t);}).catch(()=>{});};poll();const timer=setInterval(poll,1000);return()=>{live=false;clearInterval(timer);};},[]);
 if(!jobs.length&&!task)return null;
 return <section className="panel"><h2>{de?'Video- und Medienaufträge':'Video and media jobs'}</h2>
 {task&&<article className="video-job"><strong>{task.name} · {task.kind==='proxy'?'Proxy':de?'Wellenform':'Waveform'} · {task.status}</strong>{task.status==='running'&&<><progress max={1} value={task.progress}/><button className="button secondary" onClick={()=>void mediaApi.cancel(task.id)}>{de?'Abbrechen':'Cancel'}</button></>}{task.error&&<p className="inline-warning">{creativeError(task.error,de)}</p>}</article>}
 {jobs.map(j=><article className="video-job" key={j.id}><strong>{j.name} · {j.preview?(de?'Vorschau':'Preview'):'Export'} · {j.status}</strong>{j.status==='running'&&<><progress max={1} value={j.progress}/><button className="button secondary" onClick={()=>void creativeApi.cancel(j.id)}>{de?'Abbrechen':'Cancel'}</button></>}{j.error&&<p className="inline-warning">{creativeError(j.error,de)}</p>}{j.output&&<p className="project-path">{j.output}</p>}</article>)}
 </section>;
}
