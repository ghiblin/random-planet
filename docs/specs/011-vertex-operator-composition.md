# 011 — Vertex Operator Composition

**Status:** Ready for review
**Feature slug:** `vertex-operator-composition`

This is an ad-hoc structural refactor, like `005-subdivision-facade`, `006-by-concern-file-layout`, and `008-strategies-module` before it — not a roadmap phase. User-directed: each `SubdivisionStrategy` currently computes its new edge vertex through one monolithic private function (`displaced_midpoint` / `displaced_split_point`) that inlines three distinct operations back to back: picking a point on the edge, displacing it radially, and displacing it along the edge's plane normal. This feature decomposes that into three named, independently testable steps and moves the two that are shared across strategies (radial displacement, plane-normal displacement) into `planet-core/src/processor/` as factory-produced closures, composed into a single pipeline via a `compose` combinator — true functional composition, not manual call-site sequencing — so every strategy becomes (pick-point, then one composed operator) instead of three copies of similar-looking math.

## Requirements

- Every `SubdivisionStrategy`'s "compute the new vertex for this edge" logic decomposes into exactly three steps, applied in this order:
  1. **Pick the point** — a strategy-unique function/method (stays in that strategy's own file in `subdivision/strategies/`, per explicit user direction; not moved to `processor/`, even where two strategies happen to compute it identically). `UniformRedSplit` and `RadialRandomSplit` both keep their own private `exact_midpoint(a, b) -> Vertex` (byte-identical bodies, intentionally duplicated rather than shared, since picking the point is scoped as strategy-owned by this feature's requirements). `RedGreenSplit` keeps a private `gaussian_split_point(a, b, rng, split_point_variance) -> Vertex` (the `t = 0.5 + variance * z` logic already in `displaced_split_point`, with the radial/normal math stripped out).
  2. **Radial displacement** — moves the point along its own radius (the line from the origin through the point). Shared by `RadialRandomSplit` and `RedGreenSplit`; a no-op (`identity`) for `UniformRedSplit`.
  3. **Normal (plane) displacement** — moves the point along the normal of the plane defined by the edge's two original endpoints and the origin (this is the operation your request calls "tilt"; named `normal_displacement` in code to match the existing `NormalNoiseRange` type it's parameterized by, rather than introducing a new term — flag this in review if you'd rather keep "tilt" as the literal identifier). Shared by `RadialRandomSplit` and `RedGreenSplit`; a no-op (`identity`) for `UniformRedSplit`.
