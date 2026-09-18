import {useState} from 'react';
import type {ProjectAsset} from './project-api';
export default function ProjectMediaPicker({assets,de,imagesOnly=false,disabled,onAdd}:{assets:ProjectAsset[];de:boolean;imagesOnly?:boolean;disabled:boolean;onAdd:(asset:ProjectAsset)=>Promise<void>}){
 const [id,setId]=useState(''),[busy,setBusy]=useState(false);const choices=assets.filter(a=>!imagesOnly||a.kind==='image');
 return <div className="creative-toolbar project-media-picker"><label>{de?'Aus Projektbibliothek':'From project library'}<select aria-label={de?'Projektmedium hinzufügen':'Add project media'} value={id} disabled={disabled||busy} onChange={e=>setId(e.target.value)}><option value="">{de?'Medium auswählen':'Select media'}</option>{choices.map(a=><option key={a.id} value={a.id}>{a.name}</option>)}</select></label><button disabled={disabled||busy||!choices.some(a=>a.id===id)} onClick={()=>{const a=choices.find(a=>a.id===id);if(a){setBusy(true);void onAdd(a).finally(()=>setBusy(false));}}}>{de?'In Editor einfügen':'Insert into editor'}</button></div>;
}
