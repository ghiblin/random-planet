# 002 — Domain Data Model

**Status:** Ready for review
**Feature slug:** `domain-data-model`

## Requirements

- Introduce `planet-core`'s foundational value objects: `Vec3`, `Vertex`, `Triangle`, `Mesh`
- `Vec3` carries basic 3D vector math (`add`, `sub`, `scale`, `dot`, `cross`, `length`, `normalized`) — the primitive operations later phases (icosahedron construction, radial displacement, edge midpoints, subdivision) will build on
- `Mesh` is an immutable snapshot of a `Vec<Vertex>` plus a `Vec<Triangle>`, constructed through a validating constructor that rejects triangles referencing out-of-bounds vertex indices
- No icosahedron construction, no subdivision, no rendering wiring — this phase is data model only

Out of scope for this phase (later roadmap phases):
- Base icosahedron construction (`003-icosahedron-subdivision`)
- Any subdivision algorithm, `EdgeCache`, red-green triangulation (`003`–`005`)
- `Seed`, `SubdivisionDepth`, `Preset`/`PresetParams`, `ColorGradient`, the `Planet` aggregate root (later phases)
- Per-vertex color: the architecture doc (`000-architecture.md`) describes `Vertex` as "position + elevation-derived color" in its final shape, but color comes from `ColorGradient::sample(elevation)`, which doesn't exist until `006-planet-presets`. Adding an unpopulated color field now would be a half-finished field with no producer. `Vertex` in this phase holds only `position`; color is added additively in `006` per the constitution's core-first, additive-phase progression
- Wiring `planet-core` types into `planet-renderer` (buffers, rendering) — `planet-renderer`'s cube-render pipeline from `001-cube-render` is untouched by this phase

## Domain model involved

All new — `planet-core/src/lib.rs` is currently empty (scaffolded but unpopulated in `001-cube-render`).

**`planet-core/src/vec3.rs` (new):**
- `Vec3 { x: f32, y: f32, z: f32 }` — public fields, `Debug`, `Clone`, `Copy`, `PartialEq`
- `Vec3::new(x: f32, y: f32, z: f32) -> Vec3`
- `Vec3::add(&self, other: Vec3) -> Vec3`
- `Vec3::sub(&self, other: Vec3) -> Vec3`
- `Vec3::scale(&self, factor: f32) -> Vec3`
- `Vec3::dot(&self, other: Vec3) -> f32`
- `Vec3::cross(&self, other: Vec3) -> Vec3`
- `Vec3::length(&self) -> f32`
- `Vec3::normalized(&self) -> Option<Vec3>`

**`planet-core/src/mesh.rs` (new):**
- `Vertex { position: Vec3 }` — public field, `Debug`, `Clone`, `Copy`, `PartialEq`
- `Triangle { a: usize, b: usize, c: usize }` — public fields, `Debug`, `Clone`, `Copy`, `PartialEq`; indices into a `Mesh`'s vertex list
- `Triangle::new(a: usize, b: usize, c: usize) -> Triangle`
- `MeshError` — `Debug`, `Clone`, `PartialEq`, implements `std::fmt::Display` + `std::error::Error`
  - `MeshError::VertexIndexOutOfBounds { index: usize, vertex_count: usize }`
- `Mesh` — private fields (`vertices: Vec<Vertex>`, `triangles: Vec<Triangle>`), `Debug`, `Clone`, `PartialEq`
  - `Mesh::new(vertices: Vec<Vertex>, triangles: Vec<Triangle>) -> Result<Mesh, MeshError>`
  - `Mesh::vertices(&self) -> &[Vertex]`
  - `Mesh::triangles(&self) -> &[Triangle]`

**`planet-core/src/lib.rs` (updated):**
- `pub mod vec3;`
- `pub mod mesh;`

No changes to `planet-renderer`.

## Function/API contracts

- `Vec3::add`, `Vec3::sub`, `Vec3::scale`, `Vec3::dot`, `Vec3::cross` never panic for any finite `f32` component inputs and perform plain component-wise/algebraic arithmetic (no clamping, no normalization)
- `Vec3::length` returns a value `>= 0.0` for any finite input, and returns exactly `0.0` for the zero vector `Vec3::new(0.0, 0.0, 0.0)`
- `Vec3::normalized` returns `Some(unit_vector)` with `unit_vector.length()` within `1e-5` of `1.0` for any input with non-zero length, and returns `None` for the zero vector — it never panics and never divides by zero
- `Triangle::new` and `Triangle`'s fields accept any `usize` values; `Triangle` has no bounds awareness of its own — validity against a vertex list is `Mesh::new`'s responsibility
- `Mesh::new` returns `Ok(Mesh)` when every triangle's `a`, `b`, and `c` are all `< vertices.len()`, preserving the given `vertices` and `triangles` unchanged, in the given order, with no deduplication
- `Mesh::new` returns `Err(MeshError::VertexIndexOutOfBounds { index, vertex_count })` — identifying the offending index and the vertex count it was checked against — when any triangle references an index `>= vertices.len()`; it never panics
- `Mesh::new(vec![], vec![])` returns `Ok(Mesh)` with an empty vertex list and an empty triangle list
- `planet-core` has zero `unwrap()`/`panic!()` outside of tests (constitution + `rules.md`)

