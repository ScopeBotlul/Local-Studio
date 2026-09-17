# Hugging Face integration, 0.4.0

This is the first implemented part of M1. It does not complete the model manager or add inference.

## Implemented

- Native Rust HTTPS API client, explicit search, task and sort filters, cursor pagination.
- Model details resolve a branch/tag/commit to a specific commit and show repository metadata, license, gated/private status, filenames, known sizes and LFS SHA-256 values. Unknown sizes remain unknown.
- Model cards are fetched from that exact revision and rendered as escaped plain text. No model-provided HTML, remote image, script or executable code runs in the app.
- The executable-model filter correctly returns no models while no inference adapters exist. No automatic variant selection or inference occurs. Explicit file downloads are available from 0.4.0; see DOWNLOADS.md.
- Advanced personal-token sign-in validates the token with the official `whoami-v2` endpoint before storing it. One account per app profile; sign-out deletes the credential. A saved account can be explicitly checked online.
- Credentials and account metadata use Windows Credential Manager (`keyring`, Windows native backend), keyed by the canonical configuration path. No plaintext fallback, database token, localStorage token, token logging or token read IPC command. Moving a portable profile to another path/PC requires signing in again; updating program files in the same folder keeps the profile identity.
- OAuth authorization-code implementation uses a public client, S256 PKCE, unpredictable state, a temporary IPv4 loopback listener on an OS-assigned port and, from 0.3.1, the isolated integrated browser requested by the user. Host, path, method and single state/code values are checked. Unrelated callbacks are rejected; cancellation, timeout and generation checks stop late responses from restoring a signed-out account. A refresh token, if issued, stays in the same Windows credential and is used before expiry.
- Official website and account-management links open with the Windows URL association API. No shell interpreter, arbitrary URL opener or broad Tauri shell permission is exposed.

## OAuth registration completed after explicit approval

On September 17, 2026 the user explicitly approved registering Local Studio as a persistent public OAuth application with profile and repository read access. The previously rejected operation was then executed once successfully.

The exact submitted payload is [huggingface-oauth-registration.json](huggingface-oauth-registration.json). Hugging Face returned public client ID `322a729f-1375-4456-8b0e-d3dbea8b8d31`, authentication method `none`, scope `openid profile read-repos`, and redirect `http://127.0.0.1/callback`. The non-secret registration response is recorded in [huggingface-oauth-client.json](huggingface-oauth-client.json). No client secret or user credential was created or requested. The registration payload identifies the integration's original version 0.2.0; the build enabling this client is 0.2.1.

`src-tauri/huggingface-oauth.json` contains this public client ID and is embedded at build time. Browser sign-in is enabled. Registration is not repeated by the app or test suite. From 0.3.1 the user completes sign-in and consent in the integrated browser. The listener waits up to three minutes; if it times out, start sign-in again.

The anonymous provider check [oauth-provider-0.2.1.json](../.artifacts/oauth-provider-0.2.1.json) confirms an authorization request with this client and an ephemeral loopback port redirects to the official HF login page (HTTP 302 then 200). This does not prove a completed account login or token exchange. The browser inspection helper failed technically, so this provider check is HTTP-based, not a claimed visual browser verification.

## Verification boundaries

The native smoke extension `scripts/check-huggingface.mjs` is opt-in via `LOCAL_STUDIO_TEST_HF=1`. It exercises real public API calls and an intentionally invalid synthetic token, without accessing existing accounts or registering an OAuth client. The integrated OAuth test is part of `LOCAL_STUDIO_TEST_BROWSER=1`; it uses the isolated WebView profile and no system-browser session. The runner deletes any credential in its isolated test profile during cleanup. Native account storage and callback tests use unique temporary profiles and synthetic credentials; those tests are not evidence of a successful real-account login.

The user reported successful app sign-in and confirmed the username plus verified-account status after checking the connection on September 17, 2026. This is user-reported happy-path evidence, not an automated account login. Real valid personal-token login, account switching with two real accounts, refresh-token issuance/renewal and gated/private repository access still require account-based verification. Until those checks are recorded as passed, do not describe them as end-to-end verified.

Version 0.3.0 implements the embedded website described below. Downloads/pause/resume, scan/import, model installation and all inference remain open. Full M0 configuration and other unfinished milestones remain in PLAN.md.

## Network and security boundaries

No network request is made merely by reading account status or opening the model page. Search/model-detail requests send only the entered query/repository/revision and, when connected, the relevant bearer credential to fixed HTTPS Hugging Face endpoints. API redirects are refused. Returned pagination links contribute only an opaque cursor after origin/path validation; they never become arbitrary request URLs. All responses have size/time limits, and error response bodies are never returned to the UI or logs. The account token endpoint and whoami endpoint are fixed. A failed account lookup cannot persist an unverified token.

