# 018 — Restore Tangential Jitter

**Status:** Ready for review
**Feature slug:** `restore-tangential-jitter`

This is an ad-hoc corrective feature, not the next sequential `docs/roadmap.md` phase — phase 1 of a 4-phase plan fixing visual regressions introduced by `017-geodesic-terrain-rework.md`. That spec switched `Planet::subdivide` from `SubdivisionMode::RedGreenSplit` to `SubdivisionMode::UniformRedSplit` (exact chord-midpoint subdivision, no split-point jitter, no per-vertex displacement of its own) to fix two other bugs (early convergence, direction-correlated slivers from green triangulation). A side effect: every generated planet is now built from perfectly equilateral, exact-midpoint triangles — the base icosahedron's 12 vertices sit at their pristine construction positions, and every vertex created during subdivision sits at the exact arithmetic mean of its edge's endpoints. This reads visually as "every planet is a smooth, uniform geodesic ball," the opposite of the fractal/hand-crafted irregularity `constitution.md` describes ("recursive triangle subdivision with controlled randomness").

## Investigation

Two independent mechanisms produced the pre-017 irregularity, and both are gone:

1. **Base-vertex scrambling.** `010-vertex-scramble.md` added `scramble_vertices`/`VertexScrambleRange` (`planet-core/src/processor/vertex_scramble.rs`, `vertex_scramble_range.rs`) specifically so the icosahedron's 12 base vertices — otherwise always at their pristine construction positions — lose their perfect symmetry before any subdivision runs. This code is fully implemented and tested but **orphaned**: nothing calls `scramble_vertices` anywhere in the current codebase (confirmed via `grep`). `PlanetBuilder::build()` (`planet-core/src/planets/planet_builder.rs`) constructs `Mesh::icosahedron()` directly and never scrambles it.
2. **Per-split displacement.** The old `RedGreenSplit`/`RadialRandomSplit` strategies displaced every newly-created vertex along the split edge (`SplitPointVariance`, Gaussian-jittered `t` instead of exact midpoint), along the edge's normal (`NormalNoiseRange`, out-of-plane), and radially (`ElevationNoiseRange`). `017` deleted all of this along with the two strategies. `UniformRedSplit` (`planet-core/src/subdivision/strategies/uniform_red_split.rs`) hardcodes `identity()` as its `VertexOperator` and a throwaway, unused `Pcg32::seed_from_u64(0)` recreated on every `split_triangle` call — every new vertex is the exact chord midpoint, full stop.

**A key architectural finding constrains the fix:** `apply_terrain_noise` (`planet-core/src/processor/terrain_noise.rs`), which runs unconditionally as the first stage of every `Planet::subdivide` call, computes `direction = vertex.position.normalized()` and then sets the vertex's entire radius from scratch (`direction.scale(new_radius)`). Scaling a vector never changes its normalized direction, so **any purely radial displacement applied during subdivision is provably discarded** by the time a planet is rendered — only a vertex's direction on the sphere survives into the final mesh. This was confirmed by simulation. Consequently, this phase restores only the two displacement components that survive: **tangential** (along the edge, i.e. split-point variance) and **normal** (out-of-plane). A genuine radial component is out of scope here — it would require `apply_terrain_noise` to add to an incoming radius rather than overwrite it, a larger, separate change explicitly deferred to the later `exaggerate-terrain-relief` phase of this same 4-phase plan.

