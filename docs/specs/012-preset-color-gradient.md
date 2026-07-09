# 012 — Preset Color Gradient

**Status:** Ready for review
**Feature slug:** `preset-color-gradient`

This is the first slice of `docs/roadmap.md`'s "007 — Planet presets" phase (`000-architecture.md` calls it `007-planet-presets` in its own cross-references, before intervening ad-hoc specs `005-subdivision-facade`, `006-by-concern-file-layout`, `008-strategies-module`, `010-vertex-scramble`, and `011-vertex-operator-composition` pushed the number to `012`). Scope is deliberately the smallest useful slice, matching how earlier roadmap phases were split (`007-radial-randomness`, `009-irregular-subdivision`): the `ColorGradient`/`Rgb` color-mapping value types, and the `PresetParams`/`Preset` bundle, as pure `planet-core` domain types with three concrete presets (Earthy, Volcano, Rocky). No ocean quota, no `Planet` aggregate root, no wiring into `SubdivisionMode`/`subdivide`, and no UI dropdown — all deferred to later, higher-numbered specs.

## Requirements

- `planet-core` gains a new top-level concern, `color/` (sibling to `geometry/`, `subdivision/`, `processor/`, declared via `planet-core/src/color.rs` per `rules.md`'s "every module lives under a concern subdirectory" rule), holding the color value types this feature introduces
- `planet-core` gains a new public value type `Rgb` (`planet-core/src/color/rgb.rs`): three `f32` channels (`r`, `g`, `b`). Validated constructor `Rgb::new(r: f32, g: f32, b: f32) -> Result<Rgb, RgbError>` rejects any channel outside `0.0..=1.0` via `RgbError::OutOfRange { r, g, b }` (carrying all three attempted values, mirroring `ElevationNoiseRangeError::InvalidRange`'s "report the whole attempted input" convention, even though only one channel may be the actual offender). `0.0` and `1.0` themselves are valid (inclusive range); `NaN` in any channel is rejected because `(0.0..=1.0).contains(&value)` is `false` for `NaN`, the same NaN-rejection-by-comparison pattern every other validated value type in this codebase already relies on. Accessors `r()`, `g()`, `b()`. No `Default` — there is no meaningful "default color" independent of a preset, and every `Rgb` this feature actually constructs comes from an explicit preset-authored gradient stop or from `ColorGradient::sample`'s interpolation, never from a bare default
- `Rgb`'s three fields are `pub(crate)` (not fully private) — see the "Internal infallible construction" rationale below
- `planet-core` gains a new public value type `ColorGradient` (`planet-core/src/color/color_gradient.rs`): an ordered list of elevation → color stops, `stops: Vec<(f32, Rgb)>` (the field itself `pub(crate)`, same rationale). Validated constructor `ColorGradient::new(stops: Vec<(f32, Rgb)>) -> Result<ColorGradient, ColorGradientError>` rejects two distinct cases, checked in order: fewer than 2 stops (`ColorGradientError::TooFewStops { count: usize }` — at least 2 stops are required to define an interval to interpolate across; a single stop or an empty list has no interval), then, for the remaining pairs, any adjacent pair whose elevations are not strictly ascending (`ColorGradientError::StopsNotStrictlyAscending { index: usize }`, where `index` is the position of the first stop that violates `stops[index - 1].0 < stops[index].0` — this also rejects two stops sharing the same elevation, since equal is not strictly ascending)
- `ColorGradient::sample(&self, elevation: f32) -> Rgb` maps an elevation to a color via linear interpolation between the two nearest stops, clamped at the ends: if `elevation <= stops[0].0`, returns `stops[0].1` exactly (no interpolation); if `elevation >= stops[last].0`, returns `stops[last].1` exactly; otherwise finds the bracketing pair `(e0, c0)`, `(e1, c1)` with `e0 <= elevation <= e1`; if `elevation` exactly equals the bracket's upper elevation `e1`, returns `c1` verbatim (an explicit equality check, not an artifact of the interpolation arithmetic — floating-point subtraction/re-addition (`c0 + (c1 - c0) * 1.0`) is not guaranteed bit-exact to `c1` in general IEEE-754 arithmetic for arbitrary channel values, only when `c0`'s channels happen to be exactly `0.0`, so an explicit check is required for the "exact at any stop" guarantee to hold universally, not just for gradients with a black stop); otherwise computes `t = (elevation - e0) / (e1 - e0)` and returns a new `Rgb` with each channel independently lerped: `c0.r() + (c1.r() - c0.r()) * t` (and likewise for `g`, `b`). Sampling exactly at a stop's own elevation therefore always returns that stop's color exactly — the first/last stop via the outer clamp branches, every interior stop via the explicit `elevation == e1` check. Since `c0` and `c1` are both valid `Rgb` (channels in `0.0..=1.0`) and `t` is in `0.0..=1.0` for any elevation strictly inside a bracket, the interpolated result is a convex combination and is therefore always itself channel-valid by construction — `sample` builds its result via `Rgb`'s internal infallible constructor (below), never via `Rgb::new(...).expect(...)`
- **Internal infallible construction (no `unwrap`/`expect` in production code, per `constitution.md`/`rules.md`):** `sample`'s interpolated result, and every field this feature's `Preset::params()` (below) sets on `MinEdgeLength`, `ElevationNoiseRange`, `NormalNoiseRange`, and `SplitPointVariance`, are compile-time-literal or provably-in-range values that never need runtime validation, but every one of those types' only public constructor (`new`) is fallible and returns `Result`. Rather than calling `.expect(...)` on a `Result` that can never actually be `Err` in these call sites — which `rules.md`'s "No `unwrap()`/`panic!()` in production code" rule forbids outright, with no carved-out exception for "but I know it's valid" — each of these 4 pre-existing types' private field(s) become `pub(crate)` instead of fully private (`MinEdgeLength(pub(crate) f32)`, `SplitPointVariance(pub(crate) f32)`, `ElevationNoiseRange { pub(crate) low: f32, pub(crate) high: f32 }`, `NormalNoiseRange { pub(crate) low: f32, pub(crate) high: f32 }`). This is a one-token-per-field visibility widening only — no new methods, no behavior change, no change to any existing public constructor, accessor, or `Default` impl — and it is what lets code elsewhere in the same crate (`presets/`, a different module tree from `subdivision/`) construct a known-good instance directly via its own struct/tuple literal, with the validation invariant upheld by this feature's own hardcoded literals rather than re-checked at runtime. `Rgb` and `ColorGradient`, being new types introduced by this very feature, are designed with `pub(crate)` fields from the start for the identical reason, rather than retrofitted
- `planet-core` gains a new top-level concern, `presets/` (sibling to `color/`), holding the preset bundle this feature introduces. Named `presets/` (plural), not `preset/` (singular) — the concern's own primary type is `Preset`, and a `preset/preset.rs` file layout would resolve to module path `preset::preset`, tripping clippy's default-on `module_inception` lint and failing the mandatory `-D warnings` build gate. No existing concern (`geometry/`, `subdivision/`, `processor/`) has a file that shares its containing concern's exact name, so this is the first case where the collision would actually arise
- `planet-core` gains a new public value type `PresetParams` (`planet-core/src/presets/preset_params.rs`), bundling the 5 fields `000-architecture.md` specifies minus `ocean_quota` (explicitly out of scope, see below) plus `normal_noise_range` (not yet invented when `000-architecture.md` was written, but now a real, already-shipped parameter of the subdivision pipeline that a preset must be able to specify so a later wiring spec is purely additive rather than a `PresetParams` rewrite): `min_edge_length: MinEdgeLength`, `elevation_noise_range: ElevationNoiseRange`, `normal_noise_range: NormalNoiseRange`, `split_point_variance: SplitPointVariance`, `color_gradient: ColorGradient`. Constructor `PresetParams::new(min_edge_length, elevation_noise_range, normal_noise_range, split_point_variance, color_gradient) -> PresetParams` is infallible — every field is already a validated/valid value type by the time it reaches this constructor (same "Pre: none, every field already validated" contract `RedGreenSplit::new` established), so there is no `Result` and no possible construction error. Accessors: `min_edge_length()`, `elevation_noise_range()`, `normal_noise_range()`, `split_point_variance()` (all `Copy` return-by-value), `color_gradient()` (returns `&ColorGradient`, since `ColorGradient` owns a `Vec` and is not `Copy`). `PresetParams` derives `Debug, Clone, PartialEq` — not `Copy` (blocked transitively by `ColorGradient`'s `Vec`), not `Eq` (blocked transitively by every `f32`-bearing field, same as `SubdivisionMode`)
- `planet-core` gains a new public enum `Preset` (`planet-core/src/presets/preset.rs`) with 3 unit variants — `Earthy` (default), `Volcano`, `Rocky` — carrying no data of their own (unlike `SubdivisionMode`'s variants). `Preset::params(&self) -> PresetParams` returns that variant's hardcoded `PresetParams`, built directly via each component type's `pub(crate)` literal construction (never via the fallible public `new()` + `.expect(...)`, per the rationale above) — see exact values in "Domain model involved". Because `Preset` itself holds no `f32` (all variants are unit variants), it derives the full standard set: `Debug, Clone, Copy, PartialEq, Eq, Hash, Default`. `#[default]` is `Earthy` — the "plain," least-extreme preset, mirroring `SubdivisionMode::default()`'s choice of the simplest variant
- `rules.md`'s "Module structure" section gains two new `planet-core` concern-list entries: `color/` (`rgb.rs` — `Rgb`, `RgbError`; `color_gradient.rs` — `ColorGradient`, `ColorGradientError`) and `presets/` (`preset_params.rs` — `PresetParams`; `preset.rs` — `Preset`)

Out of scope:
- `ocean_quota`, the sea-level flattening post-processing step, and any change to `planet-core/src/processor/` — `000-architecture.md`'s ocean-quota mechanism is a `Planet`-aggregate-level concern (percentile over a whole generated `Mesh`'s vertex radii), which doesn't exist yet; it lands in a later, higher-numbered spec once the `Planet` aggregate root does
- The `Planet` aggregate root and `Planet::generate(preset, seed, max_depth) -> Planet` — this feature ships `Preset`/`PresetParams` as standalone, directly-constructible domain types, exactly as `min_edge_length`/`elevation_noise_range`/`split_point_variance` were kept standalone (not preset-sourced) in `007-radial-randomness`/`009-irregular-subdivision`. Consequently, `rules.md`'s "every preset-related feature file covers: determinism (same seed + preset + depth ⇒ identical Mesh), elevation distribution respects the preset's noise range, and — for presets with an ocean quota — the fraction of vertices at sea level" rule does not yet apply in full: there is no seed-driven `Mesh` generation to assert determinism or elevation-distribution over. This feature's BDD coverage instead establishes what it can at this layer — `Preset::params()` is a pure, argument-free (besides `self`) function that always returns the same `PresetParams` for the same variant, and every preset's `color_gradient` samples correctly across its own elevation range — and defers the full Mesh-level scenario set to whichever future spec adds `Planet::generate`
- Wiring `PresetParams` into `SubdivisionMode`/`subdivide` in any way — `SubdivisionMode`, `SubdivisionArgs`, `subdivide()`, `EdgeCache`, and all 3 existing strategies (`UniformRedSplit`, `RadialRandomSplit`, `RedGreenSplit`) are untouched by this feature, not even a mechanical one-line change
- A `Preset` dropdown or any other UI control in `planet-renderer` — `planet-renderer` is not touched by this feature at all, not even a demo wiring call (unlike `007-radial-randomness`/`009-irregular-subdivision`/`010-vertex-scramble`, which each added one hardcoded demo call to `app.rs`); there is nothing yet for `Preset`/`PresetParams`/`ColorGradient` to drive in the renderer, since they don't produce a `Mesh` or a per-vertex color on their own
- A non-linear (e.g. cubic/spline) or perceptual (e.g. HSL/Lab-space) interpolation mode for `ColorGradient::sample` — plain per-channel linear RGB interpolation is sufficient for a stylized generative-art gradient at this app's scale, and any color-space conversion would add a new confirmed dependency for no scoped requirement
- Gamma correction, sRGB encoding/decoding, or any color-space semantics for `Rgb`'s channels beyond "a linear `0.0..=1.0` float per channel" — how these values eventually map to GPU-side color output is `planet-renderer`'s concern, for whichever future spec wires a `Mesh`'s per-vertex color into the GPU buffer
- A `Default` impl for `Rgb` or `ColorGradient` — neither has a meaningful default independent of a preset (see Requirements)
- Any change to `Mesh`, `Vertex`, `MeshError`, `Vec3`, `Triangle`, `Seed`, `Steps`, `SubdivisionMode`, `SubdivisionArgs`, `EdgeCache`, or any existing strategy — this feature adds 4 new files under 2 new concerns and widens 4 existing types' field visibility (see above); it changes no existing type's public constructor, accessor, `Default`, or derive list
- Extracting a shared "validated-range" or "validated-scalar" generic/trait to de-duplicate `MinEdgeLength`/`SplitPointVariance`/`ElevationNoiseRange`/`NormalNoiseRange`/`Rgb`'s near-identical validation shape — pre-existing duplication across the first 4, not this feature's to fix; `Rgb`'s shape (3 channels, not 1 or 2) doesn't fit the existing pattern anyway

## Domain model involved

**`planet-core/src/color/rgb.rs` (new):**
```rust
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub(crate) r: f32,
    pub(crate) g: f32,
    pub(crate) b: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RgbError {
    OutOfRange { r: f32, g: f32, b: f32 },
}

impl fmt::Display for RgbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RgbError::OutOfRange { r, g, b } => {
                write!(f, "rgb channels must be within 0.0..=1.0, got r {r} g {g} b {b}")
            }
        }
    }
}

impl std::error::Error for RgbError {}

impl Rgb {
    pub fn new(r: f32, g: f32, b: f32) -> Result<Rgb, RgbError> {
        let in_range = |v: f32| (0.0..=1.0).contains(&v);
        if in_range(r) && in_range(g) && in_range(b) {
            Ok(Rgb { r, g, b })
        } else {
            Err(RgbError::OutOfRange { r, g, b })
        }
    }

    pub fn r(&self) -> f32 { self.r }
    pub fn g(&self) -> f32 { self.g }
    pub fn b(&self) -> f32 { self.b }
}
```

**`planet-core/src/color/color_gradient.rs` (new):**
```rust
use std::fmt;

use super::rgb::Rgb;

#[derive(Debug, Clone, PartialEq)]
pub struct ColorGradient {
    pub(crate) stops: Vec<(f32, Rgb)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorGradientError {
    TooFewStops { count: usize },
    StopsNotStrictlyAscending { index: usize },
}

impl fmt::Display for ColorGradientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorGradientError::TooFewStops { count } => {
                write!(f, "color gradient needs at least 2 stops, got {count}")
            }
            ColorGradientError::StopsNotStrictlyAscending { index } => {
                write!(f, "color gradient stops must be strictly ascending by elevation, stop {index} is not")
            }
        }
    }
}

impl std::error::Error for ColorGradientError {}

impl ColorGradient {
    pub fn new(stops: Vec<(f32, Rgb)>) -> Result<ColorGradient, ColorGradientError> {
        if stops.len() < 2 {
            return Err(ColorGradientError::TooFewStops { count: stops.len() });
        }
        for index in 1..stops.len() {
            if !(stops[index - 1].0 < stops[index].0) {
                return Err(ColorGradientError::StopsNotStrictlyAscending { index });
            }
        }
        Ok(ColorGradient { stops })
    }

    pub fn sample(&self, elevation: f32) -> Rgb {
        let last = self.stops.len() - 1;
        if elevation <= self.stops[0].0 {
            return self.stops[0].1;
        }
        if elevation >= self.stops[last].0 {
            return self.stops[last].1;
        }
        for index in 0..last {
            let (e0, c0) = self.stops[index];
            let (e1, c1) = self.stops[index + 1];
            if elevation == e1 {
                return c1; // exact stop match: return verbatim, skip interpolation arithmetic entirely
            }
            if elevation >= e0 && elevation <= e1 {
                let t = (elevation - e0) / (e1 - e0);
                return Rgb {
                    r: c0.r() + (c1.r() - c0.r()) * t,
                    g: c0.g() + (c1.g() - c0.g()) * t,
                    b: c0.b() + (c1.b() - c0.b()) * t,
                };
            }
        }
        self.stops[last].1 // unreachable given the clamps above; exhaustive fallback
    }
}
```

**`planet-core/src/color.rs` (new):**
```rust
pub mod color_gradient;
pub mod rgb;
```

**`planet-core/src/presets/preset_params.rs` (new):**
```rust
use crate::color::color_gradient::ColorGradient;
use crate::subdivision::elevation_noise_range::ElevationNoiseRange;
use crate::subdivision::min_edge_length::MinEdgeLength;
use crate::subdivision::normal_noise_range::NormalNoiseRange;
use crate::subdivision::split_point_variance::SplitPointVariance;

#[derive(Debug, Clone, PartialEq)]
pub struct PresetParams {
    min_edge_length: MinEdgeLength,
    elevation_noise_range: ElevationNoiseRange,
    normal_noise_range: NormalNoiseRange,
    split_point_variance: SplitPointVariance,
    color_gradient: ColorGradient,
}

impl PresetParams {
    pub fn new(
        min_edge_length: MinEdgeLength,
        elevation_noise_range: ElevationNoiseRange,
        normal_noise_range: NormalNoiseRange,
        split_point_variance: SplitPointVariance,
        color_gradient: ColorGradient,
    ) -> PresetParams {
        PresetParams {
            min_edge_length,
            elevation_noise_range,
            normal_noise_range,
            split_point_variance,
            color_gradient,
        }
    }

    pub fn min_edge_length(&self) -> MinEdgeLength { self.min_edge_length }
    pub fn elevation_noise_range(&self) -> ElevationNoiseRange { self.elevation_noise_range }
    pub fn normal_noise_range(&self) -> NormalNoiseRange { self.normal_noise_range }
    pub fn split_point_variance(&self) -> SplitPointVariance { self.split_point_variance }
    pub fn color_gradient(&self) -> &ColorGradient { &self.color_gradient }
}
```

**`planet-core/src/presets/preset.rs` (new)** — exact hardcoded values per variant (elevations are vertex-radius values; base/unperturbed radius is `1.0`, per `Mesh::icosahedron()`'s own normalization):
```rust
use crate::color::color_gradient::ColorGradient;
use crate::color::rgb::Rgb;
use crate::subdivision::elevation_noise_range::ElevationNoiseRange;
use crate::subdivision::min_edge_length::MinEdgeLength;
use crate::subdivision::normal_noise_range::NormalNoiseRange;
use crate::subdivision::split_point_variance::SplitPointVariance;

use super::preset_params::PresetParams;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Preset {
    #[default]
    Earthy,
    Volcano,
    Rocky,
}

impl Preset {
    pub fn params(&self) -> PresetParams {
        match self {
            Preset::Earthy => PresetParams::new(
                MinEdgeLength(0.35),
                ElevationNoiseRange { low: -0.05, high: 0.15 },
                NormalNoiseRange { low: -0.05, high: 0.05 },
                SplitPointVariance(0.1),
                ColorGradient {
                    stops: vec![
                        (0.85, Rgb { r: 0.05, g: 0.15, b: 0.45 }), // deep water
                        (0.95, Rgb { r: 0.20, g: 0.50, b: 0.60 }), // shallow water
                        (1.00, Rgb { r: 0.82, g: 0.76, b: 0.50 }), // sand
                        (1.05, Rgb { r: 0.25, g: 0.55, b: 0.20 }), // grassland
                        (1.10, Rgb { r: 0.45, g: 0.35, b: 0.25 }), // hills
                        (1.15, Rgb { r: 0.95, g: 0.95, b: 0.95 }), // snow cap
                    ],
                },
            ),
            Preset::Volcano => PresetParams::new(
                MinEdgeLength(0.25),
                ElevationNoiseRange { low: -0.05, high: 0.35 },
                NormalNoiseRange { low: -0.10, high: 0.10 },
                SplitPointVariance(0.2),
                ColorGradient {
                    stops: vec![
                        (0.95, Rgb { r: 0.10, g: 0.05, b: 0.05 }), // dark basalt
                        (1.00, Rgb { r: 0.25, g: 0.05, b: 0.02 }), // charred rock
                        (1.15, Rgb { r: 0.55, g: 0.10, b: 0.02 }), // glowing rock
                        (1.25, Rgb { r: 0.95, g: 0.35, b: 0.05 }), // molten orange
                        (1.35, Rgb { r: 1.00, g: 0.85, b: 0.30 }), // lava-yellow peak
                    ],
                },
            ),
            Preset::Rocky => PresetParams::new(
                MinEdgeLength(0.3),
                ElevationNoiseRange { low: -0.2, high: 0.2 },
                NormalNoiseRange { low: -0.15, high: 0.15 },
                SplitPointVariance(0.25),
                ColorGradient {
                    stops: vec![
                        (0.80, Rgb { r: 0.30, g: 0.28, b: 0.26 }), // dark gray
                        (0.95, Rgb { r: 0.45, g: 0.42, b: 0.38 }), // gray
                        (1.00, Rgb { r: 0.55, g: 0.52, b: 0.48 }), // mid gray
                        (1.10, Rgb { r: 0.68, g: 0.64, b: 0.58 }), // light gray
                        (1.20, Rgb { r: 0.80, g: 0.78, b: 0.74 }), // pale peak
                    ],
                },
            ),
        }
    }
}
```
(`MinEdgeLength(0.35)`/`SplitPointVariance(0.1)` construct the tuple struct directly via its now-`pub(crate)` field, exactly like each type's own `Default` impl already does internally; `ElevationNoiseRange { low, high }`/`NormalNoiseRange { low, high }`/`Rgb { r, g, b }`/`ColorGradient { stops }` construct via their now-`pub(crate)` named fields, the same mechanism.)

**`planet-core/src/presets.rs` (new):**
```rust
pub mod preset;
pub mod preset_params;
```

**`planet-core/src/lib.rs` (updated):**
```rust
pub mod color;
pub mod geometry;
pub mod presets;
pub mod processor;
pub mod subdivision;
```

**`planet-core/src/subdivision/min_edge_length.rs` (updated — visibility only):** `pub struct MinEdgeLength(pub(crate) f32);` — `new`, `value`, `Default` bodies unchanged.

**`planet-core/src/subdivision/split_point_variance.rs` (updated — visibility only):** `pub struct SplitPointVariance(pub(crate) f32);` — unchanged otherwise.

**`planet-core/src/subdivision/elevation_noise_range.rs` (updated — visibility only):**
```rust
pub struct ElevationNoiseRange {
    pub(crate) low: f32,
    pub(crate) high: f32,
}
```
— unchanged otherwise.

**`planet-core/src/subdivision/normal_noise_range.rs` (updated — visibility only):** identical field-visibility change as `ElevationNoiseRange` above.

**`planet-core/Cargo.toml` (updated):** add `[[test]] name = "rgb" harness = false`, `[[test]] name = "color_gradient" harness = false`, `[[test]] name = "preset_params" harness = false`, `[[test]] name = "preset" harness = false`. No new dependencies.

**`rules.md` (updated):** `planet-core`'s concerns list gains:
```markdown
- `color/` — elevation-to-color mapping value types, no algorithm: `rgb.rs` (`Rgb`,
  `RgbError`), `color_gradient.rs` (`ColorGradient`, `ColorGradientError`)
- `presets/` — bundles the subdivision/color knobs into named, pre-tuned presets:
  `preset_params.rs` (`PresetParams`), `preset.rs` (`Preset`)
```

No changes to `planet-renderer` at all.

## Function/API contracts

- `Rgb::new(r, g, b) -> Result<Rgb, RgbError>` — **Pre:** none. **Post:** `Ok` iff all 3 channels are within `0.0..=1.0` inclusive (rejects any channel `< 0.0`, `> 1.0`, or `NaN`); on success, `r()`/`g()`/`b()` return exactly the inputs
- `ColorGradient::new(stops) -> Result<ColorGradient, ColorGradientError>` — **Pre:** none. **Post:** `Ok` iff `stops.len() >= 2` and every adjacent pair is strictly ascending by elevation (`stops[i-1].0 < stops[i].0` for all `i`); `Err(TooFewStops)` checked first (before any ascending check, so a 0- or 1-element input always reports `TooFewStops`, never `StopsNotStrictlyAscending`), `Err(StopsNotStrictlyAscending { index })` otherwise, reporting the first offending index
- `ColorGradient::sample(&self, elevation) -> Rgb` — **Pre:** `self` was built via `ColorGradient::new` (always true — no other constructor exists). **Post:** `elevation <= stops[0].0` returns `stops[0].1` exactly (by `Rgb`'s `PartialEq`, not merely "close"); `elevation >= stops[last].0` returns `stops[last].1` exactly; sampling exactly at *any* stop's own elevation (first, last, or interior) returns that stop's `Rgb` exactly — for interior stops this is an explicit `elevation == e1` equality check inside the bracket search, not an artifact of interpolation arithmetic, since `c0 + (c1 - c0) * 1.0` is not guaranteed bit-exact to `c1` for arbitrary channel values under IEEE-754 rounding; for `elevation` strictly between two adjacent stops' elevations (and not equal to either), returns a channel-wise linear interpolation between them, with the fraction `t` computed from the bracketing pair; never panics for any finite `f32` input; every returned `Rgb`'s channels stay within `0.0..=1.0` (provable: a convex combination, `t ∈ [0, 1]`, of two channel-valid endpoints stays channel-valid)
- `PresetParams::new(min_edge_length, elevation_noise_range, normal_noise_range, split_point_variance, color_gradient) -> PresetParams` — **Pre:** none (every argument is already a validated/valid value type). **Post:** infallible; the 5 accessors return exactly what was passed in
- `Preset::params(&self) -> PresetParams` — **Pre:** none. **Post:** pure and deterministic — calling it twice on the same `Preset` variant returns `PresetParams` that are `PartialEq`-equal; returns the exact hardcoded values in "Domain model involved" for each of the 3 variants; `Preset::default().params()` equals `Preset::Earthy.params()`
- `Preset::default() -> Preset` returns `Preset::Earthy`

## BDD scenarios

`planet-core/tests/features/rgb.feature`:
```gherkin
Feature: Constructing a validated Rgb color

  Scenario: Constructing an Rgb with all channels in range succeeds
    When an Rgb is constructed with r 0.2, g 0.4, b 0.6
    Then the Rgb is constructed successfully
    And the Rgb has r 0.2, g 0.4, b 0.6

  Scenario: Constructing an Rgb with channels at the exact boundaries succeeds
    When an Rgb is constructed with r 0.0, g 1.0, b 0.0
    Then the Rgb is constructed successfully

  Scenario: Constructing an Rgb with a channel below 0.0 fails
    When an Rgb is constructed with r -0.1, g 0.5, b 0.5
    Then the construction fails with an out-of-range error of r -0.1, g 0.5, b 0.5

  Scenario: Constructing an Rgb with a channel above 1.0 fails
    When an Rgb is constructed with r 0.5, g 1.5, b 0.5
    Then the construction fails with an out-of-range error of r 0.5, g 1.5, b 0.5

  Scenario: Constructing an Rgb with a NaN channel fails
    When an Rgb is constructed with r NaN, g 0.5, b 0.5
    Then the construction fails with an out-of-range error where r is NaN
```

`planet-core/tests/features/color_gradient.feature`:
```gherkin
Feature: Sampling a ColorGradient

  Scenario: Constructing a ColorGradient with at least 2 strictly ascending stops succeeds
    When a ColorGradient is constructed with stops at elevation 0.0 color black and elevation 1.0 color white
    Then the ColorGradient is constructed successfully

  Scenario: Constructing a ColorGradient with fewer than 2 stops fails
    When a ColorGradient is constructed with a single stop at elevation 0.0 color black
    Then the construction fails with a too-few-stops error of count 1

  Scenario: Constructing a ColorGradient with non-ascending stops fails
    When a ColorGradient is constructed with stops at elevation 1.0 color white and elevation 0.0 color black
    Then the construction fails with a stops-not-strictly-ascending error at index 1

  Scenario: Constructing a ColorGradient with two stops at the same elevation fails
    When a ColorGradient is constructed with stops at elevation 0.5 color black and elevation 0.5 color white
    Then the construction fails with a stops-not-strictly-ascending error at index 1

  Scenario: Sampling below the first stop clamps to the first stop's color
    Given a ColorGradient with stops at elevation 0.0 color black and elevation 1.0 color white
    When the ColorGradient is sampled at elevation -5.0
    Then the sampled Rgb equals black

  Scenario: Sampling above the last stop clamps to the last stop's color
    Given a ColorGradient with stops at elevation 0.0 color black and elevation 1.0 color white
    When the ColorGradient is sampled at elevation 5.0
    Then the sampled Rgb equals white

  Scenario: Sampling exactly at a stop's elevation returns that stop's color exactly
    Given a ColorGradient with stops at elevation 0.0 color black, elevation 0.5 color gray, and elevation 1.0 color white
    When the ColorGradient is sampled at elevation 0.5
    Then the sampled Rgb equals gray

  Scenario: Sampling exactly at an interior stop's elevation returns that stop's color exactly, even when neither bracketing stop is black
    Given a ColorGradient with stops at elevation 0.0 color with r 0.12, g 0.34, b 0.56, elevation 0.5 color with r 0.65, g 0.43, b 0.21, and elevation 1.0 color with r 0.91, g 0.82, b 0.73
    When the ColorGradient is sampled at elevation 0.5
    Then the sampled Rgb has r 0.65, g 0.43, b 0.21

  Scenario: Sampling halfway between two stops linearly interpolates each channel
    Given a ColorGradient with stops at elevation 0.0 color black and elevation 1.0 color white
    When the ColorGradient is sampled at elevation 0.5
    Then the sampled Rgb has r 0.5, g 0.5, b 0.5
```

`planet-core/tests/features/preset_params.feature`:
```gherkin
Feature: Bundling validated subdivision and color parameters into PresetParams

  Scenario: Constructing PresetParams bundles all 5 fields unchanged
    Given a MinEdgeLength of 0.4, an ElevationNoiseRange of low -0.1 and high 0.1, a NormalNoiseRange of low -0.05 and high 0.05, a SplitPointVariance of 0.15, and a ColorGradient with stops at elevation 0.0 color black and elevation 1.0 color white
    When a PresetParams is constructed from those 5 values
    Then the PresetParams has a MinEdgeLength of 0.4
    And the PresetParams has an ElevationNoiseRange of low -0.1 and high 0.1
    And the PresetParams has a NormalNoiseRange of low -0.05 and high 0.05
    And the PresetParams has a SplitPointVariance of 0.15
    And the PresetParams's ColorGradient samples elevation 0.0 to black

  Scenario: Two PresetParams built from identical arguments are equal
    Given a MinEdgeLength of 0.4, an ElevationNoiseRange of low -0.1 and high 0.1, a NormalNoiseRange of low -0.05 and high 0.05, a SplitPointVariance of 0.15, and a ColorGradient with stops at elevation 0.0 color black and elevation 1.0 color white
    When two PresetParams are constructed from those same 5 values, separately
    Then the two PresetParams are identical
```

`planet-core/tests/features/preset.feature`:
```gherkin
Feature: Selecting a Preset's parameters

  Scenario: The Earthy preset has its configured parameters
    When Preset::Earthy's params are requested
    Then the PresetParams has a MinEdgeLength of 0.35
    And the PresetParams has an ElevationNoiseRange of low -0.05 and high 0.15
    And the PresetParams has a NormalNoiseRange of low -0.05 and high 0.05
    And the PresetParams has a SplitPointVariance of 0.1

  Scenario: The Volcano preset has its configured parameters
    When Preset::Volcano's params are requested
    Then the PresetParams has a MinEdgeLength of 0.25
    And the PresetParams has an ElevationNoiseRange of low -0.05 and high 0.35
    And the PresetParams has a NormalNoiseRange of low -0.1 and high 0.1
    And the PresetParams has a SplitPointVariance of 0.2

  Scenario: The Rocky preset has its configured parameters
    When Preset::Rocky's params are requested
    Then the PresetParams has a MinEdgeLength of 0.3
    And the PresetParams has an ElevationNoiseRange of low -0.2 and high 0.2
    And the PresetParams has a NormalNoiseRange of low -0.15 and high 0.15
    And the PresetParams has a SplitPointVariance of 0.25

  Scenario: Earthy's color gradient samples its own lowest and highest configured elevations to its first and last stops' colors
    When Preset::Earthy's params are requested
    Then sampling its color gradient at elevation 0.85 returns Rgb r 0.05, g 0.15, b 0.45
    And sampling its color gradient at elevation 1.15 returns Rgb r 0.95, g 0.95, b 0.95

  Scenario: Volcano's color gradient samples its own lowest and highest configured elevations to its first and last stops' colors
    When Preset::Volcano's params are requested
    Then sampling its color gradient at elevation 0.95 returns Rgb r 0.1, g 0.05, b 0.05
    And sampling its color gradient at elevation 1.35 returns Rgb r 1.0, g 0.85, b 0.3

  Scenario: Rocky's color gradient samples its own lowest and highest configured elevations to its first and last stops' colors
    When Preset::Rocky's params are requested
    Then sampling its color gradient at elevation 0.8 returns Rgb r 0.3, g 0.28, b 0.26
    And sampling its color gradient at elevation 1.2 returns Rgb r 0.8, g 0.78, b 0.74

  Scenario: Preset::params is deterministic
    When Preset::Rocky's params are requested twice
    Then both PresetParams are identical

  Scenario: The default Preset is Earthy
    Given the default Preset
    Then the Preset equals Preset::Earthy
```

## Acceptance criteria

1. `Rgb::new(r, g, b)` returns `Ok` iff every channel is within `0.0..=1.0` inclusive; returns `Err(RgbError::OutOfRange { r, g, b })` if any channel is `< 0.0`, `> 1.0`, or `NaN`
2. `ColorGradient::new(stops)` returns `Err(ColorGradientError::TooFewStops { count })` for 0 or 1 stops; returns `Err(ColorGradientError::StopsNotStrictlyAscending { index })` for the first adjacent pair that is not strictly ascending (including exactly-equal elevations); otherwise returns `Ok`
3. `ColorGradient::sample` clamps exactly to the first/last stop's color outside the stops' elevation range, interpolates linearly per-channel between the two bracketing stops inside it, and returns a stop's exact color when sampled exactly at that stop's elevation
4. For a 2-stop gradient from black (`0,0,0`) to white (`1,1,1`) at elevations `0.0`/`1.0`, sampling at `0.5` returns `Rgb { r: 0.5, g: 0.5, b: 0.5 }` exactly
5. `PresetParams::new` is infallible and its 5 accessors return exactly what was passed in
6. `Preset::Earthy.params()`, `Preset::Volcano.params()`, and `Preset::Rocky.params()` each return the exact `MinEdgeLength`/`ElevationNoiseRange`/`NormalNoiseRange`/`SplitPointVariance` values hardcoded in "Domain model involved", and a `ColorGradient` whose first and last stops match those listed
7. `Preset::params()` is deterministic: two calls on the same variant return `PartialEq`-equal `PresetParams`
8. `Preset::default()` equals `Preset::Earthy`; `Preset` derives `Eq` and `Hash` (compiles and is usable as a `HashMap`/`HashSet` key)
9. `planet-core/src/color/` contains exactly `rgb.rs` and `color_gradient.rs`; `planet-core/src/presets/` contains exactly `preset_params.rs` and `preset.rs`; `rules.md`'s "Module structure" section documents both new concerns
10. `MinEdgeLength`, `SplitPointVariance`, `ElevationNoiseRange`, `NormalNoiseRange` keep their exact existing public API (constructor signature, accessors, `Default` values, error variants) — only their field(s)' visibility widens from private to `pub(crate)`; every existing test for these 4 types and every existing `SubdivisionMode`/`subdivide`/strategy scenario referencing them passes unmodified
11. No `unwrap()`/`panic!()` is introduced in any production code path added or touched by this feature — `Preset::params()` and `ColorGradient::sample` construct every value type directly via `pub(crate)` fields/literals, never via a fallible public constructor plus `.expect(...)`
12. `Rgb`, `ColorGradient`, `PresetParams`, `Preset`, and their error types are all `pub` and reachable as `planet_core::color::rgb::*`, `planet_core::color::color_gradient::*`, `planet_core::presets::preset_params::PresetParams`, `planet_core::presets::preset::Preset` (verified via `cargo doc -p planet-core --no-deps`)
13. `planet-renderer` is untouched — no file under `planet-renderer/src/` changes
14. All scenarios in `rgb.feature`, `color_gradient.feature`, `preset_params.feature`, and `preset.feature` pass via real `cucumber` step definitions — no undefined/stub steps
15. Sampling a `ColorGradient` exactly at any interior stop's elevation (not just the first/last) returns that stop's `Rgb` bit-for-bit, verified with at least one non-black/non-zero-channel stop pair (not just the black/gray/white fixture) — proving the guarantee holds generally, not only when a bracket's lower endpoint happens to be `0.0`
16. `cargo test --workspace && cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo build --target wasm32-unknown-unknown -p planet-renderer` passes
