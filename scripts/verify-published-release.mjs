// Verify the public update channel independently of the signing process.
import assert from 'node:assert/strict';
import {readFile,mkdir,writeFile} from 'node:fs/promises';
import {createHash,verify} from 'node:crypto';
import path from 'node:path';
const root=path.resolve(import.meta.dirname,'..');
const version=JSON.parse(await readFile(path.join(root,'package.json'),'utf8')).version;
async function get(url){const r=await fetch(url,{signal:AbortSignal.timeout(120000),headers:{'User-Agent':'Local-Studio-release-verification'}});assert(r.ok,`${r.status}: ${url}`);return Buffer.from(await r.arrayBuffer());}
const release=JSON.parse(await get('https://api.github.com/repos/ScopeBotlul/Local-Studio/releases/latest'));
assert.equal(release.tag_name,`v${version}`);assert.equal(release.draft,false);assert.equal(release.prerelease,false);
const base=`https://github.com/ScopeBotlul/Local-Studio/releases/download/v${version}/`;
const [bytes,sig,rawKey]=await Promise.all([get(base+'update.json'),get(base+'update.sig'),readFile(path.join(root,'src-tauri/update-public-key.txt'),'utf8')]);
const key=Buffer.concat([Buffer.from('302a300506032b6570032100','hex'),Buffer.from(rawKey.trim(),'hex')]);
assert(verify(null,bytes,{key,format:'der',type:'spki'},Buffer.from(sig.toString(),'base64')),'Public manifest signature');
const manifest=JSON.parse(bytes);assert.equal(manifest.version,version);assert.equal(manifest.schema,1);
const result={passed:true,version,verifiedAt:new Date().toISOString(),signature:true,assets:[]};
for(const kind of ['installer','portable']){
 const a=manifest[kind],name=`Local-Studio-${version}-hub-${kind==='installer'?'setup.exe':'portable.zip'}`;assert.equal(a.url,base+name);
 const b=await get(a.url),sha256=createHash('sha256').update(b).digest('hex');assert.equal(b.length,a.size);assert.equal(sha256,a.sha256);
 assert.equal(createHash('sha256').update(await readFile(path.join(root,'releases',name))).digest('hex'),sha256);
 result.assets.push({kind,size:b.length,sha256});console.log('PASS public download:',name);
}
for(const name of ['update.json','update.sig',`SHA256SUMS-${version}.txt`,`Local-Studio-${version}-hub-setup.exe`,`Local-Studio-${version}-hub-portable.zip`]){
 const asset=release.assets.find(a=>a.name===name);assert(asset,name);const local=await readFile(path.join(root,'releases',name));assert.equal(asset.size,local.length);assert.equal(asset.digest,'sha256:'+createHash('sha256').update(local).digest('hex'));
}
await mkdir(path.join(root,'.artifacts'),{recursive:true});await writeFile(path.join(root,`.artifacts/github-release-verification-${version}.json`),JSON.stringify(result,null,2));
console.log('PASS latest release, signature, full package downloads and all five GitHub asset digests');
