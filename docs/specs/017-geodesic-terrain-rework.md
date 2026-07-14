# 017 — Geodesic Terrain Rework

**Status:** Ready for review
**Feature slug:** `geodesic-terrain-rework`

This is an ad-hoc corrective feature, not the next sequential `docs/roadmap.md` phase — triggered by two visual bugs reported directly against the shipped `Planet::subdivide` pipeline:

1. **Earthy shows no real improvement after depth 3.** `Planet::subdivide` (`planet-core/src/planets/planet.rs`) always builds `SubdivisionMode::RedGreenSplit { min_edge_length: params.min_edge_length(), .. }` regardless of preset, and `subdivide.feature`'s own existing scenario "Subdivision naturally stops growing once every edge in the mesh is below the threshold, even if more steps are requested" already documents the mechanism: with a `MinEdgeLength` of `0.35` (Earthy's exact configured value, per `preset.rs`), the mesh stops growing somewhere between 2 and 3 rounds from the base icosahedron (initial edge length ≈1.05) — every triangle becomes a "leaf" whose 3 edges are already below the threshold, and `subdivide()`'s per-round loop keeps calling `split_triangle` on those leaves for the remaining rounds as pure no-ops. `max_depth` beyond that point is a hard cap on a recursion that has already, independently, converged. This is not a tuning bug — it is the designed behavior of a length-threshold stopping condition (`000-architecture.md`), which decouples "how much detail" entirely from `max_depth` the moment every edge is short enough.
2. **Volcano and Rocky show blade-shaped slivers along consistent directions.** Both use the same hardcoded `RedGreenSplit`, with tighter `MinEdgeLength` (`0.25`/`0.3`) that requires more rounds to converge than Earthy's `0.35` — more rounds means more triangles pass through **green** triangulation (the 2-edges-split and 1-edge-split cases, fanned through the split edges' midpoints rather than a proper 4-way split), and `000-architecture.md`'s own documented trade-off ("a green triangle's newly created edge ... is not specially exempted ... at the cost of occasionally giving a green triangle one more round of refinement") means green triangles can compound non-equilateral shape across rounds. Combined with `SplitPointVariance`'s Gaussian jitter away from the exact midpoint, and the base icosahedron's own inherent anisotropy (12 order-5 vertices among otherwise order-6 vertices), the distortion is not random noise but compounds along the icosahedron's fixed symmetry axes — hence "along some directions," not scattered uniformly.

Investigation (`constitution.md`, `tech-stack.md`, `rules.md`, `000-architecture.md`, `docs/roadmap.md`, plus external research into procedural-planet techniques — Sebastian Lague's and others' icosphere-plus-noise pipelines, Red Blob Games' noise-redistribution/terracing techniques, and geodesic-grid literature on edge-bisection subdivision's near-equilateral guarantee) converged on one architectural fix for both bugs at once: **stop tying elevation detail to the subdivision recursion at all.** `planet-core` already contains an unused-by-`Planet` `SubdivisionMode::UniformRedSplit` strategy (`subdivision/strategies/uniform_red_split.rs`, the enum's own `#[default]` variant) — plain exact-midpoint 4-way subdivision, no length threshold, no green triangulation, no split-point jitter. Geodesic-grid literature confirms this construction stays near-equilateral by projection alone; it cannot produce a blade-shaped sliver, because it never creates the asymmetric triangulations that cause one. Swapping `Planet::subdivide` onto `UniformRedSplit` removes symptom 2 by construction and makes `max_depth` the *only* detail knob (mesh growth becomes the closed-form `20 * 4^max_depth`, with no early convergence), removing symptom 1 by construction too — provided elevation itself no longer comes from Bernoulli draws made once per split event (which is where the *old* model's information content lived, and which stops growing exactly when the recursion does), but instead from a continuous fractal-noise function sampled at each vertex's final position on the unit sphere. Every reference implementation found (Sebastian Lague, Andrew Yi, Nick Chavez, the arXiv "Comparative Analysis of Procedural Planet Generators" survey) uses exactly this shape: layered fBm noise evaluated at a vertex's direction, independent of how that vertex was produced — so depth increases always sample new positions and reveal genuinely new elevation detail, at whatever frequency content the noise is configured to carry.

This feature makes the changes itemized in "Requirements" below. One detail worth calling out up front: `Seed` wraps a `u64` (`subdivision/seed.rs`) and stays that way — `apply_terrain_noise` is the one new consumer that needs a narrower `u32` (matching `noise`-rs's `Fbm::<Perlin>::new(seed: u32)`), so the narrowing happens locally, at that one call site, via a plain `seed.value() as u32` — a true integer truncation (keeps the value's low 32 bits; Rust's integer-to-integer `as` casts wrap rather than saturate, unlike the float-to-int casts elsewhere in this codebase). `Pcg32::seed_from_u64` and `seed_from_timestamp` are both untouched by this feature; nothing about `Seed`'s own representation changes.

**Scope boundary — what this spec deliberately does *not* do:** collapsing `SubdivisionMode`/`SubdivisionArgs`'s strategy-selection shape down to a single hardcoded call (since only `UniformRedSplit` will remain) is arguably a further simplification, but it is an orthogonal cleanup with its own blast radius (`subdivision_args.rs`, `subdivide.rs`'s dispatch call, `subdivision_mode.rs` itself) independent of the terrain/noise rework this spec is scoped to fix. `SubdivisionMode` stays as a type with its one remaining variant; a follow-up phase can flatten it later if desired. Similarly, exact fBm tuning (frequency/octaves/amplitude/redistribution exponent per preset) is chosen here as a concrete, buildable starting point, not a frozen aesthetic contract — `000-architecture.md` already places actual GPU pixel output outside BDD scope, "manually verified in-browser per milestone," and this feature's own `planet-tdd` REFACTOR step is expected to retune the exact constants against real rendered output the same way every prior preset-tuning phase has.

