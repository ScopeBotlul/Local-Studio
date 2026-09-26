// Browser fixture only; never imported by the app.
import React from 'react';
import {createRoot} from 'react-dom/client';
import WindowFrame from '../src/WindowFrame';
import DownloadOverview from '../src/DownloadOverview';
import {downloads} from '../src/download-api';
import {updateApi} from '../src/update-api';
import '../src/styles.css';
export function mount(){
 window.downloadFixture={rows:[
  {id:'one',repo:'test/image-model',status:'downloading',totalBytes:100000000,downloadedBytes:10000000,bytesPerSecond:1000000},
  {id:'two',repo:'test/video-model',status:'queued',totalBytes:200000000,downloadedBytes:0,bytesPerSecond:0},
  {id:'three',repo:'test/finished',status:'completed',totalBytes:100,downloadedBytes:100,bytesPerSecond:0},
 ],opened:0,calls:0,fail:false,update:{phase:'idle'}};
 downloads.list=async()=>{window.downloadFixture.calls++;if(window.downloadFixture.fail)throw Error('offline');return structuredClone(window.downloadFixture.rows);};
 updateApi.status=async()=>structuredClone(window.downloadFixture.update);
 createRoot(document.getElementById('root')).render(<WindowFrame><div className="app-shell"><DownloadOverview language="de" onOpenDownloads={()=>window.downloadFixture.opened++}/></div></WindowFrame>);
}
