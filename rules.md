# Fractal Planet — Rules

## Naming
- Types: PascalCase
- Modules and files: snake_case
- Error types: always suffixed with `Error` (e.g. `PresetParamsError`)
- Traits: no suffix

## Module structure

`planet-core` has a single aggregate root (`Planet`), unlike a multi-aggregate domain — so it does not use an `entities/value_objects/ports` split per aggregate. Instead, one file per type, flat under `src/`:

- `vec3.rs` — `Vec3`
- `mesh.rs` — `Vertex`, `Triangle`, `Mesh`
- `icosahedron.rs` — base icosahedron construction
- `edge.rs` — `EdgeKey`, `EdgeCache`, edge split-decision logic (length threshold + Gaussian split point)
- `subdivide.rs` — red-green recursive subdivision
- `color.rs` — `ColorGradient`
- `preset.rs` — `Preset`, `PresetParams`
- `ocean.rs` — sea-level percentile calculation + geometry flattening
- `planet.rs` — `Planet` (aggregate root, `Planet::generate`)
- `lib.rs` — `pub mod` declarations only, no logic

`planet-renderer` splits by testability, not by aggregate:

- `camera.rs`, `buffers.rs`, `uniforms.rs` — pure logic, no GPU calls, natively testable
- `render.rs` — wgpu device/pipeline/draw calls (thin, not BDD-tested)
- `app.rs` — winit event loop, wasm-bindgen entry point, HTML control wiring (thin, `#[cfg(target_arch = "wasm32")]`-gated where browser-only, not BDD-tested)
- `lib.rs` / `main.rs` — wiring only, no logic

One type per file, everywhere.

When each crate is scaffolded (spec `001-cube-render`), split this into `planet-core/RULES.md` and `planet-renderer/RULES.md` with an allow/blocklist, following the pattern below:

- `planet-core` allowed: pure computation, `rand`/`rand_pcg`. Not allowed: `wgpu`, `winit`, `wasm-bindgen`, `web-sys`, `std::fs`, `std::net`, any dependency on `planet-renderer`
- `planet-renderer` allowed: `wgpu`, `winit`, `wasm-bindgen`, `web-sys`, depends on `planet-core`. Not allowed: domain/generation logic that belongs in `planet-core` (e.g. no subdivision math inline in render code)

## Error handling
- No `unwrap()`/`panic!()` in production code — permitted only in tests and examples
- Constructors that validate invariants (e.g. `PresetParams` fields in range) return `Result` with a dedicated `Error` type
- DOM/canvas lookups in `planet-renderer` (e.g. `document.get_element_by_id`) must handle `None` explicitly, never `.unwrap()`

## BDD scenario style

- Reference a fixture by how it was obtained, never bare — `Given an icosahedron mesh`, `Given a Planet generated with seed <n> and the <Preset> preset`, never `Given a mesh` or `Given a planet`
- Every subdivision-related feature file carries the same core scenario set, in this order: face-count growth per level, no duplicate vertices at shared edges, no cracks/T-junctions between red and green triangles, vertex radii stay within the preset's configured bounds. Add algorithm-specific scenarios after these
- Every preset-related feature file covers: determinism (same seed + preset + depth ⇒ identical `Mesh`), elevation distribution respects the preset's noise range, and — for presets with an ocean quota — the fraction of vertices at sea level matches the configured quota within tolerance
- `Then`/`And` steps name the field they assert on exactly as it appears in the domain model

## Commit format
- Build gate must pass before every commit
- One commit per task, on the current phase/feature branch
- Semantic commit format: `type: short imperative description`
- Common types: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`
- Squash merge commits on `main` follow the same semantic format
- Never add `Co-Authored-By` trailers to any commit
