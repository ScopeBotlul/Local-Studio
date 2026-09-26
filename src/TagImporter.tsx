import {useMemo,useState} from 'react';
import {invoke} from '@tauri-apps/api/core';
import {LoaderCircle, Tags} from 'lucide-react';
import CivitaiBrowser from './CivitaiBrowser';

export type TagGroup='Character'|'Body'|'Position'|'Camera'|'Miscellaneous';
const headings=new Set(['character','characters','general','artist','artists','copyright','copyrights','metadata','meta']);
const censor=new Set(['censored','censor','scribble censor','scribble censoring','pointless censoring','bar censor','mosaic censoring','blur censor']);
const camera=/\b(camera|view|angle|perspective|portrait|close-up|closeup|full body|upper body|from above|from below|from side|pov|depth of field)\b/i;
const position=/\b(sitting|standing|kneeling|lying|bent over|walking|running|pose|spread legs|crossed legs|arms up|hands behind)\b/i;
const body=/\b((?:\d+)?girl|(?:\d+)?boy|woman|man|breasts?|areola|nipples?|hair|eyes?|skin|thighs?|legs?|arms?|hands?|feet|navel|body|muscular|slender|curvy)\b/i;

export function parseTags(raw:string,removeCensor:boolean):Record<TagGroup,string[]>{
 const groups:Record<TagGroup,string[]>={Character:[],Body:[],Position:[],Camera:[],Miscellaneous:[]};
 let section='';const seen=new Set<string>();
 for(const source of raw.split(/\r?\n|,/)){
  const trimmed=source.trim();if(!trimmed)continue;
  const heading=trimmed.replace(/:$/,'').toLocaleLowerCase();if(headings.has(heading)){section=heading;continue;}
  let tag=trimmed.replace(/^[?•·-]+\s*/,'').replace(/\s+\d+(?:\.\d+)?\s*[kmb]?$/i,'').trim().replaceAll('_',' ');
  tag=tag.replace(/\s+/g,' ');const key=tag.toLocaleLowerCase();if(!tag||seen.has(key)||(removeCensor&&censor.has(key)))continue;seen.add(key);
  const group:TagGroup=section.startsWith('character')?'Character':camera.test(tag)?'Camera':position.test(tag)?'Position':body.test(tag)?'Body':'Miscellaneous';groups[group].push(tag);
 }
 return groups;
}
export const flatTags=(groups:Record<TagGroup,string[]>)=>(['Character','Body','Position','Camera','Miscellaneous'] as TagGroup[]).flatMap(group=>groups[group]);

function tagError(error:unknown,de:boolean){
 const code=String(error);
 const messages:Record<string,[string,string]>={
  tag_browser:['Der eingebettete Browser ist nicht mehr geöffnet oder die Seite konnte nicht gelesen werden.','The embedded browser is no longer open or the page could not be read.'],
  tag_post:['Öffne zuerst einen einzelnen unterstützten Post.','Open an individual supported post first.'],
  tag_response:['Die Tags dieses Posts konnten nicht gelesen werden.','The tags for this post could not be read.'],
  tag_url:['Diese Post-Adresse wird nicht unterstützt.','This post address is not supported.'],
  tag_network:['Die Tag-Quelle ist derzeit nicht erreichbar.','The tag source is currently unavailable.'],
 };
 const message=messages[code];
 return message?message[de?0:1]:code;
}

export default function TagImporter({de,removeCensor,onImport}:{de:boolean;removeCensor:boolean;onImport:(tags:string[])=>void}){
 const [raw,setRaw]=useState(''),[browser,setBrowser]=useState<'danbooru'|'rule34'|null>(null),[busy,setBusy]=useState(false),[error,setError]=useState('');
 const groups=useMemo(()=>parseTags(raw,removeCensor),[raw,removeCensor]),tags=flatTags(groups);
 const browserHome=browser==='rule34'?'https://rule34.xxx/index.php?page=post&s=list':'https://danbooru.donmai.us/posts';
 async function importPost(url:string){if(busy)return;setBusy(true);setError('');try{setRaw(await invoke<string>('tag_post_tags',{url}));setBrowser(null);}catch(e){setError(tagError(e,de));}finally{setBusy(false);}}
 return <details className="image-control-details tag-importer">
  <summary><Tags size={15}/>{de?' Tags importieren':' Import tags'}{tags.length?` · ${tags.length}`:''}</summary>
  <p className="hub-hint">{de?'Öffne einen einzelnen Danbooru- oder Rule34-Post im integrierten Browser und wähle dort „Tags übernehmen“. Alternativ kannst du eine kopierte Tagliste einfügen. Überschriften, Fragezeichen und Mengenangaben werden lokal entfernt; die Vorschau wird gruppiert.':'Open an individual Danbooru or Rule34 post in the embedded browser and choose “Use tags”. You can also paste a copied tag list. Headings, question marks, and counts are removed locally; the preview is grouped.'}</p>
  <div className="tag-source-actions">
   <button type="button" className={`button secondary ${browser==='danbooru'?'active':''}`} disabled={busy} onClick={()=>setBrowser('danbooru')}>Danbooru</button>
   <button type="button" className={`button secondary ${browser==='rule34'?'active':''}`} disabled={busy} onClick={()=>setBrowser('rule34')}>Rule34</button>
   {browser&&<button type="button" className="text-button" disabled={busy} onClick={()=>setBrowser(null)}>{de?'Browser schließen':'Close browser'}</button>}
  </div>
  {browser&&<CivitaiBrowser key={browser} de={de} homeUrl={browserHome} onPost={url=>void importPost(url)}/>}
  {busy&&<p className="hub-hint"><LoaderCircle className="spin" size={14}/>{de?' Tags werden geladen …':' Loading tags …'}</p>}
  {error&&<p className="notice warning" role="alert">{error}</p>}
  <textarea value={raw} onChange={event=>setRaw(event.target.value)} rows={5} placeholder={'Character\n? character name 2.4k\nGeneral\n? 1girl 8.4M'}/>
  {tags.length>0&&<div className="tag-groups">{(Object.keys(groups) as TagGroup[]).filter(group=>groups[group].length).map(group=><div key={group}><strong>{group}</strong><span>{groups[group].join(', ')}</span></div>)}</div>}
  <button type="button" className="button secondary" disabled={!tags.length} onClick={()=>onImport(tags)}>{de?'In den Prompt übernehmen':'Add to prompt'}</button>
  {removeCensor&&<p className="hub-hint">{de?'Zensur-Tags werden gemäß Einstellung entfernt.':'Censorship tags are removed by your setting.'}</p>}
 </details>;
}
