import {useEffect,useState,type ComponentProps} from 'react';
import ImageStudio from './ImageStudio';
import CanvasStudio from './CanvasStudio';
import type {ImageRequest} from './image-api';
type Props={base:ImageRequest;incoming:ImageRequest|null;onConsume:()=>void;canvas:ComponentProps<typeof CanvasStudio>}&Omit<ComponentProps<typeof ImageStudio>,'request'|'setRequest'|'selectModel'|'onRestore'|'editing'>;
export default function ImageEditingStudio({base,incoming,onConsume,canvas,...props}:Props){
 const [mode,setMode]=useState<'ai'|'layers'>('ai');
 const [request,setRequest]=useState<ImageRequest>(()=>{try{const saved=JSON.parse(localStorage.getItem('image-edit-draft')??'null');if(saved&&typeof saved.modelPath==='string'&&typeof saved.prompt==='string')return saved;}catch{/* local draft */}return {...base,reference:null};});
 useEffect(()=>{if(incoming){setRequest({...incoming,reference:incoming.reference?{...incoming.reference,mask:null}:null});setMode('ai');onConsume();}},[incoming,onConsume]);
 useEffect(()=>{try{localStorage.setItem('image-edit-draft',JSON.stringify(request));}catch{/* local draft */}},[request]);
 return <div className="image-edit-studio"><div className="edit-view-switch edit-studio-modes" role="group" aria-label={props.language==='de'?'Bildbearbeitung':'Image editing'}><button className="button secondary" type="button" aria-pressed={mode==='ai'} onClick={()=>setMode('ai')}>{props.language==='de'?'Mit KI bearbeiten':'Edit with AI'}</button><button className="button secondary" type="button" aria-pressed={mode==='layers'} onClick={()=>setMode('layers')}>{props.language==='de'?'Ebenen und Werkzeuge':'Layers and tools'}</button></div>{mode==='ai'?<ImageStudio {...props} editing request={request} setRequest={setRequest} selectModel={modelPath=>setRequest(r=>({...r,modelPath}))} onRestore={request=>setRequest({...request,reference:request.reference?{...request.reference,mask:null}:null})}/>:<CanvasStudio {...canvas}/>}</div>;
}
