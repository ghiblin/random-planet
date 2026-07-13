# 007 — Radial Randomness

**Status:** Ready for review
**Feature slug:** `radial-randomness`

This is `docs/roadmap.md`'s own "005 — Radial randomness" phase — it becomes spec number `007` only because ad-hoc refactor specs `005-subdivision-facade` and `006-by-concern-file-layout` already claimed the intervening numbers (the same numbering drift those two specs already called out for each other). Scope matches the roadmap line exactly: random radial vertex displacement on newly created vertices during subdivision. No new stopping condition, no Gaussian split point, no red/green triangulation (`006-irregular-subdivision` on the roadmap), and no `Preset`/`PresetParams`/ocean quota (`007-planet-presets` on the roadmap, which will land as a higher-numbered spec).

## Requirements

- `planet-core` gains a new public value type `Seed` (`planet-core/src/subdivision/seed.rs`), wrapping a `u64` with a private field — mirroring `Steps`'s newtype style but with no invariant to validate (every `u64` is a valid seed), so construction is infallible via `impl From<u64> for Seed` rather than a `Steps::new`-style `Result`-returning constructor
- `planet-core` gains a new public value type `ElevationNoiseRange` (`planet-core/src/subdivision/elevation_noise_range.rs`) with a validated constructor per `rules.md`'s "constructors that validate invariants return `Result` with a dedicated `Error` type" — rejects `low > high` (which also naturally rejects a `NaN` bound, since any comparison against `NaN` is `false`) via `ElevationNoiseRangeError::InvalidRange { low, high }`. Stores `low`/`high` as two plain `f32` fields (not a `std::ops::Range<f32>`) specifically so the type stays `Copy` — `Range<f32>` deliberately does not implement `Copy`, and `SubdivisionMode` (see below) needs to remain `Copy` with this type embedded in one of its variants
- `SubdivisionMode` gains a second variant, `RadialRandomSplit { seed: Seed, elevation_noise_range: ElevationNoiseRange }`, exactly as `005-subdivision-facade` anticipated ("future strategies each add a variant here, never a new public type"). `SubdivisionMode::default()` is unaffected — still resolves to `UniformRedSplit`. Because `ElevationNoiseRange` embeds `f32` fields, `SubdivisionMode` can no longer derive `Eq` (only `PartialEq`) — `f32` has no `Eq` impl; this is the one public-API-visible trait-bound change in this feature. `Copy`, `Clone`, `Debug`, `PartialEq`, `Default` are all still derived
- A new `pub(crate)` strategy, `RadialRandomSplit` (`planet-core/src/subdivision/radial_random_split.rs`), implements `SubdivisionStrategy` exactly like `UniformRedSplit` (same edge-cache-driven triangle split into 4 children — this feature changes **where new vertices sit**, never the topology or triangle count), except each newly created midpoint vertex is displaced along its own radius (the line from the origin through that vertex) by a random delta drawn from the mode's `elevation_noise_range`, using a `rand_pcg::Pcg32` seeded once (at strategy construction, from the mode's `seed`) via `Pcg32::seed_from_u64`. The mesh's pre-existing vertices (anything already in the mesh before this round, including the icosahedron's original 12) are never touched — only vertices newly created by an edge split get displaced, matching the roadmap line verbatim ("random radial vertex displacement on **newly created** vertices")
- Determinism (`constitution.md`'s non-negotiable constraint) is preserved: for a given seed, the same input `Mesh` and the same step count always produce byte-identical output. This holds because triangle iteration order is a `Vec` traversal (insertion order, not hash-map order) and each edge's compute closure — the only place the RNG is drawn from — runs exactly once per distinct edge, on that edge's first encounter, in that deterministic order. `EdgeCache`'s own hash-map internals are irrelevant to draw order: a draw happens on cache-miss (edge-key insertion), and cache-misses occur in the fixed order triangles are visited, not in hash-map iteration order
- Displacement never produces a non-positive or degenerate radius: the computed `new_radius = (original_radius + delta).max(MIN_VERTEX_RADIUS)` for a new `pub(crate) const MIN_VERTEX_RADIUS: f32 = 0.05` in `radial_random_split.rs`. Rationale (new invariant introduced by this feature, not previously specified in `000-architecture.md`): an unclamped negative delta larger in magnitude than a vertex's radius would flip that vertex through the origin, inverting the triangles that reference it and producing a degenerate/self-intersecting mesh — clamping to a small positive floor keeps every displaced vertex on the same side of the origin as before, regardless of how the caller configures `elevation_noise_range`
- Displacement is applied as a single ratio scale — `midpoint.scale(new_radius / radius)` — rather than normalizing to a unit direction and rescaling in two steps. This is not just a style choice: `radius / radius` is exactly `1.0` for any nonzero finite `radius` under IEEE 754, whereas `(1.0 / radius) * radius` (the normalize-then-rescale path) is not guaranteed to be — it can be off by a rounding ULP. Since `new_radius == radius` whenever `delta == 0.0` (the zero-width-range case), the single-ratio form guarantees the displaced position is bit-identical to the undisplaced midpoint in that case, which the two-step form does not
- The exact-zero-length-vector edge case (a midpoint whose position is the origin itself, so `radius / radius` would be a `0.0 / 0.0` division) is guarded directly — checking `radius == 0.0` before dividing — and returns the undisplaced midpoint unchanged, never panicking, mirroring the same zero-length case `Vec3::normalized` itself guards against (it just isn't called here, so the check is inlined rather than reached through `Option`). This case cannot actually arise from any edge of a mesh built from `Mesh::icosahedron()` or `Mesh::cube()` (no edge's two endpoints are antipodal through the origin), but the strategy must not assume that and must handle it defensively
- `planet-renderer`'s `App::resumed` switches its one `subdivide` call from the implicit default `SubdivisionMode` to an explicit `SubdivisionMode::RadialRandomSplit` with a new `const DEMO_SEED: u64 = 42;` and the default `ElevationNoiseRange`, so the rendered planet visibly shows the bumpy, irregular surface this feature produces instead of a perfectly smooth subdivided icosahedron. This is a temporary, hardcoded demonstration wire-up — no seed input or noise-range control is added to the UI (`000-architecture.md`: "No seed input exposed in the UI — regenerate re-seeds internally"; the "regenerate" mechanism itself is not part of this feature, see Out of scope)

Out of scope:
- Any stopping condition, Gaussian split-point placement, or red/green triangulation (`006-irregular-subdivision` on the roadmap) — every edge is still split every round, exactly as `UniformRedSplit` does today
- `Preset`, `PresetParams`, `ColorGradient`, ocean quota, or the `Planet` aggregate root (`007-planet-presets` on the roadmap) — `elevation_noise_range` here is a standalone, directly-constructed `ElevationNoiseRange`, not a field pulled from a preset
- A "regenerate" control, a non-deterministic/OS-entropy-seeded `Seed` (e.g. a `Seed::random()` using `rand::rng()`/`getrandom`), or any UI control for seed or noise range — `app.rs` uses one hardcoded demo seed and the default range; picking a fresh seed per session is deferred to whichever future phase adds the "regenerate" concept `000-architecture.md` alludes to
- Changing `Mesh`, `MeshError`, `Vec3`, `Triangle`, `Vertex`, `EdgeCache`/`EdgeKey`, `Steps`, `SubdivisionArgs`, or `subdivide`'s own signature/loop — this feature only adds a new `SubdivisionMode` variant and its backing strategy
- Any change to `UniformRedSplit`'s behavior or its existing scenario coverage (face counts, exact-midpoint math, radius ≤ 1.0 bound) — it is untouched by this feature
- A depth/step-count or noise-range UI slider — still deferred to `007-planet-presets`, as `005-subdivision-facade` already scoped out

## Domain model involved

**`planet-core/src/subdivision/seed.rs` (new):**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Seed(u64);

impl Seed {
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl From<u64> for Seed {
    fn from(value: u64) -> Seed {
        Seed(value)
    }
}
```
`Seed::default()` returns `Seed(0)` (derived — `u64::default()` is `0`). Construction is `Seed::from(value)` or `value.into()` — no inherent `new` (would be a redundant second way to do the same infallible conversion `From` already covers).

**`planet-core/src/subdivision/elevation_noise_range.rs` (new):**
```rust
const DEFAULT_ELEVATION_NOISE_LOW: f32 = -0.05;
const DEFAULT_ELEVATION_NOISE_HIGH: f32 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElevationNoiseRange {
    low: f32,
    high: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElevationNoiseRangeError {
    InvalidRange { low: f32, high: f32 },
}
// Display/std::error::Error impls mirror MeshError/StepsError's style.

impl ElevationNoiseRange {
    pub fn new(low: f32, high: f32) -> Result<ElevationNoiseRange, ElevationNoiseRangeError> {
        if low <= high {
            Ok(ElevationNoiseRange { low, high })
        } else {
            Err(ElevationNoiseRangeError::InvalidRange { low, high })
        }
    }

    pub fn low(&self) -> f32 { self.low }
    pub fn high(&self) -> f32 { self.high }
}

impl Default for ElevationNoiseRange {
    fn default() -> Self {
        ElevationNoiseRange { low: DEFAULT_ELEVATION_NOISE_LOW, high: DEFAULT_ELEVATION_NOISE_HIGH }
    }
}
```

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
}

impl SubdivisionMode {
    pub(crate) fn strategy(&self) -> Box<dyn SubdivisionStrategy> {
        match self {
            SubdivisionMode::UniformRedSplit => Box::new(UniformRedSplit),
            SubdivisionMode::RadialRandomSplit { seed, elevation_noise_range } => {
                Box::new(RadialRandomSplit::new(*seed, *elevation_noise_range))
            }
        }
    }
}
```
(`Eq` dropped from the derive list — see Requirements.)

**`planet-core/src/subdivision/radial_random_split.rs` (new):**
```rust
use rand::{RngExt, SeedableRng};
use rand_pcg::Pcg32;

use super::edge::EdgeCache;
use super::elevation_noise_range::ElevationNoiseRange;
use super::seed::Seed;
use super::subdivide::SubdivisionStrategy;
use crate::geometry::mesh::{Triangle, Vertex};

pub(crate) const MIN_VERTEX_RADIUS: f32 = 0.05;

fn displaced_midpoint(a: &Vertex, b: &Vertex, rng: &mut Pcg32, range: ElevationNoiseRange) -> Vertex {
    let midpoint = a.position.add(b.position).scale(0.5);
    let radius = midpoint.length();
    if radius == 0.0 {
        return Vertex { position: midpoint };
    }
    let delta = rng.random_range(range.low()..=range.high());
    let new_radius = (radius + delta).max(MIN_VERTEX_RADIUS);
    Vertex { position: midpoint.scale(new_radius / radius) }
}

pub(crate) struct RadialRandomSplit {
    rng: Pcg32,
    elevation_noise_range: ElevationNoiseRange,
}

impl RadialRandomSplit {
    pub(crate) fn new(seed: Seed, elevation_noise_range: ElevationNoiseRange) -> RadialRandomSplit {
        RadialRandomSplit {
            rng: Pcg32::seed_from_u64(seed.value()),
            elevation_noise_range,
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
        let range = self.elevation_noise_range;
        let ab = edges.get_or_insert_with(triangle.a, triangle.b, vertices, |a, b| {
            displaced_midpoint(a, b, &mut self.rng, range)
        });
        let bc = edges.get_or_insert_with(triangle.b, triangle.c, vertices, |a, b| {
            displaced_midpoint(a, b, &mut self.rng, range)
        });
        let ca = edges.get_or_insert_with(triangle.c, triangle.a, vertices, |a, b| {
            displaced_midpoint(a, b, &mut self.rng, range)
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

**`planet-core/src/subdivision.rs` (updated):**
```rust
mod edge;
pub mod elevation_noise_range;
mod radial_random_split;
pub mod seed;
pub mod steps;
pub mod subdivide;
pub mod subdivision_args;
pub mod subdivision_mode;
mod uniform_red_split;
```
(`radial_random_split` stays a private `mod` declaration, unreachable outside `subdivision` and its descendants — its only consumer, `subdivision_mode.rs`, already lives inside that subtree, exactly like `uniform_red_split` today.)

**`planet-core/Cargo.toml` (updated):**
- Add `[[test]] name = "seed" harness = false` and `[[test]] name = "elevation_noise_range" harness = false`
- No dependency changes — `rand` and `rand_pcg` are already present (added ahead of this feature, per `tech-stack.md`)

**`planet-renderer/src/app.rs` (updated):**
- Imports: add `use planet_core::subdivision::elevation_noise_range::ElevationNoiseRange;`, `use planet_core::subdivision::seed::Seed;`
- New `const DEMO_SEED: u64 = 42;`
- `resumed()`: the `SubdivisionArgs::new(None, None, Some(update_cb))` call becomes `SubdivisionArgs::new(None, Some(SubdivisionMode::RadialRandomSplit { seed: Seed::from(DEMO_SEED), elevation_noise_range: ElevationNoiseRange::default() }), Some(update_cb))`

No changes to `camera.rs`, `gpu/*`, `shader.wgsl`, `Mesh`, `Vec3`, `Triangle`, `Vertex`, `EdgeCache`, `Steps`, `SubdivisionArgs`, or `subdivide`'s loop.

## Function/API contracts

- `Seed::from(value)` (equivalently `value.into()`) never fails and never panics — every `u64` is a valid seed; `Seed::default()` equals `Seed::from(0)`; `Seed::value()` returns the wrapped `u64` unchanged
- `ElevationNoiseRange::new(low, high)`:
  - `low <= high` (including `low == high`, a zero-width range) returns `Ok(ElevationNoiseRange { low, high })`
  - `low > high`, or either bound is `NaN`, returns `Err(ElevationNoiseRangeError::InvalidRange { low, high })` and constructs nothing
  - `ElevationNoiseRange::default()` equals `ElevationNoiseRange::new(-0.05, 0.05).unwrap()`
- `SubdivisionMode::RadialRandomSplit { seed, elevation_noise_range }` is a `pub` enum variant; `SubdivisionMode::default()` is unaffected (`UniformRedSplit`); `SubdivisionMode` derives `Copy`, `Clone`, `Debug`, `PartialEq`, `Default` but **not** `Eq` (new, `f32`-driven restriction)
- `RadialRandomSplit` (the strategy struct) and `MIN_VERTEX_RADIUS` are `pub(crate)` — unreachable from outside `planet-core` (verified via `cargo doc -p planet-core --no-deps`)
- `subdivide(mesh, args)` with `args.mode() == SubdivisionMode::RadialRandomSplit { seed, elevation_noise_range }`:
  - Produces the same triangle/vertex counts as `UniformRedSplit` would for the same input and step count — topology is unaffected by this feature
  - Is deterministic: two calls with identical `mesh`, `steps`, `seed`, and `elevation_noise_range` produce byte-identical `Mesh`es
  - Leaves every vertex present in `mesh` before that round completely unchanged (position, index) — only vertices created by this round's edge splits are displaced
  - Produces no vertex with radius `< MIN_VERTEX_RADIUS` (`0.05`) at any step count — the per-round clamp re-applies every round, so this floor never compounds downward
  - **Superseded by `016-length-relative-displacement-noise.md`:** this bound was originally `radius > 1.0 + steps * elevation_noise_range.high()`, compounding linearly and unboundedly with step count, because `elevation_noise_range.high()` was sampled as a fixed absolute magnitude every round regardless of the edge being split. `016` changed the sampled delta to `edge_length * elevation_noise_range.high()` (a fraction of the current edge's length, not an absolute magnitude), so a lineage's per-round contribution now shrinks along with the edge lengths subdivision itself produces, rather than staying constant. The bound is now `1.0 + L0 * (eh + nh) / (1 - (0.5 + eh + nh))` — a finite limit as `steps → ∞` (for base icosahedron edge length `L0` and per-round highs `eh`/`nh`, whenever `eh + nh < 0.5`, true of every current `Preset`) — instead of growing linearly with `steps`.
  - With `elevation_noise_range` fixed at `ElevationNoiseRange::new(0.0, 0.0).unwrap()`, produces a `Mesh` identical to what `SubdivisionMode::UniformRedSplit` would produce for the same input and step count (zero-width range at zero ⇒ no actual displacement)
  - Different `seed` values (same non-zero-width range) generically produce different vertex positions (not asserted for every possible pair, but demonstrated for at least one concrete seed pair in the BDD coverage below)
- `RadialRandomSplit::split_triangle` never panics for any valid `Triangle`/`Vertex` input, including a degenerate edge whose midpoint is exactly the origin (guarded by an explicit `radius == 0.0` check before dividing, returning the undisplaced midpoint)
- `planet-renderer`'s `App::resumed` passes an explicit `SubdivisionMode::RadialRandomSplit` (not the default) to its one `subdivide` call

## BDD scenarios

`planet-core/tests/features/seed.feature`:
```gherkin
Feature: Constructing a Seed

  Scenario: Constructing a Seed from a u64 value
    When a Seed is constructed with value 42
    Then the Seed has value 42

  Scenario: The default Seed value is 0
    Given the default Seed
    Then the Seed has value 0
```

`planet-core/tests/features/elevation_noise_range.feature`:
```gherkin
Feature: Constructing a validated ElevationNoiseRange

  Scenario: Constructing an ElevationNoiseRange with low less than high succeeds
    When an ElevationNoiseRange is constructed with low -0.1 and high 0.2
    Then the ElevationNoiseRange is constructed successfully
    And the ElevationNoiseRange has low -0.1
    And the ElevationNoiseRange has high 0.2

  Scenario: Constructing an ElevationNoiseRange with equal low and high succeeds
    When an ElevationNoiseRange is constructed with low 0.0 and high 0.0
    Then the ElevationNoiseRange is constructed successfully

  Scenario: Constructing an ElevationNoiseRange with low greater than high fails
    When an ElevationNoiseRange is constructed with low 0.5 and high 0.1
    Then the construction fails with an invalid-range error of low 0.5 and high 0.1

  Scenario: The default ElevationNoiseRange has low -0.05 and high 0.05
    Given the default ElevationNoiseRange
    Then the ElevationNoiseRange has low -0.05
    And the ElevationNoiseRange has high 0.05
```

`planet-core/tests/features/subdivide.feature` (new scenarios appended, existing `UniformRedSplit` scenarios unmodified — per `rules.md`'s "every subdivision-related feature file carries the same core scenario set, in this order: face-count growth, no duplicate vertices, no cracks, vertex radii within bounds"):
```gherkin
  Scenario: Subdividing the icosahedron mesh by 1 step using SubdivisionMode::RadialRandomSplit quadruples the triangle count
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7 and the default ElevationNoiseRange
    Then the resulting Mesh has 80 triangles

  Scenario: Subdividing the icosahedron mesh with SubdivisionMode::RadialRandomSplit does not duplicate vertices at shared edges
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7 and the default ElevationNoiseRange
    Then the resulting Mesh has 42 vertices

  Scenario: Subdividing the icosahedron mesh with SubdivisionMode::RadialRandomSplit never creates cracks between adjacent triangles
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::RadialRandomSplit with seed 7 and the default ElevationNoiseRange
    Then no two vertices in the resulting Mesh have the same position

  Scenario: Subdividing the icosahedron mesh with SubdivisionMode::RadialRandomSplit keeps every vertex radius within the configured bound
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::RadialRandomSplit with seed 7 and an ElevationNoiseRange of low -0.1 and high 0.1
    Then every vertex of the resulting Mesh has a radius less than or equal to 1.2
    And every vertex of the resulting Mesh has a radius greater than or equal to 0.05

  Scenario: Subdividing with 0 steps using SubdivisionMode::RadialRandomSplit leaves the mesh unchanged
    Given an icosahedron mesh
    When the mesh is subdivided with 0 steps using SubdivisionMode::RadialRandomSplit with seed 7 and the default ElevationNoiseRange
    Then the resulting Mesh is identical to the icosahedron mesh

  Scenario: SubdivisionMode::RadialRandomSplit never displaces the mesh's original vertices
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7 and an ElevationNoiseRange of low -0.1 and high 0.1
    Then the first 12 vertices of the resulting Mesh have the same positions as the icosahedron mesh's vertices

  Scenario: SubdivisionMode::RadialRandomSplit is deterministic for a given seed
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::RadialRandomSplit with seed 7 and the default ElevationNoiseRange, producing the first Mesh
    And the same icosahedron mesh is subdivided with 2 steps using SubdivisionMode::RadialRandomSplit with seed 7 and the default ElevationNoiseRange, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: SubdivisionMode::RadialRandomSplit with different seeds produces different vertex positions
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7 and an ElevationNoiseRange of low -0.1 and high 0.1, producing the first Mesh
    And the same icosahedron mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 99 and an ElevationNoiseRange of low -0.1 and high 0.1, producing the second Mesh
    Then the first Mesh and the second Mesh are not identical

  Scenario: SubdivisionMode::RadialRandomSplit with a zero-width ElevationNoiseRange at zero behaves like SubdivisionMode::UniformRedSplit
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7 and an ElevationNoiseRange of low 0.0 and high 0.0, producing the first Mesh
    And the same icosahedron mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit, producing the second Mesh
    Then the first Mesh and the second Mesh are identical
```

## Acceptance criteria

1. `Seed::from(value).value() == value` for any `u64`; `Seed::default() == Seed::from(0)`
2. `ElevationNoiseRange::new(low, high)` succeeds iff `low <= high` (including equal bounds), otherwise returns `Err(ElevationNoiseRangeError::InvalidRange { low, high })`; `ElevationNoiseRange::default()` has `low == -0.05` and `high == 0.05`
3. `SubdivisionMode` has exactly two variants (`UniformRedSplit`, `RadialRandomSplit { seed, elevation_noise_range }`), still implements `Default` resolving to `UniformRedSplit`, and derives `Copy`/`Clone`/`Debug`/`PartialEq`/`Default` but not `Eq`
4. `RadialRandomSplit` (the strategy) and `MIN_VERTEX_RADIUS` are absent from `cargo doc -p planet-core --no-deps`'s output; `Seed`, `ElevationNoiseRange`, `ElevationNoiseRangeError`, and `SubdivisionMode::RadialRandomSplit` are present in it
5. `subdivide(&icosahedron, SubdivisionArgs::new(Some(Steps::new(1).unwrap()), Some(SubdivisionMode::RadialRandomSplit { seed: Seed::from(7), elevation_noise_range: ElevationNoiseRange::default() }), None))` produces exactly 80 triangles and 42 vertices; 2 steps produces exactly 320 triangles — identical counts to `UniformRedSplit` at the same step count
6. No two vertices in any `RadialRandomSplit` output share the same position, at any tested step count `>= 1`
7. **Superseded by `016-length-relative-displacement-noise.md`:** this criterion originally read "For an `ElevationNoiseRange` of low `-0.1`/high `0.1`, every vertex's radius after 2 steps is within `[0.05, 1.2]` (upper bound is `1.0 + steps * high`, not `1.0 + high`)" — that linear-in-`steps` upper bound no longer holds. Displacement is now scaled by the current edge's length, so the upper bound converges to a finite limit instead of growing with `steps` (see `016`'s Function/API contracts and `subdivide.feature`'s current `RadialRandomSplit`/`RedGreenSplit` radius-bound scenarios for the up-to-date figures)
8. Two `subdivide` calls with identical mesh, steps, seed, and `elevation_noise_range` produce byte-identical `Mesh`es (`PartialEq` equal); two calls differing only in seed (non-zero-width range) produce non-identical `Mesh`es
9. `subdivide` with `ElevationNoiseRange::new(0.0, 0.0).unwrap()` under `RadialRandomSplit` produces a `Mesh` identical to the same input/steps under `UniformRedSplit`
10. The icosahedron mesh's first 12 vertices are unchanged (same positions, same indices) after any number of `RadialRandomSplit` subdivision rounds `>= 1`
11. `RadialRandomSplit::split_triangle` never panics, including for a triangle whose edge midpoint lands exactly on the origin
12. `planet-renderer/src/app.rs` passes `SubdivisionMode::RadialRandomSplit { seed: Seed::from(DEMO_SEED), elevation_noise_range: ElevationNoiseRange::default() }` to its one `subdivide` call, not the default mode
13. On loading the app in-browser, the rendered planet's surface is visibly irregular/bumpy rather than a perfectly smooth subdivided icosahedron (manual/in-browser check, per `000-architecture.md`'s exemption for GPU/DOM wiring)
14. All scenarios in `seed.feature`, `elevation_noise_range.feature`, and the appended `subdivide.feature` scenarios pass via real `cucumber` step definitions — no undefined/stub steps
15. `cargo test --workspace`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` all pass
16. `cargo build --target wasm32-unknown-unknown -p planet-renderer` still succeeds
17. No new `unwrap()`/`panic!()` in production code outside tests
18. Existing `vec3.feature`, `mesh.feature`, `icosahedron.feature`, `steps.feature`, `subdivision_args.feature`, `camera.feature`, `buffers.feature`, `uniforms.feature`, `mesh_render_vertices.feature`, `mesh_render_indices.feature`, and `mesh_render_line_indices.feature` scenarios, and `subdivide.feature`'s pre-existing `UniformRedSplit` scenarios, still pass unmodified