- Steps 2 and 3 are produced by a **factory pattern** in `planet-core/src/processor/`: a function takes the operation's random parameters (an `ElevationNoiseRange` or `NormalNoiseRange`) and returns a closure — a `VertexOperator` — that computes the next vertex given the RNG, the edge's two original endpoints, and the vertex computed so far. A third factory, `identity()`, takes no parameters and returns a `VertexOperator` that returns its input vertex unchanged — used by `UniformRedSplit`, and available to any future strategy that wants to skip a step.
- `VertexOperator` is a single `pub(crate)` type alias, defined once: `Box<dyn Fn(&mut Pcg32, &Vertex, &Vertex, Vertex) -> Vertex>`. `Fn`, not `FnMut`: none of the closures mutate their own captured state (the ranges they close over are `Copy` and read-only); the only mutation is to the caller-supplied `&mut Pcg32`, which is a parameter, not captured state.
- Since `VertexOperator`'s input and output are the same shape (a `Vertex`, threaded through alongside the fixed `rng`/`a`/`b` context), it is closed under composition — so a **`compose` combinator** in `processor/compose.rs` takes two `VertexOperator`s and returns one new `VertexOperator` that applies the first, then the second, to the point the first produces. Each strategy holds **one** `pipeline: VertexOperator` field (not one field per step), built once at construction from `compose(radial_displacement(...), normal_displacement(...))` — genuine functional composition, not the strategy manually calling one operator and then the other at every edge call-site. `UniformRedSplit` doesn't need `compose` at all: its pipeline is just `identity()` directly. Calling the composed pipeline only ever needs `&self.pipeline` (never `&mut`), so it never conflicts with the strategy's own `&mut self.rng` borrow in the same expression (disjoint fields, standard Rust borrow-check, no destructuring needed).
- `compose`'s parameter order is left-to-right (`compose(first, second)` applies `first`, then `second`) — the opposite of the right-to-left convention implied by mathematical `f∘g` notation — so its doc comment states this explicitly to avoid ambiguity for a future reader.
- `MIN_VERTEX_RADIUS` (currently duplicated verbatim between `radial_random_split.rs` and `red_green_split.rs`) moves to `processor/radial_displacement.rs` as the operator's single implementation now needs it in exactly one place.
- `UniformRedSplit` gains a `new()` constructor (it was previously constructed as a bare unit-struct value, `UniformRedSplit`, at its one call site in `subdivision_mode.rs`) so it can hold its `identity()` pipeline as a field, keeping all three strategies structurally uniform. `UniformRedSplit` still takes no seed/config — `SubdivisionMode::UniformRedSplit` stays a fieldless variant. Its `split_triangle` needs *some* `Pcg32` value to hand to the shared `VertexOperator` call signature even though `identity` never reads it; it constructs one locally (`Pcg32::seed_from_u64(0)`), documented inline as unused, rather than adding an unused field or a fake `Seed` parameter that would misleadingly suggest `UniformRedSplit` is randomized.
- `RedGreenSplit`'s `maybe_split` free function becomes a private method (`&mut self`) so its closure can reach `self.pipeline` alongside `self.rng`; this also drops its `#[allow(clippy::too_many_arguments)]` (four config values move from being threaded through every call to being fields read once at construction).
- `rules.md`'s `processor/` concern-list bullet is updated to document the five new files and broaden the bullet's description: `processor/` now covers both whole-`Mesh` pre/post-processing steps (existing) and the per-vertex `VertexOperator` building blocks (including `compose`) `subdivision/strategies/` uses to build its pipeline (new).

Out of scope:
- Any change to `SubdivisionStrategy` (the trait), `EdgeCache`, `Steps`, `Seed`, `ElevationNoiseRange`, `NormalNoiseRange`, `MinEdgeLength`, `SplitPointVariance`, `SubdivisionArgs`, `subdivide()`, or `SubdivisionMode`'s variants/fields — no public signature anywhere changes
- `RedGreenSplit`'s `min_edge_length` gating (which edges get split at all, and the resulting red/green/leaf triangle topology) — this is a decision made *before* the three-step vertex-computation pipeline runs on a gated-in edge; it is not one of the three composed operators and its logic is untouched
- Any new randomness, noise range, or displacement behavior — this is a pure decomposition/relocation of existing math; no strategy's output distribution changes
- `vertex_scramble.rs`/`vertex_scramble_range.rs` (the existing whole-`Mesh` post-processing step) — untouched; it has its own, unrelated `MIN_VERTEX_RADIUS`-equivalent guard that stays local to it, since it operates on a already-fully-built `Mesh`, not mid-subdivision edges
- `planet-renderer` — untouched; it only calls `subdivide()`/`SubdivisionArgs`/`SubdivisionMode`, never the strategy structs or processor operators directly
- Deduplicating `exact_midpoint` between `UniformRedSplit` and `RadialRandomSplit` — explicitly kept strategy-local per this feature's own requirements above

## Domain model involved

**`planet-core/src/processor/vertex_operator.rs` (new):**
```rust
use rand_pcg::Pcg32;

use crate::geometry::mesh::Vertex;

pub(crate) type VertexOperator = Box<dyn Fn(&mut Pcg32, &Vertex, &Vertex, Vertex) -> Vertex>;
```
Signature reads as: given the RNG, the edge's two original endpoints (`a`, `b`), and the vertex computed so far, produce the next vertex. All three factories below produce values of this type.

**`planet-core/src/processor/compose.rs` (new):**
```rust
use rand_pcg::Pcg32;

use crate::geometry::mesh::Vertex;
use crate::processor::vertex_operator::VertexOperator;

// Applies `first`, then `second` — left-to-right, not the right-to-left order
// mathematical f∘g notation implies.
pub(crate) fn compose(first: VertexOperator, second: VertexOperator) -> VertexOperator {
    Box::new(move |rng: &mut Pcg32, a: &Vertex, b: &Vertex, point: Vertex| {
        // Sequential let-bindings, not `second(rng, a, b, first(rng, a, b, point))` —
        // the nested form borrows `rng` mutably twice in one expression and doesn't compile.
        let point = first(rng, a, b, point);
        second(rng, a, b, point)
    })
}
```