**A second finding shaped the magnitude choice.** Naively restoring split-point variance at the old scale (~±35% of edge length, matching the deleted `RedGreenSplit`'s magnitude) reintroduces exactly the kind of degenerate, near-zero-angle sliver triangle `017`'s own regression test was written to prevent — simulation showed this happens within a *single* subdivision round already (independent per-edge randomness occasionally conspires within one triangle), not only through cross-round compounding. Testing progressively smaller magnitudes (simulating the actual icosahedron + exact algorithm this feature implements, across seeds and depths 1–6) found that **split-point variance ±0.05 (t drawn from `[0.45, 0.55]`) and normal-offset fraction ±0.03 of edge length**, applied at every round with no special-casing, keeps every triangle's interior angles within `[22.1°, 118.7°]` and every vertex radius within `[0.78, 1.0]` across all tested seeds/depths — comfortably away from degenerate (a true sliver approaches 0°/180°) while still visibly breaking the exact-60°, perfectly-equilateral look. No edge-length threshold or round-limiting mechanism is needed: because each round's absolute jitter is proportional to that round's (shrinking) edge length, the total compounded effect across rounds converges rather than diverges — confirmed by the simulation converging by depth 4–6 rather than continuing to widen. Per this project's established convention (`017`, `007`, `009`: "concrete, buildable starting constants, not a frozen aesthetic contract"), these are starting values, expected to be retuned in this feature's own `planet-tdd` REFACTOR step against real rendered output, not frozen forever.

## Requirements

- `planet-core/src/planets/planet_builder.rs`: `PlanetBuilder::build()` scrambles the icosahedron base mesh — `scramble_vertices(&mesh, seed, VertexScrambleRange::default())?` — before computing colors and constructing the `Planet`. This is the **only** change to `build()`; `scramble_vertices`/`VertexScrambleRange` themselves are unchanged (already fully implemented and tested by `010-vertex-scramble.md`), this feature only adds the missing call site.
- `planet-core` gains a new `processor/` building block, `jitter.rs`, following the same shape as the existing `identity.rs` (`pub(crate) fn identity() -> VertexOperator`): a function producing a `VertexOperator` that displaces a split point tangentially (along the edge, away from the exact midpoint) and along the edge's normal (out-of-plane), each magnitude proportional to the edge's current length.
- `planet-core/src/subdivision/strategies/uniform_red_split.rs`: `UniformRedSplit` gains a real `rng: Pcg32` field (seeded once, at construction, from a real `Seed` — replacing today's discarded, freshly-reseeded-to-a-constant dummy RNG created inside every `split_triangle` call) and its `pipeline` field is built from the new `jitter()` operator instead of `identity()`.
- `planet-core/src/subdivision/subdivision_mode.rs`: `SubdivisionMode::UniformRedSplit` changes shape from a unit variant to a struct variant carrying `seed: Seed` — `SubdivisionMode::UniformRedSplit { seed: Seed }`. This is an additive, non-breaking-to-callers-that-name-every-field shape change, mirroring `010-vertex-scramble.md`'s own addendum precedent for `RadialRandomSplit`/`RedGreenSplit` gaining a `normal_noise_range` field. `.strategy()` passes the seed through: `SubdivisionMode::UniformRedSplit { seed } => Box::new(UniformRedSplit::new(*seed))`.
- `planet-core/src/planets/planet.rs`: `Planet::subdivide` constructs `SubdivisionMode::UniformRedSplit { seed: self.seed }` instead of the bare unit variant — the same `Seed` that already drives `apply_terrain_noise`, consistent with this project's existing convention of one `Seed` driving multiple independent pipeline stages.
- Every production and test construction site of `SubdivisionMode::UniformRedSplit` gains the `{ seed: ... }` field: `planet-core/tests/subdivide.rs` + `tests/features/subdivide.feature`, `planet-core/tests/subdivision_args.rs` + `tests/features/subdivision_args.feature`, `planet-core/tests/apply_terrain_noise.rs` + `tests/features/apply_terrain_noise.feature`.
- **Deliberate contract change #1** (`013-planet-aggregate-root.md`): `PlanetBuilder::build()`'s mesh is no longer byte-identical to `Mesh::icosahedron()` — it is topologically identical (same 12 vertices' indices, same 20 triangles) but positionally scrambled. The two `planet.feature` scenarios asserting "the resulting Planet's mesh is identical to the icosahedron mesh" immediately after `build()` (no subdivision) are reworded to assert the same vertex/triangle counts and topology, but *not* identical positions.
- **Deliberate contract change #2** (`017-geodesic-terrain-rework.md`): the regression scenario "Terrain noise with zero amplitude produces a near-equilateral geodesic sphere with no sliver triangles" (asserting every triangle's angles stay within 50°–75°) is loosened to match the new, still-bounded-but-wider range this feature intentionally introduces — reintroducing controlled per-split irregularity is this feature's entire point. The new bound is `15°`–`135°` (empirically verified via simulation of the exact algorithm across depths 1–6 and many seeds, converged worst case `22.1°`/`118.7°`, several degrees of margin either side, matching this project's existing practice of stating a provable/verified bound rather than a guess).
- **Deliberate contract change #3**: the existing "A new vertex sits at the exact arithmetic mean of its edge's endpoints" scenario (`subdivide.feature`) no longer holds by construction and is rewritten to assert the new vertex is *near*, but not exactly at, the midpoint, within a provable bound.
- `rules.md` updates:
  - `subdivision/` concern: `uniform_red_split.rs`'s description no longer says "no elevation/displacement logic of its own" unconditionally — it now performs tangential/normal split-point jitter (via the new `processor/jitter.rs` building block), proportional to edge length; radial elevation still lives entirely in `processor/terrain_noise.rs` as the sole post-subdivision whole-mesh step (unchanged).
  - `processor/` concern: gains `jitter.rs` (`jitter`, a `VertexOperator` building block) in the list alongside `identity.rs`.
  - `planets/` concern: `planet_builder.rs`'s description gains a clause noting it also scrambles the icosahedron's base vertices via `scramble_vertices` before storing the mesh.
- No change to `planet-renderer` — `app.rs`/`gpu/`/`scene/`/`controls/` are untouched; `App::generate` already reads every knob through `Planet::builder()...build()...subdivide()`, so it picks up this feature with no code change.

## Domain model involved

### New

**`planet-core/src/processor/jitter.rs`:**
```rust
use rand::RngExt;

use crate::geometry::mesh::Vertex;

use super::vertex_operator::VertexOperator;

const SPLIT_POINT_VARIANCE: f32 = 0.05;
const NORMAL_OFFSET_FRACTION: f32 = 0.03;

pub(crate) fn jitter() -> VertexOperator {
    Box::new(|rng, a, b, _exact_midpoint| {
        let edge = b.position.sub(a.position);
        let edge_length = edge.length();
        let t = rng.random_range((0.5 - SPLIT_POINT_VARIANCE)..=(0.5 + SPLIT_POINT_VARIANCE));
        let mut position = a.position.add(edge.scale(t));
        if let Some(normal) = a.position.cross(b.position).normalized() {
            let normal_delta =
                rng.random_range(-NORMAL_OFFSET_FRACTION..=NORMAL_OFFSET_FRACTION) * edge_length;
            position = position.add(normal.scale(normal_delta));
        }
        Vertex { position }
    })
}
```
The 4th `VertexOperator` parameter (the precomputed exact midpoint) is accepted, per the existing shared signature, but ignored — `jitter()` computes its own tangential lerp directly from `a`/`b`, exactly the way `identity()` ignores `rng`. The degenerate case (`a.position.cross(b.position)` is the zero vector — only reachable when `a`/`b` and the origin are colinear, e.g. one endpoint is the origin) skips the normal offset entirely, mirroring the zero-guard convention already established by `010-vertex-scramble.md`'s addendum for `RadialRandomSplit`/`RedGreenSplit`.

### Changed

- **`planet-core/src/subdivision/strategies/uniform_red_split.rs`:**
  ```rust
  pub(crate) struct UniformRedSplit {
      rng: Pcg32,
      pipeline: VertexOperator,
  }

  impl UniformRedSplit {
      pub(crate) fn new(seed: Seed) -> UniformRedSplit {
          UniformRedSplit {
              rng: Pcg32::seed_from_u64(seed.value()),
              pipeline: jitter(),
          }
      }
  }
  ```
  `split_triangle` reborrows `&mut self.rng` at each of the 3 edge calls (`ab`, `bc`, `ca`) instead of constructing a fresh, discarded `Pcg32` per call — this is the actual bug fix that makes the RNG state persist and advance across an entire `subdivide()` run, exactly like the deleted `RedGreenSplit`/`RadialRandomSplit` did. `exact_midpoint(a, b)` (the existing private helper) is still computed and passed as the 4th argument to `pipeline`, unused by `jitter()` but kept so the call site's shape doesn't change and any future `VertexOperator` swap-in (e.g. a testing-only `identity()`) continues to work unmodified.
- **`planet-core/src/subdivision/subdivision_mode.rs`:**
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Default)]
  pub enum SubdivisionMode {
      #[default]
      UniformRedSplit { seed: Seed },
  }

  impl SubdivisionMode {
      pub(crate) fn strategy(&self) -> Box<dyn SubdivisionStrategy> {
          match self {
              SubdivisionMode::UniformRedSplit { seed } => Box::new(UniformRedSplit::new(*seed)),
          }
      }
  }
  ```
  `#[derive(Default)]` continues to work since `Seed: Default` (already `#[derive(Default)]`, value `0`) — `SubdivisionMode::default()` is `UniformRedSplit { seed: Seed::from(0) }`.
