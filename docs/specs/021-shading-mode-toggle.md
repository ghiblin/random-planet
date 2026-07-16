# 021 — Shading mode toggle (flat/smooth)

**Supersedes:** the remaining, deferred half of the roadmap's original phase-010
framing. `020-face-edge-vertex-mesh-model.md` already delivered the hard part —
`Vertex.normal` (area-weighted, computed by `finalize_normals`) — and rendered with it
*unconditionally*, explicitly deferring "a future toggle... a small, additive
follow-up (pick `Face.normal` vs `Vertex.normal` per render vertex)". This spec is that
follow-up.

## Requirements

`mesh_render_vertices` (`planet-renderer/src/gpu/buffers.rs`) currently always reads
`vertex.normal` (the smooth, area-weighted normal) for every face-corner render vertex,
giving every generated planet continuous, facet-free shading everywhere with no way to
see the underlying flat-per-triangle facets. Add a toggle, alongside the existing `W`
wireframe toggle, that switches every corner's normal source between:
- **Smooth** (current, default behavior): each corner reads its own `Vertex.normal`.
- **Flat**: every corner of a given face instead reads that face's own single
  `Face.normal`, restoring hard facet edges.

This is renderer-only — no `planet-core` change. `Face.normal`/`Vertex.normal` already
exist and are already finalized by `finalize_normals` before any render vertex is ever
built; this spec only changes *which* of the two already-computed normals
`mesh_render_vertices` picks per corner.

## Domain model involved

No new `planet-core` type. `planet-renderer`'s existing `gpu::buffers::Vertex` struct
(`position`/`normal`/`color`, all `[f32; 3]`) is unchanged — only which mesh-side normal
feeds its `normal` field changes. Two existing `planet-core` fields are read, per corner,
depending on the new toggle:
- `Vertex.normal` (smooth, already used today)
- `Face.normal` (flat, already computed by `finalize_normals`, currently unused by the
  renderer)

New renderer-side state, mirroring `App.wireframe: bool` / `Renderer`'s
`wireframe_pipeline` + precomputed line-index-buffer pattern exactly:
- `App.flat_shading: bool` (default `false`, i.e. smooth by default — matches today's
  shipped behavior)
- `Renderer` gains a second precomputed vertex buffer (flat-normal variant), alongside
  today's single (now "smooth") vertex buffer

## Function/API contracts

```rust
// planet-renderer/src/gpu/buffers.rs
pub fn mesh_render_vertices(mesh: &Mesh, colors: &[Rgb], flat_shading: bool) -> Vec<Vertex>;
// new `flat_shading` parameter. `false` behaves exactly as today (unchanged output).
// `true`: every render vertex for a face reads that face's own `Face.normal` instead of
// its corner's `Vertex.normal`.
```

- `mesh_render_indices`/`mesh_render_line_indices`/`pack_vertex_buffer`/
  `pack_index_buffer` are unchanged — vertex *count* and index topology never depend on
  which normal is picked.

```rust
// planet-renderer/src/gpu/render.rs
impl Renderer {
    pub async fn new(window: Arc<Window>, mesh: &Mesh, colors: &[Rgb]) -> Result<Self, String>;
    // unchanged signature; internally now builds two vertex buffers
    // (`vertex_buffer_smooth`, `vertex_buffer_flat`) instead of one, via two
    // `mesh_render_vertices` calls (`flat_shading` false/true)

    pub fn set_mesh(&mut self, mesh: &Mesh, colors: &[Rgb]);
    // unchanged signature; rebuilds both vertex buffers, same as `new`

    pub fn render(&self, camera: &Camera, wireframe: bool, flat_shading: bool);
    // new `flat_shading` parameter, independent of `wireframe`: selects which of the
    // two precomputed vertex buffers is bound before the existing wireframe-driven
    // pipeline/index-buffer selection proceeds unchanged
}
```

```rust
// planet-renderer/src/app.rs
pub struct App {
    // ...unchanged fields...
    wireframe: bool,
    flat_shading: bool, // new, default false
}
// WindowEvent::KeyboardInput: a new `PhysicalKey::Code(KeyCode::KeyF)` arm, matched the
// same way as today's `KeyCode::KeyW` (state == Pressed, !repeat), flips
// `self.flat_shading`. `WindowEvent::RedrawRequested`'s existing
// `renderer.render(&self.camera, self.wireframe)` call becomes
// `renderer.render(&self.camera, self.wireframe, self.flat_shading)`.
```

## Pre/post conditions

