# Fractal Planet — Rules

## Naming
- Types: PascalCase
- Modules and files: snake_case
- Error types: always suffixed with `Error` (e.g. `PresetParamsError`)
- Traits: no suffix

## Module structure

Both crates organize `src/` by concern, not as a flat file list: every module lives
under a concern subdirectory, declared via a sibling `<concern>.rs` file (Rust 2024
module style — no `mod.rs`). The only files allowed directly under `src/` are
`lib.rs` (both crates) and `app.rs` (`planet-renderer`'s composition root — wasm-bindgen
entry point + winit event loop, wiring only). This is a documentation rule, enforced
at `planet-pr-validate` review time — the same way every other convention in this
file (naming, one-type-per-file) is enforced — not by an automated test.

`planet-core`'s concerns:
- `geometry/` — `vec3.rs` (`Vec3`), `mesh.rs` (`Vertex`, `Triangle`, `Mesh`, `MeshError`):
  spatial value types, no algorithm; plus a nested `primitives/` sub-concern
  (`icosahedron.rs`, `cube.rs`, both `pub(crate)` — exposed publicly only via
  `Mesh::icosahedron()` / `Mesh::cube()`, never directly) for mesh-construction
  functions built entirely from `geometry`'s own types
- `subdivision/` — `edge.rs` (`EdgeKey`, `EdgeCache`, `pub(crate)`), `steps.rs`
  (`Steps`, `StepsError`), `seed.rs` (`Seed`), `elevation_noise_range.rs`
  (`ElevationNoiseRange`, `ElevationNoiseRangeError`), `subdivision_mode.rs`
  (`SubdivisionMode`), `subdivision_args.rs` (`SubdivisionArgs`), `subdivide.rs`
  (`SubdivisionStrategy` `pub(crate)`, `subdivide`); plus a nested `strategies/`
  sub-concern (`uniform_red_split.rs`, `radial_random_split.rs`, both `pub(crate)`
  — exposed publicly only via `SubdivisionMode`, never directly) for the concrete
  subdivision-algorithm implementations: the recursive subdivision algorithm and
  its public configuration facade

`planet-renderer`'s concerns:
- `scene/` — `camera.rs` (`Camera`): orbit/zoom input math
- `gpu/` — `buffers.rs`, `uniforms.rs`, `render.rs`, `shader.wgsl`: everything
  wgpu-facing — mesh/preset-to-GPU-data mapping and the actual device/pipeline/draw calls
- `app.rs` (top-level) — winit event loop, wasm-bindgen entry point, HTML control wiring

Adding a new type: put it in the file for its existing concern if one fits; only
create a new concern subdirectory (and a `rules.md` entry for it, in this same list)
when no existing concern fits — never add a bare `.rs` file directly under `src/` as
a shortcut.

One type per file, everywhere (unchanged).

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
