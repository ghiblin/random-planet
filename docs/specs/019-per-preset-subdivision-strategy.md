# 019 — Per-Preset Subdivision Strategy

**Status:** Ready for review
**Feature slug:** `per-preset-subdivision-strategy`

This is an ad-hoc corrective feature, not the next sequential `docs/roadmap.md` phase — triggered directly by user feedback that generated planets still don't look like the expected result, after `017-geodesic-terrain-rework.md` and `018-restore-tangential-jitter.md`. The request explicitly asks to "keep it simple" this time: rather than another algorithm swap, this feature makes one small architectural correction and one preset-tuning change, scoped to the Earthy preset only.

**Architectural complaint:** `Planet::subdivide` (`planet-core/src/planets/planet.rs`) constructs a single, literal `SubdivisionMode::UniformRedSplit { seed: self.seed }` for every preset, regardless of which `Preset` is being generated. `017` intentionally converged all three presets onto one algorithm to fix two specific bugs (early convergence, direction-correlated slivers), but its side effect is that *no* preset can ever pick a different subdivision strategy without editing `Planet::subdivide` itself — the one remaining `SubdivisionMode` variant is forced on every preset by construction, not by preset-level configuration. This feature removes that forcing: `PresetParams` gains its own `subdivision_mode` field, exactly the way `TerrainNoise`/`ColorGradient`/`OceanQuota` are already preset-level knobs `Planet::subdivide` reads rather than hardcodes. `SubdivisionMode` still has exactly one variant (`UniformRedSplit`) after this feature — no new algorithm is introduced — but the *selection* now flows from the preset, so a future preset or phase can name a different strategy without touching `Planet::subdivide` again.

**Visual complaint:** "the geodesic split with an fBm technique works, but we need to increase the amplitude of the waves so we get more cartoonish-like results," scoped explicitly to the Earthy preset. Earthy's `TerrainNoise.amplitude` (0.12) is the smallest of the three presets' — smaller than Volcano's 0.30 and Rocky's 0.22 — which is why Earthy alone looks comparatively flat next to the other two. Bumping the number in isolation is not enough: `ColorGradient::sample` (`planet-core/src/color/color_gradient.rs`) clamps to its first/last stop's color outside the gradient's own configured range, and Earthy's current stops (`0.85`–`1.15`) are calibrated to the *old*, narrower amplitude. Raising amplitude without widening the stops would just clamp every newly-exaggerated peak/trench into the same flat snow/deep-water color already used today — the opposite of a more visible, "cartoonish" result. Both changes are therefore one coupled tuning change to Earthy's `PresetParams`.

**Explicitly out of scope:** collapsing `SubdivisionArgs`'/`SubdivisionMode`'s shape further, adding a second `SubdivisionMode` variant, or retuning Volcano/Rocky — this project's own convention (`017`, `018`) is to scope ad-hoc corrective features tightly and leave further cleanup to a later phase if wanted.

## Requirements

