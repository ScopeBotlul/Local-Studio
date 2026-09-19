import {createContext,useContext,useEffect,useLayoutEffect,useRef,useState,type ReactNode} from 'react';
import {createPortal} from 'react-dom';
import {getCurrentWindow} from '@tauri-apps/api/window';
import {Minus,Square,Copy,X} from 'lucide-react';
import {inDesktop} from './api';
import './window-frame.css';

const chromeContext=createContext({menuHost:null as HTMLDivElement|null,setTitle:(_title:string)=>{}});
export const useWindowFrame=()=>useContext(chromeContext);

export default function WindowFrame({children}:{children:ReactNode}){
 const chromeSlot=useRef<HTMLDivElement>(null);
 const [chromeHost]=useState(()=>document.createElement('div'));
 useLayoutEffect(()=>{
  // Modal dialogs make every outside DOM node inert. Keep the same chrome DOM
  // inside the topmost modal so native-window controls and editor menus work.
  let stack:HTMLDialogElement[]=[];
  const sync=(records:MutationRecord[]=[])=>{
   stack=stack.filter(d=>d.isConnected&&d.matches(':modal'));
   for(const record of records){const d=record.target;if(d instanceof HTMLDialogElement&&d.matches(':modal')){stack=stack.filter(item=>item!==d);stack.push(d);}}
   for(const d of document.querySelectorAll<HTMLDialogElement>('dialog:modal'))if(!stack.includes(d))stack.push(d);
   const modal=stack.at(-1),target=modal??chromeSlot.current;
   chromeHost.className=modal?'window-chrome-host window-chrome-modal':'window-chrome-host';
   if(target&&chromeHost.parentElement!==target)target.appendChild(chromeHost);
  };
  sync();const observer=new MutationObserver(sync);observer.observe(document.body,{subtree:true,childList:true,attributes:true,attributeFilter:['open']});
  return()=>{observer.disconnect();chromeHost.remove();};
 },[chromeHost]);
 const [menuHost,setMenuHost]=useState<HTMLDivElement|null>(null),[title,setTitle]=useState('Local Studio');
 const [maximized,setMaximized]=useState(false),[focused,setFocused]=useState(true),[error,setError]=useState('');
 const mounted=useRef(true);
 const [de,setDe]=useState(document.documentElement.lang!=='en');
 useEffect(()=>{const language=()=>setDe(document.documentElement.lang!=='en');language();const observer=new MutationObserver(language);observer.observe(document.documentElement,{attributes:true,attributeFilter:['lang']});return()=>observer.disconnect();},[]);
 useEffect(()=>{
  mounted.current=true;if(!inDesktop())return;let live=true;const off:(()=>void)[]=[];const w=getCurrentWindow();
  const update=()=>void w.isMaximized().then(v=>{if(live)setMaximized(v);}).catch(()=>{});
  update();void w.isFocused().then(v=>{if(live)setFocused(v);}).catch(()=>{});
  for(const listener of [w.onResized(update),w.onFocusChanged(e=>{if(live)setFocused(e.payload);})])void listener.then(f=>{if(live)off.push(f);else f();}).catch(()=>{});
  return()=>{live=false;mounted.current=false;off.forEach(f=>f());};
 },[]);
 async function run(action:()=>Promise<unknown>){if(!inDesktop())return;try{await action();}catch(e){if(mounted.current)setError(String(e));}}
 return <chromeContext.Provider value={{menuHost,setTitle}}><div className="window-frame">
  <div className="window-chrome-slot" ref={chromeSlot}/>
  {createPortal(<header className={`window-titlebar${focused?'':' window-inactive'}`}>
   <div className="window-left"><span className="window-logo" aria-label="Local Studio"><span className="logo logo-small" aria-hidden="true"><span/><span/><span/></span></span><div ref={setMenuHost} className="window-menu-host"/></div>
   <div className="window-drag" onMouseDown={e=>{if(e.button===0&&e.detail===1)void run(()=>getCurrentWindow().startDragging());}} onDoubleClick={()=>void run(()=>getCurrentWindow().toggleMaximize())}/>
   <div className="window-title" title={title}>{title}</div>
   <div className="window-controls"><button aria-label={de?'Minimieren':'Minimize'} title={de?'Minimieren':'Minimize'} onClick={()=>void run(()=>getCurrentWindow().minimize())}><Minus size={14}/></button><button aria-label={maximized?(de?'Wiederherstellen':'Restore'):(de?'Maximieren':'Maximize')} title={maximized?(de?'Wiederherstellen':'Restore'):(de?'Maximieren':'Maximize')} onClick={()=>void run(()=>getCurrentWindow().toggleMaximize())}>{maximized?<Copy size={12}/>:<Square size={12}/>}</button><button className="window-close" aria-label={de?'Fenster schließen':'Close window'} title={de?'Fenster schließen':'Close window'} onClick={()=>void run(()=>getCurrentWindow().close())}><X size={16}/></button></div>
  </header>,chromeHost)}
  <div className="window-content">{children}</div>
  {error&&<div className="window-error" role="alert">{error}<button onClick={()=>setError('')} aria-label={de?'Meldung schließen':'Dismiss message'}><X size={15}/></button></div>}
 </div></chromeContext.Provider>;
}
