import {useEffect,useRef,useState} from 'react';
import {listen} from '@tauri-apps/api/event';
import {invoke} from '@tauri-apps/api/core';
export function useGalleryWatch(rootId:string|undefined,onChange:()=>void){
 const [active,setActive]=useState(false);const refresh=useRef(onChange);refresh.current=onChange;
 useEffect(()=>{let live=true;let off:(()=>void)|undefined;let debounce:ReturnType<typeof setTimeout>|undefined;let pending=false;
  const change=()=>{if(pending)return;pending=true;debounce=setTimeout(()=>{pending=false;if(live)refresh.current();},150);};
  const connect=async()=>{try{const status=await invoke<{rootId:string;active:boolean}>('gallery_watch');if(live)setActive(status.active);}catch{if(live)setActive(false);}};
  void listen<{rootId:string;active:boolean}>('gallery-changed',event=>{if(rootId&&event.payload.rootId!==rootId)return;setActive(event.payload.active);change();}).then(fn=>{if(!live){fn();return;}off=fn;void connect();}).catch(()=>setActive(false));
  const retry=setInterval(()=>void connect(),30000);const focus=()=>{void connect();change();};window.addEventListener('focus',focus);
  return()=>{live=false;off?.();clearTimeout(debounce);clearInterval(retry);window.removeEventListener('focus',focus);};
 },[rootId]);return active;
}
