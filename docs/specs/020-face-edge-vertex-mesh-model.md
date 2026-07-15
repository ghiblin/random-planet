# 020 — Face/Edge/Vertex mesh domain model

**Supersedes:** the roadmap's original phase-010 framing ("smooth vertex normals as a
renderer-only, `mesh_render_vertices`-local post-process, gated behind a flat/smooth
toggle"). That framing is replaced with a real domain-model redesign in `planet-core`
itself: `Mesh`'s core representation becomes a `Face`/`Edge`/`Vertex` adjacency graph,
with normal computed as an intrinsic property of that graph rather than derived
transiently at render time.

**Color migration removed (for now):** an earlier draft of this spec also moved color
onto `Face`/`Vertex` (face color first, then vertex color as an area-weighted average of
incident face colors, mirroring the normal computation). That's dropped. Color stays
exactly as it is today: `Planet.colors: Vec<Rgb>`, computed by `planet.rs`'s
`vertex_color(radius, sea_level, gradient)` directly from each vertex's own position,
untouched by this spec. `Face`/`Vertex` carry no `color` field. This spec is normal-only.

No flat/smooth toggle is (re)built in this spec either. Once `Vertex` carries a real
`normal`, a future toggle is a small, additive follow-up (pick `Face.normal` vs
`Vertex.normal` per render vertex) — deferred, not requested here.

## Requirements

`planet-core`'s current representation — `Vertex { position }`, `Triangle { a, b, c }`
(bare vertex-index triples), `Mesh { vertices: Vec<Vertex>, triangles: Vec<Triangle> }`
— has no adjacency: a triangle doesn't know its neighbors, a vertex doesn't know which
triangles touch it. Every rendered normal today is a flat per-triangle normal computed
transiently in `planet-renderer`.

This spec introduces `Face`, `Edge`, and a richer `Vertex` into `planet-core`'s public
domain model, so that:
- Each `Face` has its own list of boundary `Edge`s (`order` = edge count, always 3
  today — this app only ever produces triangular faces, but the field generalizes
  without code elsewhere assuming a fixed 3) and its own normal.
- Each `Edge` hosts two vertex indices (`start`, `end`, directed) and a reference to
  the one `Face` it bounds. Two triangles sharing a geometric edge are represented by
  two separate `Edge` objects (one per face, opposite winding) — this is a half-edge-style
  graph without an explicit twin/opposite pointer, since nothing in this feature needs
  to walk from one face to its neighbor across a shared edge (crack-prevention during
  subdivision is unrelated — see "Determinism and crack-prevention are unaffected" below).
- Each `Vertex` gains `normal` and `edges: Vec<usize>` (indices of every `Edge` where
  this vertex is the `start` — exactly one per incident face, so no double-counting).
  Vertex normal is computed by walking `edges` → `Edge.face` → that `Face`'s
  already-computed normal, area-weighted.

### Why area-weighted, and why a two-pass, finalize-once pipeline

Carried over from the original renderer-only investigation, now applied to the domain
model directly: an unweighted average lets a vertex's many small incident triangles
(common on this project's irregular subdivision output) outvote one large one; an
unnormalized face normal `(b - a) × (c - a)` already has magnitude `2 × area`, so
accumulating the *raw* cross product per vertex and normalizing once at the end gives
the area-weighted average for free.

`Face`/`Vertex` normal cannot be computed until every position-mutating step
(subdivision, terrain noise, ocean quota) has finished — a vertex's final position, and
therefore its neighbors' face geometry, keeps changing until then. So `normal` is a
placeholder (`Vec3::ZERO`) throughout subdivision and is only populated by a single new
`finalize_normals` pass at the very end of `Planet`'s pipeline — mirroring this
project's existing `processor/` convention (whole-mesh pre/post-processing steps that
run outside the subdivision algorithm).

### Determinism and crack-prevention are unaffected

