import {useEffect, useRef, useState} from 'react';
import {ImagePlus, Maximize, Minus, Plus} from 'lucide-react';
import type {ImageJob} from './image-api';
import {imagePhases} from './ImageJobRow';

export default function ImageCanvas({preview, job, width, height, de}: {preview:string; job?:ImageJob; width:number; height:number; de:boolean}) {
  const viewport = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({width:600, height:500});
  const [zoom, setZoom] = useState<number | null>(null);
  const drag = useRef<{x:number;y:number;left:number;top:number}|null>(null);
  useEffect(() => {
    const element = viewport.current;
    if (!element) return;
    const observer = new ResizeObserver(([entry]) => setSize({width:entry.contentRect.width, height:entry.contentRect.height}));
    observer.observe(element); return () => observer.disconnect();
  }, []);
  useEffect(() => {setZoom(null);}, [job?.id]);
  const w = job?.request.width || width || 1024, h = job?.request.height || height || 1024;
  const fit = Math.max(0.05, Math.min((size.width-64)/w, (size.height-64)/h, 1));
  const scale = zoom ?? fit;
  const generating = job?.status === 'running' || job?.status === 'queued';
  const phase = job ? imagePhases[job.phase]?.[de?0:1] ?? job.phase : '';
  return <section className="image-canvas" aria-label={de?'Bild-Canvas':'Image canvas'}>
    <div className="image-canvas-toolbar"><div><strong>Canvas</strong><span>{w} × {h} px</span></div><div role="group" aria-label={de?'Vorschaugröße':'Preview zoom'}>
      <button type="button" className="text-button" disabled={!preview} onClick={()=>setZoom(Math.max(.1, scale-.25))} aria-label={de?'Verkleinern':'Zoom out'}><Minus size={15}/></button>
      <button type="button" className="text-button" disabled={!preview} onClick={()=>setZoom(1)} title={de?'Originalgröße anzeigen':'Show actual size'}>{Math.round(scale*100)}%</button>
      <button type="button" className="text-button" disabled={!preview} onClick={()=>setZoom(Math.min(4, scale+.25))} aria-label={de?'Vergrößern':'Zoom in'}><Plus size={15}/></button>
      <button type="button" className="text-button" disabled={!preview} onClick={()=>setZoom(null)} title={de?'In Canvas einpassen':'Fit to canvas'} aria-label={de?'In Canvas einpassen':'Fit to canvas'}><Maximize size={15}/></button>
    </div></div>
    <div ref={viewport} className={`image-canvas-viewport ${preview?'has-image':''}`} tabIndex={0} aria-label={de?'Bildvorschau. Vergrößerte Bilder ziehen oder mit Pfeiltasten scrollen.':'Image preview. Drag or use arrow keys to scroll zoomed images.'}
      onPointerDown={e=>{if(!preview || e.button!==0)return;drag.current={x:e.clientX,y:e.clientY,left:e.currentTarget.scrollLeft,top:e.currentTarget.scrollTop};e.currentTarget.setPointerCapture(e.pointerId);}}
      onPointerMove={e=>{if(drag.current){e.currentTarget.scrollLeft=drag.current.left-(e.clientX-drag.current.x);e.currentTarget.scrollTop=drag.current.top-(e.clientY-drag.current.y);}}}
      onPointerUp={e=>{drag.current=null;if(e.currentTarget.hasPointerCapture(e.pointerId))e.currentTarget.releasePointerCapture(e.pointerId);}}
      onPointerCancel={()=>{drag.current=null;}} onLostPointerCapture={()=>{drag.current=null;}}>
      {preview ? <div className="image-canvas-surface" style={{width:Math.max(size.width,w*scale+64),height:Math.max(size.height,h*scale+64)}}><img draggable={false} src={preview} width={w*scale} height={h*scale} alt={de?'Lokal generiertes Bild':'Locally generated image'}/></div>
        : <div className="image-canvas-empty" role="status"><div className="image-canvas-symbol"><ImagePlus size={32}/></div><h2>{generating?phase:job?.discarded?(de?'Bild verworfen':'Image discarded'):job?.status==='failed'?(de?'Generierung fehlgeschlagen':'Generation failed'):job?.status==='cancelled'?(de?'Generierung abgebrochen':'Generation cancelled'):job?.status==='completed'?(de?'Vorschau wird geladen …':'Loading preview …'):(de?'Platz für deine nächste Idee':'A space for your next idea')}</h2><p>{generating?(de?'Das Ergebnis erscheint hier, sobald der lokale Worker fertig ist.':'Your result appears here when the local worker finishes.'):job?(de?'Auftragsdetails und Aktionen findest du unter dem Canvas.':'Job details and actions are below the canvas.'):(de?'Modell wählen, Bild beschreiben und generieren.':'Choose a model, describe your image, and generate.')}</p>{job&&generating&&<progress aria-label={phase} max={job.phase==='hashing'?Math.max(1,job.modelBytes):(job.samplingSteps??job.request.steps)} value={job.phase==='hashing'?job.hashedBytes:job.phase==='sampling'?job.step:undefined}/>}</div>}
    </div>
    <div className="image-canvas-status"><span>{job?phase:(de?'Bereit für deine Eingaben':'Ready for your input')}</span><span>{job?`Seed ${job.request.seed}`:(de?'Lokal auf deinem PC':'Local on your PC')}</span></div>
  </section>;
}
