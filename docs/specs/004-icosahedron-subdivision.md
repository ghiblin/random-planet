# 004 — Icosahedron Subdivision

**Status:** Ready for review
**Feature slug:** `icosahedron-subdivision`

## Requirements

- `planet-core` gains base icosahedron construction: `icosahedron() -> Result<Mesh, MeshError>`, producing the classic 12-vertex, 20-triangle regular icosahedron with every vertex on the unit sphere (radius 1.0 from the origin)
- `planet-core` gains recursive subdivision built around the **Strategy design pattern**: a `SubdivisionStrategy` trait owns "how a single triangle is split for one round," and `subdivide(mesh: &Mesh, depth: u32, strategy: &mut dyn SubdivisionStrategy) -> Result<Mesh, MeshError>` is the algorithm-agnostic driver that repeats that strategy `depth` times. `subdivide` itself knows nothing about midpoints, thresholds, or triangle counts per split — it only orchestrates rounds and mesh reassembly
- This phase's sole concrete strategy is `UniformRedSplit`: every triangle's all 3 edges always split ("always-red" — no length threshold, no green triangles, both introduced later in `006-irregular-subdivision`), and each new edge vertex is placed at the **exact arithmetic midpoint** of that edge's two endpoints — no re-projection onto a sphere and no random displacement (both deferred to `005-radial-randomness`)
- The Strategy abstraction is what lets `005-radial-randomness` (a strategy that perturbs new vertices radially) and `006-irregular-subdivision` (a strategy with a length threshold, Gaussian split point, and red-green triangulation producing 1–4 children per triangle instead of always 4) plug in without changing `subdivide`'s signature or its round/recursion logic
- Shared edges between adjacent triangles must resolve to the same new vertex within a round (no duplicate-position vertices, no cracks/T-junctions) — implemented via an internal, algorithm-agnostic edge cache (`EdgeCache`) that any `SubdivisionStrategy` implementation can reuse; it does not itself decide *where* a new vertex goes, only deduplicates and caches whatever the strategy computes
- `planet-renderer` renders `subdivide(&icosahedron()?, 3, &mut UniformRedSplit)` in place of `Mesh::cube(1.0)` — infra validation only, reusing the existing generic `mesh_render_vertices`/`mesh_render_indices` conversion from `003-cube-mesh-wiring` unmodified
- `subdivide` and `EdgeCache` are generic over any valid input `Mesh` and any `SubdivisionStrategy`, not hardcoded to the icosahedron or to `UniformRedSplit` — later phases reuse both unchanged

Out of scope for this phase (later roadmap phases):
- Any random radial displacement of new vertices, and the concrete strategy that implements it (`005-radial-randomness`)
- Length-threshold stopping condition, Gaussian split-point placement, red-green triangulation for partially-split triangles, and the concrete strategy that implements them (`006-irregular-subdivision`)
- `Seed`, `SubdivisionDepth` validated newtype, `Preset`/`PresetParams`, `ColorGradient`, the `Planet` aggregate root, ocean quota (`007-planet-presets`)
- Subdivision-depth UI slider or preset dropdown (`007-planet-presets`) — this phase hardcodes the render depth as a constant
- Per-vertex color (still deferred to `007-planet-presets`, per `002-domain-data-model`)
- Camera, uniforms, or the wgpu pipeline/shader itself
- Any mechanism for `Preset` to select a `SubdivisionStrategy` at runtime — that wiring belongs to whichever later phase introduces `Preset`

## Domain model involved

**`planet-core/src/edge.rs` (new):**
- `EdgeKey { low: usize, high: usize }` — `pub`, `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`; `EdgeKey::new(a: usize, b: usize) -> EdgeKey` canonicalizes so `low = min(a, b)`, `high = max(a, b)`, making the key order-independent
- `EdgeCache` — `pub`, wraps a `HashMap<EdgeKey, usize>` mapping a canonical edge to the vertex index of its already-computed new vertex; carries **no split-point algorithm of its own**, so every `SubdivisionStrategy` implementation (this phase's and later phases') can share it unmodified
  - `EdgeCache::new() -> EdgeCache`
  - `EdgeCache::get_or_insert_with(&mut self, a: usize, b: usize, vertices: &mut Vec<Vertex>, compute: impl FnOnce(&Vertex, &Vertex) -> Vertex) -> usize` — if `EdgeKey::new(a, b)` is already cached, returns the cached index without calling `compute`; otherwise calls `compute(&vertices[a], &vertices[b])`, pushes the returned `Vertex` onto `vertices`, caches its new index, and returns it
