// Browser-only fixture. Never imported by the application or desktop build.
import React from 'react';
import {createRoot} from 'react-dom/client';
import UpdateDialog from '../src/UpdateDialog';
let root;
export function mount({portable=false,de=true}={}){
 root?.unmount();
 const calls=[];
 let resolveDownload;
 let resolveComfyUpdate;
 let status={phase:'available',portable,received:0,total:100,error:null,latest:{version:'99.0.0',notes:'Test release',publishedAt:'',installer:{url:'',size:100,sha256:''},portable:{url:'',size:100,sha256:''}}};
 let comfy={installed:true,path:'C:/ComfyUI',running:false,managed:false,endpoint:'http://127.0.0.1:8188',version:'0.33.0',error:null,dismissed:false,install:{phase:'idle',variant:null,totalBytes:0,receivedBytes:0,bytesPerSecond:0,error:null},update:{phase:'available',installedVersion:'0.33.0',latestVersion:'0.34.0',error:null,log:null}};
 window.__TAURI_INTERNALS__={invoke:async command=>{
  calls.push(command);
  if(command==='update_status')return structuredClone(status);
  if(command==='update_download'){
   status={...status,phase:'downloading'};
   return new Promise(resolve=>{resolveDownload=resolve;});
  }
  if(command==='update_check'){status={...status,phase:'available',error:null};return status;}
  if(command==='update_cancel'||command==='update_open_download')return;
  if(command==='comfy_status')return structuredClone(comfy);
  if(command==='comfy_update_check'){comfy={...comfy,update:{...comfy.update,phase:'available'}};return structuredClone(comfy);}
  if(command==='comfy_update'){comfy={...comfy,update:{...comfy.update,phase:'updating',log:'pulling latest changes'}};return new Promise(resolve=>{resolveComfyUpdate=resolve;});}
  if(command==='comfy_open_updater')return;
  throw Error('Unexpected IPC: '+command);
 }};
 window.updateFixture={calls,installed:0,closed:0,finishComfy(){comfy={...comfy,version:'0.34.0',update:{...comfy.update,phase:'current',installedVersion:'0.34.0',log:'Done!'}};resolveComfyUpdate(structuredClone(comfy));},finish(phase='ready'){
  status={...status,phase,error:phase==='error'?'update_integrity':null};
  resolveDownload(structuredClone(status));
 }};
 root=createRoot(document.getElementById('root'));
 root.render(<UpdateDialog de={de} version="0.27.0" automatic={true} onAutomatic={()=>{}} onInstall={()=>{window.updateFixture.installed++;root.unmount();root=null;}} onClose={()=>{window.updateFixture.closed++;root.unmount();root=null;}}/>);
}
