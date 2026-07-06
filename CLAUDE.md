# Fractal Planet — Claude Instructions

Fractal Planet is a browser app that procedurally generates fractal planet meshes from an icosahedron via recursive triangle subdivision with controlled randomness. Core logic is Rust compiled to WASM; rendering uses wgpu.

## Foundation docs — read before touching any code

- `constitution.md` — what the system is and is not; non-negotiable constraints; core-first progression model
- `tech-stack.md` — confirmed technology choices and dependency table
- `rules.md` — naming conventions, module structure, BDD scenario style, commit format
- `docs/specs/000-architecture.md` — reference technical design (domain model, subdivision algorithm, ocean-quota mechanism, rendering split)
- `docs/roadmap.md` — phased feature list and current progress
- `<crate>/RULES.md` (once scaffolded) — allowlist/blocklist for that crate

## Architecture

Two-crate Cargo workspace:

| Crate | Kind | Responsibility |
|---|---|---|
| `planet-core` | lib | Domain types, subdivision algorithm, presets, color. No I/O, no GPU, no WASM. |
| `planet-renderer` | lib (cdylib+rlib) | wgpu rendering, winit input, wasm-bindgen entry point, HTML control wiring. The only crate that touches the GPU or the browser. |

## Development workflow

This project follows spec-driven development with a BDD/TDD pipeline, coordinated by five skills:

1. `planet-spec` — discovery + writes the feature spec
2. `planet-spec-review` — hard-gate checklist before implementation starts
3. `planet-tdd` — RED-GREEN-REFACTOR implementation
4. `planet-pr-validate` — hard-gate checklist before merge (spec adherence + quality/security)
5. `planet-pr-merge` — squash-merge and cleanup

These coordinate through `.claude/fractal-planet-workflow-state.json` (untracked). Each feature/phase works in its own git worktree and branch.

**Build gate — must pass before every commit:**
```bash
cargo test --workspace && cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo build --target wasm32-unknown-unknown -p planet-renderer
```

**Git workflow:**
- `main` receives one squash commit per phase/feature
- Each phase works on a `feat/<slug>` branch in a dedicated git worktree
- All commits use semantic format: `type: short imperative description`
- Never add `Co-Authored-By` trailers
- Squash-merge into `main` when validated; delete the branch and worktree

**Current state:** Governance and skills pipeline set up. Phase `001-cube-render` is next — see `docs/roadmap.md`.

## Updating tech-stack.md

When a phase introduces a confirmed dependency: add a row to the table in `tech-stack.md` with crate name, feature flags, and which crates use it. No rationale — confirmed choices only.