The capability matches only the local `main` WebView, not its parent window. No remote origin or `hf-website` child is granted a capability. Native browser controller commands additionally check the caller WebView label. TLS verification uses Rustls roots. This is a bounded implementation review, not an assurance that all dependencies or all attack paths have been audited.

## Official references consulted September 17, 2026

- [Hugging Face native/public OAuth and loopback matching](https://huggingface.co/docs/hub/oauth)
- [OpenID provider metadata](https://huggingface.co/.well-known/openid-configuration)
- [Official OpenAPI specification, including public OAuth registration](https://huggingface.co/.well-known/openapi.json)
- [Official Hub search client](https://github.com/huggingface/huggingface_hub/blob/main/src/huggingface_hub/hf_api.py)
- [Windows ShellExecuteW](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew)

## Embedded website in 0.3.0

`Hugging Face → Website in Studio` creates a native child WebView in the main window. The website uses its own persistent `config/huggingface-webview` directory. It is hidden on page changes and reused when reopened. Back/forward, reload, home, address navigation and a default-browser button are implemented. Recognized model URLs can be handed to the existing repository detail view; this does not download or install a model.

Only exact HTTPS `huggingface.co` and `hf.co` destinations remain inside. Other permitted HTTPS links open through the Windows URL association API. File/script/local application schemes, credential-bearing URLs and local/IP destinations are rejected. Popups cannot create uncontrolled WebViews. Website downloads are denied until a real download path is provided; the UI explains how to open the page externally. External website resources remain governed by the website and WebView2, not by the local app CSP.

App OAuth secrets remain in Windows Credential Manager. Website cookies remain in the WebView profile; app logout does not log out the website. No token-to-cookie injection exists. Website login is performed by the user on the website. The browser has no privileged app capabilities; native tests observed ACL rejection for bootstrap, account status, browser control status and job listing invoked from the real remote page.

`LOCAL_STUDIO_TEST_BROWSER=1` extends the native smoke runner with real website rendering, denied IPC, HF-only address validation, actual repository handoff, tab lifecycle, scaled view bounds and restart persistence of a synthetic non-auth cookie. The runner uses an isolated WebView2 directory override, never a user's website profile. This proves persistence mechanics, not a real website login or every identity-provider flow. Parent and child WebViews are captured separately because CDP screenshots do not composite native child views. Refresh, real website login persistence, full accessibility and installer lifecycle remain unverified.

## App OAuth inside Studio, 0.3.1

The user's explicit request supersedes the earlier system-browser behavior: `Sign in with Hugging Face` now prepares the authorization URL natively and selects the embedded website view. If the child WebView does not yet exist, its first navigation uses the pending authorization URL. Existing website cookies may let Hugging Face recognize the account; the OAuth grant/token remains separate and requires the provider's normal consent rules. There is no password extraction or token-to-cookie injection.

The only additional navigation exception is the exact active `http://127.0.0.1:<ephemeral port>/callback` URL. Other ports, paths, schemes, credentials and fragments are rejected. The listener still checks state, Host, path and method; PKCE verification and token exchange remain in Rust. A generation check invalidates cancelled or superseded attempts. Completion/denial/cancellation removes the exception, returns the WebView to the HF homepage, and returns the UI to the account view. No new privileged capability is granted.

`scripts/check-integrated-oauth.mjs` uses a fresh isolated profile to load the actual HF password page, inspect the PKCE request, verify denied IPC, reject a forged callback, exercise a valid-state denial through the real child WebView, retry with new state, and cancel with listener cleanup. No real password, successful consent or fabricated successful token response is used. The previously user-confirmed external-browser login is not evidence of a completed embedded login. Real-account completion and third-party SSO remain to be checked by the user.

Security tradeoff: [RFC 8252 §8.12](https://www.rfc-editor.org/rfc/rfc8252.html#section-8.12) requires external user-agents for native OAuth. The requested embedded flow intentionally departs from that recommendation; an app-owned WebView does not provide the same separation from credentials as a system browser. PKCE and restricted capabilities do not remove that tradeoff. Hugging Face's [OAuth documentation](https://huggingface.co/docs/hub/oauth) supplies the public-client/PKCE/loopback protocol. Identity providers may restrict embedded sign-in; no provider-blocking bypass is implemented. External website links retain the existing system-browser policy.


## User confirmation and downloads, 0.4.0

The user explicitly confirmed that the integrated app login works ("okay login klappt"). This is user-reported successful authentication, not an automated refresh, account-switch or website-cookie persistence test. Version 0.4.0 adds explicit file selection, pinned verified downloads and local file inventory. Current implementation, tests and remaining M1 scope are in [DOWNLOADS.md](DOWNLOADS.md); earlier statements that downloads remain open describe the preceding releases.
