# Fractal Planet — Constitution

## What it is
- An educational/portfolio web app that procedurally generates fractal planet meshes from an icosahedron, entirely in Rust compiled to WASM
- A revisit of a university assignment: recursive triangle subdivision with controlled randomness

## What it is not
- Not a game engine
- Not a physically-accurate terrain simulator
- Not multiplayer/networked — everything runs client-side in one browser tab

## Non-negotiable constraints
- `planet-core` has zero I/O, zero GPU, and zero WASM/browser dependencies (no `wgpu`, `winit`, `wasm-bindgen`, `web-sys`) — pure computation, testable with plain `cargo test` on the host
- `Planet::generate(seed, preset, max_depth)` is **deterministic**: identical inputs always produce an identical `Mesh`. No reliance on system time, thread scheduling, or hash-map iteration order. This is what makes BDD scenarios over generated planets possible at all
- `planet-renderer` owns all GPU/WASM/browser-facing code. Anything that only makes sense in a browser (the `wasm-bindgen` entry point, DOM/canvas event wiring) must be `#[cfg(target_arch = "wasm32")]`-gated so the rest of the crate still compiles and tests natively
- Subdivision recursion is always bounded by an explicit max-depth cap, regardless of preset parameters — no preset configuration may cause unbounded or runaway recursion
- BDD scenarios must be backed by real `cucumber` step definitions — never left as markdown prose. An undefined step must fail the suite
- Build gate before every commit:
  ```bash
  cargo test --workspace && cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo build --target wasm32-unknown-unknown -p planet-renderer
  ```

## Core-first progression
- `planet-core` (domain logic) → `planet-renderer` (GPU/browser) → UI controls, in that order, per roadmap phase
- Each phase is additive; a later phase does not require rewriting an earlier one's public interface
- `planet-core`'s public API is the only thing `planet-renderer` depends on — no reverse dependency, ever

## Worktree rule — one bootstrap exception

Every feature/phase (`001-cube-render` onward) must be developed on a `feat/<slug>` branch in a dedicated worktree, never committed directly to `main` — enforced by `planet-spec`'s Phase 0 hard gate. The **one exception** is the initial governance/skills bootstrap commit itself (`constitution.md`, `tech-stack.md`, `rules.md`, `CLAUDE.md`, `docs/`, `.claude/skills/`): those files must exist on `main` before any worktree can be created from it, and before `planet-spec`'s own gate can run — there is no worktree-based way to commit the thing that defines the worktree workflow. This exception applies only to that one bootstrap commit and does not extend to any other work.
