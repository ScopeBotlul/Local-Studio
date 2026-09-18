import type {Layer} from './creative-state';
export function resizeLayer(l:Layer,handle:string,point:[number,number],ratio:boolean):Layer {
 const sx=handle.includes('e')?1:-1,sy=handle.includes('s')?1:-1,a=l.rotation*Math.PI/180,co=Math.cos(a),si=Math.sin(a);
 const anchor:[number,number]=[l.x+l.width/2-co*sx*l.width/2+si*sy*l.height/2,l.y+l.height/2-si*sx*l.width/2-co*sy*l.height/2];
 const dx=point[0]-anchor[0],dy=point[1]-anchor[1];let width=Math.max(1,Math.min(32768,sx*(co*dx+si*dy))),height=Math.max(1,Math.min(32768,sy*(-si*dx+co*dy)));
 if(ratio){const f=Math.max(width/l.width,height/l.height);width=Math.min(32768,l.width*f);height=width*l.height/l.width;if(height>32768){height=32768;width=height*l.width/l.height;}}
 const cx=anchor[0]+co*sx*width/2-si*sy*height/2,cy=anchor[1]+si*sx*width/2+co*sy*height/2;
 return {...l,x:cx-width/2,y:cy-height/2,width,height};
}
export function rotateLayer(l:Layer,point:[number,number],snap:boolean):Layer {let angle=Math.atan2(point[1]-l.y-l.height/2,point[0]-l.x-l.width/2)*180/Math.PI+90;if(snap)angle=Math.round(angle/15)*15;return {...l,rotation:angle>180?angle-360:angle};}
