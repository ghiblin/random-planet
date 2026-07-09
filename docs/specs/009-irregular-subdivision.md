# 009 — Irregular Subdivision

**Status:** Ready for review
**Feature slug:** `irregular-subdivision`

This is `docs/roadmap.md`'s own "006 — Irregular subdivision" phase — it becomes spec number `009` only because ad-hoc refactor specs `005-subdivision-facade`, `006-by-concern-file-layout`, and `008-strategies-module` already claimed the intervening numbers. Scope matches the roadmap line exactly: length-threshold stopping condition, Gaussian-distributed split point, red-green triangulation for partially-split triangles — i.e. `000-architecture.md`'s "Subdivision algorithm (red-green)" section, implemented in full for the first time. No `Preset`/`PresetParams`/`ColorGradient`/ocean quota (`007-planet-presets` on the roadmap, which will land as a higher-numbered spec).

## Requirements

- `planet-core` gains a new public value type `MinEdgeLength` (`planet-core/src/subdivision/min_edge_length.rs`), wrapping a validated `f32` — the per-edge stopping threshold from `000-architecture.md`. Constructor `MinEdgeLength::new(value: f32) -> Result<MinEdgeLength, MinEdgeLengthError>` rejects `value < 0.0` via `MinEdgeLengthError::Negative { value }` (a negative distance threshold is nonsensical; the comparison `value < 0.0` is `false` for `NaN`, so `MinEdgeLength::new(f32::NAN)` is also rejected, mirroring the NaN-rejection-by-comparison pattern `ElevationNoiseRange::new` already established). `MinEdgeLength::default()` returns `0.1` (roughly a tenth of the icosahedron's own edge length of ~1.0515, giving several rounds of headroom before the `Steps` hard cap is reached at default depth)
- `planet-core` gains a new public value type `SplitPointVariance` (`planet-core/src/subdivision/split_point_variance.rs`), wrapping a validated `f32` — the standard deviation of the Gaussian used to place a split point along an edge, from `000-architecture.md`. Constructor `SplitPointVariance::new(value: f32) -> Result<SplitPointVariance, SplitPointVarianceError>` rejects `value < 0.0` via `SplitPointVarianceError::Negative { value }` (a standard deviation cannot be negative; same NaN-rejection-by-comparison as above). `value == 0.0` is valid and collapses the Gaussian to a point mass at the mean — every split point lands exactly at `t = 0.5` (the arithmetic midpoint), the same convergence property `ElevationNoiseRange`'s zero-width case already has. `SplitPointVariance::default()` returns `0.1`
- `SubdivisionMode` gains a third variant, `RedGreenSplit { seed: Seed, elevation_noise_range: ElevationNoiseRange, min_edge_length: MinEdgeLength, split_point_variance: SplitPointVariance }`, exactly as `005-subdivision-facade` anticipated. `SubdivisionMode::default()` is unaffected — still resolves to `UniformRedSplit`. `Copy`, `Clone`, `Debug`, `PartialEq`, `Default` remain derived (no new trait-bound loss beyond the `Eq` loss `007-radial-randomness` already introduced)
- `SubdivisionStrategy`'s trait signature (`planet-core/src/subdivision/subdivide.rs`), `EdgeCache` (`planet-core/src/subdivision/edge.rs`), and `UniformRedSplit`/`RadialRandomSplit` are **not touched by this feature at all** — no signature change, no new method, not even a mechanical one-line edit to those two strategy files. The per-edge split/no-split decision is implemented entirely inside the new `RedGreenSplit` strategy's own file, invisible to the trait, to `EdgeCache`, and to every other strategy. The one exception is a small, strategy-agnostic pre-allocation optimization to `subdivide.rs`'s private `split_round` helper, described next — it benefits all 3 modes equally and changes no strategy's observable output
- `subdivide.rs`'s private `split_round` helper (called once per round by `subdivide()`, itself unchanged) gains a pre-allocation optimization: before iterating `mesh.triangles()`, it computes `let triangle_count = mesh.triangles().len();`, calls `vertices.reserve(3 * triangle_count)`, and initializes the round's `triangles` accumulator as `Vec::with_capacity(4 * triangle_count)` instead of `Vec::new()`. Both bounds are safe upper bounds for *any* `SubdivisionStrategy` implementation, not just `RedGreenSplit`: no strategy can create more than one new vertex per edge per triangle (3 edges), and no strategy can produce more than 4 children from one triangle (the classic red-split case — `RedGreenSplit`'s green and leaf cases produce fewer, so this over-allocates for those rounds, which is harmless, just unused capacity). This eliminates the reallocate-and-copy work `Vec::push`/`Vec::extend`'s amortized doubling growth would otherwise perform repeatedly within a round with many triangles, without changing what is computed, in what order, or what any strategy returns — `UniformRedSplit`'s and `RadialRandomSplit`'s existing scenario coverage is unaffected because this changes only when and how much memory is reserved, never any value
- A new `pub(crate)` strategy, `RedGreenSplit` (`planet-core/src/subdivision/strategies/red_green_split.rs`), implements `SubdivisionStrategy` with the existing, unchanged signature `fn split_triangle(&mut self, vertices: &mut Vec<Vertex>, edges: &mut EdgeCache, triangle: Triangle) -> Vec<Triangle>`. It carries the same shape of state as `RadialRandomSplit` — an RNG plus its configuration fields — and nothing else: **no cross-round memory of which triangles were previously classified red, green, or leaf**. Every call independently re-derives its answer from the triangle's current 3 edges; a triangle that was a green or leaf output of a previous round is handed back into `split_triangle` again next round exactly like any other triangle and is free to split further if any of its edges (including a newly created diagonal from a prior green split) now measures `>= min_edge_length` — see `000-architecture.md`'s "Subdivision algorithm (red-green)" section for the rationale (recomputing live, with no per-call state, is both simpler and no more expensive at this app's mesh scale than tracking prior classifications, at the cost of occasionally giving a green triangle one extra round of refinement). Algorithm for a given call:
  1. For each of the 3 edges in the fixed `ab, bc, ca` order: compute the edge's current length directly from `vertices[..].position` (no `EdgeCache` involvement for this comparison — it is a pure function of the two shared endpoint positions, so the triangle on the other side of this edge reaches the identical conclusion independently, with no cache needed to keep them in agreement). If `length < min_edge_length.value()`, this edge is **not split** — the triangulation below just reuses the edge's 2 original vertex indices directly, and `EdgeCache` is never consulted for that edge at all (no vertex is created, so there is nothing to cache or dedupe). If `length >= min_edge_length.value()`, this edge **is split** — call the existing, unmodified `EdgeCache::get_or_insert_with(a, b, vertices, compute)` (the exact method `UniformRedSplit`/`RadialRandomSplit` already use) where `compute` (1) draws `t` from `Normal::new(0.5, split_point_variance.value())` via this strategy's own `Pcg32` (seeded once at construction from `seed`, exactly as `RadialRandomSplit::new` seeds its RNG), (2) clamps `t` to `[MIN_SPLIT_T, MAX_SPLIT_T]` (new `pub(crate) const MIN_SPLIT_T: f32 = 0.05;` / `MAX_SPLIT_T: f32 = 0.95;` in this file — same degenerate-sliver-triangle rationale as `RadialRandomSplit`'s `MIN_VERTEX_RADIUS`, applied to the split-point parameter, defined independently in this file per every existing strategy's self-contained-constant convention), (3) computes the split-point position `a.position + (b.position - a.position) * t`, and (4) applies the exact radial-displacement logic `RadialRandomSplit::displaced_midpoint` uses (draw a delta from `elevation_noise_range`, clamp the new radius to a `MIN_VERTEX_RADIUS: f32 = 0.05` floor defined locally in this file, guard the exact-zero-radius case unchanged) — `get_or_insert_with`'s own cache still ensures the second triangle sharing this split edge reuses the exact same midpoint index without re-invoking `compute`, exactly as it does today for the other two strategies
  2. Tally how many of the 3 edges were split, and triangulate per `000-architecture.md`'s red-green table:
     - **3 split** (red): the classic 4-child split, identical topology to `UniformRedSplit`/`RadialRandomSplit` — `(a, mid_ab, mid_ca)`, `(b, mid_bc, mid_ab)`, `(c, mid_ca, mid_bc)`, `(mid_ab, mid_bc, mid_ca)`
     - **2 split** (green, 3 children, "fan through the two midpoints"): let `mp` be the midpoint of whichever of the two split edges comes first in the fixed `ab, bc, ca` order, and `mq` the other split edge's midpoint. The triangle's boundary, walked in its original `a → b → c` cyclic order but routed through both midpoints and straight across the one unsplit edge, visits 5 points in a fixed cycle; the fan pivots at `mp`, producing 3 triangles that each use `mp` plus one consecutive pair of the remaining 4 boundary points. Worked example — edges `bc` and `ca` split (`ab` unsplit): boundary cycle is `a, b, mid_bc, c, mid_ca` (back to `a`); `mp = mid_bc` (first in fixed order), so the fan from `mid_bc` gives `(mid_bc, c, mid_ca)`, `(mid_bc, mid_ca, a)`, `(mid_bc, a, b)`
     - **1 split** (green, 2 children): the split edge's endpoints are `p, q` (in the edge's fixed direction — `a,b` for `ab`; `b,c` for `bc`; `c,a` for `ca`) with midpoint `m`, and `r` is the third vertex. The 2 triangles are `(p, m, r)` and `(m, q, r)`
     - **0 split** (leaf): the triangle is returned unchanged — its original 3 indices, no new vertex
  3. Return the resulting `Vec<Triangle>` (1, 2, 3, or 4 triangles depending on the case above) — no bookkeeping beyond this call
- `planet-core` gains a new confirmed dependency, `rand_distr` (`Normal` distribution — `rand`'s own `distr` module dropped `Normal` in the version already pinned in this workspace; Gaussian sampling requires the separate crate). Added to `[workspace.dependencies]` and `planet-core/Cargo.toml`; `tech-stack.md` gains a row for it
- `planet-renderer`'s `App::resumed` switches its `subdivide` call from `SubdivisionMode::RadialRandomSplit` to `SubdivisionMode::RedGreenSplit`, reusing the existing `DEMO_SEED` constant and `ElevationNoiseRange::default()`, plus `MinEdgeLength::default()` and `SplitPointVariance::default()` — so the rendered planet visibly shows the red-green irregular triangulation instead of the always-fully-split radial-random mesh. Temporary, hardcoded demonstration wire-up, same as `007-radial-randomness`'s own wiring change — no new UI control
- `rules.md`'s "Module structure" section is updated: `subdivision/`'s concern-file list gains `min_edge_length.rs` (`MinEdgeLength`) and `split_point_variance.rs` (`SplitPointVariance`); its `strategies/` sub-concern list gains `red_green_split.rs`

Out of scope:
- `Preset`, `PresetParams`, `ColorGradient`, ocean quota, or the `Planet` aggregate root (`007-planet-presets` on the roadmap) — `min_edge_length`, `elevation_noise_range`, and `split_point_variance` here are standalone, directly-constructed value types, not fields pulled from a preset, exactly as `elevation_noise_range` was kept standalone in `007-radial-randomness`
- Any change whatsoever — not even a mechanical one-line touch — to `UniformRedSplit`, `RadialRandomSplit`, `SubdivisionStrategy` (the trait itself, including its `split_triangle` signature), or `EdgeCache` (either its existing method or its internal storage). All of this feature's red-green-specific logic (the length threshold, the Gaussian split point, and the per-edge triangulation) is self-contained inside `RedGreenSplit`'s own file. The **only** change to shared code is `split_round`'s pre-allocation (see Requirements) — `subdivide()`'s own per-round loop, its `update_cb` invocation, and everything else in `subdivide.rs` besides those 2 new lines in `split_round` are unchanged
- A new `SubdivisionStrategy` trait method letting a strategy report its own exact or tighter output-size hint, or computing the mesh's actual distinct-edge count before choosing a capacity (which would require walking every edge once just to size a buffer) — the generic `4*T`/`3*T` upper bound is enough at this app's scale, and either alternative would reintroduce trait surface that only benefits from strategy-specific knowledge, which this feature avoids by design
- Any cross-round memory of a triangle's prior red/green/leaf classification — this was considered (a private `settled` set inside `RedGreenSplit`) and deliberately rejected: it is not a performance win (a `HashSet` lookup against a set that only grows across the whole `subdivide()` call is, at this app's mesh scale, no cheaper than re-measuring 3 edge lengths, and carries a real memory cost the stateless approach has none of), and the only thing it would buy is strict conformance to a stricter "green never recurses, no matter what" rule that `000-architecture.md` has been revised to relax (see that doc's "Subdivision algorithm (red-green)" section, amended by this feature)
- A depth/min-edge-length/split-point-variance UI slider or any new UI control — still deferred to `007-planet-presets`, as `005-subdivision-facade` and `007-radial-randomness` already scoped out
- A "regenerate" control or non-deterministic seeding — same standing exclusion `007-radial-randomness` already established
- Any change to `Mesh`, `MeshError`, `Vec3`, `Triangle`, `Vertex`, `EdgeKey`, `Steps`, or `SubdivisionArgs`'s own fields/constructor signature — `subdivide()`'s public signature (`pub fn subdivide(mesh: &Mesh, args: SubdivisionArgs) -> Result<Mesh, MeshError>`) and its own body (the per-round loop and `update_cb` invocation) are unchanged; only `split_round`'s 2-line pre-allocation, described above, touches `subdivide.rs` at all
- Any subdivision performance optimization beyond `split_round`'s pre-allocation and the inherent no-bookkeeping of `RedGreenSplit`'s stateless per-call design — e.g. no early-exit shortcut is added for a fully-converged mesh, and no strategy-specific capacity hint is introduced (see above) — a fully-converged triangle just cheaply re-measures 3 already-short edges and returns itself unchanged every round, within the pre-allocated buffer

## Domain model involved

**`planet-core/src/subdivision/min_edge_length.rs` (new):**
```rust
const DEFAULT_MIN_EDGE_LENGTH: f32 = 0.1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinEdgeLength(f32);

#[derive(Debug, Clone, PartialEq)]
pub enum MinEdgeLengthError {
    Negative { value: f32 },
}

impl MinEdgeLength {
    pub fn new(value: f32) -> Result<MinEdgeLength, MinEdgeLengthError> {
        if value < 0.0 {
            Err(MinEdgeLengthError::Negative { value })
        } else {
            Ok(MinEdgeLength(value))
        }
    }

    pub fn value(&self) -> f32 {
        self.0
    }
}

impl Default for MinEdgeLength {
    fn default() -> Self {
        MinEdgeLength(DEFAULT_MIN_EDGE_LENGTH)
    }
}
```
(`Display`/`std::error::Error` impls for `MinEdgeLengthError` follow the exact pattern `ElevationNoiseRangeError` and `StepsError` already use — omitted here for brevity, required in the implementation.)

**`planet-core/src/subdivision/split_point_variance.rs` (new):** identical shape to `MinEdgeLength` above, with `DEFAULT_SPLIT_POINT_VARIANCE: f32 = 0.1`, `SplitPointVariance(f32)`, and `SplitPointVarianceError::Negative { value: f32 }`.

**`planet-core/src/subdivision/subdivision_mode.rs` (updated):**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SubdivisionMode {
    #[default]
    UniformRedSplit,
    RadialRandomSplit {
        seed: Seed,
        elevation_noise_range: ElevationNoiseRange,
    },
    RedGreenSplit {
        seed: Seed,
        elevation_noise_range: ElevationNoiseRange,
        min_edge_length: MinEdgeLength,
        split_point_variance: SplitPointVariance,
    },
}
```

`EdgeCache` and the `SubdivisionStrategy` trait are **not modified by this feature** — no code block for them is needed. `subdivide.rs`'s private `split_round` helper gains a 2-line pre-allocation change; `subdivide()` itself is unchanged:

**`planet-core/src/subdivision/subdivide.rs` (updated — `split_round` only):**
```rust
fn split_round(mesh: &Mesh, strategy: &mut dyn SubdivisionStrategy) -> Result<Mesh, MeshError> {
    let triangle_count = mesh.triangles().len();
    let mut vertices = mesh.vertices().to_vec();
    vertices.reserve(3 * triangle_count);
    let mut edges = EdgeCache::new();
    let mut triangles = Vec::with_capacity(4 * triangle_count);
    for triangle in mesh.triangles() {
        triangles.extend(strategy.split_triangle(&mut vertices, &mut edges, *triangle));
    }
    Mesh::new(vertices, triangles)
}
```

**`planet-core/src/subdivision/strategies/red_green_split.rs` (new):**
```rust
pub(crate) const MIN_SPLIT_T: f32 = 0.05;
pub(crate) const MAX_SPLIT_T: f32 = 0.95;
const MIN_VERTEX_RADIUS: f32 = 0.05;

pub(crate) struct RedGreenSplit {
    rng: Pcg32,
    elevation_noise_range: ElevationNoiseRange,
    min_edge_length: MinEdgeLength,
    split_point_variance: SplitPointVariance,
}

impl RedGreenSplit {
    pub(crate) fn new(
        seed: Seed,
        elevation_noise_range: ElevationNoiseRange,
        min_edge_length: MinEdgeLength,
        split_point_variance: SplitPointVariance,
    ) -> RedGreenSplit {
        // seeds rng via Pcg32::seed_from_u64(seed.value())
    }
}

impl SubdivisionStrategy for RedGreenSplit {
    fn split_triangle(
        &mut self,
        vertices: &mut Vec<Vertex>,
        edges: &mut EdgeCache,
        triangle: Triangle,
    ) -> Vec<Triangle> {
        // measure each of the 3 edges directly against min_edge_length (no EdgeCache
        // involvement for edges that aren't split); for edges that are split, call the
        // existing edges.get_or_insert_with(...) exactly as RadialRandomSplit does;
        // triangulate per the red/green/leaf rules above and return the result —
        // no state is kept between calls
    }
}
```

## Function/API contracts

- `MinEdgeLength::new(value: f32) -> Result<MinEdgeLength, MinEdgeLengthError>` — **Pre:** none. **Post:** `Ok` iff `value >= 0.0` (rejects negative and `NaN`); `MinEdgeLength::value()` returns exactly `value` on success
- `SplitPointVariance::new(value: f32) -> Result<SplitPointVariance, SplitPointVarianceError>` — **Pre:** none. **Post:** `Ok` iff `value >= 0.0` (rejects negative and `NaN`); `value() == 0.0` collapses every future split point to the exact arithmetic midpoint (`t = 0.5`)
- `subdivide(mesh: &Mesh, args: SubdivisionArgs) -> Result<Mesh, MeshError>` — signature and own body **unchanged** by this feature; its private `split_round` helper gains the pre-allocation described above. `SubdivisionMode::RedGreenSplit` works with `subdivide()`'s existing round-by-round loop unmodified: each round, `RedGreenSplit::split_triangle` is called once per current triangle (exactly as for the other two modes), and it independently re-derives its answer from that triangle's current edges every time — a triangle that was a green or leaf output of a previous round is not treated specially
- `split_round(mesh: &Mesh, strategy: &mut dyn SubdivisionStrategy) -> Result<Mesh, MeshError>` (`pub(crate)`, called only by `subdivide()`) — **Pre:** unchanged. **Post (new, applies uniformly to all 3 modes):** the round's `triangles` accumulator has capacity `>= 4 * mesh.triangles().len()` before any element is pushed into it, and `vertices` has at least `3 * mesh.triangles().len()` additional capacity reserved beyond `mesh.vertices().len()` before any new vertex is pushed — so no reallocation occurs mid-round regardless of how many edges actually split. Produces byte-identical `Mesh` contents to the pre-optimization implementation for every existing `UniformRedSplit`/`RadialRandomSplit` scenario — this is a memory/timing guarantee only, never a value change
- `RedGreenSplit::new(seed: Seed, elevation_noise_range: ElevationNoiseRange, min_edge_length: MinEdgeLength, split_point_variance: SplitPointVariance) -> RedGreenSplit` — **Pre:** none (every field is already a validated/infallible value type). **Post:** deterministic for a given `seed` — identical inputs and identical sequence of `split_triangle` calls always draw the same RNG sequence and produce byte-identical output, per `constitution.md`
- `RedGreenSplit::split_triangle(&mut self, vertices: &mut Vec<Vertex>, edges: &mut EdgeCache, triangle: Triangle) -> Vec<Triangle>` (via the unchanged `SubdivisionStrategy` trait) — **Pre:** `triangle`'s 3 indices are valid into `vertices`. **Post:** stateless with respect to `triangle`'s identity — the result depends only on the current positions of `vertices[triangle.a]`, `vertices[triangle.b]`, `vertices[triangle.c]` and `min_edge_length`, never on whether this exact triangle (or any other) was previously passed to this strategy; measures all 3 edges against `min_edge_length` and returns 1, 2, 3, or 4 triangles per the red/green/leaf rules. In particular, calling it twice in a row on the same unchanged `triangle` and `vertices` (ignoring RNG advancement for edges that split) yields the same triangulation shape both times

## BDD scenarios

All scenarios use the arbitrary fixture triangle already established in `subdivide.feature` — `A = (0,0,0)`, `B = (2,0,0)`, `C = (0,2,1)` — whose 3 edges have distinct, hand-computable lengths: `|AB| = 2.0`, `|CA| = √5 ≈ 2.236`, `|BC| = 3.0`. Combined with `split_point_variance = 0.0` (collapses every split point to the exact midpoint) and `elevation_noise_range` of `(0.0, 0.0)` (no radial displacement), every triangulation case below has an exact, hand-verifiable expected output.

Core scenario set (per `rules.md`, same order as every other subdivision feature file):

  Scenario: Subdividing the icosahedron mesh by 1 step using SubdivisionMode::RedGreenSplit with a min-edge-length below the icosahedron's own edge length quadruples the triangle count
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, the default ElevationNoiseRange, a MinEdgeLength of 0.5, and a SplitPointVariance of 0.0
    Then the resulting Mesh has 80 triangles

  Scenario: Subdividing the icosahedron mesh with SubdivisionMode::RedGreenSplit does not duplicate vertices at shared edges
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, the default ElevationNoiseRange, a MinEdgeLength of 0.5, and a SplitPointVariance of 0.0
    Then the resulting Mesh has 42 vertices

  Scenario: Subdividing the icosahedron mesh with SubdivisionMode::RedGreenSplit never creates cracks between red and green triangles
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low 0.0 and high 0.0, a MinEdgeLength of 2.1, and a SplitPointVariance of 0.0
    Then no two vertices in the resulting Mesh have the same position

  Scenario: SubdivisionMode::RedGreenSplit keeps every vertex radius within the configured bound
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low -0.1 and high 0.1, a MinEdgeLength of 0.5, and a SplitPointVariance of 0.0
    Then every vertex of the resulting Mesh has a radius less than or equal to 1.1
    And every vertex of the resulting Mesh has a radius greater than or equal to 0.05

Algorithm-specific scenarios:

  Scenario: All 3 edges above the threshold produce a red split with 4 recursable children
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low 0.0 and high 0.0, a MinEdgeLength of 1.5, and a SplitPointVariance of 0.0
    Then the resulting Mesh has 4 triangles
    And the resulting Mesh has 6 vertices

  Scenario: Exactly 2 edges above the threshold produce a green split with 3 non-recursable children fanned through their two midpoints
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low 0.0 and high 0.0, a MinEdgeLength of 2.1, and a SplitPointVariance of 0.0
    Then the resulting Mesh has 3 triangles
    And the resulting Mesh has 5 vertices

  Scenario: Exactly 1 edge above the threshold produces a green split with 2 non-recursable children
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low 0.0 and high 0.0, a MinEdgeLength of 2.5, and a SplitPointVariance of 0.0
    Then the resulting Mesh has 2 triangles
    And the resulting Mesh has 4 vertices

  Scenario: No edge above the threshold produces an unchanged leaf triangle
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low 0.0 and high 0.0, a MinEdgeLength of 3.5, and a SplitPointVariance of 0.0
    Then the resulting Mesh is identical to the source Mesh

  Scenario: Subdivision naturally stops growing once every edge in the mesh is below the threshold, even if more steps are requested
    Given an icosahedron mesh
    When the mesh is subdivided with 3 steps using SubdivisionMode::RedGreenSplit with seed 7, the default ElevationNoiseRange, a MinEdgeLength of 0.35, and a SplitPointVariance of 0.0, producing the first Mesh
    And the same icosahedron mesh is subdivided with 2 steps using SubdivisionMode::RedGreenSplit with seed 7, the default ElevationNoiseRange, a MinEdgeLength of 0.35, and a SplitPointVariance of 0.0, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: SubdivisionMode::RedGreenSplit with a MinEdgeLength of 0.0 and a SplitPointVariance of 0.0 behaves like SubdivisionMode::UniformRedSplit
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low 0.0 and high 0.0, a MinEdgeLength of 0.0, and a SplitPointVariance of 0.0, producing the first Mesh
    And the same icosahedron mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: SubdivisionMode::RedGreenSplit is deterministic for a given seed
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::RedGreenSplit with seed 7, the default ElevationNoiseRange, a MinEdgeLength of 0.35, and a SplitPointVariance of 0.1, producing the first Mesh
    And the same icosahedron mesh is subdivided with 2 steps using SubdivisionMode::RedGreenSplit with seed 7, the default ElevationNoiseRange, a MinEdgeLength of 0.35, and a SplitPointVariance of 0.1, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: A non-zero SplitPointVariance moves the split point off the exact midpoint
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low 0.0 and high 0.0, a MinEdgeLength of 1.5, and a SplitPointVariance of 0.3
    Then no vertex in the resulting Mesh sits at the exact midpoint of edge 0-1

  Scenario: MinEdgeLength rejects a negative value
    When MinEdgeLength::new is called with -0.1
    Then MinEdgeLengthError::Negative is returned with value -0.1

  Scenario: SplitPointVariance rejects a negative value
    When SplitPointVariance::new is called with -0.1
    Then SplitPointVarianceError::Negative is returned with value -0.1

## Acceptance criteria

- [ ] `MinEdgeLength::new` returns `Ok` for `0.0` and every positive `f32`, and `Err(MinEdgeLengthError::Negative { value })` for every negative `f32` and for `NaN`
- [ ] `SplitPointVariance::new` returns `Ok` for `0.0` and every positive `f32`, and `Err(SplitPointVarianceError::Negative { value })` for every negative `f32` and for `NaN`
- [ ] `SubdivisionMode::RedGreenSplit` is constructible with all 4 fields and round-trips through `PartialEq`; `SubdivisionMode::default()` still equals `SubdivisionMode::UniformRedSplit`
- [ ] For the fixture triangle (edges `2.0`, `2.236`, `3.0`) with `split_point_variance = 0.0` and zero-width `elevation_noise_range`: a `min_edge_length` of `1.5` produces exactly 4 triangles and 6 vertices (red); `2.1` produces exactly 3 triangles and 5 vertices (green, 2-split); `2.5` produces exactly 2 triangles and 4 vertices (green, 1-split); `3.5` produces a `Mesh` identical to the source (leaf, 0-split)
- [ ] Subdividing the icosahedron mesh with `SubdivisionMode::RedGreenSplit`, `min_edge_length = 0.0`, `split_point_variance = 0.0`, and a zero-width `elevation_noise_range` for any number of steps ≤ `MAX_SUBDIVISION_STEPS` produces a `Mesh` identical to the same steps under `SubdivisionMode::UniformRedSplit`
- [ ] Subdividing the icosahedron mesh with `SubdivisionMode::RedGreenSplit` and a `min_edge_length` that is below the edge length at round *k* but at or above it at round *k+1* produces the same final `Mesh` whether `steps` is set to exactly *k+1* or to any larger value up to `MAX_SUBDIVISION_STEPS`
- [ ] No two vertices in any `RedGreenSplit` result share a position (no cracks), for both the icosahedron fixture and the arbitrary-triangle fixture, across all triangulation cases (red, both green cases, leaf)
- [ ] Every vertex radius in a `RedGreenSplit` result stays within `[0.05, base_radius + elevation_noise_range.high()]` for the icosahedron fixture after 1 step (i.e. `≤ 1.1` for base radius `1.0` and `elevation_noise_range.high() == 0.1`, mirroring `RadialRandomSplit`'s own radius-bound scenario)
- [ ] `RedGreenSplit` is deterministic: identical `seed`, `elevation_noise_range`, `min_edge_length`, `split_point_variance`, source `Mesh`, and step count always produce a byte-identical result `Mesh`
- [ ] A non-zero `split_point_variance` produces a split-point vertex that is not the exact arithmetic midpoint, for at least one edge in the fixture triangle
- [ ] A triangle whose 3 current edges are all below `min_edge_length` is returned unchanged by `split_triangle`, with no RNG state drawn, regardless of whether it was freshly constructed or is a green/leaf output from a previous round being passed in again — `RedGreenSplit` never consults or stores anything keyed by the triangle's identity, only by its vertices' current positions
- [ ] `EdgeCache`'s existing test coverage passes unmodified after this feature lands
- [ ] Immediately after `split_round`'s pre-allocation lines run for a round with `T` input triangles, the `triangles` accumulator's `Vec::capacity()` is `>= 4 * T` and `vertices`'s `Vec::capacity()` is `>= (pre-round vertex count) + 3 * T`, verified directly in a unit test, before any triangle in that round has been processed
- [ ] Every existing `subdivide.feature` scenario for `SubdivisionMode::UniformRedSplit` and `SubdivisionMode::RadialRandomSplit` continues to pass unmodified after this change — the pre-allocation is capacity-only and produces byte-identical `Mesh` contents for both
- [ ] `cargo test --workspace && cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo build --target wasm32-unknown-unknown -p planet-renderer` passes