## Requirements

- `planet-core` gains `processor/terrain_noise.rs` (new file in the existing `processor/` concern): `TerrainNoise` (a 7-field validated value object: `frequency`, `octaves`, `persistence`, `lacunarity`, `amplitude`, `redistribution_exponent`, `terrace_levels`), `TerrainNoiseError`, and `apply_terrain_noise(mesh: &Mesh, seed: Seed, terrain_noise: TerrainNoise) -> Result<Mesh, MeshError>` — the whole-mesh post-processing function that samples layered (fBm) noise at each vertex's unit-sphere direction and sets its radius from a redistribution-curve- and optional-terracing-reshaped result. This is the same `Mesh -> Result<Mesh, MeshError>` processor shape `apply_ocean_quota` already established (per project memory: ocean-quota-style processors are a `processor/` `Mesh -> Mesh` function, never a `Planet` method).
- `planet-core` gains one new confirmed dependency: the `noise` crate (`noise = "0.9"`, crates.io `noise-rs`), used only for its `Fbm<Perlin>` combinator (implements `NoiseFn<f64, 3>`) — pure computation, no I/O/GPU/WASM, consistent with `planet-core`'s zero-dependency constraint in `constitution.md`. `tech-stack.md` gains a row for it once this feature lands (per `CLAUDE.md`'s instruction; not done as part of this spec).
- `apply_terrain_noise` seeds its noise generator with `Fbm::<Perlin>::new(seed.value() as u32)` — `Seed` itself is untouched (still `u64`, per `subdivision/seed.rs`); the narrowing to `noise`-rs's `u32` seed parameter happens only at this one call site, as a plain integer truncation (keeps the value's low 32 bits — Rust's `u64 as u32` cast wraps, it does not saturate, since saturating float-to-int casts are a different, unrelated case). Two `Seed`s that differ only in their upper 32 bits therefore produce identical terrain noise; see "Function/API contracts" for the precise contract this implies.
- `Planet::subdivide` switches from `SubdivisionMode::RedGreenSplit` to `SubdivisionMode::UniformRedSplit`, and composes `apply_terrain_noise` as `postprocessing_pipeline`'s unconditional first stage (before the still-optional `apply_ocean_quota` stage — sea level must be computed from *final* elevations).
- `PresetParams` shrinks from 6 fields to 3: `terrain_noise: TerrainNoise`, `color_gradient: ColorGradient`, `ocean_quota: Option<OceanQuota>` — the 4 `RedGreenSplit`-only fields (`min_edge_length`, `elevation_noise_range`, `normal_noise_range`, `split_point_variance`) are removed.
- `Preset::Earthy`/`Volcano`/`Rocky` each construct a tuned `TerrainNoise` in place of the 4 removed fields (see "Domain model involved" for the concrete starting values).
- `SubdivisionMode` shrinks to its one remaining variant, `UniformRedSplit`.
- Dead code removal: the `RedGreenSplit`/`RadialRandomSplit` strategies, `MinEdgeLength`/`SplitPointVariance`/`ElevationNoiseRange`/`NormalNoiseRange`, the `radial_displacement`/`normal_displacement`/`compose` processor functions, and every dedicated test/feature file for the above (per this project's own working style: unused code is deleted outright, not left as a disabled option or a compatibility shim) — see "Domain model involved" → "Removed" for the exact file list.
- `rules.md` updated to reflect the `processor/`/`subdivision/` module changes above.
- No change to `planet-renderer` — `app.rs`/`gpu/`/`scene/`/`controls/` are untouched.

## Domain model involved

### New

**`planet-core/src/processor/terrain_noise.rs`:**
```rust
pub struct TerrainNoise {
    frequency: f32,
    octaves: u32,
    persistence: f32,
    lacunarity: f32,
    amplitude: f32,
    redistribution_exponent: f32,
    terrace_levels: Option<u32>,
}

pub enum TerrainNoiseError {
    InvalidFrequency { frequency: f32 },
    InvalidOctaves { octaves: u32 },
    InvalidPersistence { persistence: f32 },
    InvalidLacunarity { lacunarity: f32 },
    InvalidAmplitude { amplitude: f32 },
    InvalidRedistributionExponent { redistribution_exponent: f32 },
    InvalidTerraceLevels { terrace_levels: u32 },
}
```
No `seed` field — mirrors `ElevationNoiseRange`/`NormalNoiseRange`/`MinEdgeLength`/`SplitPointVariance`'s existing convention of being pure preset-shape knobs, with `Seed` always supplied separately by `Planet::subdivide` from its own `self.seed` at the call site (exactly how `SubdivisionMode::RedGreenSplit { seed: self.seed, elevation_noise_range: params.elevation_noise_range(), .. }` is assembled today). This is what makes "regenerate" (a new seed, same preset) actually change the terrain — a seed baked into `PresetParams` would make every planet of a given preset identical regardless of seed.

`apply_terrain_noise` lives in the same file as `TerrainNoise`, following `ocean_quota.rs`'s established precedent (a validated type that exists solely to be consumed by the one function built around it, both declared in one file, rather than split the way `vertex_scramble_range.rs`/`vertex_scramble.rs` are).

### Changed

- **`planet-core/src/presets/preset_params.rs`:** `PresetParams` shrinks from 6 fields to 3 — `terrain_noise: TerrainNoise`, `color_gradient: ColorGradient`, `ocean_quota: Option<OceanQuota>`. `PresetParams::new` gains the matching 3-parameter signature; `min_edge_length()`/`elevation_noise_range()`/`normal_noise_range()`/`split_point_variance()` accessors are removed, replaced by `terrain_noise(&self) -> TerrainNoise`.
- **`planet-core/src/presets/preset.rs`:** each `Preset::params()` arm builds its own `TerrainNoise` in place of the 4 removed fields:
  - `Preset::Earthy`: `TerrainNoise::new(1.5, 4, 0.5, 2.0, 0.12, 1.4, None)` — gentle, smooth continents (no terracing), matching the existing Earthy color gradient's naturalistic read
  - `Preset::Volcano`: `TerrainNoise::new(2.5, 5, 0.55, 2.2, 0.30, 2.2, Some(6))` — high amplitude + steep redistribution for craters/peaks, 6-level terracing for a banded volcanic-strata cartoon look
  - `Preset::Rocky`: `TerrainNoise::new(3.0, 4, 0.5, 2.0, 0.22, 1.8, Some(8))` — higher frequency, 8-level terracing for a blocky, low-poly rock-formation look
- **`planet-core/src/planets/planet.rs`:** `Planet::subdivide` builds `SubdivisionArgs::new(Some(max_depth), Some(SubdivisionMode::UniformRedSplit), on_progress)` instead of `RedGreenSplit`; `postprocessing_pipeline` gains a `seed: Seed` parameter and an unconditional first stage:
  ```rust
  fn postprocessing_pipeline(params: &PresetParams, seed: Seed) -> MeshProcessor {
      let terrain_noise = params.terrain_noise();
      let mut pipeline = compose_mesh(
          identity_mesh(),
          Box::new(move |mesh: &Mesh| apply_terrain_noise(mesh, seed, terrain_noise)),
      );
      if let Some(quota) = params.ocean_quota() {
          pipeline = compose_mesh(
              pipeline,
              Box::new(move |mesh: &Mesh| apply_ocean_quota(mesh, quota)),
          );
      }
      pipeline
  }
  ```
- **`planet-core/src/subdivision/subdivision_mode.rs`:** `SubdivisionMode` shrinks to its one remaining variant, `UniformRedSplit` (`#[default]`), and `.strategy()` shrinks to the one matching arm.
- **`rules.md`:** `processor/` concern entry gains `terrain_noise.rs` (`TerrainNoise`, `TerrainNoiseError`, `apply_terrain_noise`); the sentences documenting `radial_displacement.rs`/`normal_displacement.rs`/`compose.rs`, `min_edge_length.rs`/`split_point_variance.rs`/`elevation_noise_range.rs`/`normal_noise_range.rs`, and the `RadialRandomSplit`/`RedGreenSplit` strategy files are removed.
- **`planet-core/Cargo.toml`:** gains `noise = "0.9"` under `[dependencies]`; loses the `[[test]]` entries for `elevation_noise_range`, `min_edge_length`, `split_point_variance`, `normal_noise_range`.

### Removed (dead the moment nothing constructs them)

- `subdivision/strategies/red_green_split.rs`, `subdivision/strategies/radial_random_split.rs` (and their `pub(crate) mod` lines in `subdivision/strategies.rs`)
- `subdivision/min_edge_length.rs`, `subdivision/split_point_variance.rs`, `subdivision/elevation_noise_range.rs`, `subdivision/normal_noise_range.rs` (and their `pub mod` lines in `subdivision.rs`)
- `processor/radial_displacement.rs`, `processor/normal_displacement.rs`, `processor/compose.rs` (and their `pub(crate) mod` lines in `processor.rs`) — `processor/vertex_operator.rs` and `processor/identity.rs` stay, since `UniformRedSplit` still uses `identity()`
- Test files: `tests/elevation_noise_range.rs` + `tests/features/elevation_noise_range.feature`, `tests/min_edge_length.rs` + `.feature`, `tests/split_point_variance.rs` + `.feature`, `tests/normal_noise_range.rs` + `.feature`, and every `RadialRandomSplit`-/`RedGreenSplit`-specific scenario in `tests/features/subdivide.feature` (everything from "Subdividing the icosahedron mesh by 1 step using `SubdivisionMode::RadialRandomSplit` ..." onward) and its step definitions in `tests/subdivide.rs`

### Unchanged

`Mesh`/`Vertex`/`Triangle`/`Vec3`, `Seed` (stays `u64`; the `noise`-rs narrowing lives entirely inside `apply_terrain_noise`, see "Function/API contracts"), `Steps`, `SubdivisionArgs`, `ColorGradient`/`Rgb`, `OceanQuota`/`apply_ocean_quota`, `MeshProcessor`/`identity_mesh`/`compose_mesh`, `Planet`/`PlanetError`/`PlanetBuilder`, `subdivision/strategies/uniform_red_split.rs` itself, `processor/vertex_scramble.rs` itself. `planet-renderer` entirely (`app.rs` already reads every knob through `Planet::builder()...build()`/`.subdivide(..)`, so it picks up this rework with no code change).

## Function/API contracts

### `TerrainNoise::new`

```rust
pub fn new(
    frequency: f32,
    octaves: u32,
    persistence: f32,
    lacunarity: f32,
    amplitude: f32,
    redistribution_exponent: f32,
    terrace_levels: Option<u32>,
) -> Result<TerrainNoise, TerrainNoiseError>
```
- **Pre:** any values
- **Post:** `Ok` iff every field is independently valid, else the first invalid field's corresponding `Err` variant, checked in parameter order:
  - `frequency`: finite and `> 0.0`, else `InvalidFrequency`
  - `octaves`: `1..=8` inclusive (mirrors `Steps::MAX_SUBDIVISION_STEPS`'s existing bounded-recursion spirit, applied here to bound noise-evaluation cost per vertex instead of mesh recursion), else `InvalidOctaves`
  - `persistence`: finite and `0.0..=1.0` inclusive (fBm amplitude decay per octave; `>1.0` diverges), else `InvalidPersistence`
  - `lacunarity`: finite and `1.0..=4.0` inclusive (frequency growth per octave; `<=1.0` produces no new detail per octave), else `InvalidLacunarity`
  - `amplitude`: finite and `>= 0.0`, else `InvalidAmplitude`
  - `redistribution_exponent`: finite and `> 0.0`, else `InvalidRedistributionExponent`
  - `terrace_levels`: `None`, or `Some(n)` with `n >= 2` (fewer than 2 levels collapses all elevation to one value), else `InvalidTerraceLevels`
- Accessors: `frequency() -> f32`, `octaves() -> u32`, `persistence() -> f32`, `lacunarity() -> f32`, `amplitude() -> f32`, `redistribution_exponent() -> f32`, `terrace_levels() -> Option<u32>`
- `#[derive(Debug, Clone, Copy, PartialEq)]`, matching the other single-purpose validated newtypes' derive set

### `apply_terrain_noise`

```rust
pub fn apply_terrain_noise(mesh: &Mesh, seed: Seed, terrain_noise: TerrainNoise) -> Result<Mesh, MeshError>
```
- **Pre:** `mesh` is any valid `Mesh` (zero or more vertices); `terrain_noise` is any already-validated `TerrainNoise`; `seed` is any `Seed`
- **Post:**
  1. For each vertex, `direction = vertex.position.normalized()`; if `None` (a vertex exactly at the origin), the vertex is returned unchanged (mirrors `radial_displacement`'s/`apply_ocean_quota`'s existing zero-radius guard)
  2. Otherwise, sample a noise generator built once per call — `Fbm::<Perlin>::new(seed.value() as u32)` (a plain truncating cast: keeps `seed.value()`'s low 32 bits, matching `noise`-rs's own `u32` seed width — this is the only place `Seed`'s `u64` is narrowed anywhere in this feature) configured via `.set_frequency(terrain_noise.frequency() as f64)`, `.set_octaves(terrain_noise.octaves() as usize)`, `.set_persistence(terrain_noise.persistence() as f64)`, `.set_lacunarity(terrain_noise.lacunarity() as f64)` — at `direction`'s 3 components (`NoiseFn::<f64, 3>::get([direction.x as f64, direction.y as f64, direction.z as f64])`), clamp the raw `f64` result to `[-1.0, 1.0]` and narrow to `f32` (bounded regardless of octave/persistence combination), then apply redistribution — `signed = clamped.signum() * clamped.abs().powf(terrain_noise.redistribution_exponent())` — then, if `terrace_levels` is `Some(levels)`, quantize into exactly `levels` bands (bin centers): `unit = (signed + 1.0) / 2.0` (remap `[-1,1]` to `[0,1]`), `bin = (unit * levels as f32).floor().min(levels as f32 - 1.0)`, `signed = ((bin + 0.5) / levels as f32) * 2.0 - 1.0` (remap the bin center back to `[-1,1]`) — caught during this feature's own TDD RED/GREEN cycle: the naively simpler `(signed * levels as f32).round() / levels as f32` produces up to `2 * levels + 1` distinct values over the signed `[-1, 1]` range, not `levels`, which the BDD scenario asserting "at most `levels` distinct radii" caught immediately
  3. `new_radius = (1.0 + signed * terrain_noise.amplitude()).max(MIN_VERTEX_RADIUS)` (a fresh `pub(crate) const MIN_VERTEX_RADIUS: f32 = 0.05` local to this file, same value the removed `radial_displacement.rs` used)
  4. The vertex's new position is `direction.scale(new_radius)`
  5. `Mesh::new(new_vertices, mesh.triangles().to_vec())` — vertex count and triangle topology are unchanged, only positions
- **Determinism:** identical `(mesh, seed, terrain_noise)` always produces a bit-identical output `Mesh` — a vertex's elevation is a pure function of `(seed.value() as u32, terrain_noise, direction)`, with no ordering/traversal dependence (a stronger guarantee than the old per-split-RNG model, which depended on subdivision order)
- **Seed narrowing:** because `seed.value(): u64` is truncated to `u32` before seeding the noise generator, two distinct `Seed`s that agree on their low 32 bits (i.e. differ only above bit 31) produce **identical** terrain noise. Two `Seed`s that differ anywhere in their low 32 bits produce different noise (modulo the negligible, inherent chance of an actual hash/noise-function collision, unrelated to truncation). This is a real, narrow behavioral caveat of using a 32-bit noise field with a 64-bit `Seed`, not an oversight — see "BDD scenarios" for the scenario that pins it down explicitly.
- **Bound:** every output vertex's radius lies in `[max(MIN_VERTEX_RADIUS, 1.0 - amplitude), 1.0 + amplitude]` — depth-invariant and round-count-invariant, since this is a single post-subdivision pass, unlike the old per-round-compounding model `roadmap.md`'s items 008/009 had to specifically account for
- An empty mesh (zero vertices) maps to zero vertices with no special-case branch required (unlike `apply_ocean_quota`, which needs a non-empty radius list to compute a percentile)

### `Planet::subdivide` (updated contract)

All of prior specs' postconditions continue to hold (determinism, `max_depth` honored as a hard cap, `colors().len() == mesh().vertices().len()`). This feature changes two observable behaviors:
- Mesh growth is now the closed-form `20 * 4^max_depth` triangles for **every** preset (previously only true for `UniformRedSplit`/`RadialRandomSplit`; `RedGreenSplit` could converge early) — `max_depth` is now the sole detail-density knob, with no preset-dependent early convergence
- `postprocessing_pipeline`'s first stage (`apply_terrain_noise`) always runs (every preset now carries a `TerrainNoise`, unconditionally, unlike the still-optional `ocean_quota` stage that follows it)

## BDD scenarios

### `planet-core/tests/features/terrain_noise.feature` (new — `TerrainNoise` construction/validation, matching the shared idiom of the removed `elevation_noise_range.feature`/`min_edge_length.feature`/etc: one scenario per validity rule, a boundary scenario per inclusive bound, a `NaN` scenario per `f32` field)

```gherkin
Feature: Constructing a validated TerrainNoise

  Scenario: Constructing a TerrainNoise with valid values succeeds
    When a TerrainNoise is constructed with frequency 1.5, 4 octaves, persistence 0.5, lacunarity 2.0, amplitude 0.12, redistribution exponent 1.4, and no terrace levels
    Then the TerrainNoise is constructed successfully
    And the TerrainNoise has frequency 1.5
    And the TerrainNoise has 4 octaves
    And the TerrainNoise has persistence 0.5
    And the TerrainNoise has lacunarity 2.0
    And the TerrainNoise has amplitude 0.12
    And the TerrainNoise has redistribution exponent 1.4
    And the TerrainNoise has no terrace levels

  Scenario: Constructing a TerrainNoise with terrace levels set succeeds
    When a TerrainNoise is constructed with frequency 2.5, 5 octaves, persistence 0.55, lacunarity 2.2, amplitude 0.3, redistribution exponent 2.2, and 6 terrace levels
    Then the TerrainNoise is constructed successfully
    And the TerrainNoise has 6 terrace levels

  Scenario: Constructing a TerrainNoise with a non-positive frequency fails
    When a TerrainNoise is constructed with frequency 0.0, 4 octaves, persistence 0.5, lacunarity 2.0, amplitude 0.12, redistribution exponent 1.4, and no terrace levels
    Then the construction fails with an invalid-frequency error of 0.0

  Scenario: Constructing a TerrainNoise with 0 octaves fails
    When a TerrainNoise is constructed with frequency 1.5, 0 octaves, persistence 0.5, lacunarity 2.0, amplitude 0.12, redistribution exponent 1.4, and no terrace levels
    Then the construction fails with an invalid-octaves error of 0

  Scenario: Constructing a TerrainNoise with more than 8 octaves fails
    When a TerrainNoise is constructed with frequency 1.5, 9 octaves, persistence 0.5, lacunarity 2.0, amplitude 0.12, redistribution exponent 1.4, and no terrace levels
    Then the construction fails with an invalid-octaves error of 9

  Scenario: Constructing a TerrainNoise with a persistence above 1.0 fails
    When a TerrainNoise is constructed with frequency 1.5, 4 octaves, persistence 1.1, lacunarity 2.0, amplitude 0.12, redistribution exponent 1.4, and no terrace levels
    Then the construction fails with an invalid-persistence error of 1.1

  Scenario: Constructing a TerrainNoise with a lacunarity of 1.0 or below fails
    When a TerrainNoise is constructed with frequency 1.5, 4 octaves, persistence 0.5, lacunarity 1.0, amplitude 0.12, redistribution exponent 1.4, and no terrace levels
    Then the construction fails with an invalid-lacunarity error of 1.0

  Scenario: Constructing a TerrainNoise with a negative amplitude fails
    When a TerrainNoise is constructed with frequency 1.5, 4 octaves, persistence 0.5, lacunarity 2.0, amplitude -0.1, redistribution exponent 1.4, and no terrace levels
    Then the construction fails with an invalid-amplitude error of -0.1

  Scenario: Constructing a TerrainNoise with a non-positive redistribution exponent fails
    When a TerrainNoise is constructed with frequency 1.5, 4 octaves, persistence 0.5, lacunarity 2.0, amplitude 0.12, redistribution exponent 0.0, and no terrace levels
    Then the construction fails with an invalid-redistribution-exponent error of 0.0

  Scenario: Constructing a TerrainNoise with 1 terrace level fails
    When a TerrainNoise is constructed with frequency 1.5, 4 octaves, persistence 0.5, lacunarity 2.0, amplitude 0.12, redistribution exponent 1.4, and 1 terrace level
    Then the construction fails with an invalid-terrace-levels error of 1

  Scenario: Constructing a TerrainNoise with a NaN frequency fails
    When a TerrainNoise is constructed with frequency NaN, 4 octaves, persistence 0.5, lacunarity 2.0, amplitude 0.12, redistribution exponent 1.4, and no terrace levels
    Then the construction fails with an invalid-frequency error of NaN
```

### `planet-core/tests/features/apply_terrain_noise.feature` (new — mesh-elevation behavior, mirrors `apply_ocean_quota.feature`'s style)

```gherkin
Feature: Shaping a mesh's elevation from a continuous noise field

  Scenario: Applying terrain noise never displaces a vertex beyond amplitude bounds
    Given an icosahedron mesh
    And a TerrainNoise with amplitude 0.2
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise
    Then every vertex of the resulting Mesh has a radius less than or equal to 1.2
    And every vertex of the resulting Mesh has a radius greater than or equal to 0.8

  Scenario: Applying terrain noise is deterministic for a given seed
    Given an icosahedron mesh
    And a TerrainNoise with amplitude 0.2
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise, producing the first Mesh
    And terrain noise is applied to the same icosahedron mesh with seed 7 and that TerrainNoise, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: Applying terrain noise with different seeds produces different vertex positions
    Given an icosahedron mesh
    And a TerrainNoise with amplitude 0.2
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise, producing the first Mesh
    And terrain noise is applied to the same icosahedron mesh with seed 99 and that TerrainNoise, producing the second Mesh
    Then the first Mesh and the second Mesh are not identical

  Scenario: Applying terrain noise with seeds that agree on their low 32 bits produces identical terrain
    Given an icosahedron mesh
    And a TerrainNoise with amplitude 0.2
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise, producing the first Mesh
    And terrain noise is applied to the same icosahedron mesh with seed 4294967303 and that TerrainNoise, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: Applying terrain noise with zero amplitude leaves every vertex radius unchanged
    Given an icosahedron mesh
    And a TerrainNoise with amplitude 0.0
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise
    Then every vertex of the resulting Mesh has a radius equal to the corresponding vertex's radius in the icosahedron mesh

  Scenario: Applying terrain noise with terrace levels set produces radii clustered at a bounded number of distinct values
    Given an icosahedron mesh subdivided 3 steps with SubdivisionMode::UniformRedSplit
    And a TerrainNoise with amplitude 0.3 and 6 terrace levels
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise
    Then the resulting Mesh has at most 6 distinct vertex radii, within floating-point tolerance

  Scenario: Applying terrain noise never panics when a vertex sits exactly at the origin
    Given a Mesh with a vertex exactly at the origin
    And a TerrainNoise with amplitude 0.2
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise
    Then no panic occurs

  Scenario: Applying terrain noise to an empty mesh is a no-op
    Given a Mesh with no vertices and no triangles
    And a TerrainNoise with amplitude 0.2
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise
    Then the resulting Mesh is identical to the original mesh

  Scenario: Applying terrain noise preserves vertex count and triangle topology
    Given an icosahedron mesh
    And a TerrainNoise with amplitude 0.2
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise
    Then the resulting Mesh has 12 vertices
    And the resulting Mesh has the same triangles as the icosahedron mesh

  Scenario: Terrain noise with zero amplitude produces a near-equilateral geodesic sphere with no sliver triangles
    Given an icosahedron mesh subdivided 8 steps with SubdivisionMode::UniformRedSplit
    And a TerrainNoise with amplitude 0.0
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise
    Then every triangle in the resulting Mesh has all 3 angles between 50 and 75 degrees
```

This last scenario is the direct, tuning-independent regression test for "no more blade-shaped faces along consistent directions." The bound is not a guess: subdividing the icosahedron with `UniformRedSplit` (exact chord midpoints, no renormalization mid-recursion — mathematically equivalent to barycentric grid subdivision of each flat face) and then radially projecting every vertex to a common radius (exactly what `amplitude = 0.0` does: `new_radius = 1.0` for every vertex, regardless of its noise sample) is the textbook "Class I geodesic subdivision" construction. Computing it numerically (replicating `icosahedron()` and `UniformRedSplit`'s exact chord-midpoint logic) at every depth from 0 through `MAX_SUBDIVISION_STEPS` (8) gives a converged worst case of `54.0°`/`72.0°` (monotonically approaching that bound as depth increases, never reaching it), so `50°`/`75°` is a real, verified bound with comfortable margin at every valid depth — not an empirically-tuned constant pulled from a future implementation run.

**Why this is tested at `amplitude = 0.0` and not at each preset's actual (non-zero) amplitude:** purely radial per-vertex displacement — each vertex moving only along its own direction from the origin, by a different amount than its neighbors — genuinely can distort a triangle's angles (two triangles sharing an edge, displaced very differently, absolutely can look sliver-like even on top of a perfectly uniform base mesh; this is not one of the cases where the fix is true "by construction" regardless of noise parameters). What *is* true regardless of noise parameters is that the **topology/projection** — the thing `RedGreenSplit`'s green-triangle fan + Gaussian split-point jitter broke — is fixed. Whether a specific preset's chosen `frequency`/`amplitude` combination looks acceptably smooth once real elevation is layered on top is a visual, per-preset tuning question, and per `000-architecture.md`, GPU pixel output is explicitly out of BDD scope, "manually verified in-browser per milestone" — the same treatment every other preset's aesthetic constants already get, not a new exception this feature introduces.

### `planet-core/tests/features/subdivide.feature` (reduced)

Every `SubdivisionMode::RadialRandomSplit`- and `SubdivisionMode::RedGreenSplit`-specific scenario is removed. The remaining `UniformRedSplit`-only scenarios (face-count growth per level, no duplicate vertices at shared edges, no cracks, vertex radii within `[0, 1.0]`, exact-midpoint placement, default-args fallback, update-callback behavior, 0-step no-op) are the file's entire remaining content — this is exactly `rules.md`'s mandated "core scenario set," with no algorithm-specific scenarios left to append now that only one algorithm remains.

### `planet-core/tests/features/planet.feature` (extended)

```gherkin
  Scenario: Increasing subdivision depth beyond 3 keeps growing an Earthy planet's mesh
    Given a Planet generated with seed 42 and the Earthy preset at max depth 3
    When another Planet is generated with seed 42 and the Earthy preset at max depth 5
    Then the second Planet's mesh has more vertices than the first Planet's mesh

  Scenario: Subdivision depth deterministically produces the full geodesic triangle count for every preset
    Given a Planet generated with seed 5 and the Volcano preset at max depth 8
    Then the resulting Planet's mesh has exactly 1310720 triangles

  Scenario: A Planet generated with a preset carrying terrace levels has vertex radii clustered at a bounded number of distinct values
    Given a Planet generated with seed 5 and the Volcano preset at max depth 4
    Then the resulting Planet's mesh has at most 6 distinct vertex radii, within floating-point tolerance
```

The existing "Subdivision depth is honored as a hard cap regardless of the preset's min edge length" scenario is retitled ("... regardless of preset" — `min_edge_length` no longer exists) and strengthened from an upper-bound assertion to the exact-count assertion above, since `UniformRedSplit` makes the triangle count a closed form rather than merely a bound.

### `planet-core/tests/features/preset_params.feature` / `preset.feature` (rewritten for the 3-field shape)

```gherkin
  Scenario: Constructing PresetParams bundles all 3 fields unchanged
    Given a TerrainNoise with amplitude 0.12, a ColorGradient with stops at elevation 0.0 color black and elevation 1.0 color white, and an OceanQuota of 0.2
    When a PresetParams is constructed from those 3 values
    Then the PresetParams has a TerrainNoise with amplitude 0.12
    And the PresetParams's ColorGradient samples elevation 0.0 to black
    And the PresetParams has an OceanQuota of 0.2

  Scenario: Two PresetParams built from identical arguments are equal
    Given a TerrainNoise with amplitude 0.12, a ColorGradient with stops at elevation 0.0 color black and elevation 1.0 color white, and an OceanQuota of 0.2
    When two PresetParams are constructed from those same 3 values, separately
    Then the two PresetParams are identical
```

`preset.feature`'s per-preset scenarios gain a `TerrainNoise` assertion in place of the removed `MinEdgeLength`/`ElevationNoiseRange`/`NormalNoiseRange`/`SplitPointVariance` ones (e.g. `And the Earthy preset's PresetParams has a TerrainNoise with amplitude 0.12`).

`planet-core/tests/features/seed.feature` and `planet-renderer/tests/features/seed_from_timestamp.feature` are both untouched by this feature — `Seed` stays `u64` and `seed_from_timestamp` is unchanged; the new seed-narrowing behavior is entirely local to `apply_terrain_noise` (see `apply_terrain_noise.feature` above, "seeds that agree on their low 32 bits").

## Acceptance criteria

1. `planet-core` gains `processor/terrain_noise.rs` (`TerrainNoise`, `TerrainNoiseError`, `apply_terrain_noise`), declared in `processor.rs`'s sibling-module list and added to `rules.md`'s `processor/` concern entry
2. `TerrainNoise::new` validates all 7 fields independently (frequency `> 0.0`, octaves `1..=8`, persistence `0.0..=1.0`, lacunarity `1.0..=4.0`, amplitude `>= 0.0`, redistribution exponent `> 0.0`, terrace levels `None` or `Some(n>=2)`), rejecting `NaN` for every `f32` field (unit/BDD test)
3. `apply_terrain_noise(mesh, seed, terrain_noise)` never returns a vertex radius outside `[max(MIN_VERTEX_RADIUS, 1.0 - amplitude), 1.0 + amplitude]`, for any valid `TerrainNoise` (unit/BDD test)
4. `apply_terrain_noise` is deterministic: identical `(mesh, seed, terrain_noise)` always produces a bit-identical output `Mesh`; two `Seed`s differing anywhere in their low 32 bits produce different vertex positions for a non-zero amplitude, while two `Seed`s agreeing on their low 32 bits (differing only above bit 31) produce identical output — an explicit, intentional consequence of narrowing `seed.value(): u64` to the `u32` `Fbm::new` needs (unit/BDD test)
5. `apply_terrain_noise` with `amplitude == 0.0` leaves every vertex radius unchanged (unit/BDD test)
6. `apply_terrain_noise` with `terrace_levels: Some(n)` produces at most `n` distinct vertex radii (within floating-point tolerance) on any input mesh (unit/BDD test)
7. `apply_terrain_noise` never panics on a vertex exactly at the origin, or on an empty mesh, and preserves vertex count and triangle topology exactly (unit/BDD test)
8. `PresetParams` shrinks to 3 fields (`terrain_noise`, `color_gradient`, `ocean_quota`); `MinEdgeLength`/`ElevationNoiseRange`/`NormalNoiseRange`/`SplitPointVariance` and their accessors no longer exist anywhere in `planet-core`'s public API (compile-time check)
9. `Preset::Earthy`/`Volcano`/`Rocky` each construct a distinct, valid `TerrainNoise` via `Preset::params()` (unit/BDD test)
10. `SubdivisionMode` has exactly one variant, `UniformRedSplit`; `RadialRandomSplit`/`RedGreenSplit` no longer exist anywhere in `planet-core`'s public API (compile-time check)
11. `Planet::subdivide` builds `SubdivisionMode::UniformRedSplit` unconditionally, for every preset (unit/BDD test)
12. For every preset, `Planet::subdivide(max_depth, _)`'s resulting mesh has exactly `20 * 4^max_depth` triangles — no early convergence, for any preset, at any depth up to `MAX_SUBDIVISION_STEPS` (unit/BDD test) — this is the direct regression test for "no real improvement after depth 3"
13. For a `Planet` generated with the Earthy preset, increasing `max_depth` beyond 3 (e.g. to 5) strictly increases the resulting mesh's vertex count (BDD test) — the direct behavioral regression test for symptom 1, expressed independently of the closed-form triangle count
14. `apply_terrain_noise(mesh, seed, terrain_noise)` with `terrain_noise.amplitude() == 0.0`, applied to an icosahedron subdivided 8 steps with `SubdivisionMode::UniformRedSplit`, never produces a triangle with any interior angle below `50°` or above `75°` (unit/BDD test). This bound is verified by direct numerical computation of the actual algorithm (icosahedron construction + exact chord-midpoint subdivision + radial projection), not estimated: the true converged worst case across every depth from 0 through `MAX_SUBDIVISION_STEPS` is `≈54.0°`/`≈72.0°`, monotonically approached but never reached, so `50°`/`75°` holds with several degrees of margin at every valid depth, independent of any preset's noise tuning — this is the direct, tuning-independent regression test for "no more blade-shaped faces along consistent directions." (A specific preset's chosen non-zero `amplitude`/`frequency` combination looking acceptably smooth once real elevation is layered on top remains a visual, per-preset tuning concern, out of BDD scope per `000-architecture.md`, same as every other preset's aesthetic constants.)
15. `postprocessing_pipeline` composes `apply_terrain_noise` unconditionally as its first stage (every preset now has a `TerrainNoise`, not an `Option`) and `apply_ocean_quota` conditionally as its second stage, in that order (BDD test, via `Planet::subdivide`'s existing determinism/ocean-quota scenarios continuing to pass)
16. `planet-core/Cargo.toml` gains `noise = "0.9"` under `[dependencies]` and loses the `[[test]]` entries for the 4 removed newtypes; the workspace's build gate (`cargo test --workspace && cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo build --target wasm32-unknown-unknown -p planet-renderer`) passes
17. `subdivision/strategies/red_green_split.rs`, `subdivision/strategies/radial_random_split.rs`, `subdivision/min_edge_length.rs`, `subdivision/split_point_variance.rs`, `subdivision/elevation_noise_range.rs`, `subdivision/normal_noise_range.rs`, `processor/radial_displacement.rs`, `processor/normal_displacement.rs`, `processor/compose.rs`, and their dedicated test/feature files no longer exist in the repository
18. No change to `planet-renderer` — `app.rs` is untouched by this feature; the existing preset dropdown / depth slider continue to work with no code change
19. `apply_terrain_noise`/`TerrainNoise::new` contain no `unwrap()`/`panic!()`/`.expect()` in production code
20. All BDD scenarios above are backed by real `cucumber` step definitions in their respective `.feature` files and matching step-definition modules — no scenario is left as markdown prose
21. `tech-stack.md` gains a row for the `noise` crate (exact pinned version, feature flags if any, "Used in: `planet-core`") once this feature's implementation confirms it, per `CLAUDE.md`'s dependency-table convention
22. `Seed` is unchanged (still wraps `u64`); `apply_terrain_noise` is the only place `Seed`'s value is narrowed, via `seed.value() as u32` — a plain truncating cast (keeps the low 32 bits) — passed directly into `Fbm::<Perlin>::new`. Two `Seed`s agreeing on their low 32 bits produce identical terrain noise; two differing anywhere in their low 32 bits produce different terrain noise (unit/BDD test)
