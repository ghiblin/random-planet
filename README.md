# Fractal Planet

A browser app that procedurally generates fractal planet meshes from an icosahedron via recursive triangle subdivision with controlled randomness. Core logic is Rust compiled to WASM; rendering uses [wgpu](https://wgpu.rs/). It's a revisit of a university assignment, built as an educational/portfolio project.

Not a game engine, not a physically-accurate terrain simulator, not multiplayer — everything runs client-side in one browser tab.

## Architecture

A two-crate Cargo workspace, scaffolded starting with phase `001-cube-render`:

| Crate | Kind | Responsibility |
|---|---|---|
| `planet-core` | lib | Domain types, subdivision algorithm, presets, color. No I/O, no GPU, no WASM. |
| `planet-renderer` | lib (cdylib+rlib) | wgpu rendering, winit input, wasm-bindgen entry point, HTML control wiring. The only crate that touches the GPU or the browser. |

See [`constitution.md`](constitution.md) for the non-negotiable constraints (determinism, bounded recursion, crate boundaries) and [`docs/specs/000-architecture.md`](docs/specs/000-architecture.md) for the full technical design.

## Status

See [`docs/roadmap.md`](docs/roadmap.md) for the phased feature list and current progress.

## Development

This project follows spec-driven development with a BDD/TDD pipeline — see [`CLAUDE.md`](CLAUDE.md) for the full workflow and the governing docs (`constitution.md`, `tech-stack.md`, `rules.md`).

Build gate, run before every commit:

```bash
cargo test --workspace && cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo build --target wasm32-unknown-unknown -p planet-renderer
```

Once `planet-renderer` is scaffolded, serve it locally with [Trunk](https://trunkrs.dev/):

```bash
trunk serve
```
