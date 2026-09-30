# Local Studio 0.36.32

- The GIF Studio now uses the Image Studio layout: model and generation controls on the left, a large preview canvas in the middle, and the shared folder gallery on the right. Both side panels can be resized.
- Installed Wan Safetensors and GGUF models in the configured ComfyUI diffusion-model folder appear in a model selector. Gallery images can be chosen as the AI start image or added to the frame list. The existing tag importer is available beside the GIF prompt.
- The local main-window permissions now include the Wan GIF generation command and the new model-list and source-preview commands.

## Verification

- Frontend production build, Rust check, focused Wan-catalog and GIF-encoder tests passed.
- An isolated native Windows run verified the layout, gallery selection, local image preview, a real GIF export and keyboard resizing. A screenshot was inspected.
- A GPU Wan generation was not run; that path requires a configured and running ComfyUI with a compatible model, text encoder and VAE.