- **`planet-core/src/planets/planet.rs`:** `Planet::subdivide`'s `SubdivisionArgs::new(Some(max_depth), Some(SubdivisionMode::UniformRedSplit { seed: self.seed }), on_progress)` — the only line that changes in this file.
- **`planet-core/src/planets/planet_builder.rs`:**
  ```rust
  pub fn build(self) -> Result<Planet, PlanetError> {
      let preset = self.preset.unwrap_or_default();
      let seed = self.seed.unwrap_or_default();
      let mesh = Mesh::icosahedron()?;
      let mesh = scramble_vertices(&mesh, seed, VertexScrambleRange::default())?;
      let colors = mesh
          .vertices()
          .iter()
          .map(|vertex| preset.params().color_gradient().sample(vertex.position.length()))
          .collect();
      Ok(Planet { mesh, colors, preset, seed, max_depth: None })
  }
  ```
- **`rules.md`:** as described in Requirements above.

### Unchanged

`Mesh`/`Vertex`/`Triangle`/`Vec3`, `Seed` (still `u64`, no narrowing needed here — `Pcg32::seed_from_u64` already takes `u64` directly), `EdgeCache`, `subdivide()`'s loop and round-count contract, `SubdivisionArgs`, `Steps`, `PresetParams`, `TerrainNoise`/`apply_terrain_noise`, `OceanQuota`/`apply_ocean_quota`, `ColorGradient`, `scramble_vertices`/`VertexScrambleRange` themselves (only their caller changes), `identity()`/`vertex_operator.rs` (both stay — `identity()` remains available as a `VertexOperator` building block even though nothing currently constructs a strategy with it; deleting it isn't required since it isn't specific to the old, now-removed strategies the way `red_green_split.rs` was). `planet-renderer` entirely.

