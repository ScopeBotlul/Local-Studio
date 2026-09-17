# Local Studio

Read `SPEC.md` for the full product requirements and `PLAN.md` for milestone order.
Build a real Windows desktop application with Tauri 2, React/TypeScript, Rust and SQLite.
Keep inference in isolated workers. Never present simulated results as working features.
Preserve all future scope. Finish working vertical paths before exposing new controls.
Keep user media, prompts, model files and benchmarks local. No telemetry by default.
Use explicit, typed IPC and restrict desktop capabilities to the local main window.
Do not execute model-provided code or install model dependencies automatically.
Record implementation and verification separately in `PROJECT_STATUS.md` and `TEST_MATRIX.md`.
Use `npm run build`, `npm test`, and Rust tests for changes that affect behavior.
