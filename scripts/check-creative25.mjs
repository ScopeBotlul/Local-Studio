import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {checkCreative23} from './check-creative23.mjs';
const pause=ms=>new Promise(r=>setTimeout(r,ms));
async function until(fn){for(let i=0;i<200;i++){const value=await fn();if(value)return value;await pause(100);}throw Error('Creative25 timeout');}
export async function checkCreative25(ctx){
 await checkCreative23(ctx);
 const {invoke,record,artifactRoot}=ctx;const page=ctx.getPage();
 const settings=(await invoke('bootstrap')).settings;await invoke('save_settings',{settings:{...settings,language:'en'}});await page.reload();
 await page.getByRole('button',{name:'Studio',exact:true}).first().click();await page.getByRole('tab',{name:'Image editor',exact:true}).click();
 const project=await invoke('project_snapshot'),document=project.creative.image;
 const layer=document.layers.find(l=>l.mask?.strokes.length);assert(layer);
 await page.locator('.layer-row>.text-button').nth([...document.layers].reverse().findIndex(l=>l.id===layer.id)).click();
 await page.getByRole('button',{name:'Export mask as PNG',exact:true}).click();
 const note=page.getByText('Mask saved as PNG in the gallery:',{exact:false});await note.waitFor();
 const relative=(await note.innerText()).split('Mask saved as PNG in the gallery: ')[1];assert(relative?.endsWith('.png'));
 const root=(await invoke('bootstrap')).paths.gallery;const data=await fs.readFile(path.join(root,relative));
 const preview=await invoke('canvas_preview',{id:project.id,document,format:'png',quality:100,exportSize:null,maskLayer:layer.id});
 const pixels=await page.evaluate(async({a,b})=>{async function read(src,w,h){const i=new Image();i.src=src;await i.decode();const c=document.createElement('canvas');c.width=w??i.width;c.height=h??i.height;const x=c.getContext('2d');x.imageSmoothingEnabled=false;x.drawImage(i,0,0,c.width,c.height);return {pixels:x.getImageData(0,0,c.width,c.height).data,width:c.width,height:c.height};}const source=await read(a),view=await read(b,source.width,source.height),x=source.pixels,y=view.pixels;return {equal:x.length===y.length&&x.every((v,i)=>v===y[i]),black:x.some((v,i)=>i%4===0&&v===0),white:x.some((v,i)=>i%4===0&&v===255),opaque:x.every((v,i)=>i%4!==3||v===255)};},{a:'data:image/png;base64,'+data.toString('base64'),b:preview.dataUrl});
 assert.deepEqual(pixels,{equal:true,black:true,white:true,opaque:true});
 await assert.rejects(invoke('canvas_export_mask',{id:project.id,document,layerId:'missing'}));
 assert.deepEqual((await invoke('project_snapshot')).creative,project.creative);
 record('Layer mask export UI writes a real opaque grayscale PNG matching the shared mask preview at source dimensions; invalid layer IDs are rejected and the edit recipe stays unchanged');
 const encoded=await page.evaluate(()=>{const c=document.createElement('canvas');c.width=c.height=512;const x=c.getContext('2d');x.fillStyle='#456789';x.fillRect(0,0,512,512);return c.toDataURL().split(',')[1];});
 const source=path.join(artifactRoot,'Gallery reference.png');await fs.writeFile(source,Buffer.from(encoded,'base64'));
 const imported=await invoke('gallery_import',{folder:'',sources:[source]});assert.equal(imported.imported.length,1);
 await page.getByRole('tab',{name:'Image generation',exact:true}).click();await page.getByLabel('Image prompt',{exact:true}).fill('Keep my existing prompt');
 await page.getByRole('button',{name:'Gallery',exact:true}).first().click();await page.locator('[data-gallery-path]').filter({hasText:'Gallery reference.png'}).click();
 await page.getByRole('button',{name:'Use as image reference',exact:true}).click();await page.locator('summary').filter({hasText:'Reference image and inpainting'}).click();await page.getByRole('img',{name:'Reference image',exact:true}).waitFor();
 const request=await until(async()=>(await invoke('image_workspace')).workspace.request?.reference?(await invoke('image_workspace')).workspace.request:null);
 assert.equal(request.prompt,'Keep my existing prompt');assert.equal(request.reference.width,512);assert.equal(request.reference.sha256,createHash('sha256').update(Buffer.from(encoded,'base64')).digest('hex'));assert.equal((await invoke('image_jobs')).length,0);
 record('Gallery reference button adopts the actual PNG dimensions/hash, preserves the current prompt and opens Studio without starting a generation');
}
