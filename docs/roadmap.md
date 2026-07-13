# Fractal Planet — Roadmap

Each phase below becomes a spec written by the `planet-spec` skill (`docs/specs/<NNN>-<slug>.md`), reviewed by `planet-spec-review`, implemented under `planet-tdd`, and merged via `planet-pr-validate` + `planet-pr-merge`. See `docs/specs/000-architecture.md` for the technical design these phases implement.

## Phases

- **000 — Architecture** (reference doc, not a feature spec) — complete
- **001 — Cube render** — scaffold the Cargo workspace (`planet-core`, `planet-renderer`), wire up Trunk, render a single rotating cube in the browser. Infra only, validates the wgpu/winit/wasm-bindgen/Trunk pipeline
- **002 — Domain data model** — `Vec3`, `Mesh`, `Vertex`, `Triangle` in `planet-core`, no subdivision yet
- **003 — Cube mesh wiring** — `Mesh::cube(side)` utility constructor in `planet-core`; `planet-renderer` renders that `Mesh` by deriving flat-shaded GPU vertex/index data from it instead of a hardcoded vertex table
- **004 — Icosahedron subdivision** — base icosahedron construction + uniform recursive 4-way split (always-red, exact midpoints); render the result in place of the cube
- **005 — Radial randomness** — random radial vertex displacement on newly created vertices during subdivision
- **006 — Irregular subdivision** — length-threshold stopping condition, Gaussian-distributed split point, red-green triangulation for partially-split triangles
- **007 — Planet presets** — `Preset`/`PresetParams`, color gradient, ocean-quota sea-level + flattening (Earthy), Volcano/Rocky presets, preset dropdown + depth slider UI wiring
- **008 — Length-relative displacement noise** — `radial_displacement`/`normal_displacement` currently sample a fixed absolute magnitude from `ElevationNoiseRange`/`NormalNoiseRange` every round, intentionally compounding per `007-radial-randomness.md`'s own documented bound (`radius <= 1.0 + steps * elevation_noise_range.high()`); this makes presets with a tighter `min_edge_length` (Volcano, Rocky) — which subdivide many more rounds before converging than Earthy — accumulate disproportionate, unintended-looking displacement at higher depths. Review scaling the sampled delta by the current edge's length instead of applying a fixed magnitude (both split-edge endpoints are already available in `VertexOperator`'s signature, no interface change needed), and update `007-radial-randomness.md`'s invariant language and BDD scenarios to match the new length-relative semantics
- **009 — Review the segment length cutoff rule** — `RedGreenSplit::maybe_split` compares each edge's current (possibly already-displaced) length directly against `preset.min_edge_length`, a fixed absolute threshold independent of subdivision round or overall mesh scale. Review this stopping condition alongside 008's length-relative noise rework, since both concern how edge length drives the subdivision loop — confirm the cutoff still yields the intended convergence/detail behavior once displacement noise is no longer a fixed magnitude
- **010 — Smooth vertex normals (conditional post-process)** — `mesh_render_vertices` (`planet-renderer/src/gpu/buffers.rs`) currently computes one flat per-triangle face normal, duplicated across that triangle's 3 render vertices, producing hard facet edges everywhere. Add an optional per-vertex normal computed as the normalized mean of the face normals of every triangle sharing that `Mesh` vertex index, for smooth shading across triangle boundaries — gated as a conditional post-process step (e.g. a toggle alongside the existing wireframe mode), not a replacement of flat shading

## Current state

Governance docs (`constitution.md`, `tech-stack.md`, `rules.md`) and the skills pipeline are set up. Phase `002-domain-data-model` is complete. Phase `003-cube-mesh-wiring` is next.
