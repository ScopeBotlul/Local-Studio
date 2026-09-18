// Decode a single local frame using the same WebView codecs as the gallery viewer.
// Visible-card queue limits concurrency; timeout/unmount releases decoder and source.
export function videoFrame(rootId:string,path:string,version:string,signal:AbortSignal):Promise<string>{
  return new Promise((resolve,reject)=>{
    const video=document.createElement('video');video.muted=true;video.preload='auto';video.crossOrigin='anonymous';video.playsInline=true;
    let settled=false;let seeking=false;
    const finish=(error?:unknown,png?:string)=>{if(settled)return;settled=true;clearTimeout(timer);signal.removeEventListener('abort',cancel);video.onloadedmetadata=null;video.onloadeddata=null;video.onseeked=null;video.onerror=null;video.pause();video.removeAttribute('src');video.load();error?reject(error):resolve(png!);};
    const cancel=()=>finish('gallery_thumbnail'); const timer=setTimeout(cancel,12000);signal.addEventListener('abort',cancel,{once:true});
    const capture=()=>{if(settled||video.readyState<2)return;try{const w=video.videoWidth,h=video.videoHeight;if(!w||!h||w*h>32000000||w>8192||h>8192)throw 'gallery_thumbnail_limit';const scale=Math.min(1,320/w,320/h);const c=document.createElement('canvas');c.width=Math.max(1,Math.round(w*scale));c.height=Math.max(1,Math.round(h*scale));const ctx=c.getContext('2d');if(!ctx)throw 'gallery_thumbnail';ctx.drawImage(video,0,0,c.width,c.height);finish(undefined,c.toDataURL('image/png').split(',')[1]);}catch(e){finish(e);}};
    video.onloadedmetadata=()=>{if(video.videoWidth*video.videoHeight>32000000){finish('gallery_thumbnail_limit');return;}if(Number.isFinite(video.duration)&&video.duration>0.1){seeking=true;video.currentTime=Math.min(1,video.duration/3);}else capture();};
    video.onloadeddata=()=>{if(!seeking)capture();};video.onseeked=capture;video.onerror=cancel;
    if(signal.aborted){cancel();return;}const encoded=btoa(String.fromCharCode(...new TextEncoder().encode(path))).replaceAll('+','-').replaceAll('/','_').replace(/=+$/,'');
    video.src=`http://gallery.localhost/${rootId}/${encoded}?v=${version}`;video.load();
  });
}