**`planet-core/src/processor/identity.rs` (new):**
```rust
use crate::processor::vertex_operator::VertexOperator;

pub(crate) fn identity() -> VertexOperator {
    Box::new(|_rng, _a, _b, point| point)
}
```

**`planet-core/src/processor/radial_displacement.rs` (new):**
```rust
use rand::RngExt;
use rand_pcg::Pcg32;

use crate::geometry::mesh::Vertex;
use crate::processor::vertex_operator::VertexOperator;
use crate::subdivision::elevation_noise_range::ElevationNoiseRange;

pub(crate) const MIN_VERTEX_RADIUS: f32 = 0.05;

pub(crate) fn radial_displacement(range: ElevationNoiseRange) -> VertexOperator {
    Box::new(move |rng: &mut Pcg32, _a, _b, point| {
        let radius = point.position.length();
        if radius == 0.0 {
            return point;
        }
        let delta = rng.random_range(range.low()..=range.high());
        let new_radius = (radius + delta).max(MIN_VERTEX_RADIUS);
        Vertex {
            position: point.position.scale(new_radius / radius),
        }
    })
}
```
Body is `displaced_midpoint`'s radial half, unchanged: same zero-radius guard, same single-ratio scale (preserves the existing bit-identical-when-`delta == 0.0` property), same `MIN_VERTEX_RADIUS` floor.

**`planet-core/src/processor/normal_displacement.rs` (new):**
```rust
use rand::RngExt;
use rand_pcg::Pcg32;

use crate::geometry::mesh::Vertex;
use crate::processor::vertex_operator::VertexOperator;
use crate::subdivision::normal_noise_range::NormalNoiseRange;

pub(crate) fn normal_displacement(range: NormalNoiseRange) -> VertexOperator {
    Box::new(move |rng: &mut Pcg32, a, b, point| {
        let delta = rng.random_range(range.low()..=range.high());
        match a.position.cross(b.position).normalized() {
            Some(normal) => Vertex {
                position: point.position.add(normal.scale(delta)),
            },
            None => point,
        }
    })
}
```
Body is `displaced_midpoint`'s normal half, unchanged: `delta` is still drawn unconditionally (before the `Some`/`None` match), matching current behavior of always consuming from the RNG regardless of whether the edge's cross product degenerates.

**`planet-core/src/processor.rs` (updated):**
```rust
pub(crate) mod compose;
pub(crate) mod identity;
pub(crate) mod normal_displacement;
pub(crate) mod radial_displacement;
pub(crate) mod vertex_operator;
pub mod vertex_scramble;
pub mod vertex_scramble_range;
```

**`planet-core/src/subdivision/strategies/uniform_red_split.rs` (rewritten):**
```rust
use rand::SeedableRng;
use rand_pcg::Pcg32;

use crate::geometry::mesh::{Triangle, Vertex};
use crate::processor::identity::identity;
use crate::processor::vertex_operator::VertexOperator;
use crate::subdivision::edge::EdgeCache;
use crate::subdivision::subdivide::SubdivisionStrategy;

fn exact_midpoint(a: &Vertex, b: &Vertex) -> Vertex {
    Vertex {
        position: a.position.add(b.position).scale(0.5),
    }
}

pub(crate) struct UniformRedSplit {
    pipeline: VertexOperator,
}

impl UniformRedSplit {
    pub(crate) fn new() -> UniformRedSplit {
        UniformRedSplit {
            pipeline: identity(),
        }
    }
}

impl SubdivisionStrategy for UniformRedSplit {
    fn split_triangle(
        &mut self,
        vertices: &mut Vec<Vertex>,
        edges: &mut EdgeCache,
        triangle: Triangle,
    ) -> Vec<Triangle> {
        // Unused by `identity`; only present to satisfy VertexOperator's shared call signature.
        let mut rng = Pcg32::seed_from_u64(0);
        let ab = edges.get_or_insert_with(triangle.a, triangle.b, vertices, |a, b| {
            (self.pipeline)(&mut rng, a, b, exact_midpoint(a, b))
        });
        let bc = edges.get_or_insert_with(triangle.b, triangle.c, vertices, |a, b| {
            (self.pipeline)(&mut rng, a, b, exact_midpoint(a, b))
        });
        let ca = edges.get_or_insert_with(triangle.c, triangle.a, vertices, |a, b| {
            (self.pipeline)(&mut rng, a, b, exact_midpoint(a, b))
        });

        vec![
            Triangle::new(triangle.a, ab, ca),
            Triangle::new(triangle.b, bc, ab),
            Triangle::new(triangle.c, ca, bc),
            Triangle::new(ab, bc, ca),
        ]
    }
}
```

