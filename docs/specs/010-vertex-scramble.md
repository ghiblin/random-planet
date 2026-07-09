# 010 — Vertex Scramble

**Status:** Ready for review
**Feature slug:** `vertex-scramble`

This is an ad-hoc fix, not a `docs/roadmap.md` phase. Today, `RadialRandomSplit` and `RedGreenSplit` only displace vertices *newly created* by an edge split (`009-irregular-subdivision.md`, `007-radial-randomness.md` — both explicitly scope out the mesh's pre-existing vertices, "including the icosahedron's original 12"). That means the base mesh's own vertices — the icosahedron's 12 corners — always sit at their exact pristine construction positions, no matter how much noise is configured. At typical subdivision depths this reads visually as the icosahedron's edges/symmetry still faintly visible under the randomized surface. This feature adds an optional, standalone pre-processing step that jitters a mesh's existing vertices along all three axes (not just radially) before subdivision ever runs, so the base shape itself loses its perfect symmetry.

## Requirements

- `planet-core` gains a new top-level concern, `processor/` (sibling to `geometry/` and `subdivision/`, declared via `planet-core/src/processor.rs` per `rules.md`'s "every module lives under a concern subdirectory" rule), dedicated to pre/post-processing steps that run outside the subdivision algorithm itself — functions that take an already-built `Mesh` and return a transformed one, rather than participating in the recursive split. This feature is the first occupant; `000-architecture.md`'s ocean-quota flattening (a future post-processing step, `007-planet-presets` on the roadmap) is expected to land here too, alongside subdivision's own pre-processing step added by this feature
- `planet-core` gains a new public value type `VertexScrambleRange` (`planet-core/src/processor/vertex_scramble_range.rs`): two plain `f32` fields (`low`, `high`, not a `std::ops::Range<f32>`, to stay `Copy`) — the range each per-axis multiplicative factor offset is drawn from (see the next bullet). The validated `new(low, high) -> Result<VertexScrambleRange, VertexScrambleRangeError>` constructor rejects two distinct cases: `low > high` (`VertexScrambleRangeError::InvalidRange { low, high }`, the same rule `ElevationNoiseRange` uses) and `low <= -1.0` (`VertexScrambleRangeError::LowAtOrBelowNegativeOne { low }`, new — see rationale below). Accessors `low()`/`high()`; `Default` = `(-0.05, 0.05)` (unchanged numerically, now read as "each axis's scale factor varies by up to ±5%" rather than as a position delta — this is still a fresh, independent type, not a reuse of `ElevationNoiseRange`)
- `planet-core` gains a new public pure function `scramble_vertices(mesh: &Mesh, seed: Seed, range: VertexScrambleRange) -> Result<Mesh, MeshError>` (`planet-core/src/processor/vertex_scramble.rs`), implementing the feature request's own formula. For every vertex currently in `mesh`, in order, it draws three independent random factor offsets — `a`, `b`, `c`, one per axis — from `range.low()..=range.high()`, using a `rand_pcg::Pcg32` seeded once (via `Pcg32::seed_from_u64(seed.value())`) at the start of the call. Each coordinate is transformed **multiplicatively**, scaled around its own current value rather than slid along a line through the origin: `x' = (1 + a) * x`, `y' = (1 + b) * y`, `z' = (1 + c) * z` — i.e. `V' = [(1+a)x, (1+b)y, (1+c)z]`. This is a deliberately different displacement shape from `RadialRandomSplit`/`RedGreenSplit`'s radial-only noise (`midpoint.scale(new_radius / radius)`): scrambling perturbs each axis independently around the vertex's own position, so it also breaks the icosahedron's *angular* symmetry, not just vertex radii
- A purely multiplicative transform cannot move a coordinate that is already exactly `0.0` (`(1+a) * 0.0 == 0.0` for any `a`) — and every one of `Mesh::icosahedron()`'s 12 vertices has exactly one coordinate equal to `0.0` (the classic three-golden-rectangles construction: `(-1, φ, 0)`, `(0, 1, φ)`, `(φ, 0, -1)`, etc. — each sits on one of the xy/yz/xz coordinate planes). Left unhandled, this would leave the icosahedron's three mirror-symmetry planes completely unbroken, defeating the point of this feature. To handle this, a coordinate that is exactly `0.0` is nudged **additively** instead, reusing that same axis's random draw as the delta directly — `x' = a` when `x == 0.0`, rather than `x' = (1+a) * x`. This keeps exactly 3 random draws per vertex (no extra RNG calls) and keeps the nudge's magnitude comparable to the multiplicative branch's typical effect on this mesh's unit-ish scale
- `VertexScrambleRange::new` rejects `low <= -1.0` because a factor offset of exactly `-1.0` collapses that axis's coordinate to exactly `0.0` regardless of its original value (`(1 + (-1.0)) * x == 0.0`), and any offset below `-1.0` flips the coordinate's sign (mirrors it through `0.0`) — both degrade "scramble" into either a collapse onto a plane or a reflection, neither of which is the intended small perturbation. Restricting `low > -1.0` guarantees every multiplicative factor `(1 + a)` stays strictly positive, so no axis is ever collapsed or mirrored by construction
- `scramble_vertices` never produces a vertex with a radius below the same `MIN_VERTEX_RADIUS: f32 = 0.05` floor already used by `RadialRandomSplit`/`RedGreenSplit` (`radial_random_split.rs`, `red_green_split.rs`) — mirroring their existing pattern of an independently-declared, module-local `const MIN_VERTEX_RADIUS` (no shared/extracted constant; this feature does not refactor that pre-existing duplication). After all three coordinates are transformed (multiplicatively or additively, per the rule above), if the resulting vector's length is `< MIN_VERTEX_RADIUS`, it is rescaled up to exactly `MIN_VERTEX_RADIUS` along its own (already-transformed) direction — `jittered.scale(MIN_VERTEX_RADIUS / radius)` — same single-ratio-scale technique `007-radial-randomness.md` specifies. If the resulting vector's length is exactly `0.0` (only reachable, given `low > -1.0`, if the input vertex was already at the origin on all three axes — never true for `Mesh::icosahedron()` or `Mesh::cube()`'s own vertices, since a multiplicative-branch coordinate can never reach exactly `0.0`), the position is returned unchanged (guarded before the divide), mirroring the existing zero-radius guard in both sibling strategies
- `scramble_vertices` is topology-preserving: it returns a `Mesh` with exactly the same vertex count and the exact same `triangles` list as the input (only positions change) — reusing `Mesh::new(scrambled_vertices, mesh.triangles().to_vec())`. Since triangle vertex *indices* never change and no vertex is duplicated or removed, two triangles that shared an edge before scrambling still reference the identical two vertex indices afterward — scrambling cannot introduce a crack, by construction, with no additional edge-cache bookkeeping needed
- `scramble_vertices` is not icosahedron-specific — it operates on whatever vertices are present in the `Mesh` it's given, exactly like `subdivide` itself. It is the **caller's** responsibility to invoke it on a mesh's pristine base vertices before any subdivision round runs, if the goal is "scramble the initial vertices"; the function itself has no notion of "initial" vs. "generated" vertices
- Determinism (`constitution.md`'s non-negotiable constraint) holds: identical `mesh`, `seed`, and `range` always produce a byte-identical output `Mesh` — vertices are visited in `Vec` order (never hash-map order) and the RNG is drawn from exactly once per vertex, in that fixed order
- This is a **standalone pre-processing step**, deliberately kept out of `subdivision/` and not threaded through `SubdivisionArgs`/`SubdivisionMode`/`subdivide()` at all — mirroring `000-architecture.md`'s existing precedent for ocean-quota flattening ("a post-processing step... keeps `subdivide.rs` fully preset-agnostic"). Scrambling is the pre-processing mirror of that same idea, which is exactly why it belongs in the new `processor/` concern rather than `subdivision/`: the caller calls `scramble_vertices` on its base mesh first, then passes the *result* into `subdivide` as an ordinary mesh. `SubdivisionArgs`, `SubdivisionMode`, and `subdivide`'s own signature and loop are entirely untouched by this feature — zero risk to their existing behavior or test coverage
- `rules.md`'s "Module structure" section is updated to document `planet-core`'s new `processor/` concern, listing `vertex_scramble_range.rs` (`VertexScrambleRange`) and `vertex_scramble.rs` (`scramble_vertices`), matching how `geometry/` and `subdivision/` are already documented
- `planet-renderer`'s `App::resumed` calls `scramble_vertices(&base_mesh, Seed::from(DEMO_SCRAMBLE_SEED), VertexScrambleRange::default())` (new `const DEMO_SCRAMBLE_SEED: u64 = 43;` — deliberately a different constant from the existing `DEMO_SEED: u64 = 42` used by `SubdivisionMode::RadialRandomSplit`/`RedGreenSplit`, so the two random streams are independent) once, right after constructing `base_mesh`, and uses the *scrambled* mesh both as the first frame pushed into `collected_frames` and as the `mesh` argument passed to `subdivide`. Using the scrambled mesh for frame 0 too (not just for the `subdivide` call) matters: otherwise the very first rendered frame would still show the pristine icosahedron and only "pop" into the scrambled/subdivided shape on the first animation step

Out of scope:
- Any change to `SubdivisionArgs`, `SubdivisionMode`, `subdivide`, `EdgeCache`, `RadialRandomSplit`, `RedGreenSplit`, or `UniformRedSplit` — this feature adds one new standalone function and one new value type, nothing else in the subdivision pipeline changes; `subdivision/` gains no new files at all, since both new files live under the new `processor/` concern instead
- Extracting/sharing the duplicated `MIN_VERTEX_RADIUS` constant across `radial_random_split.rs`/`red_green_split.rs`/this feature's new file — pre-existing duplication, not this feature's to fix
- A UI control (checkbox, slider) for enabling/disabling or tuning the scramble step — like `007-radial-randomness.md` before it, this wires a single hardcoded demo call in `app.rs`; exposing it as a user-facing toggle is deferred to whichever future phase adds preset/UI controls (`007-planet-presets` on the roadmap)
- Reprojecting or renormalizing scrambled vertices back onto a sphere, or preserving each vertex's original radius in any way — scrambling is intentionally the feature request's own per-axis multiplicative perturbation (`V' = [(1+a)x, (1+b)y, (1+c)z]`), not a variant of the existing radial-only noise
- A separate, independently-tunable magnitude for the zero-coordinate additive fallback — it deliberately reuses that axis's existing `VertexScrambleRange` draw, not a new parameter
- Any change to `Mesh`, `MeshError`, `Vec3`, `Triangle`, `Vertex`, `Seed`, or `ElevationNoiseRange` — `VertexScrambleRange` is a new, independent type, not a reuse or modification of `ElevationNoiseRange`

## Domain model involved

**`planet-core/src/processor/vertex_scramble_range.rs` (new):**
```rust
const DEFAULT_VERTEX_SCRAMBLE_LOW: f32 = -0.05;
const DEFAULT_VERTEX_SCRAMBLE_HIGH: f32 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VertexScrambleRange {
    low: f32,
    high: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VertexScrambleRangeError {
    InvalidRange { low: f32, high: f32 },
    LowAtOrBelowNegativeOne { low: f32 },
}
// Display/std::error::Error impls mirror ElevationNoiseRangeError's style.

impl VertexScrambleRange {
    pub fn new(low: f32, high: f32) -> Result<VertexScrambleRange, VertexScrambleRangeError> {
        if low > -1.0 && low <= high {
            Ok(VertexScrambleRange { low, high })
        } else if low <= -1.0 {
            Err(VertexScrambleRangeError::LowAtOrBelowNegativeOne { low })
        } else {
            Err(VertexScrambleRangeError::InvalidRange { low, high })
        }
    }

    pub fn low(&self) -> f32 { self.low }
    pub fn high(&self) -> f32 { self.high }
}

impl Default for VertexScrambleRange {
    fn default() -> Self {
        VertexScrambleRange {
            low: DEFAULT_VERTEX_SCRAMBLE_LOW,
            high: DEFAULT_VERTEX_SCRAMBLE_HIGH,
        }
    }
}
```

**`planet-core/src/processor/vertex_scramble.rs` (new):**
```rust
use rand::{RngExt, SeedableRng};
use rand_pcg::Pcg32;

use super::vertex_scramble_range::VertexScrambleRange;
use crate::geometry::mesh::{Mesh, MeshError, Vertex};
use crate::geometry::vec3::Vec3;
use crate::subdivision::seed::Seed;

const MIN_VERTEX_RADIUS: f32 = 0.05;

fn scrambled_component(component: f32, factor_offset: f32) -> f32 {
    if component == 0.0 {
        factor_offset
    } else {
        component * (1.0 + factor_offset)
    }
}

fn scrambled(vertex: &Vertex, rng: &mut Pcg32, range: VertexScrambleRange) -> Vertex {
    let a = rng.random_range(range.low()..=range.high());
    let b = rng.random_range(range.low()..=range.high());
    let c = rng.random_range(range.low()..=range.high());
    let position = vertex.position;
    let jittered = Vec3::new(
        scrambled_component(position.x, a),
        scrambled_component(position.y, b),
        scrambled_component(position.z, c),
    );
    let radius = jittered.length();
    if radius == 0.0 {
        return Vertex { position: jittered };
    }
    if radius < MIN_VERTEX_RADIUS {
        return Vertex { position: jittered.scale(MIN_VERTEX_RADIUS / radius) };
    }
    Vertex { position: jittered }
}

pub fn scramble_vertices(
    mesh: &Mesh,
    seed: Seed,
    range: VertexScrambleRange,
) -> Result<Mesh, MeshError> {
    let mut rng = Pcg32::seed_from_u64(seed.value());
    let vertices = mesh
        .vertices()
        .iter()
        .map(|vertex| scrambled(vertex, &mut rng, range))
        .collect();
    Mesh::new(vertices, mesh.triangles().to_vec())
}
```

**`planet-core/src/processor.rs` (new):**
```rust
pub mod vertex_scramble;
pub mod vertex_scramble_range;
```

**`planet-core/src/lib.rs` (updated):**
```rust
pub mod geometry;
pub mod processor;
pub mod subdivision;
```
(`subdivision.rs` itself is untouched — no new lines, no new files under `subdivision/`.)

**`rules.md`'s "Module structure" section — a new `planet-core`'s concerns entry added, alongside the existing `geometry/` and `subdivision/` bullets:**
```markdown
- `processor/` — pre/post-processing steps that run outside the subdivision algorithm, each
  taking an already-built `Mesh` and returning a transformed one: `vertex_scramble_range.rs`
  (`VertexScrambleRange`, `VertexScrambleRangeError`), `vertex_scramble.rs` (`scramble_vertices`)
```

**`planet-core/Cargo.toml` (updated):**
- Add `[[test]] name = "vertex_scramble_range" harness = false` and `[[test]] name = "vertex_scramble" harness = false`
- No dependency changes — `rand` and `rand_pcg` are already present

**`planet-renderer/src/app.rs` (updated):**
- Imports: add `use planet_core::processor::vertex_scramble::scramble_vertices;` and `use planet_core::processor::vertex_scramble_range::VertexScrambleRange;`
- New `const DEMO_SCRAMBLE_SEED: u64 = 43;`
- `resumed()`: right after `base_mesh` is constructed, insert
  ```rust
  let base_mesh = match scramble_vertices(
      &base_mesh,
      Seed::from(DEMO_SCRAMBLE_SEED),
      VertexScrambleRange::default(),
  ) {
      Ok(mesh) => mesh,
      Err(error) => {
          web_sys::console::error_1(&format!("failed to scramble vertices: {error}").into());
          return;
      }
  };
  ```
  (shadowing the pristine `base_mesh` binding) — every subsequent use of `base_mesh` (the `collected_frames` seed and the `subdivide` call) then already sees the scrambled mesh, with no other line in `resumed()` needing to change

No changes to `SubdivisionArgs`, `SubdivisionMode`, `subdivide`, `EdgeCache`, `camera.rs`, `gpu/*`, `shader.wgsl`, `Mesh`, `Vec3`, `Triangle`, `Vertex`, `Seed`, or `ElevationNoiseRange`.

## Function/API contracts

- `VertexScrambleRange::new(low, high)`:
  - `low > -1.0 && low <= high` (including equal bounds) returns `Ok(VertexScrambleRange { low, high })`
  - `low <= -1.0` returns `Err(VertexScrambleRangeError::LowAtOrBelowNegativeOne { low })`, regardless of `high` — checked first, since a `low` that would collapse or mirror an axis is invalid no matter what `high` is
  - Otherwise (`low > high`, or either bound is `NaN`) returns `Err(VertexScrambleRangeError::InvalidRange { low, high })`
  - `VertexScrambleRange::default()` has `low == -0.05` and `high == 0.05`
- `scramble_vertices(mesh, seed, range)`:
  - Returns a `Mesh` with the same vertex count and the identical `triangles` list as `mesh` — only vertex positions differ
  - Is deterministic: two calls with an identical `mesh`, `seed`, and `range` produce byte-identical `Mesh`es
  - With `range == VertexScrambleRange::new(0.0, 0.0).unwrap()` (zero-width at zero), produces a `Mesh` identical to `mesh` (no-op) — every factor offset is exactly `0.0`, so every multiplicative-branch coordinate is unchanged (`(1+0)*x == x`) and every additive-branch (zero) coordinate stays `0.0`
  - Different `seed` values (same non-zero-width `range`) generically produce a different `Mesh` (demonstrated for at least one concrete seed pair in BDD coverage, not asserted for every possible pair)
  - For any vertex with at least one coordinate exactly `0.0` and a non-zero-width, non-zero `range` (e.g. `VertexScrambleRange::new(0.02, 0.02).unwrap()`, deterministic), that coordinate becomes non-zero in the output — the additive fallback branch guarantees this
  - Never produces a vertex with radius `< MIN_VERTEX_RADIUS` (`0.05`), regardless of how close to `-1.0` `range.low()` is or how far from the origin an input vertex already was — a `range` whose factors are all close to `-1.0` shrinks every non-zero-coordinate axis close to (but never through) `0.0`, and the post-transform floor clamp catches the resulting small radius
  - Never panics, including when an input vertex's position is exactly the origin (all three coordinates `0.0`) combined with a zero-width `range` at `0.0` — the only combination that can drive the post-transform radius to exactly `0.0`, since `low > -1.0` guarantees every multiplicative-branch coordinate is non-zero whenever its input was already non-zero
  - Works on any valid `Mesh`, not only ones produced by `Mesh::icosahedron()` — demonstrated with an arbitrary 3-vertex mesh in BDD coverage, matching `subdivide`'s own "not icosahedron-specific" precedent
- `planet-renderer`'s `App::resumed` calls `scramble_vertices` exactly once, before its one `subdivide` call, and both the first collected animation frame and the mesh handed to `subdivide` are the scrambled result (never the pristine `Mesh::icosahedron()` output)

## BDD scenarios

`planet-core/tests/features/vertex_scramble_range.feature`:
```gherkin
Feature: Constructing a validated VertexScrambleRange

  Scenario: Constructing a VertexScrambleRange with low less than high succeeds
    When a VertexScrambleRange is constructed with low -0.1 and high 0.2
    Then the VertexScrambleRange is constructed successfully
    And the VertexScrambleRange has low -0.1
    And the VertexScrambleRange has high 0.2

  Scenario: Constructing a VertexScrambleRange with equal low and high succeeds
    When a VertexScrambleRange is constructed with low 0.0 and high 0.0
    Then the VertexScrambleRange is constructed successfully

  Scenario: Constructing a VertexScrambleRange with low greater than high fails
    When a VertexScrambleRange is constructed with low 0.5 and high 0.1
    Then the construction fails with an invalid-range error of low 0.5 and high 0.1

  Scenario: Constructing a VertexScrambleRange with low at exactly -1.0 fails
    When a VertexScrambleRange is constructed with low -1.0 and high 0.0
    Then the construction fails with a low-at-or-below-negative-one error of low -1.0

  Scenario: Constructing a VertexScrambleRange with low just above -1.0 succeeds
    When a VertexScrambleRange is constructed with low -0.999 and high 0.0
    Then the VertexScrambleRange is constructed successfully

  Scenario: The default VertexScrambleRange has low -0.05 and high 0.05
    Given the default VertexScrambleRange
    Then the VertexScrambleRange has low -0.05
    And the VertexScrambleRange has high 0.05
```

`planet-core/tests/features/vertex_scramble.feature`:
```gherkin
Feature: Scrambling a mesh's existing vertices along all three axes

  Scenario: Scrambling the icosahedron mesh's vertices changes the resulting Mesh
    Given an icosahedron mesh
    When the icosahedron mesh is scrambled with seed 7 and a VertexScrambleRange of low -0.1 and high 0.1
    Then the resulting Mesh is not identical to the icosahedron mesh

  Scenario: Scrambling preserves vertex count and triangle topology
    Given an icosahedron mesh
    When the icosahedron mesh is scrambled with seed 7 and a VertexScrambleRange of low -0.1 and high 0.1
    Then the resulting Mesh has 12 vertices
    And the resulting Mesh has the same triangles as the icosahedron mesh

  Scenario: Scrambling with a zero-width VertexScrambleRange at zero leaves the mesh unchanged
    Given an icosahedron mesh
    When the icosahedron mesh is scrambled with seed 7 and a VertexScrambleRange of low 0.0 and high 0.0
    Then the resulting Mesh is identical to the icosahedron mesh

  Scenario: Scrambling is deterministic for a given seed
    Given an icosahedron mesh
    When the icosahedron mesh is scrambled with seed 7 and a VertexScrambleRange of low -0.1 and high 0.1, producing the first Mesh
    And the same icosahedron mesh is scrambled with seed 7 and a VertexScrambleRange of low -0.1 and high 0.1, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: Scrambling with different seeds produces different vertex positions
    Given an icosahedron mesh
    When the icosahedron mesh is scrambled with seed 7 and a VertexScrambleRange of low -0.1 and high 0.1, producing the first Mesh
    And the same icosahedron mesh is scrambled with seed 99 and a VertexScrambleRange of low -0.1 and high 0.1, producing the second Mesh
    Then the first Mesh and the second Mesh are not identical

  Scenario: Scrambling never pushes a vertex below the minimum vertex radius
    Given a Mesh with a vertex at position 10.0, 10.0, 10.0
    When that mesh is scrambled with seed 7 and a VertexScrambleRange of low -0.999 and high -0.999
    Then every vertex of the resulting Mesh has a radius greater than or equal to 0.05

  Scenario: Scrambling never panics when a vertex sits exactly at the origin
    Given a Mesh with a vertex exactly at the origin
    When that mesh is scrambled with seed 7 and a VertexScrambleRange of low 0.0 and high 0.0
    Then no panic occurs

  Scenario: Scrambling moves a vertex off a coordinate plane it started on
    Given an icosahedron mesh
    When the icosahedron mesh is scrambled with seed 7 and a VertexScrambleRange of low 0.02 and high 0.02
    Then no vertex of the resulting Mesh has a coordinate equal to 0.0

  Scenario: Scrambling an arbitrary mesh proves it is not icosahedron-specific
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When that mesh is scrambled with seed 7 and a VertexScrambleRange of low -0.1 and high 0.1
    Then the resulting Mesh has 3 vertices
```

## Acceptance criteria

1. `VertexScrambleRange::new(low, high)` succeeds iff `low > -1.0 && low <= high` (including equal bounds); `low <= -1.0` returns `Err(VertexScrambleRangeError::LowAtOrBelowNegativeOne { low })` regardless of `high`; otherwise (`low > high`, or a `NaN` bound) returns `Err(VertexScrambleRangeError::InvalidRange { low, high })`; `VertexScrambleRange::default()` has `low == -0.05` and `high == 0.05`
2. `scramble_vertices` is `pub` and reachable as `planet_core::processor::vertex_scramble::scramble_vertices`; `VertexScrambleRange`/`VertexScrambleRangeError` are `pub` and reachable as `planet_core::processor::vertex_scramble_range::*` (verified via `cargo doc -p planet-core --no-deps`)
3. `planet-core/src/processor/` contains exactly `vertex_scramble.rs` and `vertex_scramble_range.rs`; `planet-core/src/subdivision/` gains no new files; `rules.md`'s "Module structure" section documents the new `processor/` concern, mirroring how `geometry/` and `subdivision/` are already documented
4. `scramble_vertices(&Mesh::icosahedron().unwrap(), Seed::from(7), VertexScrambleRange::new(-0.1, 0.1).unwrap())` returns a `Mesh` with exactly 12 vertices and the identical 20-triangle list as `Mesh::icosahedron().unwrap()`, but not identical vertex positions
5. Two `scramble_vertices` calls with identical mesh, seed, and range produce byte-identical `Mesh`es (`PartialEq` equal); two calls differing only in seed (non-zero-width range) produce non-identical `Mesh`es
6. `scramble_vertices` with `VertexScrambleRange::new(0.0, 0.0).unwrap()` returns a `Mesh` identical to its input
7. For an input vertex whose coordinates are all far from the origin (e.g. `(10.0, 10.0, 10.0)`) and a `VertexScrambleRange` whose deterministic factor is close to `-1.0` (e.g. `low == high == -0.999`, giving a `0.001` scale factor on every axis), the output vertex's radius is still `>= 0.05` — the floor clamp catches the resulting near-zero radius
8. `scramble_vertices` never panics for an input vertex exactly at the origin scrambled with a zero-width `VertexScrambleRange` at `0.0` — the only combination that can drive the post-transform radius to exactly `0.0`
9. For an input vertex with a coordinate exactly `0.0` (e.g. any `Mesh::icosahedron()` vertex) and a deterministic, non-zero `VertexScrambleRange` (e.g. `low == high == 0.02`), that coordinate is non-zero in the output — proving the additive fallback actually moves vertices off the coordinate planes they started on
10. `scramble_vertices` succeeds on a non-icosahedron `Mesh` (an arbitrary 3-vertex, 1-triangle mesh), returning the same vertex/triangle counts as the input
11. `SubdivisionArgs`, `SubdivisionMode`, `subdivide`, and `EdgeCache`'s public contracts, source, and existing test coverage (`subdivide.feature`, `subdivision_args.feature`, all `SubdivisionMode` variant scenarios) are byte-for-byte unmodified by this feature
12. `planet-renderer/src/app.rs` calls `scramble_vertices(&base_mesh, Seed::from(DEMO_SCRAMBLE_SEED), VertexScrambleRange::default())` once, immediately after constructing `base_mesh`, and both `collected_frames`'s first entry and the mesh passed to `subdivide` are the scrambled result
13. On loading the app in-browser, the rendered planet's base shape no longer visibly resembles a pristine icosahedron (no flat, symmetric facets/edges bleeding through the randomized surface) at frame 0 or at any subsequent frame — manual/in-browser check, per `000-architecture.md`'s exemption for GPU/DOM wiring
14. All scenarios in `vertex_scramble_range.feature` and `vertex_scramble.feature` pass via real `cucumber` step definitions — no undefined/stub steps
15. `cargo test --workspace`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` all pass
16. `cargo build --target wasm32-unknown-unknown -p planet-renderer` still succeeds
17. No new `unwrap()`/`panic!()` in production code outside tests
18. Existing `vec3.feature`, `mesh.feature`, `icosahedron.feature`, `steps.feature`, `seed.feature`, `elevation_noise_range.feature`, `min_edge_length.feature`, `split_point_variance.feature`, `subdivision_args.feature`, `subdivide.feature`, `camera.feature`, `buffers.feature`, `uniforms.feature`, `mesh_render_vertices.feature`, `mesh_render_indices.feature`, and `mesh_render_line_indices.feature` scenarios still pass unmodified
