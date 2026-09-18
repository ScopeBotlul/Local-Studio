import {useEffect,useState} from 'react';
import {creativeApi,creativeError,type VideoJob} from './creative-state';
import {ai,aiError,aiPhase,type SpeechJob} from './ai-api';
import {mediaApi,type MediaTask} from './media-tools';
export default function VideoJobs({de}:{de:boolean}) {
 const [speech,setSpeech]=useState<SpeechJob[]>([]);
 const [jobs,setJobs]=useState<VideoJob[]>([]),[task,setTask]=useState<MediaTask|null>(null);
 useEffect(()=>{let live=true;const poll=()=>{void ai.jobs().then(j=>{if(live)setSpeech(j);}).catch(()=>{});void creativeApi.jobs().then(j=>{if(live)setJobs(j);}).catch(()=>{});void mediaApi.status().then(t=>{if(live)setTask(t);}).catch(()=>{});};poll();const timer=setInterval(poll,1000);return()=>{live=false;clearInterval(timer);};},[]);
 if(!jobs.length&&!task&&!speech.length)return null;
 return <section className="panel"><h2>{de?'Video-, Medien- und Sprachaufträge':'Video, media and speech jobs'}</h2>
 {speech.map(j=><article className="video-job" key={j.id}><strong>{j.name} · {de?'Transkription':'Transcription'} · {aiPhase(j.status==='running'?j.phase:j.status,de)}</strong>{j.status==='running'&&<><progress max={1} value={j.progress??undefined}/><button className="button secondary" onClick={()=>void ai.cancelSpeech(j.id)}>{de?'Abbrechen':'Cancel'}</button></>}{j.error&&<p className="inline-warning">{aiError(j.error,de)}</p>}{j.status==='completed'&&<p>{j.captions.length} {de?'Untertitel · im Videoschnitt prüfen und übernehmen':'captions · review and apply in Video editor'}</p>}</article>)}
 {task&&<article className="video-job"><strong>{task.name} · {task.kind==='proxy'?'Proxy':de?'Wellenform':'Waveform'} · {task.status}</strong>{task.status==='running'&&<><progress max={1} value={task.progress}/><button className="button secondary" onClick={()=>void mediaApi.cancel(task.id)}>{de?'Abbrechen':'Cancel'}</button></>}{task.error&&<p className="inline-warning">{creativeError(task.error,de)}</p>}</article>}
 {jobs.map(j=><article className="video-job" key={j.id}><strong>{j.name} · {j.preview?(de?'Vorschau':'Preview'):'Export'} · {j.status}</strong>{j.status==='running'&&<><progress max={1} value={j.progress}/><button className="button secondary" onClick={()=>void creativeApi.cancel(j.id)}>{de?'Abbrechen':'Cancel'}</button></>}{j.error&&<p className="inline-warning">{creativeError(j.error,de)}</p>}{j.output&&<p className="project-path">{j.output}</p>}</article>)}
 </section>;
}
