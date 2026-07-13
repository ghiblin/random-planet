# 014 — Ocean Quota

**Status:** Ready for review
**Feature slug:** `ocean-quota`

This is the third slice of `docs/roadmap.md`'s "007 — Planet presets" phase, continuing after `013-planet-aggregate-root` (which shipped `Planet`/`PlanetBuilder` but explicitly deferred ocean-quota sea-level flattening: *"It lands in a later, higher-numbered spec once this one is merged — shaped as a `processor/` whole-mesh post-processing function ... mirroring `processor/vertex_scramble.rs`'s existing `scramble_vertices(mesh, seed, range) -> Result<Mesh, MeshError>` shape exactly ... That future spec's `Planet::subdivide()` will call it once, on the fully-subdivided `Mesh`, after `subdivide()` returns and before per-vertex color sampling."*). This feature is that later spec.

Per `000-architecture.md`'s "Ocean quota (Earthy preset)" section, sea level is a property of the *whole final elevation distribution*, computed once after subdivision completes, not by clamping radius mid-recursion:

> 1. Generate the full `Mesh` via subdivision (no ocean-specific logic in `subdivide.rs`)
> 2. If `preset.ocean_quota` is `Some(q)`: sort all final vertex radii, take the value at the `q`-th percentile **by vertex count** (approximate — not area-weighted) → this is `sea_level`
> 3. **Flatten**: any vertex with radius `< sea_level` gets its radius raised to exactly `sea_level`, producing a literal constant-radius "ocean" region
> 4. Color every vertex via `ColorGradient::sample(final_radius)` — ocean vertices, sharing the same radius, render with the same color

This feature adds: a validated `OceanQuota` newtype, a pure `apply_ocean_quota(mesh, quota) -> Result<Mesh, MeshError>` processor function implementing the algorithm above, a new `ocean_quota: Option<OceanQuota>` field on `PresetParams` (Earthy carries `Some`, Volcano/Rocky carry `None`), and a **composable post-processing pipeline** inside `Planet::subdivide` that `apply_ocean_quota` becomes the first stage of. It does **not** touch `planet-renderer` — `Planet::subdivide` already reads every knob from `self.preset.params()` internally, so `app.rs`'s existing `Planet::builder()...build()` / `.subdivide(..)` call sites pick up ocean-quota flattening automatically, with no code change, the moment `Preset::Earthy` carries a quota.