## Function/API contracts

### `jitter() -> VertexOperator`

- **Pre:** none (matches `identity()`'s contract — always constructible)
- **Post:** returns a `VertexOperator` closure `(rng: &mut Pcg32, a: &Vertex, b: &Vertex, _point: Vertex) -> Vertex` that:
  1. Draws `t` uniformly from `[0.45, 0.55]` and computes `position = a.position + (b.position - a.position) * t` (a point on the edge, generally not the exact midpoint)
  2. If `a.position.cross(b.position)` is non-zero, draws a normal offset uniformly from `[-0.03, 0.03] * edge_length` and adds it along the normalized cross product to `position`; otherwise leaves `position` unchanged from step 1
  3. Returns `Vertex { position }`
- **Determinism:** for a fixed `rng` state and fixed `a`/`b`, always produces the same result (2 draws from `rng`, in fixed order: `t` then normal offset)
- **Bound:** the returned position's distance from the exact midpoint `(a.position + b.position) / 2` is at most `sqrt(0.05² + 0.03²) * edge_length ≈ 0.058 * edge_length` (Pythagorean combination of the two orthogonal-ish offset components; a safe, provable upper bound, not merely empirical)

### `UniformRedSplit::new(seed: Seed) -> UniformRedSplit`

- **Pre:** any `Seed`
- **Post:** returns a `UniformRedSplit` whose internal `rng` is freshly seeded from `seed.value()` and whose `pipeline` is `jitter()`. Two `UniformRedSplit`s constructed from the same `Seed` and driven through an identical sequence of `split_triangle` calls (same triangles, same order) produce byte-identical results — same reasoning as every other seeded strategy in this codebase (fixed draw order per edge, `Vec`/traversal order, no hash-map iteration)

### `SubdivisionMode::UniformRedSplit { seed: Seed }` (updated contract)

- All of `subdivide()`'s prior postconditions continue to hold unconditionally for every preset: exact `20 * 4^depth` triangle count (topology is untouched by this feature — only vertex *positions* change), no cracks/duplicate vertices at shared edges (the `EdgeCache` still guarantees each edge's split vertex is computed once and shared), `max_depth` still a hard cap
- **New:** for a pristine (non-scrambled) icosahedron mesh, every vertex created by subdivision lies within `0.058 * edge_length` of its edge's exact arithmetic mean (see `jitter()`'s bound above) and no vertex's radius exceeds `1.0` (empirically verified across depths 1–6, many seeds: the un-jittered radial component is bounded above by the max of the mesh's own pre-existing vertices, since neither the tangential lerp nor the normal offset used here can push a new vertex's radius above what the base mesh already contains — confirmed by simulation, `max = 1.0000` in every tested case) — the existing "every vertex of the resulting Mesh has a radius less than or equal to 1.0" scenario is therefore **unchanged**, not widened
- **New (lower bound):** every vertex's radius is empirically bounded below by `0.7` for a pristine icosahedron subdivided by this strategy (observed minimum `0.778` across depths 1–6, many seeds, with margin) — a new scenario, not a modification of an existing one