**`planet-core/src/subdivision/strategies/radial_random_split.rs` (rewritten):**
```rust
use rand::SeedableRng;
use rand_pcg::Pcg32;

use crate::geometry::mesh::{Triangle, Vertex};
use crate::processor::compose::compose;
use crate::processor::normal_displacement::normal_displacement;
use crate::processor::radial_displacement::radial_displacement;
use crate::processor::vertex_operator::VertexOperator;
use crate::subdivision::edge::EdgeCache;
use crate::subdivision::elevation_noise_range::ElevationNoiseRange;
use crate::subdivision::normal_noise_range::NormalNoiseRange;
use crate::subdivision::seed::Seed;
use crate::subdivision::subdivide::SubdivisionStrategy;

fn exact_midpoint(a: &Vertex, b: &Vertex) -> Vertex {
    Vertex {
        position: a.position.add(b.position).scale(0.5),
    }
}

pub(crate) struct RadialRandomSplit {
    rng: Pcg32,
    pipeline: VertexOperator,
}

impl RadialRandomSplit {
    pub(crate) fn new(
        seed: Seed,
        elevation_noise_range: ElevationNoiseRange,
        normal_noise_range: NormalNoiseRange,
    ) -> RadialRandomSplit {
        RadialRandomSplit {
            rng: Pcg32::seed_from_u64(seed.value()),
            pipeline: compose(
                radial_displacement(elevation_noise_range),
                normal_displacement(normal_noise_range),
            ),
        }
    }
}

impl SubdivisionStrategy for RadialRandomSplit {
    fn split_triangle(
        &mut self,
        vertices: &mut Vec<Vertex>,
        edges: &mut EdgeCache,
        triangle: Triangle,
    ) -> Vec<Triangle> {
        let ab = edges.get_or_insert_with(triangle.a, triangle.b, vertices, |a, b| {
            (self.pipeline)(&mut self.rng, a, b, exact_midpoint(a, b))
        });
        let bc = edges.get_or_insert_with(triangle.b, triangle.c, vertices, |a, b| {
            (self.pipeline)(&mut self.rng, a, b, exact_midpoint(a, b))
        });
        let ca = edges.get_or_insert_with(triangle.c, triangle.a, vertices, |a, b| {
            (self.pipeline)(&mut self.rng, a, b, exact_midpoint(a, b))
        });

        vec![
            Triangle::new(triangle.a, ab, ca),
            Triangle::new(triangle.b, bc, ab),
            Triangle::new(triangle.c, ca, bc),
            Triangle::new(ab, bc, ca),
        ]
    }
}
```
(`MIN_VERTEX_RADIUS` no longer declared here — moved to `processor/radial_displacement.rs`.)

