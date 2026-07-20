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
entry point + winit event loop, wiring only). `planet-renderer/src/bin/generation_worker.rs`
is a separate, Cargo-standard exception to this same rule — `src/bin/*.rs` is its own
sanctioned location for extra binary targets, distinct from the `src/`-direct-files
restriction above; it is the Web Worker's own wasm-bindgen entry point (composition
root for the generation-off-the-main-thread flow), thin wiring only, same as `app.rs`.
This is a documentation rule, enforced at `planet-pr-validate` review time — the same
way every other convention in this file (naming, one-type-per-file) is enforced — not
by an automated test.

`planet-core`'s concerns:
- `geometry/` — `vec3.rs` (`Vec3`), `vertex.rs` (`Vertex`), `edge.rs` (`Edge`),
  `face.rs` (`Face`), `mesh.rs` (`Mesh`, `MeshError`): spatial value types, no
  algorithm. `Mesh` is a `Face`/`Edge`/`Vertex` adjacency graph — `Vertex` carries
  `position`, `normal`, and `edges` (indices of every `Edge` where it is the
  `start`); `Edge` carries `start`/`end` vertex indices and the one `Face` it bounds
  (two triangles sharing a geometric edge each get their own `Edge`, no shared/twin
  pointer); `Face` carries its boundary `edges`, `order` (`edges.len()`, always 3
  today), and `normal`. `Mesh::new` takes bare `Vec<Vec3>` positions plus
  `Vec<(usize, usize, usize)>` triangle index-triples (no dedicated `Triangle` type)
  and builds the graph; `Vertex.normal`/`Face.normal` start as `Vec3::ZERO`
  placeholders, populated only by `processor/finalize_normals.rs`'s
  `finalize_normals` once every position-mutating step has completed; plus a nested
  `primitives/` sub-concern (`icosahedron.rs`, `cube.rs`, both `pub(crate)` — exposed
  publicly only via `Mesh::icosahedron()` / `Mesh::cube()`, never directly) for
  mesh-construction functions built entirely from `geometry`'s own types