`subdivision::edge::{EdgeKey, EdgeCache}` (the `pub(crate)` split-decision bookkeeping
that guarantees two triangles sharing an edge agree on the same midpoint vertex,
preventing cracks) is an entirely separate mechanism from the new public `Edge` type
and is untouched by this spec. Subdivision's recursive algorithm (`subdivide.rs`,
`uniform_red_split.rs`) keeps building each round from plain vertex-index triples
exactly as today; only the *final* `Mesh::new` call at each round's boundary now builds
the richer Face/Edge/Vertex.edges graph instead of a flat triangle list. This keeps the
already-shipped, already-tested subdivision engine's hot path essentially unchanged.

### Rendered-planet impact

Facet edges disappear on land terrain — `Vertex.normal` is now genuinely continuous
across triangle boundaries. Color is unaffected by this spec: the Earthy preset's
ocean/coastline color cutoff stays exactly as sharp as it is today, since color sampling
doesn't change. Only *lighting* (driven by normal) softens at the coastline — and even
there, the flattened ocean cap's own faces are already coplanar with the sphere, so
their flat and smooth normals were already nearly identical; the boundary ring between
ocean and land is where the two normal modes diverge most visibly.

## Domain model involved

### `planet-core/src/geometry/mesh.rs` (rewritten)

```rust
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,      // Vec3::ZERO until finalize_normals runs
    pub edges: Vec<usize>, // indices into Mesh::edges(); empty until Mesh::new builds the graph
}
impl Vertex {
    pub(crate) fn at(position: Vec3) -> Vertex // placeholder normal/edges — replaces today's `Vertex { position }` literal at every construction site (jitter, terrain noise, ocean quota, vertex scramble, icosahedron/cube primitives)
}

pub struct Edge {
    pub start: usize, // vertex index
    pub end: usize,   // vertex index
    pub face: usize,  // face index this edge bounds
}

pub struct Face {
    pub edges: Vec<usize>, // indices into Mesh::edges(), in winding order
    pub order: usize,      // == edges.len(); always 3 today
    pub normal: Vec3,      // Vec3::ZERO until finalize_normals runs
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
    faces: Vec<Face>,
}
```

- `geometry/mesh.rs` has no new cross-concern dependency — with color removed from this
  spec, `Vertex`/`Face`/`Edge` are all still plain spatial value types, no reference to
  `color::rgb::Rgb` at all. `rules.md`'s module-structure list only needs updating to
  mention the new `Face`/`Edge` types and `Vertex`'s new `normal`/`edges` fields.

### `planet-core/src/processor/finalize_normals.rs` (new)

```rust
pub fn finalize_normals(mesh: &Mesh) -> Mesh
```

- Pass 1, over `mesh.faces()`: derive each face's 3 corner positions via its edges'
  `start` vertices; compute the raw (unnormalized) cross product → normalize for
  `face.normal` (falls back to `Vec3::ZERO` if the face is degenerate/zero-area).
  Accumulate that raw cross product into running per-vertex sums for each of the
  face's 3 corners — computed once per face, reused both for the face's own normal and
  every incident vertex's aggregate, no redundant recomputation.
- Pass 2, over `mesh.vertices()`: normalize each vertex's accumulated raw sum →
  `vertex.normal` (`Vec3::ZERO` fallback if the sum is zero — vertex touched by no
  triangle, or only degenerate ones).
- Returns a new `Mesh` with identical topology (positions, edges, `Face.edges`/`order`
  all unchanged) — only `Face.normal` and `Vertex.normal` differ from the input.
- `pub`, not `pub(crate)`: `planet-renderer`'s `app.rs` calls it directly on
  mid-subdivision animation frames (see below), the same way it already calls
  `ColorGradient::sample` directly on intermediate meshes today — consistent with the
  existing precedent that `rules.md`'s "obtain every Mesh via Planet's lifecycle" rule
  is about how a Mesh's *geometry* is produced, not about deriving read-only shading
  data from a mesh Planet already produced.

