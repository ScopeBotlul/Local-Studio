import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
const exec=promisify(execFile),pause=ms=>new Promise(r=>setTimeout(r,ms));
export async function popupMenu(page,pid,group,label){
 await page.getByRole('menuitem',{name:group,exact:true}).click();
 for(let i=0;i<100;i++){
  try{const {stdout}=await exec('python',['scripts/native-popup27.py',String(pid),...(label?[label]:[])],{windowsHide:true,timeout:10000});const result=JSON.parse(stdout);await page.waitForFunction(group=>[...document.querySelectorAll('[role="menubar"] [role="menuitem"]')].find(b=>b.textContent===group)?.getAttribute('aria-expanded')==='false',group);return result;}
  catch(e){if(!String(e).includes('No native popup'))throw e;await pause(100);}
 }
 throw Error('Native popup did not appear: '+group);
}
