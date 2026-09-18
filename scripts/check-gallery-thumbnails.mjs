import assert from 'node:assert/strict';
import { promises as fs } from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
const pause=ms=>new Promise(r=>setTimeout(r,ms));
async function until(fn,timeout=20000){const end=Date.now()+timeout;while(Date.now()<end){if(await fn())return;await pause(120);}throw new Error('Thumbnail condition timed out');}
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
export async function checkGalleryThumbnails({getPage,invoke,stop,launch,artifactRoot,record}) {
  let page=getPage(); const gallery=(await invoke('bootstrap')).paths.gallery;
  const fixture=await page.evaluate(()=>{
    const canvas=document.createElement('canvas');canvas.width=1600;canvas.height=900;
    const ctx=canvas.getContext('2d');ctx.fillStyle='#274060';ctx.fillRect(0,0,1600,900);ctx.fillStyle='#a78bfa';ctx.fillRect(200,200,450,450);
    return canvas.toDataURL('image/png').split(',')[1];
  });
  const source=Buffer.from(fixture,'base64'); await invoke('gallery_create_folder',{folder:'',name:'Raster-Test'});
  for(let i=0;i<61;i++)await fs.writeFile(path.join(gallery,'Raster-Test',`Bild-${String(i).padStart(2,'0')}.png`),source);
  const query={folder:'Raster-Test',search:'',kind:'image',recursive:true,offset:0};
  const list=await invoke('gallery_list',{query}); assert.equal(list.total,61);assert.equal(list.entries.length,50);
  const e=list.entries[0]; const request={rootId:list.rootId,path:e.path,version:e.thumbnailVersion};
  const first=await invoke('gallery_thumbnail',{query:request});assert.equal(first.cached,false);assert.equal(first.cacheStored,true);assert.deepEqual([first.width,first.height],[320,180]);
  const second=await invoke('gallery_thumbnail',{query:request});assert.equal(second.cached,true);assert.equal(second.dataUrl,first.dataUrl);
  await assert.rejects(invoke('gallery_thumbnail',{query:{...request,rootId:'wrong'}}),/gallery_changed/);
  await assert.rejects(invoke('gallery_thumbnail',{query:{...request,path:'../outside.png'}}),/gallery_path/);
  await page.getByRole('button',{name:'Gallery',exact:true}).first().click();await page.getByLabel('Search files',{exact:true}).fill('Raster-Test');
  const rows=page.locator('.gallery-file');await until(async()=>await rows.count()===50&&/Raster-Test/.test(await rows.first().innerText()));
  await page.getByRole('button',{name:'Grid view',exact:true}).click();await page.locator('.gallery-items-grid').scrollIntoViewIfNeeded();
  await until(async()=>await rows.first().locator('.gallery-thumbnail img').count()===1);
  assert.equal(await rows.first().locator('.gallery-thumbnail img').evaluate(i=>i.complete&&i.naturalWidth===320&&i.naturalHeight===180),true);
  // Inspect actual decoded cards; the Tauri invoke property is not replaceable.
  await pause(500);
  const loadedBefore=await rows.locator('.gallery-thumbnail img').count();assert(loadedBefore>0&&loadedBefore<50,`Lazy loading: ${loadedBefore}`);
  await page.screenshot({path:path.join(artifactRoot,'gallery-grid-light.png')});
  await page.locator('.gallery-items-grid').evaluate(el=>{el.scrollTop=el.scrollHeight;});
  await until(async()=>await rows.locator('.gallery-thumbnail img').count()>loadedBefore);
  const last=rows.last();await last.click();const selected=await last.getAttribute('data-gallery-path');
  await until(()=>page.locator('.gallery-viewer img').evaluate(i=>i.complete&&i.naturalWidth===1600));
  await page.getByRole('button',{name:'List view',exact:true}).click();assert.equal(await page.locator('.gallery-thumbnail').count(),0);assert.equal(await page.locator('.gallery-file[aria-pressed="true"]').getAttribute('data-gallery-path'),selected);
  await page.locator('.gallery-pagination button').last().click();await until(async()=>await rows.count()===11);await page.locator('.gallery-pagination button').first().click();await until(async()=>await rows.count()===50);
  record('Real native image grid uses decoded 320px thumbnails, loads only visible cards, scrolls lazily and retains selection in list view; 61-file pagination remains bounded to 50');
  await stop(true);await launch();page=getPage();
  assert.equal((await invoke('gallery_thumbnail',{query:request})).cached,true);
  await page.getByRole('button',{name:'Gallery',exact:true}).first().click();assert.equal(await page.getByRole('button',{name:'List view',exact:true}).getAttribute('aria-pressed'),'true');
  await page.getByLabel('Search files',{exact:true}).fill('Raster-Test');await page.getByRole('button',{name:'Grid view',exact:true}).click();
  await page.getByRole('button',{name:'Rebuild thumbnails',exact:true}).click();await page.getByText('Preview cache cleared.',{exact:false}).waitFor();
  await page.locator('.gallery-file').first().scrollIntoViewIfNeeded();await until(async()=>await page.locator('.gallery-thumbnail img').count()>0);
  for(let i=0;i<61;i++)assert.equal(digest(await fs.readFile(path.join(gallery,'Raster-Test',`Bild-${String(i).padStart(2,'0')}.png`))),digest(source));
  record('Thumbnail cache and list preference survive native restart; UI cache rebuild regenerates visible previews and all 61 originals remain byte-identical');
  // Replace a fixture externally: the stale version is refused and polling refreshes the card.
  const changed=await page.evaluate(()=>{const c=document.createElement('canvas');c.width=300;c.height=600;const x=c.getContext('2d');x.fillStyle='#D04D64';x.fillRect(0,0,300,600);return c.toDataURL().split(',')[1];});
  await fs.writeFile(path.join(gallery,e.path),Buffer.from(changed,'base64'));
  await assert.rejects(invoke('gallery_thumbnail',{query:request}),/gallery_changed/);
  await page.getByLabel('Search files',{exact:true}).fill(e.name);
  await until(async()=>await page.locator('.gallery-file').count()===1&&await page.locator('.gallery-thumbnail img').count()===1&&await page.locator('.gallery-thumbnail img').evaluate(i=>i.complete&&i.naturalWidth===160&&i.naturalHeight===320));
  const portrait=await page.locator('.gallery-thumbnail').boundingBox();assert(Math.abs(portrait.width/portrait.height-4/3)<0.03,'Portrait thumbnail stays within the fixed 4:3 card area');
  await fs.writeFile(path.join(gallery,'Raster-Test','Kaputt.png'),Buffer.from('broken PNG fixture'));
  await page.getByLabel('Search files',{exact:true}).fill('Kaputt');await page.getByText('No thumbnail',{exact:true}).waitFor();await page.getByText('Preview unavailable.',{exact:false}).waitFor();
  assert.equal(await fs.readFile(path.join(gallery,'Raster-Test','Kaputt.png'),'utf8'),'broken PNG fixture');
  // Use actual persisted theme settings for the second visual check.
  const settings=(await invoke('bootstrap')).settings;await invoke('save_settings',{settings:{...settings,theme:'dark'}});await stop(true);await launch();page=getPage();
  await page.getByRole('button',{name:'Gallery',exact:true}).first().click();await page.getByLabel('Search files',{exact:true}).fill('Raster-Test');await until(async()=>await page.locator('.gallery-file').count()===50);await page.locator('.gallery-file').first().scrollIntoViewIfNeeded();await until(async()=>await page.locator('.gallery-thumbnail img').count()>0);
  await page.screenshot({path:path.join(artifactRoot,'gallery-grid-dark.png')});
  record('External image replacement refreshes the grid using file versions; damaged image keeps an explicit fallback and unchanged original; light and dark native screenshots captured');
}
