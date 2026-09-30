# Local Studio 0.36.31

- GIF Studio can create a GIF from local image frames or generate frames locally with a selected Wan image-to-video model through ComfyUI.
- The Wan path accepts a start image, movement prompt or tags, negative prompt, output dimensions and frame count. It validates the local ComfyUI nodes and exact installed component names before starting.
- Temporary imports and decoded frames are removed after the GIF is stored in the selected gallery folder.

## Verification

- TypeScript/Vite production build, Rust compiler check and the two focused GIF encoder tests passed.
- No Wan GPU generation was run for this release: the local ComfyUI API on port 8188 was unavailable during verification. The app reports that unavailable engine state before submitting a workflow.