### `PlanetBuilder::build()` (updated contract)

- **Pre:** unchanged (`self.preset`, `self.seed` both optional, defaulted)
- **Post:** the resulting `Planet.mesh` has exactly 12 vertices and the identical 20-triangle topology as `Mesh::icosahedron()`, but is **not** bit-identical to it for any seed (deterministic: same seed ⇒ same scrambled mesh; different seeds ⇒ generically different scrambled meshes — both properties already guaranteed by `scramble_vertices` itself, per `010-vertex-scramble.md`)

## BDD scenarios

### `planet-core/tests/features/subdivide.feature` (updated)

Every existing `When ... using SubdivisionMode::UniformRedSplit` step gains an explicit seed, e.g. `using SubdivisionMode::UniformRedSplit with seed 7`, per `rules.md`'s "reference a fixture by how it was obtained, never bare" convention (mirroring how the deleted `RedGreenSplit`/`RadialRandomSplit` scenarios always named their seed explicitly).

```gherkin
  Scenario: A new vertex is displaced from its edge's exact midpoint, bounded by the edge's length
    Given an icosahedron mesh
    And the two vertices of the first triangle's first edge in the icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit with seed 7
    Then a vertex exists in the resulting Mesh within 0.06 times the edge's length of the exact midpoint of the two given vertices
    And no vertex in the resulting Mesh sits at the exact midpoint of the two given vertices

  Scenario: Subdividing the icosahedron mesh with different seeds produces different vertex positions
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit with seed 7, producing the first Mesh
    And the same icosahedron mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit with seed 99, producing the second Mesh
    Then the first Mesh and the second Mesh are not identical

  Scenario: Subdividing the icosahedron mesh is deterministic for a given seed
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit with seed 7, producing the first Mesh
    And the same icosahedron mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit with seed 7, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: Subdividing the icosahedron mesh never pushes a vertex below a safe minimum radius
    Given an icosahedron mesh
    When the mesh is subdivided with 6 steps using SubdivisionMode::UniformRedSplit with seed 7
    Then every vertex of the resulting Mesh has a radius greater than or equal to 0.7
```
The pre-existing "never pushes vertices beyond the base radius" (`≤ 1.0`) scenario is unchanged (see Function/API contracts above — this bound continues to hold).