**`planet-core/src/subdivision/strategies/red_green_split.rs` (rewritten):**
```rust
use rand::{RngExt, SeedableRng};
use rand_distr::StandardNormal;
use rand_pcg::Pcg32;

use crate::geometry::mesh::{Triangle, Vertex};
use crate::processor::compose::compose;
use crate::processor::normal_displacement::normal_displacement;
use crate::processor::radial_displacement::radial_displacement;
use crate::processor::vertex_operator::VertexOperator;
use crate::subdivision::edge::EdgeCache;
use crate::subdivision::elevation_noise_range::ElevationNoiseRange;
use crate::subdivision::min_edge_length::MinEdgeLength;
use crate::subdivision::normal_noise_range::NormalNoiseRange;
use crate::subdivision::seed::Seed;
use crate::subdivision::split_point_variance::SplitPointVariance;
use crate::subdivision::subdivide::SubdivisionStrategy;

pub(crate) const MIN_SPLIT_T: f32 = 0.05;
pub(crate) const MAX_SPLIT_T: f32 = 0.95;

fn gaussian_split_point(
    a: &Vertex,
    b: &Vertex,
    rng: &mut Pcg32,
    split_point_variance: SplitPointVariance,
) -> Vertex {
    // Equivalent to Normal::new(0.5, split_point_variance.value()).sample(rng) — see
    // rand_distr's own Distribution<F> impl for Normal, which computes exactly
    // `mean + std_dev * StandardNormal.sample(rng)` — but without Normal::new's
    // fallible (non-finite std_dev) construction step, which production code must
    // never unwrap/expect on.
    let z: f32 = rng.sample(StandardNormal);
    let t = (0.5 + split_point_variance.value() * z).clamp(MIN_SPLIT_T, MAX_SPLIT_T);
    Vertex {
        position: a.position.add(b.position.sub(a.position).scale(t)),
    }
}

pub(crate) struct RedGreenSplit {
    rng: Pcg32,
    min_edge_length: MinEdgeLength,
    split_point_variance: SplitPointVariance,
    pipeline: VertexOperator,
}

impl RedGreenSplit {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        seed: Seed,
        elevation_noise_range: ElevationNoiseRange,
        normal_noise_range: NormalNoiseRange,
        min_edge_length: MinEdgeLength,
        split_point_variance: SplitPointVariance,
    ) -> RedGreenSplit {
        RedGreenSplit {
            rng: Pcg32::seed_from_u64(seed.value()),
            min_edge_length,
            split_point_variance,
            pipeline: compose(
                radial_displacement(elevation_noise_range),
                normal_displacement(normal_noise_range),
            ),
        }
    }

    fn maybe_split(
        &mut self,
        a: usize,
        b: usize,
        vertices: &mut Vec<Vertex>,
        edges: &mut EdgeCache,
    ) -> Option<usize> {
        let length = vertices[b].position.sub(vertices[a].position).length();
        if length < self.min_edge_length.value() {
            return None;
        }
        let split_point_variance = self.split_point_variance;
        Some(edges.get_or_insert_with(a, b, vertices, |va, vb| {
            let point = gaussian_split_point(va, vb, &mut self.rng, split_point_variance);
            (self.pipeline)(&mut self.rng, va, vb, point)
        }))
    }
}

impl SubdivisionStrategy for RedGreenSplit {
    fn split_triangle(
        &mut self,
        vertices: &mut Vec<Vertex>,
        edges: &mut EdgeCache,
        triangle: Triangle,
    ) -> Vec<Triangle> {
        let ab = self.maybe_split(triangle.a, triangle.b, vertices, edges);
        let bc = self.maybe_split(triangle.b, triangle.c, vertices, edges);
        let ca = self.maybe_split(triangle.c, triangle.a, vertices, edges);

        match (ab, bc, ca) {
            (Some(ab), Some(bc), Some(ca)) => vec![
                Triangle::new(triangle.a, ab, ca),
                Triangle::new(triangle.b, bc, ab),
                Triangle::new(triangle.c, ca, bc),
                Triangle::new(ab, bc, ca),
            ],
            (Some(ab), Some(bc), None) => vec![
                Triangle::new(ab, triangle.b, bc),
                Triangle::new(ab, bc, triangle.c),
                Triangle::new(ab, triangle.c, triangle.a),
            ],
            (None, Some(bc), Some(ca)) => vec![
                Triangle::new(bc, triangle.c, ca),
                Triangle::new(bc, ca, triangle.a),
                Triangle::new(bc, triangle.a, triangle.b),
            ],
            (Some(ab), None, Some(ca)) => vec![
                Triangle::new(ab, triangle.b, triangle.c),
                Triangle::new(ab, triangle.c, ca),
                Triangle::new(ab, ca, triangle.a),
            ],
            (Some(ab), None, None) => vec![
                Triangle::new(triangle.a, ab, triangle.c),
                Triangle::new(ab, triangle.b, triangle.c),
            ],
            (None, Some(bc), None) => vec![
                Triangle::new(triangle.b, bc, triangle.a),
                Triangle::new(bc, triangle.c, triangle.a),
            ],
            (None, None, Some(ca)) => vec![
                Triangle::new(triangle.c, ca, triangle.b),
                Triangle::new(ca, triangle.a, triangle.b),
            ],
            (None, None, None) => vec![triangle],
        }
    }
}
```
(`MIN_VERTEX_RADIUS` no longer declared here either — same single new home. `maybe_split` drops `#[allow(clippy::too_many_arguments)]`: 4 params instead of 8, since noise ranges are now baked into `self.pipeline` instead of threaded through every call.)

