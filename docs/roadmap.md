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

## Current state

Governance docs (`constitution.md`, `tech-stack.md`, `rules.md`) and the skills pipeline are set up. Phase `002-domain-data-model` is complete. Phase `003-cube-mesh-wiring` is next.
