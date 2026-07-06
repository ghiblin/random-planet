# Fractal Planet — Tech Stack

## Language & edition
- Rust 2024 edition, workspace-wide

## Workspace structure
- `planet-core` — library crate. Pure domain: `Vec3`/`Mesh`, icosahedron construction, red-green recursive subdivision, presets, color gradient, ocean-quota sea-level calculation. No I/O, no GPU, no WASM
- `planet-renderer` — library crate (`crate-type = ["cdylib", "rlib"]`). wgpu rendering pipeline, winit event loop/input, wasm-bindgen entry point, HTML control wiring. Pure-logic submodules (camera math, mesh→buffer packing, preset→uniform mapping) stay platform-agnostic and natively testable; actual GPU calls and DOM/browser glue are `#[cfg(target_arch = "wasm32")]`-gated where applicable
- Shared `[workspace.dependencies]` for versions used by both crates

## Build tooling (not a crate dependency)
- **Trunk** — serves `index.html`, drives `wasm-bindgen`/`wasm-opt`, dev server with live reload for the `planet-renderer` WASM build

## Confirmed dependencies

| Crate | Feature flags | Used in |
|---|---|---|
| `wgpu` | default | `planet-renderer` |
| `winit` | default | `planet-renderer` |
| `wasm-bindgen` | default | `planet-renderer` |
| `web-sys` | grows per DOM API touched (e.g. `HtmlCanvasElement`, `HtmlSelectElement`, `HtmlInputElement`) | `planet-renderer` |
| `rand` | default | `planet-core` |
| `rand_pcg` | default | `planet-core` (seeded RNG — required for deterministic generation) |
| `cucumber` | default | `planet-core`, `planet-renderer` (dev) |
| `tokio` | `macros`, `rt-multi-thread` (dev-only) | `planet-core`, `planet-renderer` (dev — cucumber test harness) |
