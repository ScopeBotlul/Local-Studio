export interface CompareView { zoom:number; x:number; y:number }
export const initialView:CompareView={zoom:1,x:0,y:0};
export function clampView(view:CompareView):CompareView {
 const zoom=Math.min(8,Math.max(1,Number.isFinite(view.zoom)?view.zoom:1));const edge=(zoom-1)/2;
 const clamp=(n:number)=>Math.min(edge,Math.max(-edge,Number.isFinite(n)?n:0))||0;
 return {zoom,x:clamp(view.x),y:clamp(view.y)};
}
export function wipePosition(clientX:number,left:number,width:number):number {
 return width>0?Math.max(0,Math.min(100,100*(clientX-left)/width)):50;
}
