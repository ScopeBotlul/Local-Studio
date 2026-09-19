import {useEffect,useState} from 'react';
import {ArrowLeftRight} from 'lucide-react';
import {validImageDimensions, type ImageRequest} from './image-api';
const formats = [[1024,1024,'1:1'],[768,1152,'2:3'],[1152,768,'3:2'],[768,1344,'4:7'],[1344,768,'7:4'],[512,512,'512']] as const;
export default function ImageDimensions({request,onChange,onValidityChange,de}:{request:ImageRequest;onChange:(patch:Partial<ImageRequest>)=>void;onValidityChange:(valid:boolean)=>void;de:boolean}) {
  const [custom,setCustom]=useState(false);
  const [draft,setDraft]=useState({width:String(request.width),height:String(request.height)});
  useEffect(()=>{setDraft({width:String(request.width),height:String(request.height)});onValidityChange(validImageDimensions(request.width,request.height));},[request.width,request.height,onValidityChange]);
  function choose(width:number,height:number) {
    setDraft({width:String(width),height:String(height)});onValidityChange(validImageDimensions(width,height));onChange({width,height});
  }
  function edit(key:'width'|'height',value:string) {
    const next={...draft,[key]:value};setDraft(next);
    const width=Number(next.width),height=Number(next.height),valid=validImageDimensions(width,height);
    onValidityChange(valid);
    if(valid)onChange({width,height});
  }
  const match = formats.find(([w,h])=>w===request.width&&h===request.height);
  const showCustom = custom || !match;
  const valid=validImageDimensions(Number(draft.width),Number(draft.height));
  return <section className="image-dimensions"><div className="section-heading"><h3>{de?'Bildformat':'Image size'}</h3><button type="button" className="text-button" onClick={()=>choose(request.height,request.width)} aria-label={de?'Breite und Höhe tauschen':'Swap width and height'}><ArrowLeftRight size={15}/></button></div>
    <div className="image-size-modes" role="group" aria-label={de?'Auflösungsmodus':'Resolution mode'}><button type="button" aria-pressed={!showCustom} onClick={()=>{setCustom(false);choose(match?request.width:1024,match?request.height:1024);}}>{de?'Vorlagen':'Presets'}</button><button type="button" aria-pressed={showCustom} onClick={()=>setCustom(true)}>{de?'Eigene Größe':'Custom size'}</button></div>
    {!showCustom?<div className="image-size-presets">{formats.map(([w,h,label])=><button type="button" key={`${w}-${h}`} aria-pressed={request.width===w&&request.height===h} onClick={()=>choose(w,h)}><strong>{label}</strong><small>{w} × {h}</small></button>)}</div>:<div className="image-parameters">{(['width','height'] as const).map(key=><label className="field-label" key={key}>{key==='width'?(de?'Breite':'Width'):(de?'Höhe':'Height')}<input aria-label={key==='width'?'Image width':'Image height'} type="number" min={256} max={2048} step={64} value={draft[key]} aria-invalid={!valid} onChange={e=>edit(key,e.target.value)}/></label>)}</div>}
    <p className={valid?'hub-hint':'notice warning'} role={valid?undefined:'alert'}>{de?'SDXL: 256–2048 px, 64er-Schritte, maximal 2.097.152 Pixel. Größere Bilder benötigen mehr VRAM.':'SDXL: 256–2048 px, multiples of 64, at most 2,097,152 pixels. Larger images require more VRAM.'}</p>
    {request.reference&&(request.reference.width!==request.width||request.reference.height!==request.height)&&<p className="notice warning">{de?'Referenzbild benötigt':'Reference requires'} {request.reference.width} × {request.reference.height}. <button type="button" className="text-button" onClick={()=>choose(request.reference!.width,request.reference!.height)}>{de?'Größe übernehmen':'Use reference size'}</button></p>}
  </section>;
}
