# 013 — Planet Aggregate Root

**Status:** Ready for review
**Feature slug:** `planet-aggregate-root`

This is the second slice of `docs/roadmap.md`'s "007 — Planet presets" phase, continuing after `012-preset-color-gradient` (which shipped `Rgb`/`ColorGradient`/`PresetParams`/`Preset` as standalone, directly-constructible `planet-core` value types, explicitly deferring the `Planet` aggregate root itself). This feature ships that aggregate root: `Planet::generate(preset, seed, max_depth, on_progress)`, wiring the preset's subdivision knobs into the existing `subdivide`/`SubdivisionMode::RedGreenSplit` pipeline and producing one `Rgb` per vertex via the preset's `ColorGradient`.

**`Planet` is also established as `planet-core`'s intended entry point for every consumer outside the crate** — `planet-renderer` (and any future consumer) must obtain every generated `Mesh` via `Planet`/`Planet::generate`, never by calling `Mesh::icosahedron()`, `subdivide()`, `SubdivisionMode`, `scramble_vertices()`, or any other generation primitive directly. This is a **documentation/review convention, not a compiler-enforced one**: every one of those types stays `pub` (see "Why visibility stays as-is" below), because `planet-core`'s own BDD/unit test suite lives under `planet-core/tests/`, which Rust compiles as a separate crate that can only see `pub` items, never `pub(crate)` — locking these down for real would break all 17 existing test files (`subdivide.rs`, `preset.rs`, `preset_params.rs`, `vertex_scramble.rs`, `subdivision_args.rs`, and more) built up over specs `004`–`012`, and reconciling that would mean migrating that entire test suite from `tests/*.rs` into in-crate `#[cfg(test)]` modules — a large, disruptive rewrite of the test architecture that is out of scope for adding one aggregate root. So the boundary is enforced the same way `rules.md`'s existing module-structure convention already is: **at `planet-pr-validate` review time**, not by `cargo build`.

Because `planet-renderer`'s `app.rs` currently violates this convention today — it directly imports and calls `Mesh::icosahedron()`, `subdivide()`, `SubdivisionArgs`, `SubdivisionMode`, `scramble_vertices()`, `Seed`, and all four noise-range types to drive its demo animation — this feature **does** touch `planet-renderer`, despite `012`'s precedent of pure-domain slices with no renderer touch. `app.rs` is rewired to call `Planet::generate` exclusively (see "`app.rs` migration" below). This is the one exception to `constitution.md`'s core-first ordering this feature makes, and only because the new convention requires it to hold from the moment it's introduced — not because this feature is doing renderer/UI work ahead of schedule.