1. `SubdivisionMode` (`planet-core/src/subdivision/subdivision_mode.rs`) reverts to a seedless, preset-shape enum — `UniformRedSplit` becomes a unit variant again (`#[default]`), matching `TerrainNoise`'s and `ColorGradient`'s existing convention of carrying no `Seed` of their own. `018` added a `seed: Seed` field to this same variant for a real reason (a persistent, per-run `Pcg32` inside `UniformRedSplit`) — that reason doesn't go away, it just moves: seed becomes a parameter passed alongside the mode, not baked into it, exactly the way `apply_terrain_noise(mesh, seed, terrain_noise)` already takes `TerrainNoise` and `Seed` as two separate arguments rather than one merged type.
2. `SubdivisionMode::strategy` gains a `seed: Seed` parameter: `pub(crate) fn strategy(&self, seed: Seed) -> Box<dyn SubdivisionStrategy>`.
3. `SubdivisionArgs` (`planet-core/src/subdivision/subdivision_args.rs`) gains a 4th field, `seed: Seed`, supplied independently of `mode` — `SubdivisionArgs::new(steps: Option<Steps>, mode: Option<SubdivisionMode>, seed: Option<Seed>, update_cb: Option<UpdateCallback>) -> SubdivisionArgs`, with a `seed() -> Seed` accessor. Omitting `seed` falls back to `Seed::default()` (value `0`), the same fallback convention every other `Option` field in this constructor already uses.
4. `subdivide()` (`planet-core/src/subdivision/subdivide.rs`) builds its strategy via `args.mode.strategy(args.seed)` instead of `args.mode.strategy()`.
5. `PresetParams` (`planet-core/src/presets/preset_params.rs`) gains a 4th field, `subdivision_mode: SubdivisionMode`, with a matching `PresetParams::new` parameter and a `subdivision_mode() -> SubdivisionMode` accessor.
6. `Preset::Earthy`/`Volcano`/`Rocky` (`planet-core/src/presets/preset.rs`) each construct `SubdivisionMode::UniformRedSplit` explicitly as their 4th `PresetParams::new` argument. No preset picks a different variant in this feature (only one variant exists), but the value now comes from the preset's own definition rather than a literal inside `Planet::subdivide`.
7. `Planet::subdivide` (`planet-core/src/planets/planet.rs`) builds `SubdivisionArgs::new(Some(max_depth), Some(params.subdivision_mode()), Some(self.seed), on_progress)` — `params.subdivision_mode()` replaces the literal `SubdivisionMode::UniformRedSplit { seed: self.seed }`; `self.seed` moves to the new, separate `seed` argument.
8. `Preset::Earthy`'s `TerrainNoise.amplitude` increases from `0.12` to `0.30` — comparable in magnitude to Volcano's existing `0.30`, but reading differently in practice because Earthy's lower frequency/octave count (`1.5`/`4` vs Volcano's `2.5`/`5`) and gentler redistribution exponent (`1.4` vs Volcano's `2.2`) produce broad, rolling shapes rather than Volcano's sharper terraced peaks — a "cartoonish, exaggerated continent" look rather than Volcano's "craggy volcanic" look, at a similar overall elevation range. All other `TerrainNoise` fields for Earthy (`frequency`, `octaves`, `persistence`, `lacunarity`, `redistribution_exponent`, `terrace_levels`) are unchanged.
9. `Preset::Earthy`'s `ColorGradient` stops widen to match amplitude `0.3`'s actual reachable radius range (`new_radius = (1.0 + signed * amplitude).max(MIN_VERTEX_RADIUS)` bounds every vertex to `[0.7, 1.3]`, per `apply_terrain_noise`'s existing, unchanged contract), preserving the old 6 stops' relative proportions across the widened range: `0.70` deep water, `0.90` shallow water, `1.00` sand, `1.10` grassland, `1.20` hills, `1.30` snow cap (previously `0.85`/`0.95`/`1.00`/`1.05`/`1.10`/`1.15`). Stop colors (the `Rgb` values) are unchanged — only the elevation thresholds move.
10. No change to `Preset::Volcano`/`Preset::Rocky`'s `TerrainNoise` or `ColorGradient` — this feature's visual retuning is scoped to Earthy only.
11. Every production and test construction site of `SubdivisionMode::UniformRedSplit { seed: ... }` and `SubdivisionArgs::new(...)` updates to the new seedless-mode / separate-seed-argument shape: `planet-core/src/planets/planet.rs`, `planet-core/tests/subdivide.rs` + `tests/features/subdivide.feature`, `planet-core/tests/subdivision_args.rs` + `tests/features/subdivision_args.feature`, `planet-core/tests/apply_terrain_noise.rs` + `tests/features/apply_terrain_noise.feature`.
12. `rules.md` updates: `subdivision/` concern's `subdivision_mode.rs` description no longer says `UniformRedSplit` carries a `seed` field; `subdivision_args.rs`'s description gains its new `seed` field; `presets/` concern's `preset_params.rs` description gains the 4th field.
13. No change to `planet-renderer` — `app.rs`/`gpu/`/`scene/`/`controls/` are untouched; `App::generate` already reads every knob through `Planet::builder()...build()...subdivide()`, so it picks up this feature with no code change.

## Domain model involved

### Changed

**`planet-core/src/subdivision/subdivision_mode.rs`:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SubdivisionMode {
    #[default]
    UniformRedSplit,
}

impl SubdivisionMode {
    pub(crate) fn strategy(&self, seed: Seed) -> Box<dyn SubdivisionStrategy> {
        match self {
            SubdivisionMode::UniformRedSplit => Box::new(UniformRedSplit::new(seed)),
        }
    }
}
```
(`UniformRedSplit::new(seed: Seed)` itself, in `subdivision/strategies/uniform_red_split.rs`, is unchanged by this feature — it already takes `Seed` as a plain parameter, per `018`.)

**`planet-core/src/subdivision/subdivision_args.rs`:**
```rust
pub struct SubdivisionArgs {
    pub(crate) steps: Steps,
    pub(crate) mode: SubdivisionMode,
    pub(crate) seed: Seed,
    pub(crate) update_cb: Option<UpdateCallback>,
}

impl SubdivisionArgs {
    pub fn new(
        steps: Option<Steps>,
        mode: Option<SubdivisionMode>,
        seed: Option<Seed>,
        update_cb: Option<UpdateCallback>,
    ) -> SubdivisionArgs {
        SubdivisionArgs {
            steps: steps.unwrap_or_default(),
            mode: mode.unwrap_or_default(),
            seed: seed.unwrap_or_default(),
            update_cb,
        }
    }

    // ...steps(), mode() unchanged, plus:
    pub fn seed(&self) -> Seed {
        self.seed
    }
}
```

**`planet-core/src/subdivision/subdivide.rs`:** the one call-site line changes: `let mut strategy = args.mode.strategy(args.seed);`.

**`planet-core/src/presets/preset_params.rs`:**
```rust
pub struct PresetParams {
    terrain_noise: TerrainNoise,
    color_gradient: ColorGradient,
    ocean_quota: Option<OceanQuota>,
    subdivision_mode: SubdivisionMode,
}

impl PresetParams {
    pub fn new(
        terrain_noise: TerrainNoise,
        color_gradient: ColorGradient,
        ocean_quota: Option<OceanQuota>,
        subdivision_mode: SubdivisionMode,
    ) -> PresetParams { /* ... */ }

    // ...existing accessors, plus:
    pub fn subdivision_mode(&self) -> SubdivisionMode {
        self.subdivision_mode
    }
}
```

**`planet-core/src/presets/preset.rs`:** each `Preset::params()` arm's `PresetParams::new(...)` call gains a 4th argument, `SubdivisionMode::UniformRedSplit`, for all three presets. Earthy's arm additionally changes:
- `TerrainNoise.amplitude`: `0.12` → `0.30` (all other `TerrainNoise` fields unchanged)
- `ColorGradient.stops`: elevation thresholds `0.85, 0.95, 1.00, 1.05, 1.10, 1.15` → `0.70, 0.90, 1.00, 1.10, 1.20, 1.30` (the 6 `Rgb` colors themselves unchanged)

**`planet-core/src/planets/planet.rs`:**
```rust
let args = SubdivisionArgs::new(
    Some(max_depth),
    Some(params.subdivision_mode()),
    Some(self.seed),
    on_progress,
);
```

### Unchanged

`Mesh`/`Vertex`/`Triangle`/`Vec3`, `Seed`, `Steps`, `SubdivisionStrategy`, `UniformRedSplit`/`jitter()`, `TerrainNoise`/`apply_terrain_noise` (function bodies untouched — only Earthy's *arguments* to `TerrainNoise` change), `OceanQuota`/`apply_ocean_quota`, `ColorGradient` (the type/`sample` logic itself — only Earthy's *stop values* change), `MeshProcessor`/`identity_mesh`/`compose_mesh`, `PlanetBuilder`, `Planet`'s other fields/methods. `Preset::Volcano`/`Preset::Rocky`'s `TerrainNoise`/`ColorGradient` values. `planet-renderer` entirely.

## Function/API contracts

### `SubdivisionMode::strategy` (updated contract)

```rust
pub(crate) fn strategy(&self, seed: Seed) -> Box<dyn SubdivisionStrategy>
```
- **Pre:** any `SubdivisionMode`, any `Seed`
- **Post:** returns a freshly constructed strategy seeded from `seed` — for the one existing variant, `UniformRedSplit::new(seed)`, with the exact same determinism guarantee `018` already established (identical `(mode, seed)` ⇒ identical resulting strategy behavior across an identical sequence of `split_triangle` calls)

### `SubdivisionArgs::new` (updated contract)

```rust
pub fn new(
    steps: Option<Steps>,
    mode: Option<SubdivisionMode>,
    seed: Option<Seed>,
    update_cb: Option<UpdateCallback>,
) -> SubdivisionArgs
```
- **Pre:** any combination of `Some`/`None` per field
- **Post:** each `None` falls back to that field's `Default` (`Steps::default()`, `SubdivisionMode::default()` i.e. `UniformRedSplit`, `Seed::default()` i.e. value `0`); `seed()` returns whatever `Seed` was resolved, independent of `mode`

### `PresetParams::new` (updated contract)

```rust
pub fn new(
    terrain_noise: TerrainNoise,
    color_gradient: ColorGradient,
    ocean_quota: Option<OceanQuota>,
    subdivision_mode: SubdivisionMode,
) -> PresetParams
```
- **Pre:** any already-validated `TerrainNoise`/`ColorGradient`, any `Option<OceanQuota>`, any `SubdivisionMode`
- **Post:** bundles all 4 values unchanged; `subdivision_mode()` returns exactly what was passed in

### `Planet::subdivide` (updated contract)

All prior postconditions continue to hold (determinism, `max_depth` honored as a hard cap, exact `20 * 4^max_depth` triangle count, `colors().len() == mesh().vertices().len()`). Two additional guarantees:
- The `SubdivisionMode` used for a given generation is always `self.preset.params().subdivision_mode()` — never a value independent of `self.preset`
- For the Earthy preset specifically, every vertex's radius after `apply_terrain_noise` (before any `OceanQuota` flattening) lies in `[0.7, 1.3]` (previously `[0.88, 1.12]`), per `apply_terrain_noise`'s existing, unchanged bound formula applied to the new amplitude `0.3`

## BDD scenarios

### `planet-core/tests/features/subdivision_args.feature` (updated)

```gherkin
  Scenario: Constructing SubdivisionArgs with an explicit mode and seed
    When SubdivisionArgs is constructed with 3 steps, SubdivisionMode::UniformRedSplit, and seed 7
    Then the SubdivisionArgs has 3 steps
    And the SubdivisionArgs has mode SubdivisionMode::UniformRedSplit
    And the SubdivisionArgs has seed 7

  Scenario: Omitting seed defaults to seed 0
    When SubdivisionArgs is constructed with 3 steps, SubdivisionMode::UniformRedSplit, and no seed
    Then the SubdivisionArgs has seed 0

  Scenario: Omitting mode defaults to UniformRedSplit
    Given 3 steps
    When SubdivisionArgs is constructed with those steps, no mode, and no seed
    Then the SubdivisionArgs has the default UniformRedSplit mode
```

### `planet-core/tests/features/subdivide.feature` (updated)

Every existing scenario's construction step changes shape only (seed moves from `SubdivisionMode::UniformRedSplit { seed: N }` to a separate `SubdivisionArgs` seed argument) — the scenario list and its assertions (face-count growth, no duplicate vertices, no cracks, determinism, differing-seed divergence, radius bounds `[0.7, 1.0]`, update-callback behavior, 0-step no-op) are otherwise unchanged from `018`'s version of this file.

```gherkin
  Scenario: Subdividing the icosahedron mesh by 1 step using SubdivisionMode::UniformRedSplit quadruples the triangle count
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit and seed 7
    Then the resulting Mesh has 80 triangles
```

### `planet-core/tests/features/preset_params.feature` (updated)

```gherkin
  Scenario: Constructing PresetParams bundles all 4 fields unchanged
    Given a TerrainNoise with amplitude 0.12, a ColorGradient with stops at elevation 0.0 color black and elevation 1.0 color white, an OceanQuota of 0.2, and SubdivisionMode::UniformRedSplit
    When a PresetParams is constructed from those 4 values
    Then the PresetParams has a TerrainNoise with amplitude 0.12
    And the PresetParams's ColorGradient samples elevation 0.0 to black
    And the PresetParams has an OceanQuota of 0.2
    And the PresetParams has subdivision mode SubdivisionMode::UniformRedSplit
```

### `planet-core/tests/features/preset.feature` (updated)

```gherkin
  Scenario: The Earthy preset's PresetParams carries the cartoonish-amplitude TerrainNoise
    When the Earthy preset's PresetParams is retrieved
    Then the PresetParams has a TerrainNoise with amplitude 0.3
    And the PresetParams has subdivision mode SubdivisionMode::UniformRedSplit

  Scenario: The Earthy preset's ColorGradient samples the new widened elevation stops
    When the Earthy preset's PresetParams is retrieved
    Then the PresetParams's ColorGradient samples elevation 0.7 to the deep-water color
    And the PresetParams's ColorGradient samples elevation 1.3 to the snow-cap color

  Scenario: Volcano and Rocky presets are unaffected by this feature
    When the Volcano preset's PresetParams is retrieved
    Then the PresetParams has a TerrainNoise with amplitude 0.3
    And the PresetParams has subdivision mode SubdivisionMode::UniformRedSplit
    When the Rocky preset's PresetParams is retrieved
    Then the PresetParams has a TerrainNoise with amplitude 0.22
    And the PresetParams has subdivision mode SubdivisionMode::UniformRedSplit
```

### `planet-core/tests/features/planet.feature` (extended)

```gherkin
  Scenario: An Earthy Planet's post-subdivision vertex radii reflect the increased cartoonish amplitude
    Given a Planet generated with seed 5 and the Earthy preset at max depth 3
    Then every vertex of the resulting Planet's mesh has a radius greater than or equal to 0.7
    And every vertex of the resulting Planet's mesh has a radius less than or equal to 1.3

  Scenario: A Planet's subdivision mode comes from its preset, not a value independent of preset
    Given a Planet generated with seed 5 and the Earthy preset at max depth 3
    When another Planet is generated with seed 5 and the Volcano preset at max depth 3
    Then both resulting Planets' meshes have exactly 1280 triangles
```

The second scenario is the direct regression test for "no preset-independent hardcoded mode": both presets currently resolve to the same `UniformRedSplit` variant (so the assertion is topological, the one thing observable without a second variant existing), but it pins down that `Planet::subdivide` reads the mode from each preset's own `PresetParams` rather than from a single shared literal — a future preset naming a different `SubdivisionMode` variant would only need to change `Preset::params()`, not this scenario or `Planet::subdivide`.

## Acceptance criteria

1. `SubdivisionMode` has exactly one variant, `UniformRedSplit`, as a unit variant carrying no `Seed` (compile-time check + unit test)
2. `SubdivisionMode::strategy(&self, seed: Seed)` takes `seed` as a parameter and constructs `UniformRedSplit::new(seed)` (unit test)
3. `SubdivisionArgs` carries a `seed: Seed` field independent of `mode`; `SubdivisionArgs::new`'s 3rd parameter is `seed: Option<Seed>`, defaulting to `Seed::default()` when `None`; `seed() -> Seed` accessor exists (unit/BDD test)
4. `subdivide()` constructs its strategy via `args.mode.strategy(args.seed)` (unit test, e.g. asserting two `SubdivisionArgs` with the same `mode` but different `seed` produce different subdivision results)
5. `PresetParams` has a 4th field, `subdivision_mode: SubdivisionMode`, with a matching `PresetParams::new` parameter and `subdivision_mode() -> SubdivisionMode` accessor (unit/BDD test)
6. `Preset::Earthy`/`Volcano`/`Rocky` each construct `SubdivisionMode::UniformRedSplit` via `Preset::params()`'s 4th `PresetParams::new` argument (unit/BDD test)
7. `Planet::subdivide` builds `SubdivisionArgs::new(Some(max_depth), Some(params.subdivision_mode()), Some(self.seed), on_progress)` — no literal `SubdivisionMode::UniformRedSplit { .. }` construction remains inside `planet.rs` (compile-time/code-review check, enforced at `planet-pr-validate` time per `rules.md`'s existing convention for structural rules)
8. `Preset::Earthy`'s `TerrainNoise.amplitude` is `0.3`; all other `TerrainNoise` fields for Earthy are unchanged from `017`'s values (unit/BDD test)
9. `Preset::Earthy`'s `ColorGradient` has exactly 6 stops at elevations `0.70, 0.90, 1.00, 1.10, 1.20, 1.30`, with the same 6 `Rgb` colors (in the same order) as before this feature (unit/BDD test)
10. `Preset::Volcano`/`Preset::Rocky`'s `TerrainNoise`/`ColorGradient` values are bit-identical to their pre-feature values (unit/BDD test — direct regression guard that only Earthy was retuned)
11. For a `Planet` generated with the Earthy preset, every post-`apply_terrain_noise` vertex radius lies in `[0.7, 1.3]` (BDD test) — the direct regression test for "increase the amplitude... more cartoonish-like results"
12. `Planet::subdivide`'s resolved `SubdivisionMode` for a given `Planet` always equals that `Planet`'s own `preset.params().subdivision_mode()` (BDD test, per the "not a value independent of preset" scenario above)
13. Every production/test construction site of the old `SubdivisionMode::UniformRedSplit { seed: ... }` shape and 3-argument `SubdivisionArgs::new` is updated to the new shape; the workspace's build gate (`cargo test --workspace && cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo build --target wasm32-unknown-unknown -p planet-renderer`) passes
14. `rules.md`'s `subdivision/` and `presets/` concern entries are updated per Requirements above
15. No `unwrap()`/`panic!()`/`.expect()` in production code (existing convention, unaffected by this feature)
16. All BDD scenarios above are backed by real `cucumber` step definitions in their respective `.feature` files and matching step-definition modules — no scenario left as markdown prose
17. No change to `planet-renderer` — `app.rs` is untouched; the existing preset dropdown / depth slider continue to work with no code change
18. Manual, in-browser check (per `000-architecture.md`'s GPU/pixel-output exemption): a freshly generated Earthy planet visibly shows a more exaggerated, "cartoonish" elevation range (taller peaks, deeper basins, more saturated use of the full color gradient) than before this feature, without introducing degenerate sliver triangles (still bounded by `018`'s existing `15°`–`135°` angle regression test, unaffected by this feature since it doesn't touch subdivision jitter magnitude)
