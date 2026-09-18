import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {createHash,verify} from 'node:crypto';
import path from 'node:path';
const root=path.resolve(import.meta.dirname,'..');
const read=p=>readFile(path.join(root,p));
const version=JSON.parse(await read('package.json')).version;
for(const p of ['package-lock.json','src-tauri/tauri.conf.json'])assert.equal(JSON.parse(await read(p)).version,version,p);
const bytes=await read('releases/update.json');
const signature=Buffer.from((await read('releases/update.sig')).toString(),'base64');
const publicKey=Buffer.concat([Buffer.from('302a300506032b6570032100','hex'),Buffer.from((await read('src-tauri/update-public-key.txt')).toString().trim(),'hex')]);
assert(verify(null,bytes,{key:publicKey,format:'der',type:'spki'},signature),'Manifest signature');
const manifest=JSON.parse(bytes);assert.equal(manifest.schema,1);assert.equal(manifest.version,version);assert(Buffer.byteLength(manifest.notes)<=16000);
const sums=(await read(`releases/SHA256SUMS-${version}.txt`)).toString();
for(const [key,kind] of [['installer','setup.exe'],['portable','portable.zip']]){
 const name=`Local-Studio-${version}-hub-${kind}`;
 const b=await read(`releases/${name}`);const hash=createHash('sha256').update(b).digest('hex');
 assert.equal(manifest[key].url,`https://github.com/ScopeBotlul/Local-Studio/releases/download/v${version}/${name}`);
 assert.equal(manifest[key].size,b.length);assert.equal(manifest[key].sha256,hash);assert(sums.split(/\r?\n/).includes(`${hash}  ${name}`));
 console.log(`PASS ${name}: exact signed size and SHA-256`);
}
console.log(`PASS ${version}: manifest matches embedded public key, versions and release URLs`);
