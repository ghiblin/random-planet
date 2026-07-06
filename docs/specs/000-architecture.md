# 000 — Architecture

**Status:** Approved (brainstorming session, 2026-07-06)

This is a reference design doc, not a feature spec — it does not follow the five-section format (`planet-spec` skill) that `001`-`006` will use. Individual phase specs should point back here for algorithm/domain detail rather than re-deriving it.

## Overview

The app starts from an icosahedron and recursively subdivides its triangular faces, perturbing vertex positions along their radius, to produce an irregular planet-like mesh. `planet-core` computes this mesh; `planet-renderer` uploads it to the GPU and renders it in a browser via WASM.

## Domain model (`planet-core`)

**Value objects** (immutable, no identity):
- `Vec3` — 3D point/vector, basic math ops
- `Mesh` — `Vec<Vertex>` (position + elevation-derived color) + `Vec<Triangle>` (3 vertex indices); an immutable snapshot
- `Seed`, `SubdivisionDepth` — validated newtypes (`u64`; `u32` capped, e.g. 1..=8)
- `Preset` — enum (`Earthy`, `Volcano`, `Rocky`, ...), each carrying `PresetParams`:
  - `min_edge_length: f32` — per-edge stopping threshold for subdivision
  - `elevation_noise_range: Range<f32>` — radial perturbation applied to new vertices
  - `split_point_variance: f32` — std-dev of the Gaussian used to place a split point along an edge
  - `color_gradient: ColorGradient`
  - `ocean_quota: Option<f32>` — fraction of vertices (by count) that must end up at sea level; `None` for presets with no liquid ocean
- `ColorGradient` — elevation → color stops + `sample(elevation) -> Rgb`

**Aggregate root:**
- `Planet` — the only type with a lifecycle. `Planet::generate(preset: Preset, seed: Seed, max_depth: SubdivisionDepth) -> Planet`, holding the resulting `Mesh` and the `Preset` used. **Must be deterministic** — see `constitution.md`.

**Internal mechanism** (not public domain vocabulary): an `EdgeCache` keyed by canonical `(min_idx, max_idx)` vertex-index pairs, storing each edge's split decision and midpoint vertex index, so both triangles sharing an edge agree. This is what prevents cracks.

Single aggregate ⇒ no `entities/value_objects/ports` subfolder split (see `rules.md`); one file per type, flat under `planet-core/src/`.

## Subdivision algorithm (red-green)

Per triangle, per recursion level: for each of its 3 edges, consult the `EdgeCache`. If uncached, compute the edge's current length from its (possibly already-perturbed) endpoint positions:

- If `length < preset.min_edge_length` → mark **not split**
- Otherwise → mark **split**, with the split point at `t = clamp(gaussian(mean=0.5, std=preset.split_point_variance), t_min, t_max)` along the edge, then displace the new vertex along its radius by a random amount drawn from `preset.elevation_noise_range`

Triangulate based on how many of the 3 edges ended up split:

| Edges split | Label | Children | Recurses further? |
|---|---|---|---|
| 3 | red | 4 (classic subdivision) | yes |
| 2 | green | 3 (fan through the two midpoints) | no |
| 1 | green | 2 | no |
| 0 | leaf | 1 (unchanged) | no |

Green triangles are **not** recursed into further — this avoids compounding sliver-triangle distortion across levels (see brainstorming discussion: red-green refinement is a known FEM technique, but repeatedly recursing into green triangles degrades triangle quality).

The subdivision-depth UI control is a **hard cap** on recursion levels, independent of `min_edge_length` — it bounds runaway recursion regardless of preset (per `constitution.md`).

## Ocean quota (Earthy preset)

Sea level is computed **after** the final subdivision round completes, not by clamping radius during recursion — a quota is a property of the whole final elevation distribution, and per-round clamping cannot guarantee a specific fraction of the surface ends up at/below a level (it can only bound extremes, and compounds unpredictably across variable-depth branches).

Algorithm:
1. Generate the full `Mesh` via subdivision (no ocean-specific logic in `subdivide.rs`)
2. If `preset.ocean_quota` is `Some(q)`: sort all final vertex radii, take the value at the `q`-th percentile **by vertex count** (approximate — not area-weighted; acceptable for a generative/visual feature, see brainstorming discussion) → this is `sea_level`
3. **Flatten**: any vertex with radius `< sea_level` gets its radius raised to exactly `sea_level`, producing a literal constant-radius "ocean" region
4. Color every vertex via `ColorGradient::sample(final_radius)` — ocean vertices, sharing the same radius, render with the same color

This keeps `subdivide.rs` fully preset-agnostic; "ocean" is a concept that lives only in the post-processing step, driven by `PresetParams.ocean_quota`.

## Rendering (`planet-renderer`)

wgpu + winit, targeting native (for tests) and `wasm32-unknown-unknown` (for the browser build via Trunk).

BDD-testable logic lives in GPU-free modules:
- `camera.rs` — orbit/zoom math (mouse-delta → yaw/pitch, scroll → distance with clamping)
- `buffers.rs` — `Mesh` → vertex/index buffer layout (byte packing)
- `uniforms.rs` — `Preset` → light direction / gradient uniform mapping

Not BDD-tested, manually verified in-browser per milestone: actual `wgpu::Device`/pipeline/draw-call code, and the `wasm-bindgen`/DOM event wiring that connects browser mouse events and HTML controls to the logic above. Per the brainstorming decision, actual GPU pixel output is explicitly out of BDD scope — no headless-render/pixel-readback testing.

## UI controls

- Camera orbit (mouse drag) and zoom (scroll)
- Subdivision-depth slider (hard recursion cap, see above)
- Preset selector dropdown
- No seed input exposed in the UI — "regenerate" re-seeds internally

## Presets

Each `Preset` variant is a named `PresetParams` bundle. Confirmed presets: Earthy (with `ocean_quota`), Volcano, Rocky — exact parameter values are an implementation detail of spec `006-planet-presets`, not fixed here.