### `planet-core/tests/features/subdivision_args.feature` (updated)

`Given`/`When` steps constructing `SubdivisionMode::UniformRedSplit` gain an explicit seed (e.g. "the UniformRedSplit mode with seed 7"); the "Omitting mode defaults to UniformRedSplit" scenario's assertion becomes "the SubdivisionArgs has the default UniformRedSplit mode" (seed `0`), matching `SubdivisionMode::default()`.

### `planet-core/tests/features/apply_terrain_noise.feature` (updated)

The two `Given an icosahedron mesh subdivided N steps with SubdivisionMode::UniformRedSplit` steps gain an explicit seed (e.g. "with SubdivisionMode::UniformRedSplit and seed 7") — these scenarios assert properties of `apply_terrain_noise`'s output (radius bounds, terrace-level clustering), unaffected in substance by which seed drives the input mesh's jitter.

**Deliberate contract change** — the bound in "Terrain noise with zero amplitude produces a near-equilateral geodesic sphere with no sliver triangles" widens from `50°`–`75°` to `15°`–`135°`:
```gherkin
  Scenario: Terrain noise with zero amplitude produces a geodesic sphere with no degenerate sliver triangles
    Given an icosahedron mesh subdivided 8 steps with SubdivisionMode::UniformRedSplit and seed 7
    And a TerrainNoise with amplitude 0.0
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise
    Then every triangle in the resulting Mesh has all 3 angles between 15 and 135 degrees
```
This is a deliberate, documented loosening, not a regression: `017` introduced this bound specifically against *direction-correlated* slivers caused by green triangulation's asymmetric triangle fans (a topology bug, fixed by `017` switching to `UniformRedSplit`'s uniform 4-way split, which this feature does not touch). This feature reintroduces bounded, per-split *positional* randomness on top of that same safe topology — a different, intentional kind of irregularity, verified by simulating the exact algorithm (not the green-triangulation bug returning).

### `planet-core/tests/features/planet.feature` (updated)

**Deliberate contract change** — the two scenarios asserting `PlanetBuilder::build()`'s mesh is pristine:
```gherkin
  Scenario: Building a Planet with no fields set falls back to each field's default
    Given a Planet built with no fields set
    Then the resulting Planet's preset is Earthy
    And the resulting Planet's seed is 0
    And the resulting Planet's mesh has 12 vertices
    And the resulting Planet's mesh has the same triangles as the icosahedron mesh
    And the resulting Planet's mesh is not identical to the icosahedron mesh
    And the resulting Planet has no max depth set

  Scenario: Creating a Planet does not subdivide it
    Given a Planet created with the Earthy preset and seed 1
    Then the resulting Planet's seed is 1
    And the resulting Planet's mesh has 12 vertices
    And the resulting Planet's mesh has the same triangles as the icosahedron mesh
    And the resulting Planet's mesh is not identical to the icosahedron mesh
    And the resulting Planet has no max depth set
```
All other `planet.feature` scenarios are unaffected in substance (they assert determinism, counts, bounds, color-gradient correctness — all preserved) and need no wording changes beyond what already names seeds/presets explicitly.

