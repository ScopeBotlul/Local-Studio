// Browser-only fixture. Never imported by the application or desktop build.
import React from 'react';
import {createRoot} from 'react-dom/client';
import UpdateDialog from '../src/UpdateDialog';
let root;
export function mount({portable=false,de=true}={}){
 root?.unmount();
 const calls=[];
 let resolveDownload;
 let status={phase:'available',portable,received:0,total:100,error:null,latest:{version:'99.0.0',notes:'Test release',publishedAt:'',installer:{url:'',size:100,sha256:''},portable:{url:'',size:100,sha256:''}}};
 window.__TAURI_INTERNALS__={invoke:async command=>{
  calls.push(command);
  if(command==='update_status')return structuredClone(status);
  if(command==='update_download'){
   status={...status,phase:'downloading'};
   return new Promise(resolve=>{resolveDownload=resolve;});
  }
  if(command==='update_check'){status={...status,phase:'available',error:null};return status;}
  if(command==='update_cancel'||command==='update_open_download')return;
  throw Error('Unexpected IPC: '+command);
 }};
 window.updateFixture={calls,installed:0,closed:0,finish(phase='ready'){
  status={...status,phase,error:phase==='error'?'update_integrity':null};
  resolveDownload(structuredClone(status));
 }};
 root=createRoot(document.getElementById('root'));
 root.render(<UpdateDialog de={de} version="0.27.0" automatic={true} onAutomatic={()=>{}} onInstall={()=>{window.updateFixture.installed++;root.unmount();root=null;}} onClose={()=>{window.updateFixture.closed++;root.unmount();root=null;}}/>);
}
