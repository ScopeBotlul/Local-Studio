// Anonymous provider check. Does not approve consent, exchange a code or use a user account.
import { randomBytes, createHash } from 'node:crypto';
import { promises as fs } from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';

const root = path.resolve(import.meta.dirname, '..');
const { clientId } = JSON.parse(await fs.readFile(path.join(root, 'src-tauri/huggingface-oauth.json'), 'utf8'));
assert.match(clientId, /^[A-Za-z0-9_-]+$/);
const url = new URL('https://huggingface.co/oauth/authorize');
const verifier = randomBytes(32).toString('base64url');
url.search = new URLSearchParams({
  client_id: clientId, redirect_uri: 'http://127.0.0.1:49282/callback',
  response_type: 'code', scope: 'openid profile read-repos',
  state: randomBytes(32).toString('base64url'),
  code_challenge: createHash('sha256').update(verifier).digest('base64url'), code_challenge_method: 'S256',
}).toString();
const response = await fetch(url, { redirect: 'manual', signal: AbortSignal.timeout(30000) });
const redirect = response.headers.get('location');
assert.ok(redirect, `Expected anonymous authorization to redirect to sign-in, got HTTP ${response.status}`);
const login = new URL(redirect, url);
assert.equal(login.origin, 'https://huggingface.co');
assert.equal(login.pathname, '/login');
const loginResponse = await fetch(login, { redirect: 'manual', signal: AbortSignal.timeout(30000) });
assert.equal(loginResponse.status, 200);
const html = await loginResponse.text();
assert.ok(html.includes('Log In') || html.includes('Log in'));
const report = { checkedAt: new Date().toISOString(), clientId, authorizationStatus: response.status, anonymousDestination: login.origin + login.pathname, loginPageStatus: loginResponse.status, passed: true, limitation: 'Anonymous redirect and login page only; no real account consent or token exchange.' };
await fs.mkdir(path.join(root, '.artifacts'), { recursive: true });
await fs.writeFile(path.join(root, '.artifacts/oauth-provider-0.2.1.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