### `planet-core/src/geometry/mesh.rs` — `Mesh` methods

```rust
impl Mesh {
    pub fn new(positions: Vec<Vec3>, triangles: Vec<(usize, usize, usize)>) -> Result<Mesh, MeshError>
    pub(crate) fn with_repositioned(&self, positions: Vec<Vec3>) -> Mesh
    pub fn vertices(&self) -> &[Vertex]
    pub fn edges(&self) -> &[Edge]
    pub fn faces(&self) -> &[Face]
    pub fn icosahedron() -> Result<Mesh, MeshError>
    pub fn cube(side: f32) -> Result<Mesh, MeshError>
}
```

- No dedicated `Triangle` type — `Mesh::new` takes bare `Vec<Vec3>` positions (not
  `Vec<Vertex>`) plus `Vec<(usize, usize, usize)>` index-triples, a plain tuple with no
  named-struct ceremony for a shape that's only ever consumed positionally
  (`.0`/`.1`/`.2`) right where it's built. Internally it validates indices (same
  `MeshError::VertexIndexOutOfBounds` check as today), builds one `Face` +3 `Edge`s per
  input triple, then a final pass groups edges by `start` vertex into each
  `Vertex.edges`. `Vertex.normal`/`Face.normal` start as placeholders.
- `Mesh::with_repositioned` is the new position-only-update path for
  `terrain_noise`/`ocean_quota`/`vertex_scramble`: same `edges`/`faces` (and each
  vertex's `edges` list, `normal`) untouched, only `position` replaced per vertex,
  positionally by index. Infallible — topology is provably unchanged from an
  already-valid `Mesh`, so no `MeshError` case applies.
- `mesh.triangles()` is removed; `mesh.faces()`/`mesh.edges()` replace it everywhere.

### `planet-core/src/processor/{terrain_noise,ocean_quota,vertex_scramble}.rs`

- Each already only computes a new `Vec3` position per vertex; each switches its final
  `Mesh::new(vertices, mesh.triangles().to_vec())` call to
  `Ok(mesh.with_repositioned(new_positions))`. Public signatures
  (`Result<Mesh, MeshError>`) stay unchanged for minimal ripple — they always return
  `Ok(...)` now, but changing them to an infallible return type is out of scope (avoids
  touching `MeshProcessor`'s type alias and every composed pipeline call site for a
  simplification nobody asked for).

### `planet-core/src/subdivision/strategies/uniform_red_split.rs`, `processor/jitter.rs`

- `exact_midpoint`/`jitter`'s `Vertex { position }` literals become `Vertex::at(position)`.
  No other change — the recursive split algorithm itself (index-triple children,
  `EdgeCache` split-decision agreement) is untouched.

### `planet-core/src/geometry/primitives/{icosahedron,cube}.rs`

- Build `Vec<Vec3>` positions directly (no `Vertex` wrapper) instead of today's
  `Vertex { position }` mapping; today's `Triangle::new(a, b, c)` calls become plain
  `(a, b, c)` tuples.

### `planet-core/src/planets/planet.rs`, `planet_builder.rs`

- `Planet.colors: Vec<Rgb>`, `Planet::colors()`, and `planet.rs`'s `vertex_color` free
  function are **unchanged** — no migration, exactly as today.
- `Planet::subdivide`'s pipeline gains one more step at the end:
  `finalize_normals(&mesh)`, after the existing `colors` computation (order between the
  two doesn't matter — they're independent derivations from the same finalized
  positions).
- `PlanetBuilder::build` likewise calls `finalize_normals(&mesh)` before constructing
  the `Planet`, alongside its existing unchanged `colors` computation.

### `planet-renderer/src/gpu/buffers.rs`, `render.rs`, `app.rs`

- `mesh_render_vertices(mesh: &Mesh, colors: &[Rgb]) -> Vec<Vertex>` **keeps** its
  `colors` parameter (color is still a separate, per-vertex-index array, unchanged) but
  drops its own flat-normal computation — it walks `mesh.faces()` → each face's `edges`
  → each edge's `start` vertex, reads `vertex.normal` directly (smooth shading,
  unconditionally — no flat/smooth toggle in this spec) and looks up
  `colors[edge.start]` exactly as it looks up `mesh.vertices()[triangle.a]` etc. today.
