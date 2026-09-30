# Local Studio 0.36.33

- Fixed a Wan GIF failure where a Wan 2.1 model completed sampling but ComfyUI tried to decode its 16-channel latent with a Wan 2.2 VAE expecting 48 channels. The adapter now selects a VAE that matches the model generation.
- Wan 2.2 models use the dedicated `Wan22ImageToVideoLatent` node and a Wan 2.2 VAE. Missing compatible VAEs are reported before sampling. ComfyUI channel-mismatch and GPU-memory errors have more specific messages.

## Verification

- The affected local ComfyUI log and installed VAE inventory identified the mismatch. Frontend build, Rust check and focused model/VAE matching test passed.
- Isolated native end-to-end runs produced real GIF files with the installed Wan 2.1 GGUF and Wan 2.2 Safetensors models. These were five-frame, one-step checks of execution and decoding; they do not establish output quality at normal settings.
