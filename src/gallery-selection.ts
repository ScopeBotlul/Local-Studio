import type { GalleryEntry } from './gallery-api';
export function selectGesture(entries: GalleryEntry[], selected: Record<string,GalleryEntry>, path: string, anchor: string | null, ctrl: boolean, shift: boolean) {
  const entry=entries.find(e=>e.path===path);
  if(!entry?.fileId)return selected;
  const next:Record<string,GalleryEntry>=ctrl?{...selected}:{};
  if(shift){
    const start=entries.findIndex(e=>e.path===anchor),end=entries.indexOf(entry);
    for(const e of entries.slice(Math.min(start<0?end:start,end),Math.max(start<0?end:start,end)+1))if(e.fileId)next[e.path]=e;
  }else if(ctrl){if(next[path])delete next[path];else next[path]=entry;}
  const ids=new Set<string>();
  return Object.fromEntries(Object.entries(next).filter(([,e])=>!!e.fileId&&!ids.has(e.fileId)&&!!ids.add(e.fileId)).slice(0,50));
}