- `mesh_render_indices`/`mesh_render_line_indices` switch from `mesh.triangles().len()`
  to `mesh.faces().len()` (same generated index sequence — every face still has
  `order == 3` today).
- `Renderer::new`/`Renderer::set_mesh` keep their `colors: &[Rgb]` parameter, unchanged.
- `app.rs`'s `generate()`: the `on_progress` callback keeps its existing direct
  `ColorGradient` sampling per intermediate animation frame (`app.rs:194-201`,
  unchanged) and additionally calls `finalize_normals(mesh)` before storing each frame,
  so the growth animation also renders with smooth shading. The `Frames` type alias
  (`Rc<RefCell<(Vec<(Mesh, Vec<Rgb>)>, usize)>>`) is unchanged.

## Function/API contracts

Summarized above per file. The externally-visible new/changed public surface:

```rust
// planet-core
pub struct Vertex { pub position: Vec3, pub normal: Vec3, pub edges: Vec<usize> }
pub struct Edge { pub start: usize, pub end: usize, pub face: usize }
pub struct Face { pub edges: Vec<usize>, pub order: usize, pub normal: Vec3 }

impl Mesh {
    pub fn new(positions: Vec<Vec3>, triangles: Vec<(usize, usize, usize)>) -> Result<Mesh, MeshError>;
    pub fn vertices(&self) -> &[Vertex];
    pub fn edges(&self) -> &[Edge];
    pub fn faces(&self) -> &[Face];
    // icosahedron(), cube() unchanged in signature
}

pub fn finalize_normals(mesh: &Mesh) -> Mesh;

impl Planet {
    pub fn mesh(&self) -> &Mesh;   // unchanged signature; vertices/faces now carry a real normal
    pub fn colors(&self) -> &[Rgb]; // unchanged, untouched
}

// planet-renderer
pub fn mesh_render_vertices(mesh: &Mesh, colors: &[Rgb]) -> Vec<Vertex>; // colors parameter unchanged
```

## BDD scenarios

### `Mesh::new` graph construction (rewrites `planet-core/tests/features/mesh.feature`)

```gherkin
Feature: Building a Face/Edge/Vertex graph from positions and triangle indices

  Scenario: Building a Mesh from a single triangle index-triple produces one Face with 3 Edges and 3 Vertices, each with one incident edge
    Given 3 positions forming a single triangle
    And a triangle index-triple (0, 1, 2)
    When a Mesh is built from those positions and triangle index-triples
    Then the Mesh has 1 face with order 3
    And the Mesh has 3 edges
    And each of the 3 vertices has exactly 1 edge in its edges list

  Scenario: Two triangles sharing an edge each get their own Edge object, and the shared vertices see both incident faces
    Given a Mesh constructed by Mesh::cube with side 1.0
    When inspecting vertex 0's edges
    Then vertex 0 has exactly 6 edges, one per incident face

  Scenario: Building a Mesh with an out-of-bounds triangle index fails
    Given 2 positions
    And a triangle index-triple (0, 1, 5)
    When a Mesh is built from those positions and triangle index-triples
    Then building fails with VertexIndexOutOfBounds for index 5
```

### `finalize_normals` (new: `planet-core/tests/features/finalize_normals.feature`)

