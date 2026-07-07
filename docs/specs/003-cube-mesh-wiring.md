# 003 — Cube Mesh Wiring

**Status:** Ready for review
**Feature slug:** `cube-mesh-wiring`

## Requirements

- `planet-renderer` depends on `planet-core` (new workspace path dependency)
- `planet_core::mesh::Mesh` gains a `Mesh::cube(side: f32) -> Result<Mesh, MeshError>` utility constructor: 8 unique corner `Vertex`es, 12 `Triangle`s (2 per face), forming an axis-aligned cube of edge length `side` centered at the origin
- `Mesh::cube` rejects a negative `side`: it returns `Err(MeshError::NegativeCubeSide { side })` rather than silently building a mesh, since a negative edge length has no sensible geometric meaning
- The rendered cube's geometry comes from `Mesh::cube(1.0)` — not a hardcoded per-face vertex table
- `planet-renderer` gains pure, natively-testable conversion logic that turns any `Mesh` into flat-shaded GPU vertex/index data: one renderer `Vertex` (position + normal) per triangle corner, with the face normal computed from the triangle's own geometry via `Vec3::cross`/`Vec3::normalized` — because a `Mesh`'s shared corner vertices cannot carry a single correct flat normal (a cube corner touches three faces with three different normals)
- The existing `FACES` const, `cube_vertices()`, and `cube_indices()` in `planet-renderer/src/buffers.rs` are removed and replaced by this `Mesh`-based path
- `render.rs`'s buffer setup is rewired to source its vertex/index bytes from `Mesh::cube(1.0)` plus the new conversion functions instead of the old table; the wgpu pipeline/draw-call logic itself is unchanged

Out of scope for this phase (later roadmap phases):
- Base icosahedron construction or any subdivision (`004-icosahedron-subdivision` onward) — the rendered shape stays a cube
- Per-vertex color (still deferred to `007-planet-presets`, per `002-domain-data-model`)
- Any change to camera, uniforms, or the wgpu pipeline/shader itself

## Domain model involved

**`planet-core/src/mesh.rs` (updated):**
- Add `Mesh::cube(side: f32) -> Result<Mesh, MeshError>` — for `side < 0.0`, returns `Err(MeshError::NegativeCubeSide { side })` without building any geometry; otherwise builds the 8 corners of an axis-aligned cube of edge length `side` centered at the origin, and 12 triangles (2 per face, consistent outward winding), returning `Mesh::new(vertices, triangles)`
- Add `MeshError::NegativeCubeSide { side: f32 }` — a new variant on the existing `MeshError` enum, identifying the rejected `side` value; covered by the same `Debug`, `Clone`, `PartialEq`, `Display`, `std::error::Error` derivations/impls as the existing `VertexIndexOutOfBounds` variant
- No other change to `Vec3`, `Vertex`, `Triangle`, or `Mesh::new`/`vertices`/`triangles`

**`planet-core/tests/features/mesh.feature` / `planet-core/tests/mesh.rs` (updated):**
- New scenarios covering `Mesh::cube` are added alongside the existing `Mesh::new` scenarios from `002-domain-data-model` (same file — `Mesh::cube` is another constructor on the same type)

**`planet-renderer/src/buffers.rs` (rewritten):**
- Removed: `FACES`, `cube_vertices()`, `cube_indices()` — no renderer-local cube constructor; cube geometry now comes entirely from `planet_core::mesh::Mesh::cube`
- Renderer-local `Vertex { position: [f32; 3], normal: [f32; 3] }` — unchanged, still the GPU-facing per-triangle-corner type
- `mesh_render_vertices(mesh: &Mesh) -> Vec<Vertex>` (new) — for each triangle, emits 3 renderer `Vertex`es (one per corner), all three sharing the same computed face normal
- `mesh_render_indices(mesh: &Mesh) -> Vec<u16>` (new) — emits `0..3 * mesh.triangles().len()` as `u16`, matching `mesh_render_vertices`'s per-triangle-unrolled layout
- `pack_vertex_buffer(&[Vertex]) -> Vec<u8>`, `pack_index_buffer(&[u16]) -> Vec<u8>` — unchanged

