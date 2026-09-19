# Local Studio

Read `SPEC.md` for the full product requirements and `PLAN.md` for milestone order.
Build a real Windows desktop application with Tauri 2, React/TypeScript, Rust and SQLite.
Keep inference in isolated workers. Never present simulated results as working features.
Preserve all future scope. Finish working vertical paths before exposing new controls.
Keep user media, prompts, model files and benchmarks local. No telemetry by default.
Use explicit, typed IPC and restrict desktop capabilities to the local main window.
Do not execute model-provided code or install model dependencies automatically.
Record implementation and verification separately in `PROJECT_STATUS.md` and `TEST_MATRIX.md`.
Current user instruction (2026-09-19): tests and builds are authorized again, but do not run every check for every change. Test the changed areas and their direct dependencies; choose frontend, Rust, native UI, inference or installer checks according to the actual impact. Reuse relevant existing evidence for unchanged areas. Run the full regression suite only for broad changes that justify it or when explicitly requested. Do not repeat unrelated AI, generation, privacy or installer suites solely because a new build was produced. Record the scope and limits of verification accurately.
Current publishing instruction (2026-09-19): upload new builds to ScopeBotlul/Local-Studio on GitHub by default, including the installer, portable archive, checksums and existing signed update metadata. This is standing authorization to publish builds; do not request confirmation again unless the scope changes materially. Preserve signing keys and publish no credentials, user data or model weights. Clearly disclose incomplete checks or known limitations in release notes; never describe a partially verified build as fully verified. The user may explicitly request a local-only build as an exception.