## Acceptance criteria

1. `planet-core/src/processor/jitter.rs` exists, exposing `pub(crate) fn jitter() -> VertexOperator`, declared in `processor.rs`'s sibling-module list and added to `rules.md`'s `processor/` concern entry
2. `jitter()`'s tangential component draws `t` uniformly from `[0.45, 0.55]`; its normal component draws a delta uniformly from `[-0.03, 0.03] * edge_length` and skips this step when `a.position.cross(b.position)` is the zero vector (unit/BDD test)
3. `UniformRedSplit::new(seed)` seeds a real, persistent `Pcg32` from `seed.value()` at construction — not a fresh, discarded RNG per `split_triangle` call — and `split_triangle`'s 3 edge computations (`ab`, `bc`, `ca`) advance that same RNG state in a fixed order (unit test, e.g. asserting the 2nd and 3rd edges of a triangle receive different jitter than the 1st for a non-trivial seed)
4. `SubdivisionMode::UniformRedSplit` is a struct variant carrying `seed: Seed`; every production/test construction site is updated (compile-time check); `SubdivisionMode::default()` is `UniformRedSplit { seed: Seed::from(0) }`
5. `Planet::subdivide` constructs `SubdivisionMode::UniformRedSplit { seed: self.seed }` (unit/BDD test)
6. For a pristine (non-scrambled) icosahedron mesh subdivided by `UniformRedSplit` at any depth 1–6, every newly-created vertex lies within `0.06 * edge_length` of its edge's exact midpoint, and no such vertex is exactly at the midpoint, for at least one concrete seed (BDD test)
7. For a pristine icosahedron mesh subdivided by `UniformRedSplit` at any depth up to `MAX_SUBDIVISION_STEPS` (8), every vertex radius lies in `[0.7, 1.0]` (BDD test; the upper bound `1.0` is the pre-existing, unchanged scenario — this feature does not widen it; the lower bound `0.7` is new)
8. `UniformRedSplit`/`SubdivisionMode::UniformRedSplit` remain deterministic: identical `(mesh, seed)` always produces a bit-identical output `Mesh`; different seeds generically produce different output (BDD test)
9. `PlanetBuilder::build()`'s resulting mesh has 12 vertices and the identical 20-triangle topology as `Mesh::icosahedron()`, but is not bit-identical to it, for at least one seed; is deterministic per seed (BDD test)
10. `apply_terrain_noise`'s "no degenerate sliver triangles" regression scenario (renamed from `017`'s "near-equilateral... no sliver triangles") asserts every triangle's 3 angles lie between 15° and 135°, using `SubdivisionMode::UniformRedSplit` with an explicit seed and 8 subdivision steps — this bound is empirically verified (not guessed) by simulating the exact algorithm this feature implements, across depths 1–6 and many seeds, converged worst case `22.1°`/`118.7°` (BDD test)
11. `rules.md`'s `subdivision/`, `processor/`, and `planets/` concern entries are updated per Requirements above
12. `cargo test --workspace`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo build --target wasm32-unknown-unknown -p planet-renderer` all pass
13. No `unwrap()`/`panic!()`/`.expect()` in production code (existing convention, unaffected by this feature)
14. All BDD scenarios above are backed by real `cucumber` step definitions — no scenario left as markdown prose
15. `identity()`/`vertex_operator.rs`/`scramble_vertices`/`VertexScrambleRange` remain unchanged and fully covered by their existing test suites (`identity.rs`'s own unit test, `vertex_scramble.feature`, `vertex_scramble_range.feature`) — this feature adds one new caller and one new sibling building block, nothing about their own contracts changes
16. Manual, in-browser check (per `000-architecture.md`'s GPU/pixel-output exemption): a freshly generated planet, at every preset, visibly shows non-equilateral, irregular triangle structure at both the coarse (original-20-face) and fine (subdivided) scale — not a perfectly smooth geodesic ball
