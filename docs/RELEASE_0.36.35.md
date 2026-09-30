# Local Studio 0.36.35

- The GIF studio now offers a native Vulkan engine alongside ComfyUI. It runs Wan image-to-video through the bundled, integrity-checked `stable-diffusion.cpp` runtime and supports Vulkan-capable NVIDIA, AMD, and Intel graphics devices.
- Vulkan mode accepts a Wan I2V/TI2V diffusion model, UMT5 text encoder, and matching Wan VAE as separate local Safetensors or GGUF files. A prompt remains optional when a start image is supplied.
- Model weights may be moved between GPU memory and system RAM automatically. Diffusion stays on Vulkan while VAE decoding runs on the CPU to reduce peak VRAM use on handheld hardware.
- Vulkan results use the same temporary preview and explicit gallery-save flow as ComfyUI results. No model dependencies or model-provided code are installed or executed.

## Verification

- Frontend production build, Rust compiler check, strict native video-input test, and runtime-integrity permission generation passed.
- The bundled Vulkan runtime generated five real 512 × 288 frames from a start image with a local Wan 2.2 TI2V model, UMT5 encoder, and matching VAE. The one-step run verifies the execution path, not normal-quality output.