**Why a composable pipeline, not a single `if let Some(quota) = ...` call:** ocean quota is the first of what will be a series of whole-mesh post-processing steps applied after subdivision completes, each independently enabled by its own optional `PresetParams` field (a future preset might, say, carry both an ocean quota *and* a separate post-subdivision step with no relationship to it). `subdivision/strategies` already solves the analogous problem one level down — composing several optional *per-vertex* operations (`radial_displacement`, `normal_displacement`) into one pipeline via `processor/vertex_operator.rs`'s `VertexOperator` type alias, `processor/identity.rs`'s `identity()` neutral element, and `processor/compose.rs`'s pairwise `compose(first, second)` combinator, each applied left-to-right. This feature introduces the exact same three-piece shape one level up, for *whole-mesh* transformations: `MeshProcessor` (`processor/mesh_processor.rs`), `identity_mesh()` (`processor/identity_mesh.rs`), `compose_mesh(first, second)` (`processor/compose_mesh.rs`) — all `pub(crate)`, matching the per-vertex trio's exact visibility (internal composition machinery, never called directly outside `planet-core`, exposed to consumers only through `Planet::subdivide`'s resulting `Mesh`). `Planet::subdivide` builds its pipeline by folding every configured optional post-processing knob onto `identity_mesh()` via `compose_mesh`, in a fixed, one-line-per-knob sequence — so the *next* post-processing step this roadmap phase adds requires exactly one new optional `PresetParams` field and one new `if let Some(cfg) = params.new_field() { ... }` line in that fold, no other change to `Planet::subdivide`'s shape or to any other part of this pipeline.

The one behavioral difference from the per-vertex `compose`: `MeshProcessor` is fallible (`Fn(&Mesh) -> Result<Mesh, MeshError>`, since `apply_ocean_quota` and every future whole-mesh step propagate `Mesh::new`'s validation), where `VertexOperator` is infallible. `compose_mesh` therefore short-circuits — if the first stage's `Result` is `Err`, the second stage never runs and the error propagates immediately — a case the per-vertex `compose` has no equivalent of.

**Why `OceanQuota` is a validated newtype, not a raw `Option<f32>` field:** every other `PresetParams` field (`MinEdgeLength`, `ElevationNoiseRange`, `NormalNoiseRange`, `SplitPointVariance`) is a validated newtype with its own `Error` type, per `rules.md`'s "constructors that validate invariants ... return `Result` with a dedicated `Error` type." `000-architecture.md`'s domain-model listing writes every one of these fields as a bare primitive type (`min_edge_length: f32`, `ocean_quota: Option<f32>`) — that doc predates the newtype convention specs `007`–`012` actually established, so its primitive annotations are not binding on implementation. A quota outside `0.0..=1.0` is meaningless (a negative or >100% fraction of vertices), so it gets the same construction-time validation as every sibling field.

**Why `OceanQuota` and `apply_ocean_quota` share one file (`processor/ocean_quota.rs`) instead of splitting the type into its own file the way `vertex_scramble_range.rs` sits beside `vertex_scramble.rs`:** `VertexScrambleRange` and `scramble_vertices` are independently meaningful — `VertexScrambleRange` has its own construction/validation identity apart from the one function that happens to consume it today. `OceanQuota`, by contrast, exists solely to be consumed by this one function, immediately — the same relationship `subdivide.rs` already has between its `pub(crate) trait SubdivisionStrategy` and its public `subdivide` function, both declared in one file. That precedent (a tightly-coupled type + the single function built around it, in one file) is followed here rather than the `vertex_scramble_range.rs`/`vertex_scramble.rs` precedent (two independently-reusable types). Test files still split along the same two concerns as if they were separate (see "BDD scenarios" below), keeping construction-validation tests and mesh-transformation-behavior tests from mixing in one sprawling feature file.

## Requirements

- `planet-core` gains `processor/ocean_quota.rs` (new file in the existing `processor/` concern, added to `rules.md`'s module-structure list):
  - `pub struct OceanQuota(pub(crate) f32)` — `pub(crate)` field, matching `MinEdgeLength`'s exact pattern, so `preset.rs`'s compile-time-known-valid literals can be constructed directly (`OceanQuota(0.4)`) without going through `::new()`/`.expect(..)` in production code
  - `pub enum OceanQuotaError { OutOfRange { value: f32 } }`, `Display`/`std::error::Error` impls, mirroring `MinEdgeLengthError`'s shape
  - `OceanQuota::new(value: f32) -> Result<OceanQuota, OceanQuotaError>` — `Ok` iff `(0.0..=1.0).contains(&value)`, else `Err(OceanQuotaError::OutOfRange { value })`
  - `OceanQuota::value(&self) -> f32`
  - `#[derive(Debug, Clone, Copy, PartialEq)]` on `OceanQuota` (matches `MinEdgeLength`); `impl Default for OceanQuota` returning `OceanQuota(0.3)` (a `DEFAULT_OCEAN_QUOTA: f32 = 0.3` constant, mirroring `MinEdgeLength`'s exact `DEFAULT_MIN_EDGE_LENGTH` pattern) — every one of the 5 existing validated newtypes (`MinEdgeLength`, `ElevationNoiseRange`, `NormalNoiseRange`, `SplitPointVariance`, `VertexScrambleRange`) carries a `Default` impl whose only actual caller is its own BDD step-def file (confirmed: none of the 5 are used for any production fallback path either), purely to support a standard "the default `<Type>` has value/low/high `<...>`" scenario — `OceanQuota` follows the same unbroken convention rather than being a one-off exception
  - `pub fn apply_ocean_quota(mesh: &Mesh, quota: OceanQuota) -> Result<Mesh, MeshError>` — the whole-mesh post-processing function:
    1. Collect each vertex's radius: `let mut radii: Vec<f32> = mesh.vertices().iter().map(|v| v.position.length()).collect();`
    2. If `radii.is_empty()`, return `Ok(mesh.clone())` immediately (no percentile is defined for zero vertices, and there is nothing to flatten)
    3. `radii.sort_by(f32::total_cmp);` — NaN-safe, no `unwrap()`/`panic!()` (vertex radii are always finite in practice, but `total_cmp` never panics regardless)
    4. `let index = ((quota.value() * radii.len() as f32) as usize).min(radii.len() - 1);`
    5. `let sea_level = radii[index];`
    6. Map every vertex: if `vertex.position.length() < sea_level`, replace it with `vertex.position.normalized()` scaled to `sea_level` (`Vertex { position: direction.scale(sea_level) }`); if `normalized()` is `None` (a vertex exactly at the origin), leave the vertex unchanged — mirrors `radial_displacement`'s and `scramble_vertices`'s existing zero-radius guard. Otherwise (radius `>= sea_level`) leave the vertex unchanged
    7. `Mesh::new(new_vertices, mesh.triangles().to_vec())` — reuses `Mesh`'s own index-bounds validation; topology (triangle indices) is untouched, so this call cannot fail in practice, but the `Result` is propagated via `?` regardless, matching every other processor function's shape
  - `processor.rs`'s sibling-module declarations gain `pub mod ocean_quota;` (alphabetically placed)
- `planet-core` gains three new `pub(crate)` files in `processor/` — the whole-mesh pipeline machinery, mirroring `vertex_operator.rs`/`identity.rs`/`compose.rs`'s exact shape and visibility one level up (per-vertex → whole-mesh):
  - `processor/mesh_processor.rs` — `pub(crate) type MeshProcessor = Box<dyn Fn(&Mesh) -> Result<Mesh, MeshError>>;`
  - `processor/identity_mesh.rs` — `pub(crate) fn identity_mesh() -> MeshProcessor`, returning `Box::new(|mesh: &Mesh| Ok(mesh.clone()))`
  - `processor/compose_mesh.rs` — `pub(crate) fn compose_mesh(first: MeshProcessor, second: MeshProcessor) -> MeshProcessor`, applying `first` then `second` (left-to-right, matching `compose`'s own documented order), short-circuiting via `?` if `first` returns `Err` (so `second` never runs)
  - Each gets its own `#[cfg(test)] mod tests` in-file (not a `tests/*.rs`/`.feature` pair) — matching exactly how `vertex_operator.rs`/`identity.rs`/`compose.rs` are tested today, since all three are `pub(crate)` and therefore invisible to the separate-crate `planet-core/tests/` suite
  - `processor.rs`'s sibling-module declarations gain `pub(crate) mod mesh_processor;`, `pub(crate) mod identity_mesh;`, `pub(crate) mod compose_mesh;` (alphabetically placed among the existing `pub(crate)` entries)
- `planet-core/src/presets/preset_params.rs`: `PresetParams` gains a 6th field `ocean_quota: Option<OceanQuota>`; `PresetParams::new` gains a 6th parameter of the same type/position (after `color_gradient`, matching field declaration order); new accessor `pub fn ocean_quota(&self) -> Option<OceanQuota>`
- `planet-core/src/presets/preset.rs`: all three `Preset::params()` match arms updated for the new `PresetParams::new` arity:
  - `Preset::Earthy` passes `Some(OceanQuota(0.4))` — the confirmed value for this feature: roughly the lowest 40% of Earthy's final vertex radii become a flat ocean
  - `Preset::Volcano` and `Preset::Rocky` both pass `None` — neither preset has a liquid ocean concept (matches `000-architecture.md`'s "Confirmed presets: Earthy (with `ocean_quota`), Volcano, Rocky")
- `planet-core/src/planets/planet.rs`: `Planet::subdivide` gains a private helper function and one wiring step between `subdivide()` returning and `colors` being sampled:
  ```rust
  fn postprocessing_pipeline(params: &PresetParams) -> MeshProcessor {
      let mut pipeline = identity_mesh();
      if let Some(quota) = params.ocean_quota() {
          pipeline = compose_mesh(pipeline, Box::new(move |mesh: &Mesh| apply_ocean_quota(mesh, quota)));
      }
      pipeline
  }
  ```
  ```rust
  let mesh = subdivide(&self.mesh, args)?;
  let mesh = postprocessing_pipeline(&params)(&mesh)?;
  let colors = mesh
      .vertices()
      .iter()
      .map(|vertex| params.color_gradient().sample(vertex.position.length()))
      .collect();
  ```
  `postprocessing_pipeline` is a plain private (module-private, not `pub`/`pub(crate)`) function local to `planet.rs` — it is `Planet::subdivide`'s own wiring logic (the same role that function already plays for `SubdivisionMode::RedGreenSplit`'s knobs), not a reusable domain type, so it needs no dedicated file or `rules.md` entry of its own. Its behavior is exercised end-to-end through `Planet::subdivide`'s own BDD scenarios (see below), the same way its existing `SubdivisionMode` wiring already is. `Planet::subdivide`'s new imports: `crate::processor::ocean_quota::apply_ocean_quota`, `crate::processor::mesh_processor::MeshProcessor`, `crate::processor::identity_mesh::identity_mesh`, `crate::processor::compose_mesh::compose_mesh`. No new `PlanetError` variant — every stage returns `Result<Mesh, MeshError>`, and `PlanetError`'s existing `From<MeshError>` impl already covers the pipeline's overall `Result` via `?`
- No change to `PlanetBuilder`/`Planet::builder()...build()` — creation never subdivides, so ocean-quota flattening (which only makes sense on a fully-subdivided mesh, per `000-architecture.md`) never runs at creation time; a freshly-built `Planet`'s mesh is still the untouched base icosahedron regardless of its preset's `ocean_quota`
- No change to `planet-renderer` — `app.rs` already obtains its demo `Planet` exclusively via `Planet::builder()...build()` / `.subdivide(..)` (per `013`'s crate-boundary migration), so it picks up ocean-quota flattening automatically once `Preset::Earthy` (its `DEMO_PRESET`) carries a quota
- `rules.md`'s `processor/` concern list gains: `ocean_quota.rs` (`OceanQuota`, `OceanQuotaError`, `apply_ocean_quota`) alongside the existing `vertex_scramble_range.rs`/`vertex_scramble.rs` entries, plus a new sentence in the same bullet naming the whole-mesh pipeline building blocks `Planet::subdivide` composes (`mesh_processor.rs` (`MeshProcessor`, `pub(crate)`), `identity_mesh.rs` (`identity_mesh`, `pub(crate)`), `compose_mesh.rs` (`compose_mesh`, `pub(crate)`)) — worded to mirror the existing sentence about the per-vertex `VertexOperator` building blocks (`vertex_operator.rs`, `identity.rs`, `radial_displacement.rs`, `normal_displacement.rs`, `compose.rs`) exactly

## Domain model involved

**`planet-core/src/processor/ocean_quota.rs` (new):**
```rust
use std::fmt;

use crate::geometry::mesh::{Mesh, MeshError, Vertex};

const DEFAULT_OCEAN_QUOTA: f32 = 0.3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OceanQuota(pub(crate) f32);

#[derive(Debug, Clone, PartialEq)]
pub enum OceanQuotaError {
    OutOfRange { value: f32 },
}

impl fmt::Display for OceanQuotaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OceanQuotaError::OutOfRange { value } => {
                write!(f, "ocean quota must be between 0.0 and 1.0, got {value}")
            }
        }
    }
}

impl std::error::Error for OceanQuotaError {}

impl OceanQuota {
    pub fn new(value: f32) -> Result<OceanQuota, OceanQuotaError> {
        if (0.0..=1.0).contains(&value) {
            Ok(OceanQuota(value))
        } else {
            Err(OceanQuotaError::OutOfRange { value })
        }
    }

    pub fn value(&self) -> f32 {
        self.0
    }
}

impl Default for OceanQuota {
    fn default() -> Self {
        OceanQuota(DEFAULT_OCEAN_QUOTA)
    }
}

pub fn apply_ocean_quota(mesh: &Mesh, quota: OceanQuota) -> Result<Mesh, MeshError> {
    let mut radii: Vec<f32> = mesh.vertices().iter().map(|v| v.position.length()).collect();
    if radii.is_empty() {
        return Ok(mesh.clone());
    }
    radii.sort_by(f32::total_cmp);
    let index = ((quota.value() * radii.len() as f32) as usize).min(radii.len() - 1);
    let sea_level = radii[index];

    let vertices = mesh
        .vertices()
        .iter()
        .map(|vertex| {
            let radius = vertex.position.length();
            if radius < sea_level {
                match vertex.position.normalized() {
                    Some(direction) => Vertex {
                        position: direction.scale(sea_level),
                    },
                    None => *vertex,
                }
            } else {
                *vertex
            }
        })
        .collect();

    Mesh::new(vertices, mesh.triangles().to_vec())
}
```

**`planet-core/src/processor/mesh_processor.rs` (new):**
```rust
use crate::geometry::mesh::{Mesh, MeshError};

pub(crate) type MeshProcessor = Box<dyn Fn(&Mesh) -> Result<Mesh, MeshError>>;
```

**`planet-core/src/processor/identity_mesh.rs` (new):**
```rust
use crate::processor::mesh_processor::MeshProcessor;

pub(crate) fn identity_mesh() -> MeshProcessor {
    Box::new(|mesh| Ok(mesh.clone()))
}

#[cfg(test)]
mod tests {
    use super::identity_mesh;
    use crate::geometry::mesh::{Mesh, Vertex};
    use crate::geometry::vec3::Vec3;

    #[test]
    fn identity_mesh_returns_the_mesh_unchanged() {
        let mesh = Mesh::new(
            vec![Vertex {
                position: Vec3::new(1.0, 2.0, 3.0),
            }],
            vec![],
        )
        .expect("valid mesh fixture");

        let result = identity_mesh()(&mesh).expect("identity never fails");

        assert_eq!(result, mesh);
    }
}
```

**`planet-core/src/processor/compose_mesh.rs` (new):**
```rust
use crate::geometry::mesh::Mesh;
use crate::processor::mesh_processor::MeshProcessor;

pub(crate) fn compose_mesh(first: MeshProcessor, second: MeshProcessor) -> MeshProcessor {
    Box::new(move |mesh: &Mesh| {
        let mesh = first(mesh)?;
        second(&mesh)
    })
}
```

**`planet-core/src/presets/preset_params.rs` (modified):** gains `ocean_quota: Option<OceanQuota>` field, `ocean_quota` parameter on `new`, `ocean_quota(&self) -> Option<OceanQuota>` accessor — otherwise unchanged from its current 5-field shape.

**`planet-core/src/presets/preset.rs` (modified):** each `PresetParams::new(..)` call gains a 6th argument — `Some(OceanQuota(0.4))` for `Preset::Earthy`, `None` for `Preset::Volcano` and `Preset::Rocky`.

**`planet-core/src/planets/planet.rs` (modified):** `Planet::subdivide` gains the private `postprocessing_pipeline` helper and the pipeline wiring step shown in "Requirements" above, plus the four corresponding `use` imports.

Existing types this feature calls but does not modify: `Mesh` / `Mesh::new` / `MeshError` / `Vertex` (`geometry/mesh.rs`), `Vec3::length`/`normalized`/`scale` (`geometry/vec3.rs`), `Planet`/`PlanetError` (`planets/planet.rs`, aside from the pipeline wiring step), `Preset` (`presets/preset.rs`), `PresetParams` (aside from the new field/accessor).

## Function/API contracts

### `OceanQuota::new`

```rust
pub fn new(value: f32) -> Result<OceanQuota, OceanQuotaError>
```

- **Pre:** `value` is any `f32`
- **Post:** `Ok(OceanQuota(value))` iff `0.0 <= value <= 1.0` (inclusive both ends); `Err(OceanQuotaError::OutOfRange { value })` otherwise (including negative values, values `> 1.0`, and non-finite values, since `(0.0..=1.0).contains` is `false` for `NaN`/`±inf`)
- `OceanQuota::default()` returns `OceanQuota(0.3)` — a fixed, valid constant, matching the "every validated newtype gets a `Default`" convention shared by `MinEdgeLength`/`ElevationNoiseRange`/`NormalNoiseRange`/`SplitPointVariance`/`VertexScrambleRange`

### `apply_ocean_quota`

```rust
pub fn apply_ocean_quota(mesh: &Mesh, quota: OceanQuota) -> Result<Mesh, MeshError>
```

- **Pre:** `mesh` is any valid `Mesh` (zero or more vertices); `quota` is any valid `OceanQuota` (already validated at construction, `0.0..=1.0`)
- **Post:**
  - Returns `Ok(Mesh)` for every valid `(mesh, quota)` pair — `Mesh::new`'s only failure mode is an out-of-bounds triangle index, which cannot occur since triangle indices and vertex count are passed through unchanged
  - Vertex count and triangle list are identical between input and output (only vertex *positions* may change)
  - No returned vertex has a radius strictly less than the computed `sea_level` (the value at the `quota`-th percentile, by vertex count, of the input mesh's sorted radii) — every vertex that was below `sea_level` is raised to exactly `sea_level`; every vertex already at or above `sea_level` is returned bit-identical to its input
  - Deterministic: identical `(mesh, quota)` always produces a bit-identical output `Mesh` — the function is a pure computation over its inputs with no randomness
  - `quota` of `OceanQuota::new(0.0).unwrap()` (only in test code) is a no-op: `sea_level` resolves to the mesh's own minimum radius, so no vertex is strictly below it
  - `quota` of `OceanQuota::new(1.0).unwrap()` flattens every vertex to the mesh's own maximum radius (every vertex except the one(s) already at the max gets raised to it)
  - A `mesh` with zero vertices returns `Ok` with the input mesh unchanged (no percentile is computable, nothing to flatten)
  - A vertex exactly at the origin (radius `0.0`) is never mutated and never causes a panic, regardless of `quota` — mirrors `scramble_vertices`'/`radial_displacement`'s existing zero-radius guard

### `identity_mesh`

```rust
pub(crate) fn identity_mesh() -> MeshProcessor
```

- **Pre:** none
- **Post:** the returned `MeshProcessor`, called with any `&Mesh`, always returns `Ok` with a `Mesh` bit-identical to its input — the neutral element for `compose_mesh` folding

### `compose_mesh`

```rust
pub(crate) fn compose_mesh(first: MeshProcessor, second: MeshProcessor) -> MeshProcessor
```

- **Pre:** `first`/`second` are any `MeshProcessor` values
- **Post:**
  - Calling the returned `MeshProcessor` with `mesh` is equivalent to calling `first(mesh)`, and — only if that returned `Ok(intermediate)` — calling `second(&intermediate)`, returning that call's result directly
  - If `first(mesh)` returns `Err`, the returned `MeshProcessor` returns that same `Err` immediately and `second` is never invoked (short-circuit; the one behavioral difference from the always-infallible per-vertex `compose`)
  - `compose_mesh(identity_mesh(), p)` and `compose_mesh(p, identity_mesh())` both behave identically to `p` alone, for any `MeshProcessor` `p` (identity-element law)

### `Planet::subdivide` (updated contract)

All of spec `013`'s existing postconditions for `Planet::subdivide` continue to hold unchanged (determinism, `max_depth` honored, `colors().len() == mesh().vertices().len()`), with one clarification surfaced by this feature's own TDD cycle: `on_progress`'s per-round invocations report `subdivide()`'s own raw, pre-post-processing `Mesh` at each round (unchanged from `013` — the callback is wired directly into `subdivide()`'s internal per-round reporting, upstream of `postprocessing_pipeline`), so the callback's *last* invocation's mesh is only guaranteed to equal the returned `Planet`'s `mesh()` when no post-processing step ran (i.e. `ocean_quota()` is `None`) — this was always implicit in `013`'s wiring order but had no way to be observed before this feature introduced the first non-identity post-processing step. `013`'s "reports the base mesh and every subdivision round" scenario is updated from `Preset::Earthy` to `Preset::Volcano` (`ocean_quota: None`) to keep asserting that exact equality meaningfully, rather than weakening the assertion itself. This feature adds:
- If `self.preset().params().ocean_quota()` is `Some(quota)`, the returned `Planet`'s `mesh()` is exactly `apply_ocean_quota(&subdivided_mesh, quota)`'s output (where `subdivided_mesh` is what `subdivide()` itself would have returned) — i.e. flattening happens strictly after subdivision, before color sampling. This holds regardless of whether it is expressed as a direct call or, as this feature implements it, as one stage of `postprocessing_pipeline`'s composed `MeshProcessor` — the pipeline's *observable* behavior for a preset with only an ocean quota configured is identical to calling `apply_ocean_quota` directly
- If `self.preset().params().ocean_quota()` is `None`, `Planet::subdivide`'s behavior is byte-for-byte identical to `013`'s (`postprocessing_pipeline` returns `identity_mesh()` unchanged, a true no-op) — this covers `Preset::Volcano` and `Preset::Rocky` unconditionally, and confirms this feature does not change their behavior at all
- For `max_depth = Steps::new(0).unwrap()`: the pre-subdivision mesh passed into the (no-op, since 0 rounds ran) pipeline is the base icosahedron, whose 12 vertices all share the same radius (bit-identical, since `Mesh::icosahedron()` scales every vertex by the same precomputed factor from equal-magnitude coordinate triples) — so the ocean-quota stage is a true no-op here too, and `013`'s existing "Generating a Planet at zero max depth is exactly the base icosahedron, colored" scenario (which already uses the `Earthy` preset) continues to hold unmodified

## BDD scenarios

### `planet-core/tests/features/ocean_quota.feature` (new — `OceanQuota` construction/validation, matching `min_edge_length.feature`/`elevation_noise_range.feature`/`normal_noise_range.feature`/`split_point_variance.feature`/`vertex_scramble_range.feature`'s exact shared idiom: scenario titles of the form "Constructing a `<Type>` with `<condition>` succeeds/fails", a `When a `<Type>` is constructed with value `<v>`` step, `Then the `<Type>` is constructed successfully` + `And the `<Type>` has value `<v>`` for success, `Then the construction fails with a(n) `<error-name>` error of `<v>`` naming the actual error variant for failure, a `NaN` scenario (present on both other single-`f32` newtypes, `MinEdgeLength`/`SplitPointVariance`), and a "the default `<Type>` has value `<v>`" scenario)

```gherkin
Feature: Constructing a validated OceanQuota

  Scenario: Constructing an OceanQuota with a value within 0.0 and 1.0 succeeds
    When an OceanQuota is constructed with value 0.4
    Then the OceanQuota is constructed successfully
    And the OceanQuota has value 0.4

  Scenario: Constructing an OceanQuota with the boundary value 0.0 succeeds
    When an OceanQuota is constructed with value 0.0
    Then the OceanQuota is constructed successfully
    And the OceanQuota has value 0.0

  Scenario: Constructing an OceanQuota with the boundary value 1.0 succeeds
    When an OceanQuota is constructed with value 1.0
    Then the OceanQuota is constructed successfully
    And the OceanQuota has value 1.0

  Scenario: Constructing an OceanQuota with a negative value fails
    When an OceanQuota is constructed with value -0.1
    Then the construction fails with an out-of-range error of -0.1

  Scenario: Constructing an OceanQuota with a value above 1.0 fails
    When an OceanQuota is constructed with value 1.5
    Then the construction fails with an out-of-range error of 1.5

  Scenario: Constructing an OceanQuota with NaN fails
    When an OceanQuota is constructed with value NaN
    Then the construction fails with an out-of-range error of NaN

  Scenario: The default OceanQuota has value 0.3
    Given the default OceanQuota
    Then the OceanQuota has value 0.3
```

### `planet-core/tests/features/apply_ocean_quota.feature` (new — mesh-flattening behavior, mirrors `vertex_scramble.feature`'s style)

```gherkin
Feature: Flattening a mesh's lowest-radius vertices to a shared sea level

  Scenario: Flattening raises every vertex below the computed sea level to a shared radius
    Given a Mesh with vertices at radii 0.9, 1.0, 1.1, 1.2
    When that mesh is flattened with an OceanQuota of 0.5
    Then the resulting Mesh has vertex radii 1.1, 1.1, 1.1, 1.2

  Scenario: Flattening with an OceanQuota of 0.0 leaves the mesh unchanged
    Given a Mesh with vertices at radii 0.9, 1.0, 1.1
    When that mesh is flattened with an OceanQuota of 0.0
    Then the resulting Mesh is identical to the original mesh

  Scenario: Flattening with an OceanQuota of 1.0 raises every vertex to the mesh's maximum radius
    Given a Mesh with vertices at radii 0.9, 1.0, 1.1
    When that mesh is flattened with an OceanQuota of 1.0
    Then the resulting Mesh has vertex radii 1.1, 1.1, 1.1

  Scenario: Flattening preserves vertex count and triangle topology
    Given an icosahedron mesh
    When the icosahedron mesh is flattened with an OceanQuota of 0.4
    Then the resulting Mesh has 12 vertices
    And the resulting Mesh has the same triangles as the icosahedron mesh

  Scenario: Flattening a mesh with all-equal radii is a no-op
    Given an icosahedron mesh
    When the icosahedron mesh is flattened with an OceanQuota of 0.4
    Then the resulting Mesh is identical to the icosahedron mesh

  Scenario: Flattening never panics when a vertex sits exactly at the origin
    Given a Mesh with a vertex exactly at the origin
    When that mesh is flattened with an OceanQuota of 0.9
    Then no panic occurs

  Scenario: Flattening an empty mesh is a no-op
    Given a Mesh with no vertices and no triangles
    When that mesh is flattened with an OceanQuota of 0.5
    Then the resulting Mesh is identical to the original mesh
```

### `planet-core/tests/features/planet.feature` (extended)

Per `rules.md`'s "every preset-related feature file covers ... for presets with an ocean quota, the fraction of vertices at sea level matches the configured quota within tolerance" — this scenario is new to `planet.feature`:

```gherkin
  Scenario: A Planet generated with the Earthy preset has approximately its configured ocean quota's fraction of vertices at sea level
    Given a Planet generated with seed 11 and the Earthy preset at max depth 4
    Then the fraction of the resulting Planet's mesh vertices at its minimum vertex radius is within 0.05 of the Earthy preset's configured OceanQuota
```

### `planet-core/tests/features/preset.feature` / `preset_params.feature` (extended)

Existing per-preset scenarios gain an `OceanQuota` assertion line each:
- Earthy: `And the PresetParams has an OceanQuota of 0.4`
- Volcano: `And the PresetParams has no OceanQuota`
- Rocky: `And the PresetParams has no OceanQuota`

`preset_params.feature`'s two existing scenarios ("Constructing PresetParams bundles all 5 fields unchanged", "Two PresetParams built from identical arguments are equal") are updated to 6 fields, with an `OceanQuota` value added to the fixture and a matching `Then the PresetParams has an OceanQuota of <value>` step.

## Acceptance criteria

1. `planet-core` gains `processor/ocean_quota.rs` (`OceanQuota`, `OceanQuotaError`, `apply_ocean_quota`), declared in `processor.rs`'s sibling-module list and added to `rules.md`'s `processor/` concern entry
2. `OceanQuota::new(value) -> Result<OceanQuota, OceanQuotaError>` accepts exactly `0.0..=1.0` inclusive and rejects everything else, including `NaN` (unit/BDD test, covering both boundary values, both out-of-range directions, and `NaN`); `OceanQuota::default()` returns `OceanQuota(0.3)` (unit/BDD test)
3. `apply_ocean_quota(mesh, quota) -> Result<Mesh, MeshError>` never returns a vertex with radius strictly less than the quota-th-percentile-by-count `sea_level` computed from the input mesh's own vertex radii (unit/BDD test)
4. `apply_ocean_quota` leaves every vertex already at or above `sea_level` bit-identical to its input, and preserves vertex count and triangle topology exactly (unit/BDD test)
5. `apply_ocean_quota` with `OceanQuota` value `0.0` is a no-op on any mesh (unit/BDD test)
6. `apply_ocean_quota` with `OceanQuota` value `1.0` raises every vertex to the mesh's own maximum radius (unit/BDD test)
7. `apply_ocean_quota` never panics on a mesh containing a vertex exactly at the origin, for any valid `OceanQuota` (unit/BDD test)
8. `apply_ocean_quota` never panics on an empty mesh (zero vertices, zero triangles) and returns it unchanged (unit/BDD test)
9. `apply_ocean_quota` is deterministic: identical `(mesh, quota)` always produces a bit-identical output `Mesh` (unit test)
10. `PresetParams` gains a 6th field `ocean_quota: Option<OceanQuota>`, a matching 6th `new` parameter, and an `ocean_quota(&self) -> Option<OceanQuota>` accessor (unit/BDD test)
11. `Preset::Earthy.params().ocean_quota()` equals `Some(OceanQuota(0.4))`; `Preset::Volcano.params().ocean_quota()` and `Preset::Rocky.params().ocean_quota()` both equal `None` (unit/BDD test)
12. `MeshProcessor`, `identity_mesh()`, `compose_mesh(first, second)` exist as `pub(crate)` in `processor/mesh_processor.rs`/`identity_mesh.rs`/`compose_mesh.rs`, added to `rules.md`'s `processor/` concern entry (compile-time check + in-file unit tests)
13. `identity_mesh()()` always returns its input mesh unchanged (in-file unit test, mirroring `identity.rs`'s own existing test)
14. `compose_mesh(first, second)` applies `first` then `second`, in that order, on a `mesh` where the two stages are independently observable (e.g. one changes vertex count semantics, the other flags it was called) (in-file unit test, mirroring `compose.rs`'s own existing "applies first then second" test)
15. `compose_mesh(first, second)` short-circuits: if `first` returns `Err`, the composed processor returns that `Err` immediately and `second` is never invoked (in-file unit test — this has no equivalent in the infallible per-vertex `compose` and is new coverage this feature adds)
16. `Planet::subdivide`'s private `postprocessing_pipeline` helper builds its `MeshProcessor` by folding `identity_mesh()` with `compose_mesh` once per configured optional post-processing knob on `params` — today, exactly one such knob (`ocean_quota`) — so a `Planet` whose preset has `ocean_quota: None` gets `identity_mesh()` itself as its pipeline (a true no-op), and one whose preset has `ocean_quota: Some(quota)` gets a pipeline equivalent to calling `apply_ocean_quota(_, quota)` directly (BDD test, via `Planet::subdivide`'s existing scenarios)
17. For a `Planet` created with `Preset::Earthy`, `.subdivide(max_depth, _)` (`max_depth > 0`) produces a `mesh()` where no vertex radius is below that mesh's own computed `sea_level`, and the fraction of vertices sitting at the mesh's minimum radius is within `0.05` of `0.4` (the configured quota) for a mesh with enough vertices to make the percentile meaningful (e.g. `max_depth >= 4`) (BDD test)
18. For a `Planet` created with `Preset::Volcano` or `Preset::Rocky`, `.subdivide(max_depth, _)`'s resulting `mesh()` is byte-for-byte identical to what `013`'s pre-ocean-quota `Planet::subdivide` would have produced (i.e. `apply_ocean_quota` is never invoked for these presets, and the pipeline resolves to `identity_mesh()`) (unit/BDD test)
19. `013`'s existing "Generating a Planet at zero max depth is exactly the base icosahedron, colored" scenario (using `Preset::Earthy`) continues to pass unmodified — the post-processing pipeline is a no-op on the pristine icosahedron
20. `PlanetError` gains no new variant; every pipeline stage's `Result<Mesh, MeshError>` propagates through `Planet::subdivide` via the existing `From<MeshError> for PlanetError` impl and `?`
21. `apply_ocean_quota`/`OceanQuota::new`/`identity_mesh`/`compose_mesh`/`postprocessing_pipeline` contain no `unwrap()`/`panic!()`/`.expect()` in production code (`radii.sort_by(f32::total_cmp)` is the NaN-safe, non-panicking sort used instead of `partial_cmp().unwrap()`)
22. No changes to `planet-renderer` — `app.rs` is untouched by this feature
23. All BDD scenarios above are backed by real `cucumber` step definitions in their respective `.feature` files and matching step-definition modules (`tests/ocean_quota.rs`, `tests/apply_ocean_quota.rs`, extensions to `tests/planet.rs`, `tests/preset.rs`, `tests/preset_params.rs`); `MeshProcessor`/`identity_mesh`/`compose_mesh` are covered by in-file `#[cfg(test)]` unit tests instead (per their `pub(crate)` visibility) — no scenario is left as markdown prose and no `pub(crate)` behavior is left untested
24. Build gate passes: `cargo test --workspace && cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo build --target wasm32-unknown-unknown -p planet-renderer`