**`planet-core/src/subdivision/subdivision_mode.rs` (updated — one call site only):**
```rust
SubdivisionMode::UniformRedSplit => Box::new(UniformRedSplit::new()),
```
(Every other match arm, the enum definition, and all imports are unchanged.)

**`rules.md`'s `processor/` bullet (updated):**
```markdown
- `processor/` — reusable vertex- and mesh-transformation building blocks: whole-mesh
  pre/post-processing steps that run outside the subdivision algorithm, each taking
  an already-built `Mesh` and returning a transformed one (`vertex_scramble_range.rs`
  (`VertexScrambleRange`, `VertexScrambleRangeError`), `vertex_scramble.rs`
  (`scramble_vertices`)); plus the per-vertex `VertexOperator` building blocks
  `subdivision/strategies/` composes into a pipeline to compute each newly split
  vertex (`vertex_operator.rs` (`VertexOperator`, `pub(crate)`), `identity.rs`
  (`identity`, `pub(crate)`), `radial_displacement.rs` (`radial_displacement`,
  `MIN_VERTEX_RADIUS`, `pub(crate)`), `normal_displacement.rs`
  (`normal_displacement`, `pub(crate)`), `compose.rs` (`compose`, `pub(crate)`))
```

## Function/API contracts

- No `pub` function, method, struct, enum, or trait anywhere in `planet-core` changes name, signature, or visibility — `cargo doc -p planet-core --no-deps`'s public item listing is byte-identical before and after. `VertexOperator`, `identity`, `radial_displacement`, `normal_displacement`, `compose` are all new but `pub(crate)`, never part of the public surface.
- `UniformRedSplit::new() -> UniformRedSplit` is a new `pub(crate)` constructor; its one caller (`SubdivisionMode::strategy()`) is the only other file touched by this change.
- `RadialRandomSplit::new(seed, elevation_noise_range, normal_noise_range) -> RadialRandomSplit` and `RedGreenSplit::new(seed, elevation_noise_range, normal_noise_range, min_edge_length, split_point_variance) -> RedGreenSplit` keep their exact existing signatures — only their bodies change (building a single `pipeline: VertexOperator` field via `compose(radial_displacement(...), normal_displacement(...))` instead of copying the ranges directly).
- `compose(first: VertexOperator, second: VertexOperator) -> VertexOperator` is the only combinator this feature introduces; it is not variadic (no `Vec<VertexOperator>`/slice-folding form) since no strategy needs more than two non-identity operators today — a future strategy needing a third would either nest `compose(compose(a, b), c)` or, if that pattern recurs, motivate a variadic form at that time.
- `SubdivisionStrategy::split_triangle`'s signature and every strategy's implementation of it keep identical behavior for the topology/gating logic (triangle counts, red/green/leaf selection, edge-cache reuse) — only how each edge's new vertex position is computed changes internally, and only in decomposition, not formula: for any given sequence of RNG draws, `radial_displacement`/`normal_displacement`'s output for a given input is identical to the corresponding half of the old `displaced_midpoint`/`displaced_split_point`.
- No test in `planet-core/tests/**` asserts an exact hardcoded vertex position for a specific seed (confirmed by reading every scenario referencing a seed in `subdivide.feature`, `seed.feature`, and every `Then` step touching `position` in `subdivide.rs`) — every seed-dependent assertion is a property check (radius bounds, coplanarity via dot product, equality/inequality between two independently-run subdivisions, or "matches `UniformRedSplit`'s output when all noise ranges are zero-width"). This means the refactor is free to change the exact order/count of RNG draws internally as long as these properties keep holding — which they do, since each operator's internal draw pattern (unconditional `random_range` call before any branch, zero/degenerate guards before drawing) is preserved unchanged from the pre-refactor code, just relocated.

