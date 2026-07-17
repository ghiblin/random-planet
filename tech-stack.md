# Fractal Planet — Tech Stack

## Language & edition
- Rust 2024 edition, workspace-wide

## Workspace structure
- `planet-core` — library crate. Pure domain: `Vec3`/`Mesh`, icosahedron construction, uniform geodesic subdivision, fBm terrain-noise elevation shaping, presets, color gradient, ocean-quota sea-level calculation. No I/O, no GPU, no WASM
- `planet-renderer` — library crate (`crate-type = ["cdylib", "rlib"]`) plus a second wasm-bindgen binary target, `src/bin/generation_worker.rs`, built as a Web Worker so planet generation never blocks the main thread regardless of subdivision depth. wgpu rendering pipeline, winit event loop/input, wasm-bindgen entry point, HTML control wiring. Pure-logic submodules (camera math, mesh→buffer packing, preset→uniform mapping, the `worker/protocol.rs` message types) stay platform-agnostic and natively testable; actual GPU calls and DOM/browser glue — including `generation_worker.rs`'s own `fn main()` — are `#[cfg(target_arch = "wasm32")]`-gated where applicable
- Shared `[workspace.dependencies]` for versions used by both crates

## Build tooling (not a crate dependency)
- **Trunk** — serves `index.html`, drives `wasm-bindgen`/`wasm-opt`, dev server with live reload for the `planet-renderer` WASM build

## Confirmed dependencies

| Crate | Feature flags | Used in |
|---|---|---|
| `wgpu` | default | `planet-renderer` |
| `winit` | default | `planet-renderer` |
| `wasm-bindgen` | default | `planet-renderer` |
| `wasm-bindgen-futures` | default | `planet-renderer` (drives the async adapter/device setup on the wasm event loop) |
| `web-sys` | grows per DOM API touched (currently `console`, `Document`, `Element`, `Node`, `EventTarget`, `HtmlInputElement`, `HtmlDialogElement`, `Performance`, `Worker`, `MessageEvent`, `DedicatedWorkerGlobalScope`) | `planet-renderer` |
| `js-sys` | default | `planet-renderer` (reads `Date.now()` to seed each Start-click's `Planet`) |
| `rand` | default | `planet-core` |
| `rand_pcg` | default | `planet-core` (seeded RNG — required for deterministic generation) |
| `noise` | default | `planet-core` (`Fbm<Perlin>` — layered fractal-noise elevation field sampled at each vertex's unit-sphere direction, `processor/terrain_noise.rs`) |
| `getrandom` | `wasm_js` | `planet-renderer` (wasm32-only — required for `rand`'s transitive `getrandom` dependency to build on `wasm32-unknown-unknown`) |
| `console_error_panic_hook` | default | `planet-renderer` (wasm32-only — installed once at startup so panics print a real message/location to the browser console instead of a bare `unreachable` trap) |
| `cucumber` | default | `planet-core`, `planet-renderer` (dev) |
| `tokio` | `macros`, `rt-multi-thread` (dev-only) | `planet-core`, `planet-renderer` (dev — cucumber test harness) |