## BDD scenarios

`planet-core/tests/features/vec3.feature`:

```gherkin
Feature: Vec3 basic math operations

  Scenario: Adding two vectors sums their components
    Given a Vec3 of (1.0, 2.0, 3.0)
    And a second Vec3 of (4.0, 5.0, 6.0)
    When the two vectors are added
    Then the resulting Vec3 is (5.0, 7.0, 9.0)

  Scenario: Subtracting two vectors
    Given a Vec3 of (5.0, 7.0, 9.0)
    And a second Vec3 of (4.0, 5.0, 6.0)
    When the second vector is subtracted from the first
    Then the resulting Vec3 is (1.0, 2.0, 3.0)

  Scenario: Scaling a vector by a scalar
    Given a Vec3 of (1.0, 2.0, 3.0)
    When the vector is scaled by 2.0
    Then the resulting Vec3 is (2.0, 4.0, 6.0)

  Scenario: Dot product of two orthogonal vectors is zero
    Given a Vec3 of (1.0, 0.0, 0.0)
    And a second Vec3 of (0.0, 1.0, 0.0)
    When the dot product of the two vectors is computed
    Then the result is 0.0

  Scenario: Cross product of two orthogonal unit vectors
    Given a Vec3 of (1.0, 0.0, 0.0)
    And a second Vec3 of (0.0, 1.0, 0.0)
    When the cross product of the two vectors is computed
    Then the resulting Vec3 is (0.0, 0.0, 1.0)

  Scenario: Length of a vector
    Given a Vec3 of (3.0, 4.0, 0.0)
    When the vector's length is computed
    Then the result is 5.0

  Scenario: Normalizing a non-zero vector produces a unit vector
    Given a Vec3 of (3.0, 4.0, 0.0)
    When the vector is normalized
    Then the resulting Vec3 has a length of 1.0

  Scenario: Normalizing a zero-length vector returns nothing
    Given a Vec3 of (0.0, 0.0, 0.0)
    When the vector is normalized
    Then normalization returns nothing
```

`planet-core/tests/features/mesh.feature`:

```gherkin
Feature: Mesh construction and validation

  Scenario: Constructing a Mesh with all triangle indices in bounds succeeds
    Given a list of 3 vertices
    And a Triangle referencing indices 0, 1, 2
    When a Mesh is constructed from the vertices and the triangle
    Then the Mesh is constructed successfully
    And the Mesh's vertices match the given list
    And the Mesh's triangles match the given list

  Scenario: Constructing a Mesh with an out-of-bounds triangle index fails
    Given a list of 3 vertices
    And a Triangle referencing indices 0, 1, 3
    When a Mesh is constructed from the vertices and the triangle
    Then the construction fails with a vertex-index-out-of-bounds error

  Scenario: Constructing an empty Mesh succeeds
    Given an empty list of vertices
    And an empty list of triangles
    When a Mesh is constructed from the vertices and the triangles
    Then the Mesh is constructed successfully
    And the Mesh has zero vertices
    And the Mesh has zero triangles
```

## Acceptance criteria

1. `Vec3` exposes `new`, `add`, `sub`, `scale`, `dot`, `cross`, `length`, `normalized`, all covered by real `cucumber` step definitions in `vec3.feature` (no undefined/stub steps)
2. `Vec3::add`, `sub`, `scale`, `dot`, `cross` never panic for finite `f32` inputs
3. `Vec3::length` is `>= 0.0` for any finite input and is exactly `0.0` for the zero vector
4. `Vec3::normalized` returns `Some` with length within `1e-5` of `1.0` for any non-zero-length vector, and `None` for the zero vector, without panicking
5. `Triangle` holds exactly three `usize` indices (`a`, `b`, `c`) and is constructible independently of any `Mesh`
6. `Mesh::new` returns `Ok(Mesh)` when every triangle index is `< vertices.len()`, preserving vertices and triangles unchanged and in order
7. `Mesh::new` returns `Err(MeshError::VertexIndexOutOfBounds)` — never panics — when any triangle references an index `>= vertices.len()`
8. `Mesh::new(vec![], vec![])` succeeds and produces a `Mesh` with zero vertices and zero triangles
9. All scenarios in `vec3.feature` and `mesh.feature` pass via real `cucumber` step definitions
10. `cargo test --workspace`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` all pass
11. `cargo build --target wasm32-unknown-unknown -p planet-renderer` still succeeds
12. `planet-core` contains no `unwrap()`/`panic!()` outside of tests
13. `planet-renderer`'s existing camera/buffers/uniforms tests from `001-cube-render` still pass unmodified — this phase does not touch `planet-renderer`
