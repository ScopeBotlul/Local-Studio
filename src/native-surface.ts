export type SurfaceRect={left:number;top:number;right:number;bottom:number};
export type NativeBounds={x:number;y:number;width:number;height:number};

export function clipNativeSurface(surface:SurfaceRect,content:SurfaceRect,viewport:{width:number;height:number}):NativeBounds|null{
 const left=Math.max(0,content.left,surface.left);
 const top=Math.max(0,content.top,surface.top);
 const right=Math.min(viewport.width,content.right,surface.right);
 const bottom=Math.min(viewport.height,content.bottom,surface.bottom);
 const width=right-left,height=bottom-top;
 return width>=32&&height>=32?{x:left,y:top,width,height}:null;
}

export function contentRect(element:Element):SurfaceRect{
 const content=element.closest('.window-content');
 const rect=(content??document.documentElement).getBoundingClientRect();
 return {left:rect.left,top:rect.top,right:rect.right,bottom:rect.bottom};
}