Ocean-quota sea-level flattening (`000-architecture.md`'s "Ocean quota (Earthy preset)" section) is **not** part of this feature — `PresetParams` has no `ocean_quota` field yet (`012` deferred it), and adding a whole percentile-and-flatten post-processing step on top of first wiring the aggregate root together would bundle two independently reviewable changes into one slice, breaking with the narrow-increment pattern every prior spec in this phase (`007`, `009`, `010`, `011`, `012`) has followed. It lands in a later, higher-numbered spec once this one is merged — shaped as a `processor/` whole-mesh post-processing function (e.g. `processor/ocean_quota.rs`, taking `&Mesh` and returning `Result<Mesh, MeshError>`), mirroring `processor/vertex_scramble.rs`'s existing `scramble_vertices(mesh, seed, range) -> Result<Mesh, MeshError>` shape exactly, per `rules.md`'s definition of `processor/` as "whole-mesh pre/post-processing steps that run outside the subdivision algorithm, each taking an already-built `Mesh` and returning a transformed one." That future spec's `Planet::generate` will call it once, on the fully-subdivided `Mesh`, after `subdivide()` returns and before per-vertex color sampling (color must be sampled from each vertex's *final*, possibly-flattened radius).

A `Preset` dropdown and a depth slider remain out of scope — those are genuine UI-control work for a later spec, unaffected by the `app.rs` migration this feature does make (which only changes *how* `app.rs`'s existing hardcoded demo obtains its `Mesh`, not what it lets the user control).

## Requirements

- `planet-core` gains a new top-level concern, `planets/` (sibling to `geometry/`, `subdivision/`, `processor/`, `color/`, `presets/`), holding the two new public items this feature introduces. Named `planets/` (plural), not `planet/` (singular) — mirroring `012`'s identical `presets/`-not-`preset/` naming decision: the concern's own primary type is `Planet`, and a `planet/planet.rs` file layout resolves to module path `planet::planet`, tripping clippy's default-on `module_inception` lint and failing the mandatory `-D warnings` build gate. `rules.md`'s "Module structure" section gains a new `planet-core` concern-list entry: `planets/` (`planet.rs` — `Planet`, `GenerationProgress`)
- `planet-core` gains a new public type `Planet` (`planet-core/src/planets/planet.rs`) — the aggregate root `000-architecture.md` describes as "the only type with a lifecycle." Fields (all private): `mesh: Mesh`, `colors: Vec<Rgb>`, `preset: Preset`. Accessors: `mesh(&self) -> &Mesh`, `colors(&self) -> &[Rgb]`, `preset(&self) -> Preset`. `colors()[i]` is always the color of `mesh().vertices()[i]` — the two slices are the same length and index-aligned by construction; there is no combined `(Vertex, Rgb)` pair type, since `Mesh`/`Vertex` are untouched by this feature and a zipped accessor is trivial to build at any call site via `mesh().vertices().iter().zip(colors())`. `Planet` derives `Debug, Clone, PartialEq` — not `Copy` (blocked transitively by `Mesh`'s and `Vec<Rgb>`'s owned `Vec`s), not `Eq` (blocked transitively by every `f32` position/channel reachable through `Mesh`/`Rgb`)
- `planet-core` gains a new public type alias `GenerationProgress` (`planet-core/src/planets/planet.rs`): `pub type GenerationProgress = Box<dyn FnMut(&Mesh, usize)>;` — structurally identical to `subdivision_args::UpdateCallback` (same underlying `Box<dyn FnMut(&Mesh, usize)>` shape) but declared fresh in the `planets/` concern so a consumer wiring a progress callback through `Planet::generate` never needs to reach into `subdivision::subdivision_args` directly, keeping `Planet`'s own module self-contained as the one thing external code imports from
- `Planet::generate(preset: Preset, seed: Seed, max_depth: Steps, on_progress: Option<GenerationProgress>) -> Result<Planet, MeshError>` — the sole public constructor, and the only way to obtain a `Planet`. Reuses the existing `Steps` type for the recursion-depth argument rather than inventing `000-architecture.md`'s `SubdivisionDepth` newtype: `Steps` already is exactly that type (a validated `usize` hard-capped at `MAX_SUBDIVISION_STEPS = 8`), and `009-irregular-subdivision`/`012-preset-color-gradient` both established the precedent of reusing an existing, already-shipped type over introducing a redundant one. `MeshError` is `Planet::generate`'s error type because both `Mesh::icosahedron()` and `subdivide()` — the two fallible calls this function makes internally — already return `Result<Mesh, MeshError>`; `Planet::generate` propagates via `?` rather than `.expect(...)`-ing a call that never actually fails for these fixed, valid inputs, per `rules.md`'s "No `unwrap()`/`panic!()` in production code" rule
- **`on_progress` callback contract:** when `Some`, the callback is invoked once with the freshly constructed, pre-subdivision base icosahedron and round `0`, then once per completed subdivision round (`1..=max_depth.value()`) with that round's `Mesh` — exactly mirroring `subdivide`'s own existing per-round `update_cb` semantics, plus the one extra round-`0` invocation for the base mesh (which `subdivide` itself has no opportunity to report, since it is only ever handed an already-built mesh). This lets a consumer reconstruct the exact same "watch it subdivide" animation `app.rs` already builds today, without reaching around `Planet` to call `subdivide` directly. When `max_depth` is `Steps::new(0)`, the callback is still invoked exactly once (round `0`, the base icosahedron) — subdivision itself runs zero rounds, but the base-mesh notification is unconditional
- **Algorithm**, in order:
  1. `let params = preset.params();`
  2. `let base = Mesh::icosahedron()?;`
  3. `let mut on_progress = on_progress;` then, if `Some`, invoke it once with `(&base, 0)`
  4. Build `SubdivisionArgs::new(Some(max_depth), Some(SubdivisionMode::RedGreenSplit { seed, elevation_noise_range: params.elevation_noise_range(), normal_noise_range: params.normal_noise_range(), min_edge_length: params.min_edge_length(), split_point_variance: params.split_point_variance() }), on_progress)` — `on_progress` (type `Option<GenerationProgress>`) is passed directly as `SubdivisionArgs::new`'s third parameter (type `Option<UpdateCallback>`); since `GenerationProgress` and `UpdateCallback` are both type aliases for the identical `Box<dyn FnMut(&Mesh, usize)>`, no conversion is needed. Every other `RedGreenSplit` field comes directly from an accessor `PresetParams` already exposes; no new accessor or field is added to `PresetParams` by this feature
  5. `let mesh = subdivide(&base, args)?;`
  6. `let colors = mesh.vertices().iter().map(|vertex| params.color_gradient().sample(vertex.position.length())).collect();` — elevation is the vertex's radius (distance from the mesh origin), matching every existing radial-displacement/ocean-quota reference to "radius"/"elevation" in this codebase
  7. `Ok(Planet { mesh, colors, preset })`
- **Why visibility stays as-is:** no change to any existing type's `pub`/`pub(crate)` status — `Mesh`, `Vertex`, `MeshError`, `Vec3`, `Triangle`, `Seed`, `Steps`, `PresetParams`, `Preset`, `ColorGradient`, `Rgb`, `SubdivisionMode`, `SubdivisionArgs`, `subdivide`, `scramble_vertices`, `VertexScrambleRange`, `EdgeCache`, and every existing strategy keep exactly the visibility they have today (see the crate-boundary convention above for why: `planet-core/tests/` needs `pub` to compile at all). `PresetParams` gains no `ocean_quota` field in this feature either (deferred, see above)
- `rules.md` gains a new "Crate boundaries" rule (sibling to "Module structure"): consumers of `planet-core` — currently only `planet-renderer` — must obtain every generated `Mesh` via `Planet`/`Planet::generate`, never via `Mesh::icosahedron()`/`subdivide()`/`SubdivisionMode`/`scramble_vertices()`/any other generation primitive directly. Reading an already-obtained `Mesh`'s own data (`vertices()`/`triangles()`, e.g. `planet-renderer`'s `gpu/buffers.rs`) is unaffected — the rule is about how a `Mesh` is *produced*, not how its data is *read*. Enforced at `planet-pr-validate` review time, exactly like the existing module-structure convention, not by the compiler (see above for why a `pub(crate)` lockdown isn't used instead)

### `app.rs` migration

`planet-renderer/src/app.rs` currently:
- imports `planet_core::geometry::mesh::Mesh`, `processor::vertex_scramble::scramble_vertices`, `processor::vertex_scramble_range::VertexScrambleRange`, `subdivision::{elevation_noise_range, min_edge_length, normal_noise_range, seed, split_point_variance, subdivide, subdivision_args, subdivision_mode}` directly
- builds the base mesh as `Mesh::icosahedron()` then `scramble_vertices(&base_mesh, Seed::from(DEMO_SCRAMBLE_SEED), VertexScrambleRange::default())`
- seeds its `collected_frames` animation buffer with that scrambled base mesh, then calls `subdivide(&base_mesh, args)` with a hand-built `SubdivisionArgs`/`SubdivisionMode::RedGreenSplit` (using `ElevationNoiseRange::default()`, `NormalNoiseRange::default()`, `MinEdgeLength::default()`, `SplitPointVariance::default()`) whose `update_cb` pushes each round's `Mesh` into `collected_frames`

This feature changes `app.rs` to:
- import only `planet_core::geometry::mesh::Mesh` (for the `frames: Vec<Mesh>` field's type and the progress-callback signature) and `planet_core::planets::planet::{Planet, GenerationProgress}` plus `planet_core::presets::preset::Preset` and `planet_core::subdivision::seed::Seed`/`steps::Steps` — no more direct `subdivision::subdivide`/`subdivision_args`/`subdivision_mode`/noise-range imports, and no more `processor::vertex_scramble`/`vertex_scramble_range` imports
- replace `DEMO_SCRAMBLE_SEED`/the `scramble_vertices` call with a new `const DEMO_PRESET: Preset = Preset::Earthy;` — **the vertex-scramble demo effect is dropped**, since scrambling is not part of `Planet::generate`'s pipeline and `app.rs` may no longer call `scramble_vertices` directly under the new convention. This is a deliberate, accepted behavior change: the pre-`012` "scrambled icosahedron" visual is a casualty of `Planet` becoming the sole entry point, not a regression this feature tries to work around
- build its `on_progress: GenerationProgress` closure exactly as today's `update_cb` (pushing each received `&Mesh` into the same `Rc<RefCell<Vec<Mesh>>>` frame collector), then call `Planet::generate(DEMO_PRESET, Seed::from(DEMO_SEED), Steps::default(), Some(on_progress))?` — no separate manual seeding of `collected_frames` before the call, since the callback's unconditional round-`0` invocation (see above) now supplies that first frame instead
- otherwise keep the existing frame-playback logic (`self.frames`, `self.current_frame`, `RedrawRequested` advancing through frames) unchanged — this feature does not touch rendering, camera, or input handling

## Domain model involved

**`planet-core/src/planets/planet.rs` (new):**
```rust
use crate::color::rgb::Rgb;
use crate::geometry::mesh::{Mesh, MeshError};
use crate::presets::preset::Preset;
use crate::subdivision::seed::Seed;
use crate::subdivision::steps::Steps;
use crate::subdivision::subdivide::subdivide;
use crate::subdivision::subdivision_args::SubdivisionArgs;
use crate::subdivision::subdivision_mode::SubdivisionMode;

pub type GenerationProgress = Box<dyn FnMut(&Mesh, usize)>;

#[derive(Debug, Clone, PartialEq)]
pub struct Planet {
    mesh: Mesh,
    colors: Vec<Rgb>,
    preset: Preset,
}

impl Planet {
    pub fn generate(
        preset: Preset,
        seed: Seed,
        max_depth: Steps,
        on_progress: Option<GenerationProgress>,
    ) -> Result<Planet, MeshError> {
        let params = preset.params();
        let base = Mesh::icosahedron()?;
        let mut on_progress = on_progress;
        if let Some(callback) = on_progress.as_mut() {
            callback(&base, 0);
        }
        let args = SubdivisionArgs::new(
            Some(max_depth),
            Some(SubdivisionMode::RedGreenSplit {
                seed,
                elevation_noise_range: params.elevation_noise_range(),
                normal_noise_range: params.normal_noise_range(),
                min_edge_length: params.min_edge_length(),
                split_point_variance: params.split_point_variance(),
            }),
            on_progress,
        );
        let mesh = subdivide(&base, args)?;
        let colors = mesh
            .vertices()
            .iter()
            .map(|vertex| params.color_gradient().sample(vertex.position.length()))
            .collect();
        Ok(Planet { mesh, colors, preset })
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn colors(&self) -> &[Rgb] {
        &self.colors
    }

    pub fn preset(&self) -> Preset {
        self.preset
    }
}
```

Existing types this feature calls but does not modify: `Mesh` / `Mesh::icosahedron()` / `MeshError` (`geometry/mesh.rs`), `Seed` (`subdivision/seed.rs`), `Steps` (`subdivision/steps.rs`), `SubdivisionMode::RedGreenSplit` (`subdivision/subdivision_mode.rs`), `SubdivisionArgs`/`UpdateCallback` (`subdivision/subdivision_args.rs`), `subdivide` (`subdivision/subdivide.rs`), `Preset` / `Preset::params()` (`presets/preset.rs`), `PresetParams`'s 5 accessors (`presets/preset_params.rs`), `ColorGradient::sample` (`color/color_gradient.rs`), `Rgb` (`color/rgb.rs`).

## Function/API contracts

### `Planet::generate`

```rust
pub fn generate(
    preset: Preset,
    seed: Seed,
    max_depth: Steps,
    on_progress: Option<GenerationProgress>,
) -> Result<Planet, MeshError>
```

- **Pre:** none beyond what each argument's own type already guarantees — `Preset` is a unit-variant enum (always valid), `Seed` wraps any `u64` (always valid), `Steps` is already validated on construction (capped at `MAX_SUBDIVISION_STEPS = 8` by `Steps::new`, or `Steps::default()` = 3), `on_progress` may be `None` or any `FnMut(&Mesh, usize)` closure. `Planet::generate` performs no additional validation of its own
- **Post:**
  - Returns `Ok(Planet)` for every valid `(preset, seed, max_depth, on_progress)` — the only way this can return `Err` is if `Mesh::icosahedron()` or `subdivide()` themselves fail, which does not happen for `Planet::generate`'s fixed, always-valid internal call shape
  - **Deterministic:** identical `(preset, seed, max_depth)` always produce a `Planet` with bit-identical `mesh()` (same vertex positions in the same order, same triangles) and bit-identical `colors()`, regardless of whether or what `on_progress` callback is supplied — required by `constitution.md`
  - `planet.colors().len() == planet.mesh().vertices().len()` always holds
  - For every index `i`: `planet.colors()[i] == preset.params().color_gradient().sample(planet.mesh().vertices()[i].position.length())`
  - `planet.preset() == preset` (the exact `Preset` variant passed in, returned unchanged)
  - For `max_depth` equal to `Steps::new(0).unwrap()`, `planet.mesh()` is structurally identical to `Mesh::icosahedron().unwrap()` (no subdivision rounds run) — 12 vertices, 20 triangles, colors sampled at the base icosahedron's unit radius
  - Subdivision never runs more than `max_depth` rounds regardless of `preset`'s `min_edge_length` — the existing hard-cap guarantee `subdivide`/`Steps` already provide, inherited unchanged through this wiring
  - When `on_progress` is `Some`, it is invoked exactly `max_depth.value() + 1` times: once for round `0` (the base icosahedron, before any subdivision), then once per completed round `1..=max_depth.value()`. When `on_progress` is `None`, `Planet::generate`'s behavior (its returned `Planet`) is identical, just without the notifications

## BDD scenarios

Feature file: `planet-core/tests/features/planet.feature`. Per `rules.md`'s BDD scenario style, every fixture is referenced by how it was obtained (`Given a Planet generated with seed <n> and the <Preset> preset...`, never bare `Given a planet`), and — per `rules.md`'s "every preset-related feature file covers: determinism..., elevation distribution respects the preset's noise range, and — for presets with an ocean quota — the fraction of vertices at sea level" — this file covers the first two (the third does not yet apply, since no `Preset` carries an `ocean_quota` in this feature; see "Out of scope").

```gherkin
Feature: Planet aggregate generation

  Scenario: Generating a Planet is deterministic for identical inputs
    Given a Planet generated with seed 42 and the Earthy preset at max depth 3
    When another Planet is generated with seed 42 and the Earthy preset at max depth 3
    Then the two Planets have identical meshes
    And the two Planets have identical colors

  Scenario: A different seed produces a different Planet
    Given a Planet generated with seed 42 and the Earthy preset at max depth 3
    When another Planet is generated with seed 43 and the Earthy preset at max depth 3
    Then the two Planets do not have identical meshes

  Scenario: Every vertex's color matches the preset's color gradient sampled at its radius
    Given a Planet generated with seed 7 and the Volcano preset at max depth 2
    Then every vertex's color in the resulting Planet equals the Volcano preset's color gradient sampled at that vertex's radius

  Scenario: Generating a Planet keeps every vertex radius within the preset's configured bound
    Given a Planet generated with seed 3 and the Rocky preset at max depth 2
    Then every vertex of the resulting Planet's mesh has a radius less than or equal to 1.4
    And every vertex of the resulting Planet's mesh has a radius greater than or equal to 0.05

  Scenario: A Planet generated at zero max depth is exactly the base icosahedron, colored
    Given a Planet generated with seed 1 and the Earthy preset at max depth 0
    Then the resulting Planet's mesh is identical to the icosahedron mesh
    And the resulting Planet has exactly 12 colors

  Scenario: Subdivision depth is honored as a hard cap regardless of the preset's min edge length
    Given a Planet generated with seed 5 and the Volcano preset at max depth 8
    Then the resulting Planet's mesh has no more triangles than 8 rounds of subdivision can produce from an icosahedron

  Scenario: The optional progress callback reports the base mesh and every subdivision round
    Given a recording progress callback
    When a Planet is generated with seed 9 and the Earthy preset at max depth 2 using that callback
    Then the progress callback was invoked 3 times
    And the progress callback's 1st invocation received round 0 with the base icosahedron mesh
    And the progress callback's 3rd invocation received round 2 with the resulting Planet's mesh

  Scenario: The optional progress callback still reports the base mesh at zero max depth
    Given a recording progress callback
    When a Planet is generated with seed 9 and the Earthy preset at max depth 0 using that callback
    Then the progress callback was invoked 1 time
    And the progress callback's 1st invocation received round 0 with the base icosahedron mesh
```

## Acceptance criteria

1. `planet-core` gains a new `planets/` concern (`planets/planet.rs`, declared via a sibling `planets.rs` `mod` file per Rust 2024 module style), added to `rules.md`'s module-structure list
2. `Planet::generate(preset: Preset, seed: Seed, max_depth: Steps, on_progress: Option<GenerationProgress>) -> Result<Planet, MeshError>` exists and is the only public constructor of `Planet`
3. Given identical `(preset, seed, max_depth)`, two calls to `Planet::generate` produce `Planet`s with bit-identical `mesh()` and bit-identical `colors()`, regardless of `on_progress` (unit/BDD test)
4. Given a different `seed` (all else equal), `Planet::generate` produces a `Planet` whose `mesh()` differs from the first (unit/BDD test)
5. `planet.colors().len() == planet.mesh().vertices().len()` holds for every generated `Planet` (unit/BDD test)
6. For every vertex index `i`, `planet.colors()[i] == preset.params().color_gradient().sample(planet.mesh().vertices()[i].position.length())` (unit/BDD test)
7. `planet.preset()` returns the exact `Preset` variant passed to `Planet::generate` (unit test)
8. For `max_depth = Steps::new(0).unwrap()`, `planet.mesh()` is structurally identical to `Mesh::icosahedron().unwrap()` — same vertex count (12), triangle count (20), positions, and indices (unit/BDD test)
9. No generated `Planet`'s mesh exceeds `max_depth` subdivision rounds regardless of `preset.params().min_edge_length()` (unit/BDD test, mirroring `subdivide.feature`'s existing hard-cap coverage)
10. When `on_progress` is `Some`, it is invoked exactly `max_depth.value() + 1` times, with round `0` reporting the base icosahedron and rounds `1..=max_depth.value()` reporting each completed round's `Mesh`, in order (unit/BDD test, including the `max_depth = 0` edge case)
11. `Planet::generate` contains no `unwrap()`/`panic!()`/`.expect()` in production code; both fallible internal calls (`Mesh::icosahedron()`, `subdivide()`) propagate their `MeshError` via `?`
12. `rules.md` gains the new "Crate boundaries" rule described above
13. `planet-renderer/src/app.rs` no longer imports anything from `planet_core::subdivision::{subdivide, subdivision_args, subdivision_mode, elevation_noise_range, normal_noise_range, min_edge_length, split_point_variance}` or `planet_core::processor::{vertex_scramble, vertex_scramble_range}`; it obtains its demo `Planet` exclusively via `Planet::generate`, and its animation-frame collection behaves identically to today's (same frame count, same per-round mesh content) except the base frame now comes from the progress callback's round-`0` invocation instead of manual pre-seeding, and the base mesh is no longer scrambled
14. All 8 BDD scenarios above are backed by real `cucumber` step definitions in `planet-core/tests/features/planet.feature` and a matching step-definition module — no scenario is left as markdown prose
15. Build gate passes: `cargo test --workspace && cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo build --target wasm32-unknown-unknown -p planet-renderer`
