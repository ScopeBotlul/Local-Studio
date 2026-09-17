import assert from 'node:assert/strict';
import path from 'node:path';
import http from 'node:http';
function callbackStatus(url) {
  return new Promise((resolve, reject) => {
    const request = http.get(url, { agent: false, timeout: 5000 }, response => { response.resume(); resolve(response.statusCode); });
    request.on('error', error => reject(new Error(`Fresh loopback probe failed: ${error.code}`)));
    request.on('timeout', () => request.destroy(new Error('Loopback probe timeout')));
  });
}
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
async function until(work, message) {
  for (let i=0;i<120;i++) { const value=await work(); if(value) return value; await pause(250); }
  throw new Error(message);
}
function authorization(value, depth=0) {
  if(depth>5) return null;
  try {
    const url=new URL(value,'https://huggingface.co');
    if(url.origin==='https://huggingface.co' && url.pathname==='/oauth/authorize' && url.searchParams.has('state')) return url;
    for(const nested of url.searchParams.values()) { const found=authorization(nested,depth+1); if(found) return found; }
  }catch{}
  return null;
}
export async function checkIntegratedOAuth(page,browser,artifactRoot,invoke,record) {
  await page.getByRole('button',{name:'Hugging Face',exact:true}).click();
  await page.getByRole('button',{name:'Sign in with Hugging Face',exact:true}).click();
  await page.getByRole('status').getByText('Complete sign-in here in the integrated browser.',{exact:true}).waitFor();
  const website=await until(()=>browser.contexts().flatMap(c=>c.pages()).find(p=>p!==page && p.url().startsWith('https://huggingface.co/')),'Embedded OAuth WebView missing');
  const auth=await until(()=>authorization(website.url()),'OAuth parameters missing from embedded login redirect');
  assert.equal(auth.searchParams.get('code_challenge_method'),'S256');
  const callback=new URL(auth.searchParams.get('redirect_uri'));
  assert.equal(callback.hostname,'127.0.0.1');assert.equal(callback.pathname,'/callback');
  await website.waitForURL(url=>url.pathname==='/login');
  await website.locator('input[type="password"]').waitFor({timeout:30000});
  await website.screenshot({path:path.join(artifactRoot,'hf-integrated-oauth-login.png')});
  assert.equal((await invoke('hf_status')).pending,true);
  const denial=await website.evaluate(async()=>{try{await window.__TAURI_INTERNALS__.invoke('hf_status');return false;}catch{return true;}});
  assert.equal(denial,true);
  record('Sign-in button opens real HF login in integrated WebView with PKCE and no privileged IPC');

  const forged=new URL(callback);forged.search=new URLSearchParams({state:'wrong',code:'synthetic'}).toString();
  assert.equal(await callbackStatus(forged),400);
  assert.equal((await invoke('hf_status')).pending,true);
  // Exercise the real WebView -> exact loopback navigation with an explicit denial,
  // never a fabricated successful token response or account login.
  const declined=new URL(callback);declined.search=new URLSearchParams({state:auth.searchParams.get('state'),error:'access_denied'}).toString();
  await website.evaluate(url=>{location.href=url;},declined.href);
  await until(async()=>!(await invoke('hf_status')).pending,'Denied OAuth did not finish');
  assert.equal((await invoke('hf_status')).error,'authorization_denied');
  assert.equal((await invoke('hf_status')).account,null);
  await page.getByRole('heading',{name:'Hugging Face account',exact:true}).waitFor();
  await until(async()=>!(await invoke('hf_browser_state')).visible,'OAuth WebView not hidden after denial');
  record('Embedded OAuth callback rejects forged state and returns to account view on denied consent');

  await page.getByRole('button',{name:'Sign in with Hugging Face',exact:true}).click();
  const second=await until(()=>{const value=authorization(website.url());return value && value.searchParams.get('state')!==auth.searchParams.get('state') ? value : null;},'Fresh OAuth state missing on retry');
  const secondCallback=second.searchParams.get('redirect_uri');
  await page.getByRole('button',{name:'Cancel sign-in',exact:true}).click();
  await until(async()=>!(await invoke('hf_status')).pending,'Cancel did not complete');
  await page.getByRole('heading',{name:'Hugging Face account',exact:true}).waitFor();
  await pause(500);
  await assert.rejects(fetch(secondCallback,{signal:AbortSignal.timeout(2000)}));
  assert.equal((await invoke('hf_status')).account,null);
  record('Integrated OAuth retry creates fresh state; cancellation closes listener without signing in');
}
