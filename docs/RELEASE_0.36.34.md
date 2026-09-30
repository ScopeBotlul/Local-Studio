# Local Studio 0.36.34

- GIFs remain temporary after generation and appear in the gallery only after **Save to gallery**. Each saved GIF receives a UUID filename, so existing media cannot be overwritten by a repeated name.
- Image-to-GIF no longer requires a prompt. It accepts a start image with a compatible Wan I2V or TI2V model; known text-to-video-only models are excluded and receive a specific explanation.
- Gallery media can be moved to Local Studio's recoverable trash from a right-click menu in both the main gallery and studio gallery.

## Verification

- The production frontend build, focused Rust GIF persistence tests and focused Wan model classification tests passed.
- A short isolated native image-to-GIF run with an empty prompt was used to verify the local ComfyUI path. This checks execution and saving behavior, not output quality at normal generation settings.
