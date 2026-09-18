import { describe,it,expect } from 'vitest';
import { selectGesture } from './gallery-selection';
import type { GalleryEntry } from './gallery-api';
const entries=Array.from({length:6},(_,i)=>({path:String(i),fileId:String(i)} as GalleryEntry));
describe('gallery selection gestures',()=>{
  it('toggles Ctrl clicks and clears bulk selection on a plain preview click',()=>{
    const first=selectGesture(entries,{},'2',null,true,false);
    const second=selectGesture(entries,first,'4','2',true,false);
    expect(Object.keys(second)).toEqual(['2','4']);
    expect(Object.keys(selectGesture(entries,second,'2','4',true,false))).toEqual(['4']);
    expect(selectGesture(entries,second,'3','2',false,false)).toEqual({});
  });
  it('selects the visible range in both directions and supports additive Ctrl+Shift',()=>{
    expect(Object.keys(selectGesture(entries,{},'1','4',false,true))).toEqual(['1','2','3','4']);
    expect(Object.keys(selectGesture(entries,{'0':entries[0]},'4','2',true,true))).toEqual(['0','2','3','4']);
    expect(Object.keys(selectGesture(entries,{},'3','missing',false,true))).toEqual(['3']);
  });
  it('skips missing identities and duplicate hard links',()=>{
    const rows=[entries[0],{...entries[1],fileId:null},{...entries[2],fileId:'0'},entries[3]];
    expect(Object.keys(selectGesture(rows,{},'3','0',false,true))).toEqual(['0','3']);
  });
});