**Preconditions:**
- `mesh`'s `Face.normal`/`Vertex.normal` have already been finalized (non-placeholder)
  by `finalize_normals` — the same precondition `mesh_render_vertices` already relies on
  today for `Vertex.normal`; this spec extends that same reliance to `Face.normal`.

**Postconditions:**
- `mesh_render_vertices(mesh, colors, false)` is byte-for-byte identical to today's
  (parameterless) behavior — a pure regression guarantee, not a behavior change.
- `mesh_render_vertices(mesh, colors, true)`: every render vertex belonging to the same
  face carries an identical normal, equal to that face's own `Face.normal` — including
  the zero-vector fallback `finalize_normals` already produces for a degenerate
  (zero-area) face, with no panic.
- `Renderer::render` binds the vertex buffer matching the current `flat_shading` value;
  this choice is fully orthogonal to `wireframe`'s existing pipeline/index-buffer
  choice — all four combinations (smooth+solid, smooth+wireframe, flat+solid,
  flat+wireframe) render without panicking.
- Toggling `App.flat_shading` (via `KeyF`) requires no `Planet`/`Mesh` regeneration —
  both vertex buffers are already precomputed whenever the mesh last changed, exactly
  like `wireframe`'s precomputed line-index buffer costs no regeneration to toggle.

## BDD scenarios

Extends `planet-renderer/tests/features/mesh_render_vertices.feature` (pure, GPU-free
logic — the only part of this feature that's BDD-testable per `constitution.md`; the
`Renderer` buffer/pipeline wiring and `App`'s keyboard handling are GPU/winit-facing and
manually verified in-browser, same as `wireframe` today).

```gherkin
Feature: Converting a Mesh into render vertices

  Scenario: Converting a Mesh into render vertices with smooth shading assigns each render vertex its source vertex's normal
    Given a Mesh constructed by Mesh::cube with side 1.0
    And normals finalized for that mesh
    When the mesh is converted into render vertices with smooth shading
    Then each render vertex's normal equals its source vertex's normal

  Scenario: Converting a Mesh into render vertices with flat shading assigns every corner of a face that face's own normal
    Given a Mesh constructed by Mesh::cube with side 1.0
    And normals finalized for that mesh
    When the mesh is converted into render vertices with flat shading
    Then every render vertex belonging to the same face has that face's normal

  Scenario: Converting a Mesh with a degenerate face into render vertices with flat shading falls back to a zero normal without panicking
    Given a Mesh with 3 vertices at the same position and a triangle index-triple (0, 1, 2)
    And normals finalized for that mesh
    When the mesh is converted into render vertices with flat shading
    Then every render vertex belonging to that face has normal (0.0, 0.0, 0.0)
```

(The existing "empty Mesh" and "color assignment" scenarios are unaffected by this
spec and keep calling the new function with an explicit `flat_shading` argument — no
scenario text change needed beyond adding the parameter to their `When` step's fixed
`false` argument in the step definition.)

## Acceptance criteria

1. `mesh_render_vertices` takes a new `flat_shading: bool` parameter; every existing
   call site (`Renderer::new`, `Renderer::set_mesh`, existing tests) is updated.
2. `flat_shading == false` produces output identical to today's implementation (no
   behavior change) — verified by the existing "each render vertex's normal equals its
   source vertex's normal" scenario, now run explicitly under `flat_shading: false`.
3. `flat_shading == true`: every render vertex belonging to a given face has a normal
   equal to that face's own `Face.normal`.
4. `flat_shading == true` on a degenerate (zero-area) face yields `(0.0, 0.0, 0.0)` for
   every one of that face's render vertices, no panic — mirrors `finalize_normals`'s own
   degenerate fallback.
5. `Renderer` precomputes both a smooth and a flat vertex buffer on every mesh change
   (`Renderer::new`, `Renderer::set_mesh`); `Renderer::render(camera, wireframe,
   flat_shading)` binds the buffer matching `flat_shading`, independent of `wireframe`'s
   pipeline/index-buffer selection.
6. `App` gains a `flat_shading: bool` field (default `false`) toggled by `KeyF`
   (press-only, non-repeat — mirrors `KeyW`'s existing handling exactly); the next
   `RedrawRequested` renders with the new mode, with no `Planet`/`Mesh` regeneration.
7. Build gate passes: `cargo test --workspace && cargo fmt --check && cargo clippy
   --workspace --all-targets -- -D warnings && cargo build --target
   wasm32-unknown-unknown -p planet-renderer`.
8. Manual, in-browser (not BDD-tested per `constitution.md`): pressing `F` toggles a
   generated planet between faceted (flat) and continuous (smooth) shading, independent
   of the `W` wireframe toggle and independent of which growth-animation frame is
   currently displayed.
