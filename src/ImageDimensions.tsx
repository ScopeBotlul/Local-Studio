import {useEffect,useState} from 'react';
import {ArrowLeftRight} from 'lucide-react';
import {validImageDimensions,imageDimensionIssue,suggestImageDimensions,IMAGE_MAX_SIDE, type ImageRequest} from './image-api';
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
  const issue=imageDimensionIssue(Number(draft.width),Number(draft.height));
  const suggestion=issue?suggestImageDimensions(Number(draft.width),Number(draft.height)):null;
  const issueText=issue==='range'?(de?'Breite und Höhe müssen zwischen 256 und 4096 Pixel liegen.':'Width and height must be between 256 and 4096 pixels.'):issue==='step'?(de?'Breite und Höhe müssen durch 64 teilbar sein. Zum Beispiel passt 1920 × 1088 statt 1920 × 1080.':'Width and height must be divisible by 64. For example, use 1920 × 1088 instead of 1920 × 1080.'):(de?'Diese Größe überschreitet 4.194.304 Pixel. Zum Beispiel sind 2048 × 2048 möglich.':'This size exceeds 4,194,304 pixels. For example, 2048 × 2048 is allowed.');
  return <section className="image-dimensions"><div className="section-heading"><h3>{de?'Bildformat':'Image size'}</h3><button type="button" className="text-button" onClick={()=>choose(request.height,request.width)} aria-label={de?'Breite und Höhe tauschen':'Swap width and height'}><ArrowLeftRight size={15}/></button></div>
    <div className="image-size-modes" role="group" aria-label={de?'Auflösungsmodus':'Resolution mode'}><button type="button" aria-pressed={!showCustom} onClick={()=>{setCustom(false);choose(match?request.width:1024,match?request.height:1024);}}>{de?'Vorlagen':'Presets'}</button><button type="button" aria-pressed={showCustom} onClick={()=>setCustom(true)}>{de?'Eigene Größe':'Custom size'}</button></div>
    {!showCustom?<div className="image-size-presets">{formats.map(([w,h,label])=><button type="button" key={`${w}-${h}`} aria-pressed={request.width===w&&request.height===h} onClick={()=>choose(w,h)}><strong>{label}</strong><small>{w} × {h}</small></button>)}</div>:<div className="image-parameters">{(['width','height'] as const).map(key=><label className="field-label" key={key}>{key==='width'?(de?'Breite':'Width'):(de?'Höhe':'Height')}<input aria-label={key==='width'?'Image width':'Image height'} type="number" min={256} max={IMAGE_MAX_SIDE} step={64} value={draft[key]} aria-invalid={!valid} aria-describedby="image-dimensions-hint" onChange={e=>edit(key,e.target.value)}/></label>)}</div>}
    <p id="image-dimensions-hint" className={valid?'hub-hint':'notice warning'} role={valid?undefined:'alert'}>{valid?(de?'SDXL: 256–4096 px pro Seite, 64er-Schritte, maximal 4.194.304 Pixel insgesamt.':'SDXL: 256–4096 px per side, multiples of 64, at most 4,194,304 pixels in total.'):issueText} {!valid&&suggestion&&<button type="button" className="text-button" onClick={()=>choose(suggestion.width,suggestion.height)}>{suggestion.width} × {suggestion.height} {de?'übernehmen':'apply'}</button>}</p>
    {valid&&request.width*request.height>2097152&&<p className="notice warning">{de?'Hohe Auflösung: benötigt deutlich mehr VRAM und Rechenzeit. Bei Speichermangel kann der Auftrag abbrechen. Größer bedeutet nicht automatisch bessere Bildqualität.':'High resolution: requires substantially more VRAM and time. The job may fail if memory runs out. Larger does not automatically mean better image quality.'}</p>}
    {request.reference&&(request.reference.width!==request.width||request.reference.height!==request.height)&&<p className="notice warning">{de?'Referenzbild benötigt':'Reference requires'} {request.reference.width} × {request.reference.height}. <button type="button" className="text-button" onClick={()=>choose(request.reference!.width,request.reference!.height)}>{de?'Größe übernehmen':'Use reference size'}</button></p>}
  </section>;
}
