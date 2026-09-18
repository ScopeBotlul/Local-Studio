export type ViewCommand='undo'|'redo'|'imageFit'|'imageActual'|'imageReset';
type Context={priority:number;actions:Partial<Record<ViewCommand,()=>void>>};
const contexts=new Map<string,Context>();
export function setMenuContext(id:string,value:Context){contexts.set(id,value);window.dispatchEvent(new Event('menu-context'));return()=>{contexts.delete(id);window.dispatchEvent(new Event('menu-context'));};}
export function menuContext(){return [...contexts.values()].sort((a,b)=>b.priority-a.priority)[0];}