## BDD scenarios

Per `constitution.md`'s BDD rule (scenarios are reserved for domain behavior) and the precedent set by `008-strategies-module` and `010-vertex-scramble`, this is a structural refactor with zero intended behavior change, so no new `.feature` file is introduced. All required coverage already exists in `planet-core/tests/features/subdivide.feature` (38 scenarios total across the 3 `SubdivisionMode` variants):

**Happy path** (already passing, exercises pick-point → radial → normal in sequence):
```gherkin
Scenario: SubdivisionMode::RadialRandomSplit keeps every vertex radius within the configured bound
  Given an icosahedron mesh
  When the mesh is subdivided with 2 steps using SubdivisionMode::RadialRandomSplit with seed 7, an ElevationNoiseRange of low -0.1 and high 0.1, and the default NormalNoiseRange
  Then every vertex of the resulting Mesh has a radius less than or equal to 1.3
  And every vertex of the resulting Mesh has a radius greater than or equal to 0.05
```

**Boundary/edge case** (already passing, exercises the relocated radial operator's zero-guard on a degenerate edge):
```gherkin
Scenario: SubdivisionMode::RadialRandomSplit never panics when an edge's midpoint is exactly the origin
  Given a Mesh with an edge whose midpoint is the origin
  And a Triangle referencing indices 0, 1, 2
  When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7, the default ElevationNoiseRange, and the default NormalNoiseRange
  Then no panic occurs
```

All 38 scenarios in `subdivide.feature`, plus `seed.feature` (2), `elevation_noise_range.feature` (4), `normal_noise_range.feature` (4), `min_edge_length.feature` (4+), and `split_point_variance.feature` (4+), must pass unmodified in content and outcome — this refactor's correctness is proven entirely by them continuing to pass, exactly as `008`'s relocation was.

## Acceptance criteria

1. `planet-core/src/processor/` contains `compose.rs`, `identity.rs`, `normal_displacement.rs`, `radial_displacement.rs`, `vertex_operator.rs` in addition to the existing `vertex_scramble.rs`/`vertex_scramble_range.rs`; all five new files are `pub(crate)`
2. `MIN_VERTEX_RADIUS` exists in exactly one place in the entire crate (`processor/radial_displacement.rs`) — `grep -rn "MIN_VERTEX_RADIUS" planet-core/src` returns matches only in that file (plus `vertex_scramble.rs`'s own, unrelated, untouched constant of the same name/value)
3. `UniformRedSplit`, `RadialRandomSplit`, and `RedGreenSplit` each hold exactly one `pipeline: VertexOperator` field, built once in `new()` — `identity()` for `UniformRedSplit`, `compose(radial_displacement(...), normal_displacement(...))` for the other two — and each strategy's `split_triangle` (via `maybe_split` for `RedGreenSplit`) applies its strategy-unique pick-point step followed by exactly one call to `self.pipeline`, for every edge
4. `UniformRedSplit::new()` exists and is used at `subdivision_mode.rs`'s one construction site; no other `pub`/`pub(crate)` signature in `subdivision_mode.rs` changes
5. `cargo doc -p planet-core --no-deps`'s public item listing is byte-identical before and after this feature
6. All 38 scenarios in `subdivide.feature`, plus every scenario in `seed.feature`, `elevation_noise_range.feature`, `normal_noise_range.feature`, `min_edge_length.feature`, and `split_point_variance.feature`, pass unmodified in Given/When/Then text and expected outcome
7. No `planet-core/tests/**` file requires changes
8. `rules.md`'s `processor/` bullet documents all five new files and their `pub(crate)` visibility, and its description now covers both whole-`Mesh` pre/post-processing steps and per-vertex `VertexOperator` building blocks
9. `cargo test --workspace`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` all pass
10. `cargo build --target wasm32-unknown-unknown -p planet-renderer` still succeeds
11. `RedGreenSplit::maybe_split` no longer carries `#[allow(clippy::too_many_arguments)]` (down to 4 parameters plus `&mut self`)
12. No new `unwrap()`/`panic!()` in production code