```gherkin
Feature: Computing Face and Vertex normals from final mesh geometry

  Scenario: Finalizing normals on a cube Mesh gives every face its flat normal and every vertex an area-weighted average
    Given a Mesh constructed by Mesh::cube with side 1.0
    When normals are finalized
    Then every face's normal has unit length
    And vertex 0's normal is approximately (-0.577, -0.577, -0.577)

  Scenario: A vertex shared by faces of unequal area weights its normal toward the larger face
    Given a Mesh where vertex 0 is shared by one large face facing (0.0, 0.0, 1.0) and one small face facing (1.0, 0.0, 0.0)
    When normals are finalized
    Then vertex 0's normal is approximately (0.01, 0.0, 1.0)

  Scenario: A vertex referenced only by degenerate faces never panics and falls back to a zero normal
    Given a Mesh with 3 vertices at the same position
    And a triangle index-triple (0, 1, 2)
    When normals are finalized
    Then no panic occurs
    And every vertex's normal is (0.0, 0.0, 0.0)
```

### Migration impact — existing feature files requiring corresponding updates

`mesh.feature`, `icosahedron.feature`, `subdivide.feature` (rename `.triangles()` /
"triangle" vocabulary to `.faces()` / "face", per `rules.md`'s "Then/And steps name the
field they assert on exactly as it appears in the domain model"), `terrain_noise.feature`,
`ocean_quota.feature`, `vertex_scramble.feature` (add a regression scenario:
repositioning leaves each vertex's `edges`/`normal` unchanged), plus
`planet-renderer/tests/features/{mesh_render_vertices,mesh_render_indices,mesh_render_line_indices}.feature`
(the `colors` parameter is unchanged, but drop the flat-normal/degenerate-flat-normal
scenarios since `mesh_render_vertices` no longer computes normals itself — it only reads
already-finalized `Vertex.normal`). `planet.feature`/`color_gradient.feature` are
**not** affected — `Planet::colors()` and color sampling are untouched.

## Acceptance criteria

1. `Mesh::new(positions, triangles)` builds exactly one `Face` (`order == 3`) and 3
   `Edge`s per input `(usize, usize, usize)` triple; every `Vertex.edges` contains
   exactly one entry per incident face (no duplicates, no omissions).
2. `Mesh::new` rejects an out-of-bounds triangle index with `MeshError::VertexIndexOutOfBounds`,
   same as today.
3. `Mesh::with_repositioned` preserves `edges`, `faces`, and every vertex's `edges`/`normal`
   unchanged — only `position` differs.
4. `finalize_normals` gives every non-degenerate face a unit-length normal.
5. `finalize_normals` gives every vertex touched by at least one non-degenerate face a
   unit-length normal — verified via the unequal-area fixture (`≈(0.01, 0.0, 1.0)`),
   locking in "area-weighted, not unweighted."
6. `finalize_normals` gives a vertex touched only by degenerate faces (or no faces) a
   `Vec3::ZERO` normal, with no panic.
7. `Planet::mesh()` returns a `Mesh` whose vertices/faces carry a final, non-placeholder
   `normal` after both `Planet::subdivide` and `PlanetBuilder::build`. `Planet::colors()`
   is unchanged and still present.
8. `mesh_render_vertices(mesh, colors)` produces one render vertex per face corner, each
   reading `vertex.normal` directly and `colors[edge.start]` for color — smooth shading
   unconditionally, color sampling unchanged.
9. Build gate passes: `cargo test --workspace && cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo build --target wasm32-unknown-unknown -p planet-renderer`.
10. Manual, in-browser (not BDD-tested per `constitution.md`): a generated planet renders
    with continuous, facet-free shading; the growth animation (intermediate subdivision
    rounds) also renders smoothly, confirming `app.rs`'s `finalize_normals` call on
    animation frames works correctly.
11. `rules.md`'s module-structure list is updated to document `geometry/mesh.rs`'s new
    `Face`/`Edge` types and `Vertex`'s new `normal`/`edges` fields, and to add
    `processor/finalize_normals.rs` to the `processor/` concern's file list.
