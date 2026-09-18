import assert from 'node:assert/strict';
import { promises as fs } from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
const pause=ms=>new Promise(resolve=>setTimeout(resolve,ms));
async function until(fn,timeout=20000) { const end=Date.now()+timeout;while(Date.now()<end){const v=await fn();if(v)return v;await pause(150);}throw new Error('Gallery condition timed out'); }
const query={folder:'',search:'',kind:'all',recursive:true,offset:0};
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
export async function checkGallery({getPage,invoke,stop,launch,artifactRoot,record}) {
  let page=getPage(); const gallery=(await invoke('bootstrap')).paths.gallery;
  const saved=(await invoke('image_jobs')).find(j=>j.savedPath);
  if(saved) {
    const relative=path.relative(gallery,saved.savedPath).replaceAll('\\','/');
    await page.getByRole('button',{name:'Gallery',exact:true}).first().click();
    await page.locator('.gallery-file').filter({hasText:path.basename(path.dirname(saved.savedPath))}).click();
    const count=(await invoke('image_jobs')).length;
    await page.getByRole('button',{name:'Restore settings',exact:true}).click();
    await until(async()=>(await invoke('image_workspace')).workspace.request?.prompt===saved.request.prompt);
    assert.deepEqual((await invoke('image_workspace')).workspace.request,saved.request);
    assert.equal((await invoke('image_jobs')).length,count);
    assert.equal(await page.getByRole('button',{name:'Generate image',exact:true}).isDisabled(),true);
    assert.deepEqual((await invoke('gallery_detail',{path:relative})).request,saved.request);
    record('Actual generated gallery image retains immutable model/prompt/parameters and restores Studio without starting generation');
  }
  const source=path.join(artifactRoot,'gallery-sources');await fs.mkdir(source,{recursive:true});
  // Actual browser-encoded image/video fixtures. These are playback tests, not AI generation.
  const encoded=await page.evaluate(async()=>{
    const canvas=document.createElement('canvas');canvas.width=320;canvas.height=180;const context=canvas.getContext('2d');context.fillStyle='#274060';context.fillRect(0,0,320,180);context.fillStyle='#a78bfa';context.fillRect(40,40,100,100);
    const png=canvas.toDataURL('image/png').split(',')[1];
    const stream=canvas.captureStream(15), chunks=[];const recorder=new MediaRecorder(stream,{mimeType:'video/webm;codecs=vp8'});
    const finished=new Promise(resolve=>{recorder.onstop=async()=>{const bytes=new Uint8Array(await new Blob(chunks,{type:'video/webm'}).arrayBuffer());let binary='';for(const b of bytes)binary+=String.fromCharCode(b);resolve(btoa(binary));};});recorder.ondataavailable=e=>chunks.push(e.data);recorder.start();
    for(let i=0;i<15;i++){context.fillStyle=i%2?'#274060':'#477090';context.fillRect(0,0,320,180);await new Promise(r=>setTimeout(r,70));}
    recorder.stop();const webm=await finished;stream.getTracks().forEach(track=>track.stop());return {png,webm};
  });
  await fs.writeFile(path.join(source,'Testbild ü.png'),Buffer.from(encoded.png,'base64'));
  await fs.writeFile(path.join(source,'Bewegung.webm'),Buffer.from(encoded.webm,'base64'));
  const samples=44100*2, wav=Buffer.alloc(44+samples*2);wav.write('RIFF');wav.writeUInt32LE(wav.length-8,4);wav.write('WAVEfmt ',8);wav.writeUInt32LE(16,16);wav.writeUInt16LE(1,20);wav.writeUInt16LE(1,22);wav.writeUInt32LE(44100,24);wav.writeUInt32LE(88200,28);wav.writeUInt16LE(2,32);wav.writeUInt16LE(16,34);wav.write('data',36);wav.writeUInt32LE(samples*2,40);for(let i=0;i<samples;i++)wav.writeInt16LE(Math.round(Math.sin(i*440*2*Math.PI/44100)*1000),44+i*2);
  await fs.writeFile(path.join(source,'Musik.wav'),wav);
  await page.getByRole('button',{name:'Gallery',exact:true}).first().click();
  await page.getByRole('button',{name:'New folder',exact:true}).click();await page.getByLabel('Folder name',{exact:true}).fill('Medien-Test');await page.getByRole('button',{name:'Create folder',exact:true}).click();
  await page.getByRole('button',{name:'Medien-Test',exact:true}).click();
  let imported=await invoke('gallery_import',{folder:'Medien-Test',sources:['Testbild ü.png','Bewegung.webm','Musik.wav'].map(n=>path.join(source,n))});assert.deepEqual(imported.errors,[]);assert.equal(imported.imported.length,3);
  for(const filename of ['Testbild ü.png','Bewegung.webm','Musik.wav'])assert.equal(hash(await fs.readFile(path.join(source,filename))),hash(await fs.readFile(path.join(gallery,'Medien-Test',filename))));
  const rows=page.locator('.gallery-file');await until(async()=>await rows.count()===3);
  await rows.filter({hasText:'Testbild ü.png'}).click();await until(()=>page.locator('.gallery-viewer img').evaluate(i=>i.complete&&i.naturalWidth===320));
  const zoom=page.getByLabel('Image zoom',{exact:true});await zoom.focus();await zoom.press('Home');
  const fitted=await page.locator('.gallery-viewer img').evaluate(i=>i.getBoundingClientRect().width);
  for(let i=0;i<4;i++)await zoom.press('ArrowRight');
  // Scrollbars can reduce the viewport once the image is zoomed. Measure the
  // current usable viewport instead of assuming its pre-zoom width is unchanged.
  await until(()=>page.locator('.gallery-viewer img').evaluate((i,fitted)=>{const v=i.closest('.gallery-image-scroll');const expected=2*Math.min(v.clientWidth/i.naturalWidth,v.clientHeight/i.naturalHeight)*i.naturalWidth;return i.getBoundingClientRect().width>1.9*fitted&&Math.abs(i.getBoundingClientRect().width-expected)<2;},fitted),3000);
  const scroll=page.locator('.gallery-image-scroll');const box=await scroll.boundingBox();
  await page.mouse.move(box.x+box.width*.7,box.y+box.height*.5);await page.mouse.down();await page.mouse.move(box.x+box.width*.4,box.y+box.height*.5,{steps:6});await page.mouse.up();
  assert(await scroll.evaluate(el=>el.scrollLeft)>0,'Dragging the zoomed image pans the view');
  await page.getByRole('button',{name:'Fit',exact:true}).click();assert.equal(await scroll.evaluate(el=>el.scrollLeft),0);
  record('Real gallery folder creation and verified image/video/audio copy import; originals unchanged; Unicode image decodes and zoom works');
  await rows.filter({hasText:'Musik.wav'}).click();await until(()=>page.locator('audio').evaluate(a=>a.readyState>=1&&a.duration===2));
  await page.locator('audio').evaluate(async a=>{a.muted=true;await a.play();});await until(()=>page.locator('audio').evaluate(a=>a.currentTime>0.2));await page.locator('audio').evaluate(a=>a.pause());
  await rows.filter({hasText:'Bewegung.webm'}).click();await until(()=>page.locator('video').evaluate(v=>v.readyState>=1&&v.videoWidth===320));
  await page.locator('video').evaluate(async v=>{v.muted=true;await v.play();});await until(()=>page.locator('video').evaluate(v=>v.currentTime>0.2));await page.locator('video').evaluate(v=>v.pause());
  await page.screenshot({path:path.join(artifactRoot,'gallery-video.png')});
  record('Native gallery plays real local PCM WAV audio and VP8 WebM video through restricted media protocol; no autoplay');
  await fs.copyFile(path.join(source,'Testbild ü.png'),path.join(gallery,'Medien-Test','Explorer-Zugang.png'));await until(async()=>await rows.count()===4);
  await page.getByLabel('Media type',{exact:true}).selectOption('audio');await until(async()=>await rows.count()===1);assert.match(await rows.innerText(),/Musik/);
  await page.getByLabel('Media type',{exact:true}).selectOption('all');await page.getByLabel('Search files',{exact:true}).fill('EXPLORER');await until(async()=>await rows.count()===1&&/Explorer-Zugang/.test(await rows.innerText()));await page.getByLabel('Search files',{exact:true}).fill('');
  imported=await invoke('gallery_import',{folder:'Medien-Test',sources:[path.join(source,'Musik.wav')]});assert.equal(imported.imported.length,1);assert.notEqual(imported.imported[0],'Medien-Test/Musik.wav');assert.equal(hash(await fs.readFile(path.join(gallery,'Medien-Test','Musik.wav'))),hash(wav));
  await assert.rejects(invoke('gallery_detail',{path:'../outside.wav'}),/gallery_path/);await assert.rejects(invoke('gallery_create_folder',{folder:'',name:'../escape'}),/gallery_name/);
  const partial=await invoke('gallery_import',{folder:'Medien-Test',sources:[path.join(source,'missing.png'),path.join(source,'Musik.wav')]});assert.equal(partial.errors.length,1);assert.equal(partial.imported.length,1);
  record('Explorer-added files appear automatically; media/search filters work; duplicate import never overwrites; invalid paths and partial errors are explicit');
  // Direct range request from trusted main webview, matching the media player's access.
  const detail=await invoke('gallery_detail',{path:'Medien-Test/Musik.wav'});
  const read=await page.evaluate(async url=>{const r=await fetch(url,{headers:{Range:'bytes=4-15'}});return {status:r.status,range:r.headers.get('Content-Range'),bytes:[...new Uint8Array(await r.arrayBuffer())]};},detail.url);
  assert.equal(read.status,206);assert.deepEqual(read.bytes,[...wav.subarray(4,16)]);
  await fs.writeFile(path.join(gallery,'Medien-Test','unsupported.mp4'),'invalid video fixture');await until(async()=>await rows.filter({hasText:'unsupported.mp4'}).count()===1);await rows.filter({hasText:'unsupported.mp4'}).click();await page.getByText('Preview unavailable.',{exact:false}).waitFor();
  await rows.filter({hasText:'Testbild ü.png'}).click();await page.screenshot({path:path.join(artifactRoot,'gallery-images.png')});
  record('Media range endpoint returns exact requested bytes; damaged media shows a playback error without changing the file');
  const before=(await invoke('gallery_list',{query})).entries.filter(e=>e.path.startsWith('Medien-Test/')).map(e=>e.path).sort();await stop(true);await launch();page=getPage();
  const after=(await invoke('gallery_list',{query})).entries.filter(e=>e.path.startsWith('Medien-Test/')).map(e=>e.path).sort();assert.deepEqual(after,before);
  await page.getByRole('button',{name:'Gallery',exact:true}).first().click();await page.getByLabel('Search files',{exact:true}).fill('Testbild');await until(()=>page.locator('.gallery-viewer img').evaluate(i=>i.complete&&i.naturalWidth===320));
  record('Real gallery files and folders remain available across clean app restart independently of image-job history');
}