- `subdivision/` — `edge.rs` (`EdgeKey`, `EdgeCache`, `pub(crate)`), `steps.rs`
  (`Steps`, `StepsError`), `seed.rs` (`Seed`), `subdivision_mode.rs`
  (`SubdivisionMode` — a seedless, preset-shape enum selecting a subdivision
  algorithm, mirroring `TerrainNoise`'s own no-`Seed`-of-its-own convention),
  `subdivision_args.rs` (`SubdivisionArgs` — bundles `steps`, `mode`, and `seed`
  independently, since a strategy needs a `Seed` supplied at construction time but
  that `Seed` is never part of `mode`'s own shape), `subdivide.rs`
  (`SubdivisionStrategy` `pub(crate)`, `subdivide`; `subdivide`'s per-round hook
  (`SubdivisionArgs.update_cb`, type `UpdateCallback`) is a mesh *transform*, not a
  read-only observer — `Box<dyn FnMut(Mesh, usize) -> Result<Mesh, MeshError>>`. Each
  round's `Mesh` is handed to the hook by value; whatever it returns becomes the mesh
  the *next* round subdivides, propagating `Err` immediately. A hook that returns its
  input unchanged reproduces plain observation; `Planet::subdivide` uses this to fold
  `apply_terrain_noise_for_round` into the subdivision loop itself, one more revealed
  octave per round, instead of as a single pass after subdivision completes); plus a
  nested `strategies/` sub-concern (`uniform_red_split.rs`, `pub(crate)` — exposed
  publicly only via `SubdivisionMode`, never directly) for the concrete
  subdivision-algorithm implementation: uniform, near-exact-midpoint geodesic
  subdivision (each new split vertex is nudged tangentially along its edge and along
  the edge's normal, proportional to that edge's current length, via
  `processor/jitter.rs`'s `VertexOperator`) — radial elevation lives entirely in
  `processor/terrain_noise.rs`, applied per subdivision round via `Planet::subdivide`'s
  `update_cb`, not as a separate post-subdivision pass
- `processor/` — reusable vertex- and mesh-transformation building blocks: whole-mesh
  pre/post-processing steps that run outside the subdivision algorithm, each taking
  an already-built `Mesh` and returning a transformed one (`vertex_scramble_range.rs`
  (`VertexScrambleRange`, `VertexScrambleRangeError`), `vertex_scramble.rs`
  (`scramble_vertices`), `ocean_quota.rs` (`OceanQuota`, `OceanQuotaError`,
  `apply_ocean_quota`), `terrain_noise.rs` (`TerrainNoise`, `TerrainNoiseError`,
  `apply_terrain_noise`, `apply_terrain_noise_for_round`) — samples layered (fBm)
  noise at each vertex's unit-sphere direction and reshapes it with a redistribution
  curve and optional terracing; `apply_terrain_noise_for_round` reveals only
  `revealed_octaves` (clamped to `[1, octaves()]`) of the configured fBm octaves,
  letting a caller fold elevation into a subdivision round-by-round instead of as one
  whole-mesh pass — `apply_terrain_noise(mesh, seed, tn)` is defined as
  `apply_terrain_noise_for_round(mesh, seed, tn, tn.octaves())`, so its own contract is
  unchanged);
  plus the per-vertex `VertexOperator` building blocks `subdivision/strategies/`
  composes into a pipeline to compute each newly split vertex (`vertex_operator.rs`
  (`VertexOperator`, `pub(crate)`), `jitter.rs` (`jitter`, `pub(crate)` — displaces
  a split point tangentially along its edge and along the edge's normal, each
  magnitude proportional to the edge's current length));
  plus `finalize_normals.rs` (`finalize_normals`, `pub` — computes each `Face`'s flat
  normal and each `Vertex`'s area-weighted normal from the mesh's final geometry;
  called once by `Planet::subdivide`/`PlanetBuilder::build` after every
  position-mutating step, and directly by `planet-renderer`'s
  `src/bin/generation_worker.rs` on mid-subdivision animation frames, the same way it
  already calls `ColorGradient::sample` directly on those frames)
- `color/` — elevation-to-color mapping value types, no algorithm: `rgb.rs` (`Rgb`,
  `RgbError`), `color_gradient.rs` (`ColorGradient`, `ColorGradientError`)
- `presets/` — bundles the subdivision/color knobs into named, pre-tuned presets:
  `preset_params.rs` (`PresetParams` — `terrain_noise`, `color_gradient`, `ocean_quota`,
  and `subdivision_mode`; the last is the preset's own choice of `SubdivisionMode`,
  read by `Planet::subdivide` instead of hardcoded there, so different presets can
  name different subdivision strategies without changing `planet.rs`), `preset.rs`
  (`Preset`)
- `planets/` — the aggregate root, split into its two lifecycle operations:
  `planet.rs` (`Planet` — including its `subdivide` method, `PlanetError`,
  `GenerationProgress`, `PostprocessProgress`), `planet_builder.rs` (`PlanetBuilder`
  — creation only, no subdivision; scrambles the icosahedron's base vertices via
  `processor/vertex_scramble.rs`'s `scramble_vertices` before storing the mesh),
  `postprocess_stage.rs` (`PostprocessStage` — one remaining variant, `OceanQuota`,
  the sole coarse, observable stage left after subdivision completes, reported via
  `PostprocessProgress` only when the preset configures an `OceanQuota` — never
  called otherwise. Terrain noise is no longer a discrete post-subdivision "stage":
  it is folded into every subdivision round via `subdivide.rs`'s `update_cb`,
  already covered by the round-by-round `GenerationProgress` callback that fires
  regardless)

`planet-renderer`'s concerns:
- `scene/` — `camera.rs` (`Camera`): orbit/zoom input math; `growth_animation.rs`
  (`GrowthAnimation`, `FRAME_INTERVAL_MS`): paces the subdivision growth-animation
  frame reveal by wall-clock elapsed time, no browser API calls — the actual
  `Performance` timestamp reads stay in `app.rs`
- `gpu/` — `buffers.rs`, `uniforms.rs`, `render.rs`, `shader.wgsl`: everything
  wgpu-facing — mesh/preset-to-GPU-data mapping and the actual device/pipeline/draw calls
- `controls/` — `preset_select.rs` (`parse_preset`), `depth_slider.rs` (`MIN_DEPTH`,
  `MAX_DEPTH`, `DepthParseError`, `parse_depth`), `seed_from_timestamp.rs`
  (`seed_from_timestamp`): pure DOM-value parsing/validation and timestamp-to-seed
  conversion, no browser API calls — the actual element lookups/listeners/clock reads
  stay in `app.rs`
- `worker/` — `protocol.rs` (`StartRequest`, `WorkerMessage`): plain, natively-testable
  message payload types shared between the main-thread entry point (`app.rs`) and the
  generation-worker entry point (`src/bin/generation_worker.rs`); the actual `JsValue`
  `postMessage`/`onmessage` encode/decode conversions live in the same file, `#[cfg(target_arch
  = "wasm32")]`-gated, same class of thin wiring as `app.rs`'s DOM glue
- `app.rs` (top-level) — winit event loop, wasm-bindgen entry point, HTML control wiring

Adding a new type: put it in the file for its existing concern if one fits; only
create a new concern subdirectory (and a `rules.md` entry for it, in this same list)
when no existing concern fits — never add a bare `.rs` file directly under `src/` as
a shortcut.

One type per file, everywhere (unchanged).

## Crate boundaries

Consumers of `planet-core` — currently only `planet-renderer` — must obtain every
generated `Mesh` via `Planet`'s own lifecycle operations (`Planet::builder()...build()`
to create one, `Planet::subdivide()` to subdivide one), never via `Mesh::icosahedron()`,
`subdivide()`, `SubdivisionMode`, `scramble_vertices()`, or any other generation
primitive directly. Reading an already-obtained `Mesh`'s own data (`vertices()`,
`edges()`, `faces()`, e.g. `planet-renderer`'s `gpu/buffers.rs`) is unaffected — the
rule is about how a `Mesh` is *produced*, not how its data is *read*. Deriving
read-only shading data from an already-produced `Mesh` is likewise unaffected — e.g.
`finalize_normals`, called directly by `planet-renderer`'s `src/bin/generation_worker.rs`
on mid-subdivision animation frames, the same way it already calls `ColorGradient::sample`
on them.

This is a documentation rule, enforced at `planet-pr-validate` review time, not by the
compiler: every generation primitive stays `pub` because `planet-core`'s own BDD/unit
test suite lives under `planet-core/tests/`, which Rust compiles as a separate crate
that can only see `pub` items, never `pub(crate)` — a real visibility lockdown would
break that entire test suite.

## Error handling
- No `unwrap()`/`panic!()` in production code — permitted only in tests and examples
- Constructors that validate invariants (e.g. `PresetParams` fields in range) return `Result` with a dedicated `Error` type
- DOM/canvas lookups in `planet-renderer` (e.g. `document.get_element_by_id`) must handle `None` explicitly, never `.unwrap()`

## BDD scenario style

- Reference a fixture by how it was obtained, never bare — `Given an icosahedron mesh`, `Given a Planet generated with seed <n> and the <Preset> preset`, never `Given a mesh` or `Given a planet`
- Every subdivision-related feature file carries the same core scenario set, in this order: face-count growth per level, no duplicate vertices at shared edges, no cracks/T-junctions between adjacent triangles, vertex radii stay within the preset's configured bounds. Add algorithm-specific scenarios after these
- Every preset-related feature file covers: determinism (same seed + preset + depth ⇒ identical `Mesh`), elevation distribution respects the preset's noise range, and — for presets with an ocean quota — the fraction of vertices at sea level matches the configured quota within tolerance
- `Then`/`And` steps name the field they assert on exactly as it appears in the domain model

## Commit format
- Build gate must pass before every commit
- One commit per task, on the current phase/feature branch
- Semantic commit format: `type: short imperative description`
- Common types: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`
- Squash merge commits on `main` follow the same semantic format
- Never add `Co-Authored-By` trailers to any commit
