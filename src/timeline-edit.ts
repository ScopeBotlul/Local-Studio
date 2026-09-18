import {length,poseAt,type Clip,type Timeline} from './creative-state';
export function trimClip(clip:Clip,edge:'in'|'out',delta:number,sourceDuration=86400):Clip {
  if(Math.abs(delta)<1e-8)return clip;
  const oldLength=length(clip),minimum=Math.max(.04,.04/clip.speed);
  if(edge==='in'){
    const shift=Math.max(-clip.start,-clip.sourceIn/clip.speed,Math.min(oldLength-minimum,delta));
    const duration=oldLength-shift;
    const keyframes=[{...poseAt(clip,Math.max(0,shift)),time:0},...clip.keyframes.filter(k=>k.time>shift).map(k=>({...k,time:k.time-shift}))];
    if(keyframes.length>100)return clip;
    return {...clip,start:clip.start+shift,sourceIn:clip.sourceIn+shift*clip.speed,fadeIn:Math.min(clip.fadeIn,duration/2),fadeOut:Math.min(clip.fadeOut,duration/2),keyframes};
  }
  const duration=Math.max(minimum,Math.min(3600-clip.start,(sourceDuration-clip.sourceIn)/clip.speed,oldLength+delta));
  const keyframes=clip.keyframes.at(-1)!.time<=duration?clip.keyframes:[...clip.keyframes.filter(k=>k.time<duration),{...poseAt(clip,duration),time:duration}];
  return {...clip,sourceOut:clip.sourceIn+duration*clip.speed,fadeIn:Math.min(clip.fadeIn,duration/2),fadeOut:Math.min(clip.fadeOut,duration/2),keyframes};
}
export function movable(t:Timeline,ids:string[]){return t.clips.filter(c=>ids.includes(c.id)&&!t.tracks.find(tr=>tr.id===c.trackId)?.locked);}
export function moveClips(t:Timeline,ids:string[],delta:number,snap:boolean,cursor:number,scale:number):{timeline:Timeline;guide:number|null}{
  const clips=movable(t,ids);if(!clips.length)return {timeline:t,guide:null};
  let d=Math.round(delta*t.fps)/t.fps,guide:number|null=null;
  const low=-Math.min(...clips.map(c=>c.start)),high=3600-Math.max(...clips.map(c=>c.start+length(c)));
  if(snap){const targets=[0,cursor,...t.clips.filter(c=>!ids.includes(c.id)).flatMap(c=>[c.start,c.start+length(c)])];let distance=8/scale;
    for(const edge of clips.flatMap(c=>[c.start,c.start+length(c)]))for(const target of targets){const diff=target-(edge+d);if(Math.abs(diff)<distance&&d+diff>=low&&d+diff<=high){distance=Math.abs(diff);guide=target;}}
    if(guide!==null){const edge=clips.flatMap(c=>[c.start,c.start+length(c)]).reduce((a,b)=>Math.abs(a+d-guide!)<Math.abs(b+d-guide!)?a:b);d=guide-edge;}
  }
  d=Math.max(low,Math.min(high,d));const set=new Set(clips.map(c=>c.id));return {timeline:{...t,clips:t.clips.map(c=>set.has(c.id)?{...c,start:c.start+d}:c)},guide};
}
export function snapEdge(t:Timeline,clip:Clip,edge:'in'|'out',delta:number,cursor:number,scale:number,snap:boolean){
  const point=edge==='in'?clip.start:clip.start+length(clip);let at=Math.round((point+delta)*t.fps)/t.fps;
  if(snap){let distance=8/scale;for(const target of [0,cursor,...t.clips.filter(c=>c.id!==clip.id).flatMap(c=>[c.start,c.start+length(c)])])if(Math.abs(target-at)<distance){distance=Math.abs(target-at);at=target;}}
  return at-point;
}