**`planet-renderer/src/render.rs` (updated, thin wiring only):**
- Replaces `pack_vertex_buffer(&cube_vertices())` / `pack_index_buffer(&cube_indices())` with: `planet_core::mesh::Mesh::cube(1.0)` once (propagating `MeshError` into the existing `Result<Self, String>` via `.map_err(|e| e.to_string())?`, matching the file's existing adapter/device error-propagation style), then `pack_vertex_buffer(&mesh_render_vertices(&mesh))` / `pack_index_buffer(&mesh_render_indices(&mesh))`
- No other change: pipeline setup, draw call, and buffer creation calls stay indexed exactly as before

**`planet-renderer/Cargo.toml` (updated):**
- Add `planet-core = { path = "../planet-core" }` to `[dependencies]`

No changes to `camera.rs`, `uniforms.rs`, `shader.wgsl`, or `app.rs`.

## Function/API contracts

- `Mesh::cube(side)` returns `Err(MeshError::NegativeCubeSide { side })` when `side < 0.0` — it never panics, and never builds a `Mesh` in this case
- `Mesh::cube(side)` returns `Ok(Mesh)` with exactly 8 vertices and 12 triangles for any finite `side >= 0.0`
- Vertices are the 8 corners of an axis-aligned cube centered at the origin; each coordinate is `±half` where `half = side / 2.0`
- `side == 0.0` produces a valid, fully degenerate `Mesh` whose 8 vertices all coincide at the origin
- Every triangle's three indices are distinct and `< 8`; triangles are wound consistently outward — verified downstream by `mesh_render_vertices`'s `+X`-face normal check
- `mesh_render_vertices(mesh)` returns exactly `3 * mesh.triangles().len()` `Vertex` values, in triangle order; the 3 vertices for a given triangle carry the positions of `mesh.vertices()[triangle.a]`, `[triangle.b]`, `[triangle.c]` respectively (in that order) and an identical normal
- The normal for a triangle's vertices is `(b - a).cross(c - a).normalized()` where `a`, `b`, `c` are the triangle's three vertex positions in order; when that cross product is degenerate (`Vec3::normalized` returns `None` — zero-area triangle, e.g. from `Mesh::cube(0.0)`), the emitted normal is `Vec3::new(0.0, 0.0, 0.0)` — `mesh_render_vertices` never panics, for any valid `Mesh` including one containing degenerate triangles
- `mesh_render_vertices(&Mesh::new(vec![], vec![]).unwrap())` returns an empty `Vec<Vertex>`
- `mesh_render_indices(mesh)` returns `[0, 1, 2, ..., 3 * mesh.triangles().len() - 1]` as `u16`; this function assumes `3 * mesh.triangles().len() <= u16::MAX` (true for the cube's 12 triangles; revisited if a future phase renders larger meshes through this same path)
- `mesh_render_indices(&Mesh::new(vec![], vec![]).unwrap())` returns an empty `Vec<u16>`
- Applying `mesh_render_vertices` to `Mesh::cube(1.0)`'s output reproduces the same six outward-facing face normals the old `FACES` table encoded — in particular, the two triangles making up the `+X` face both produce the normal `(1.0, 0.0, 0.0)`
- `planet-core` and `planet-renderer` have zero new `unwrap()`/`panic!()` in production code outside tests (constitution + `rules.md`); `MeshError` from `Mesh::cube` is propagated via `Result`, never unwrapped, in `render.rs`

## BDD scenarios

`planet-core/tests/features/mesh.feature` (new scenarios, added to the existing file):

```gherkin
  Scenario: Constructing a cube mesh with side 1.0 succeeds with the expected vertex and triangle counts
    Given a Mesh constructed by Mesh::cube with side 1.0
    Then the Mesh is constructed successfully
    And the Mesh has 8 vertices
    And the Mesh has 12 triangles

  Scenario: Every triangle in a cube mesh references three distinct vertex indices
    Given a Mesh constructed by Mesh::cube with side 1.0
    Then every triangle in the Mesh has three distinct vertex indices
    And every triangle index in the Mesh is less than 8

  Scenario: Constructing a cube mesh with side 2.0 doubles the distance from the origin to every vertex
    Given a Mesh constructed by Mesh::cube with side 1.0
    And a Mesh constructed by Mesh::cube with side 2.0
    Then every vertex of the side-2.0 Mesh is twice as far from the origin as the corresponding vertex of the side-1.0 Mesh

  Scenario: Constructing a cube mesh with a negative side fails
    When a Mesh is constructed by Mesh::cube with side -1.0
    Then the construction fails with a negative-cube-side error

  Scenario: Constructing a cube mesh with side 0.0 produces a degenerate mesh with all vertices at the origin
    Given a Mesh constructed by Mesh::cube with side 0.0
    Then the Mesh is constructed successfully
    And every vertex of the Mesh is at the origin
```

`planet-renderer/tests/features/mesh_render_vertices.feature`:

```gherkin
Feature: Converting a Mesh into flat-shaded render vertices

  Scenario: Converting a cube Mesh into render vertices produces one vertex per triangle corner
    Given a Mesh constructed by Mesh::cube with side 1.0
    When the mesh is converted into render vertices
    Then the render vertex list has 36 vertices
    And every triangle's three render vertices share an identical normal
    And every render vertex normal has unit length

  Scenario: Converting the cube mesh's +X face triangles produces an outward-facing normal
    Given a Mesh constructed by Mesh::cube with side 1.0
    When the mesh is converted into render vertices
    Then the +X face triangles have the normal (1.0, 0.0, 0.0)

  Scenario: Converting a Mesh with a degenerate triangle never panics and yields a zero normal
    Given a Mesh with 3 vertices at the same position
    And a Triangle referencing indices 0, 1, 2
    When the mesh is converted into render vertices
    Then no panic occurs
    And every render vertex normal is (0.0, 0.0, 0.0)

  Scenario: Converting an empty Mesh into render vertices produces an empty list
    Given an empty Mesh with no vertices and no triangles
    When the mesh is converted into render vertices
    Then the render vertex list is empty
```

`planet-renderer/tests/features/mesh_render_indices.feature`:

```gherkin
Feature: Converting a Mesh into render indices

  Scenario: Converting a cube Mesh into render indices produces sequential indices
    Given a Mesh constructed by Mesh::cube with side 1.0
    When the mesh is converted into render indices
    Then the render index list is 0 through 35 in order

  Scenario: Converting an empty Mesh into render indices produces an empty list
    Given an empty Mesh with no vertices and no triangles
    When the mesh is converted into render indices
    Then the render index list is empty
```

## Acceptance criteria

1. `Mesh::cube(1.0)` returns `Ok(Mesh)` with exactly 8 vertices and 12 triangles
2. Every triangle returned by `Mesh::cube` has three distinct indices, each `< 8`
3. `Mesh::cube(2.0)`'s vertices are each twice as far from the origin as `Mesh::cube(1.0)`'s corresponding vertices
4. `Mesh::cube(0.0)` succeeds and produces a `Mesh` whose 8 vertices all sit at the origin
5. `Mesh::cube(-1.0)` returns `Err(MeshError::NegativeCubeSide { side: -1.0 })` without panicking
6. `mesh_render_vertices(&mesh)` returns exactly `3 * mesh.triangles().len()` `Vertex` values, in triangle order, with positions matching the triangle's referenced vertices
7. For every triangle, all three of its emitted render vertices share an identical normal
8. For a non-degenerate triangle, the emitted normal has unit length (within `1e-5`)
9. For a degenerate triangle, the emitted normal is `(0.0, 0.0, 0.0)` and `mesh_render_vertices` does not panic
10. `mesh_render_vertices` applied to `Mesh::cube(1.0)`'s output yields normal `(1.0, 0.0, 0.0)` for the `+X` face's two triangles
11. `mesh_render_vertices` applied to an empty `Mesh` returns an empty `Vec<Vertex>`
12. `mesh_render_indices(&mesh)` returns `0..3 * mesh.triangles().len()` as `u16`, in order
13. `mesh_render_indices` applied to an empty `Mesh` returns an empty `Vec<u16>`
14. `render.rs` builds its vertex/index buffers via `Mesh::cube(1.0)` + `mesh_render_vertices()`/`mesh_render_indices()`; `FACES`, `cube_vertices()`, `cube_indices()` no longer exist anywhere in the crate
15. `planet-renderer/Cargo.toml` declares a workspace path dependency on `planet-core`
16. All scenarios in `mesh.feature` (new `Mesh::cube` scenarios), `mesh_render_vertices.feature`, and `mesh_render_indices.feature` pass via real `cucumber` step definitions — no undefined/stub steps
17. `cargo test --workspace`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` all pass
18. `cargo build --target wasm32-unknown-unknown -p planet-renderer` still succeeds
19. No new `unwrap()`/`panic!()` in `planet-core` or `planet-renderer`'s production code outside tests; `Mesh::cube`'s `MeshError` is propagated via `Result`, never unwrapped, in `render.rs`
20. Existing `camera.rs`/`uniforms.rs` BDD scenarios from `001-cube-render` still pass unmodified