- `EdgeKey`/`EdgeCache` must be `pub`, not `pub(crate)`: `SubdivisionStrategy::split_triangle` (below) takes `&mut EdgeCache`, and since `SubdivisionStrategy` itself must be `pub` (`subdivide` is `pub` and called across the crate boundary by `planet-renderer`, so every type in its trait parameter's method signatures must be at least as visible — a `pub(crate)` `EdgeCache` here would fail to compile under `-D warnings` with `private_interfaces: type EdgeCache is more private than the item SubdivisionStrategy::split_triangle`, verified experimentally). This supersedes `000-architecture.md`'s framing of the edge cache as "not public domain vocabulary" — that predates the Strategy-pattern design in this phase; `EdgeKey`/`EdgeCache` remain implementation-facing (aimed at `SubdivisionStrategy` implementors, not typical `Mesh` consumers) despite being technically `pub`
- No direct BDD scenarios for `EdgeKey`/`EdgeCache` — exercised indirectly through `subdivide`'s scenarios (dedup, no cracks)

**`planet-core/src/icosahedron.rs` (new):**
- `icosahedron() -> Result<Mesh, MeshError>` — builds the 12 vertices as the standard `(0, ±1, ±φ)`-permutation construction (`φ = (1.0 + 5.0_f32.sqrt()) / 2.0`), each scaled by the closed-form factor `1.0 / (1.0 + φ * φ).sqrt()` so every vertex lands at exactly radius 1.0 — computed directly via `Vec3::scale`, not `Vec3::normalized`, since the common scale factor is known in closed form and this sidesteps handling a `None` case that can never occur
- 20 hardcoded triangles referencing the 12 vertex indices, wound so every triangle's face normal points outward (same direction as its centroid from the origin)
- Delegates final assembly to `Mesh::new(vertices, triangles)`, propagating its `Result` via `?` — mirrors `Mesh::cube`'s existing pattern of returning `Result<Mesh, MeshError>` even though, given the fixed hardcoded indices, the `Err` branch is unreachable in practice; this avoids any `unwrap()`/`expect()` in production code (per `rules.md`)

**`planet-core/src/subdivide.rs` (new):**
- `pub trait SubdivisionStrategy { fn split_triangle(&mut self, vertices: &mut Vec<Vertex>, edges: &mut EdgeCache, triangle: Triangle) -> Vec<Triangle>; }` — the Strategy interface. An implementation may append new vertices to `vertices`, may read/write the round's shared `edges` cache, and returns the child triangles that replace `triangle`. It decides everything: how many children (this phase always 4), whether/where each edge splits, whether new vertices are displaced. `subdivide` never inspects or assumes this
- `&mut self` (not `&self`) because a future strategy (`005`, `006`) needs to draw from a seeded RNG, which requires mutable state; `UniformRedSplit` in this phase has no state and ignores the mutability
- `pub fn subdivide(mesh: &Mesh, depth: u32, strategy: &mut dyn SubdivisionStrategy) -> Result<Mesh, MeshError>` — repeats a single round `depth` times, calling `strategy.split_triangle` once per current triangle each round with a fresh per-round `EdgeCache`; `depth == 0` returns a mesh equal to the input, unchanged, without calling `strategy`
- Private helper `fn split_round(mesh: &Mesh, strategy: &mut dyn SubdivisionStrategy) -> Result<Mesh, MeshError>` — clones `mesh.vertices()` into a growable `Vec`, creates a fresh `EdgeCache`, iterates `mesh.triangles()` calling `strategy.split_triangle(&mut vertices, &mut edges, *triangle)` and concatenating the results, then assembles `Mesh::new(vertices, triangles)`, propagating via `?` (unreachable `Err` in practice, same rationale as `icosahedron()`)
- Neither `subdivide` nor `split_round` contains any icosahedron-specific or "always-red"-specific logic — that all lives in `UniformRedSplit`

**`planet-core/src/uniform_red_split.rs` (new):**
- `pub struct UniformRedSplit;` — a stateless, zero-sized concrete `SubdivisionStrategy`, this phase's only implementation
- `impl SubdivisionStrategy for UniformRedSplit`: `split_triangle` requests all 3 edge midpoints via `edges.get_or_insert_with(a, b, vertices, |va, vb| Vertex { position: va.position.add(vb.position).scale(0.5) })` (exact arithmetic mean, no displacement), then always returns the 4 classic children `(a, m_ab, m_ca)`, `(b, m_bc, m_ab)`, `(c, m_ca, m_bc)`, `(m_ab, m_bc, m_ca)`, preserving the parent's winding order

**`planet-core/src/lib.rs` (updated):**
- `pub mod edge;`
- `pub mod icosahedron;`
- `pub mod subdivide;`
- `pub mod uniform_red_split;`

**`planet-core/tests/features/icosahedron.feature` / `planet-core/tests/icosahedron.rs` (new):**
- BDD coverage for `icosahedron()`

**`planet-core/tests/features/subdivide.feature` / `planet-core/tests/subdivide.rs` (new):**
- BDD coverage for `subdivide()` driven with `UniformRedSplit`, following `rules.md`'s mandatory subdivision scenario set (face-count growth, no duplicate vertices at shared edges, no cracks, radius bound) plus algorithm-specific scenarios, including one proving `subdivide` is generic over the mesh (not icosahedron-specific)

**`planet-renderer/src/render.rs` (updated, thin wiring only):**
- Replaces `Mesh::cube(1.0)` with a call to `planet_core::subdivide::subdivide(&planet_core::icosahedron::icosahedron().map_err(|e| e.to_string())?, SUBDIVISION_DEPTH, &mut planet_core::uniform_red_split::UniformRedSplit).map_err(|e| e.to_string())?`, matching the file's existing `Result<Self, String>` error-propagation style
- New local constant `const SUBDIVISION_DEPTH: u32 = 3;` — temporary hardcoded value, replaced by the depth slider in `007-planet-presets`
- No other change: buffer packing (`pack_vertex_buffer`/`pack_index_buffer` via `mesh_render_vertices`/`mesh_render_indices`), pipeline setup, and draw call stay exactly as wired in `003-cube-mesh-wiring`

No changes to `camera.rs`, `uniforms.rs`, `shader.wgsl`, `app.rs`, `buffers.rs`, or `mesh.rs`.

## Function/API contracts

- `icosahedron()` never panics; it returns `Ok(Mesh)` with exactly 12 vertices and 20 triangles for every call (deterministic, no inputs)
- Every vertex returned by `icosahedron()` has `position.length()` within `1e-5` of `1.0`
- Every triangle returned by `icosahedron()` has three distinct indices, each `< 12`
- Every triangle returned by `icosahedron()` is wound outward: for triangle `(a, b, c)` with positions `pa, pb, pc`, the centroid `(pa + pb + pc) * (1.0/3.0)` and the face normal `(pb - pa).cross(pc - pa)` have a positive dot product
- `EdgeCache::get_or_insert_with(a, b, vertices, compute)` calls `compute` at most once per distinct canonical edge across the cache's lifetime; a second call with the same `(a, b)` (in either order) returns the previously cached index and does not call `compute` again
- `subdivide(mesh, 0, strategy)` returns `Ok(mesh.clone())` — identical vertices and triangles, same order — and never calls `strategy.split_triangle`
- `subdivide(mesh, depth, strategy)` for `depth >= 1` never panics and never produces a triangle referencing an out-of-bounds vertex index, provided `strategy` itself only returns triangles indexing into the `vertices` it was given (true for `UniformRedSplit`)
- `subdivide` is strategy-agnostic: swapping `strategy` changes only the resulting `Mesh`'s content, never `subdivide`'s or `split_round`'s control flow — no `if`/`match` in `subdivide.rs` branches on which concrete strategy is in use
- With `UniformRedSplit`, each round exactly quadruples the triangle count: `subdivide(mesh, depth, &mut UniformRedSplit).triangles().len() == mesh.triangles().len() * 4_usize.pow(depth)`
- With `UniformRedSplit`, each round adds exactly one new vertex per unique edge in the input mesh (no duplicates for edges shared between triangles) — for the icosahedron specifically (12 vertices, 30 edges, 20 triangles), one round produces exactly 42 vertices
- No two vertices in a `subdivide` result (with `UniformRedSplit`) occupy the same position (proves no cracks/duplicate midpoints at shared edges)
- Every new vertex introduced by `UniformRedSplit` sits at exactly `vertices[a].position.add(vertices[b].position).scale(0.5)` for its edge's endpoints `a`, `b` — verified by direct computation, not re-projected onto any sphere
- Every vertex produced by `subdivide` with `UniformRedSplit` applied (at any depth) to `icosahedron()`'s output has `position.length() <= 1.0 + 1e-5` — exact-midpoint splitting of points at or inside the unit sphere can only move new points toward the center, never beyond it; this is this phase's stand-in for the preset-driven radius bound that `007-planet-presets` will introduce
- `planet-core` has zero new `unwrap()`/`panic!()` in production code outside tests (constitution + `rules.md`); `icosahedron()`'s and `split_round`'s internal `Mesh::new` calls propagate via `?`, never unwrapped
- `render.rs` builds its vertex/index buffers from `subdivide(&icosahedron()?, 3, &mut UniformRedSplit)?` instead of `Mesh::cube(1.0)`; `mesh_render_vertices`/`mesh_render_indices` are unmodified and still used as-is

## BDD scenarios

`planet-core/tests/features/icosahedron.feature`:

```gherkin
Feature: Base icosahedron construction

  Scenario: Constructing the icosahedron produces the expected vertex and triangle counts
    Given an icosahedron mesh
    Then the Mesh is constructed successfully
    And the Mesh has 12 vertices
    And the Mesh has 20 triangles

  Scenario: Every vertex of the icosahedron mesh lies on the unit sphere
    Given an icosahedron mesh
    Then every vertex of the Mesh has a radius of 1.0

  Scenario: Every triangle in the icosahedron mesh references three distinct vertex indices
    Given an icosahedron mesh
    Then every triangle in the Mesh has three distinct vertex indices
    And every triangle index in the Mesh is less than 12

  Scenario: Every triangle in the icosahedron mesh is wound outward
    Given an icosahedron mesh
    Then every triangle's face normal points away from the origin
```

`planet-core/tests/features/subdivide.feature`:

```gherkin
Feature: Recursive subdivision via a pluggable SubdivisionStrategy

  Scenario: Subdividing the icosahedron mesh once with the uniform red-split strategy quadruples the triangle count
    Given an icosahedron mesh
    When the mesh is subdivided to depth 1 using the uniform red-split strategy
    Then the resulting Mesh has 80 triangles

  Scenario: Subdividing the icosahedron mesh to depth 2 grows the triangle count geometrically
    Given an icosahedron mesh
    When the mesh is subdivided to depth 2 using the uniform red-split strategy
    Then the resulting Mesh has 320 triangles

  Scenario: Subdividing the icosahedron mesh once does not duplicate vertices at shared edges
    Given an icosahedron mesh
    When the mesh is subdivided to depth 1 using the uniform red-split strategy
    Then the resulting Mesh has 42 vertices

  Scenario: Subdividing the icosahedron mesh never creates cracks between adjacent triangles
    Given an icosahedron mesh
    When the mesh is subdivided to depth 2 using the uniform red-split strategy
    Then no two vertices in the resulting Mesh have the same position

  Scenario: Subdividing the icosahedron mesh never pushes vertices beyond the base radius
    Given an icosahedron mesh
    When the mesh is subdivided to depth 2 using the uniform red-split strategy
    Then every vertex of the resulting Mesh has a radius less than or equal to 1.0

  Scenario: A new vertex sits at the exact arithmetic mean of its edge's endpoints
    Given an icosahedron mesh
    And the two vertices of the first triangle's first edge in the icosahedron mesh
    When the mesh is subdivided to depth 1 using the uniform red-split strategy
    Then a vertex exists in the resulting Mesh at the exact midpoint of the two given vertices

  Scenario: Subdividing to depth 0 leaves the mesh unchanged regardless of strategy
    Given an icosahedron mesh
    When the mesh is subdivided to depth 0 using the uniform red-split strategy
    Then the resulting Mesh is identical to the icosahedron mesh

  Scenario: The uniform red-split strategy subdivides an arbitrary single-triangle mesh, proving subdivide is not icosahedron-specific
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided to depth 1 using the uniform red-split strategy
    Then the resulting Mesh has 4 triangles
    And the resulting Mesh has 6 vertices
```

## Acceptance criteria

1. `icosahedron()` returns `Ok(Mesh)` with exactly 12 vertices and 20 triangles
2. Every vertex from `icosahedron()` has a radius within `1e-5` of `1.0`
3. Every triangle from `icosahedron()` has three distinct indices, each `< 12`
4. Every triangle from `icosahedron()` is wound outward (face normal · centroid-direction `> 0`)
5. `SubdivisionStrategy` is a trait with a `split_triangle(&mut self, vertices: &mut Vec<Vertex>, edges: &mut EdgeCache, triangle: Triangle) -> Vec<Triangle>` method; `subdivide` takes `strategy: &mut dyn SubdivisionStrategy` and contains no logic specific to any concrete strategy
6. `EdgeCache` contains no split-point computation of its own — `get_or_insert_with` takes the computation as a `compute` closure parameter
7. `UniformRedSplit` is the only type in `planet-core` implementing `SubdivisionStrategy` in this phase, and it alone contains the "always split all 3 edges, exact midpoint, 4 children" logic
8. `subdivide(mesh, 0, &mut UniformRedSplit)` returns a `Mesh` equal to the input, and `split_triangle` is called zero times
9. `subdivide(icosahedron, 1, &mut UniformRedSplit)` produces exactly 80 triangles and exactly 42 vertices
10. `subdivide(icosahedron, 2, &mut UniformRedSplit)` produces exactly 320 triangles
11. `subdivide` applied to a non-icosahedron single-triangle `Mesh` with `UniformRedSplit` produces exactly 4 triangles and 6 vertices, demonstrating genericity over both the mesh and the strategy
12. No two vertices in any `subdivide`-with-`UniformRedSplit` output share the same position, at any tested depth `>= 1`
13. Every vertex in `subdivide(icosahedron, depth, &mut UniformRedSplit)`'s output has radius `<= 1.0 + 1e-5`, at any tested depth
14. A new vertex's position from `UniformRedSplit` exactly equals the arithmetic mean of its edge's two endpoint positions
15. `subdivide` never panics and never produces a triangle with an out-of-bounds vertex index, for any valid input `Mesh`, any `depth`, and `UniformRedSplit` as the strategy
16. `render.rs` builds its vertex/index buffers from `subdivide(&icosahedron()?, 3, &mut UniformRedSplit)?`; `Mesh::cube` is no longer called anywhere in `planet-renderer`
17. All scenarios in `icosahedron.feature` and `subdivide.feature` pass via real `cucumber` step definitions — no undefined/stub steps
18. `cargo test --workspace`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` all pass
19. `cargo build --target wasm32-unknown-unknown -p planet-renderer` still succeeds
20. No new `unwrap()`/`panic!()` in `planet-core` or `planet-renderer`'s production code outside tests
21. Existing `mesh.feature`, `vec3.feature`, `camera`/`uniforms`/`buffers` BDD scenarios from prior phases still pass unmodified
