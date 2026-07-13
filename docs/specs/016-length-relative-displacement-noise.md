# 016 — Length-Relative Displacement Noise

**Status:** Ready for review
**Feature slug:** `length-relative-displacement-noise`

This is `docs/roadmap.md`'s own "008 — Length-relative displacement noise" phase — it becomes spec number `016` only because eight ad-hoc refactor/feature specs (`005`, `006`, `008` [`strategies-module`], `010` [`vertex-scramble`], `011`–`015`) already claimed the intervening numbers, the same numbering drift every prior spec in this line has already called out for its neighbors. Scope matches the roadmap line: `radial_displacement`/`normal_displacement` (`planet-core/src/processor/`) currently sample a fixed absolute magnitude from `ElevationNoiseRange`/`NormalNoiseRange` every round, regardless of how long or short the edge being split currently is. Presets with a tight `min_edge_length` (`Volcano` at `0.25`, `Rocky` at `0.3`, versus `Earthy`'s `0.35`) subdivide many more rounds before their edges converge below that threshold, so by the time an edge is only a few hundredths of a unit long, the noise sampled for it is still drawn from the same range that was tuned for a full-size icosahedron edge (~`1.05` units) — displacement several times larger than the local edge it's applied to, which is what produces the reported "too spiky" look. This feature scales the sampled delta by the current edge's length instead of applying it as a fixed magnitude, so displacement stays proportionate to local mesh resolution no matter how many rounds a preset's tighter threshold forces.

## Requirements

- `radial_displacement(range: ElevationNoiseRange) -> VertexOperator` (`planet-core/src/processor/radial_displacement.rs`) and `normal_displacement(range: NormalNoiseRange) -> VertexOperator` (`planet-core/src/processor/normal_displacement.rs`) both change their sampled `delta` from an absolute magnitude to `edge_length * sampled_fraction`, where `edge_length = b.position.sub(a.position).length()` is computed from the two edge endpoints (`a`, `b`) **already passed into every `VertexOperator` call** — `VertexOperator`'s existing signature (`Box<dyn Fn(&mut Pcg32, &Vertex, &Vertex, Vertex) -> Vertex>`, `planet-core/src/processor/vertex_operator.rs`) is unchanged; both functions currently ignore `a`/`b` (`radial_displacement` even names them `_a, _b`) and only need to stop ignoring them
- `range.low()`/`range.high()` on both `ElevationNoiseRange` and `NormalNoiseRange` are deliberately, semantically redefined: from "absolute displacement magnitude in mesh units" to **"a fraction of the edge currently being split."** `ElevationNoiseRange::new(-0.05, 0.15)` now means "between -5% and +15% of this edge's length," not "between -0.05 and +0.15 mesh units." This is a documented breaking change in *meaning*, not in the types themselves — no field, method, or constructor on either range type is renamed or resized; only what the numbers represent changes, exactly as the roadmap phase describes ("scaling the sampled delta by the current edge's length instead of applying a fixed magnitude")
- Because `radial_displacement`/`normal_displacement` are the shared building blocks both `RadialRandomSplit` and `RedGreenSplit` compose via `compose(radial_displacement(...), normal_displacement(...))` (per `011-vertex-operator-composition.md`), both strategies pick up length-relative displacement automatically — **no changes to either strategy's own file, to `compose`, to `identity`, or to `UniformRedSplit`** (which uses `identity()` and never samples noise at all, so it is entirely unaffected)
- The existing `radius == 0.0` guard in `radial_displacement` and the existing degenerate-cross-product guard in `normal_displacement` are unchanged. A **new**, strictly stronger guarantee falls out of the length-relative formula for free: whenever `edge_length == 0.0` (a degenerate, coincident-endpoint edge), `delta` is always exactly `0.0` regardless of the sampled fraction or the configured range's bounds — a zero-length edge can never be displaced, full stop, without needing a separate explicit check
- `RedGreenSplit::maybe_split`'s stopping comparison (`length < self.min_edge_length.value()`, `planet-core/src/subdivision/strategies/red_green_split.rs`) is explicitly **unchanged** by this feature — this feature only changes how a split point, once the decision to split has already been made, gets displaced; it never changes whether an edge gets split. (Roadmap phase 009 owns reviewing that comparison itself.)
- `docs/specs/007-radial-randomness.md`'s documented invariant — `` radius <= 1.0 + steps * elevation_noise_range.high() `` — is stale under the new semantics and is updated (documentation-only edit to that historical spec file) to reference the length-relative formula this spec establishes (see Function/API contracts)
- `planet-core/tests/features/subdivide.feature`'s two existing hardcoded radius-bound scenarios (`RadialRandomSplit` at 2 steps, `RedGreenSplit` at 1 step) are recomputed under the new formula and their asserted upper bounds updated (see BDD scenarios)
- A new `subdivide.feature` scenario demonstrates the actual fix: subdividing to a high step count (`MAX_SUBDIVISION_STEPS = 8`) with a tight `MinEdgeLength` (forcing many rounds, the `Volcano`/`Rocky`-style case the roadmap item calls out) keeps every vertex radius within a bound that is **independent of step count**, unlike the old formula's `1.0 + steps * high()`, which grows without limit as steps increase
- `planet-core/tests/features/planet.feature`'s `Rocky`-preset radius-bound scenario (line 18-21, currently asserting `<= 1.4`) uses `Rocky`'s real, nonzero `split_point_variance` (`0.25`), which defeats the simple zero-variance edge-halving argument the other two updated scenarios rely on (a highly off-center Gaussian split point can leave one child edge close to the parent's full length instead of roughly half). Its exact new numeric bound is **not hand-derived in this spec** — it is reverified empirically during `planet-tdd` by running the actual implementation with its fixed seed/preset/depth and reading off the real maximum, then asserting a safely-rounded-up bound against that concrete output. The *kind* of assertion (an upper and lower radius bound) is unchanged; only the asserted value is recomputed from real output instead of a formula

Out of scope:
- Changing `MinEdgeLength`'s stopping-condition comparison, or anything else about when an edge is or isn't split (roadmap phase 009)
- Changing `VertexOperator`'s signature, `compose`, `identity`, `gaussian_split_point`, `exact_midpoint`, `EdgeCache`, `MIN_VERTEX_RADIUS`, `MIN_SPLIT_T`/`MAX_SPLIT_T`, or either strategy's `split_triangle` triangulation logic
- Renaming `ElevationNoiseRange`/`NormalNoiseRange`, their fields, or their `low()`/`high()` methods — only their real-world meaning changes, not their shape
- Retuning `Preset::params()`'s actual numeric literals (`Earthy`/`Volcano`/`Rocky`'s `ElevationNoiseRange`/`NormalNoiseRange` values) to "look better" under the new semantics — the same numbers carry over unchanged as literals; whether they still produce a good-looking planet under the new interpretation is a follow-up tuning concern, not this feature
- `SplitPointVariance`'s Gaussian sampling or its own clamp bounds
- Any UI/control change in `planet-renderer` — this is a pure `planet-core` generation-logic change, no new knob is exposed

## Domain model involved

- `radial_displacement(range: ElevationNoiseRange) -> VertexOperator` (`planet-core/src/processor/radial_displacement.rs`) — existing `pub(crate)` function, body changes only, signature unchanged
- `normal_displacement(range: NormalNoiseRange) -> VertexOperator` (`planet-core/src/processor/normal_displacement.rs`) — existing `pub(crate)` function, body changes only, signature unchanged
- `VertexOperator` (`planet-core/src/processor/vertex_operator.rs`) — unchanged type alias; its existing `&Vertex, &Vertex` parameters (the edge's two endpoints) already supply everything the new `edge_length` computation needs
- `ElevationNoiseRange`, `NormalNoiseRange` (`planet-core/src/subdivision/elevation_noise_range.rs`, `.../normal_noise_range.rs`) — structurally unchanged (same fields, same validated constructor, same defaults); this feature reinterprets what their `low()`/`high()` values mean at the point of use, not their construction or validation
- `Preset::params()` (`planet-core/src/presets/preset.rs`) — its three variants' `ElevationNoiseRange`/`NormalNoiseRange` numeric literals are untouched by this feature, but their real-world effect changes because of the new interpretation (out of scope: retuning them)
- `RadialRandomSplit`, `RedGreenSplit` (`planet-core/src/subdivision/strategies/`) — unchanged files; both automatically inherit length-relative behavior through the `radial_displacement`/`normal_displacement` building blocks they already compose

## Function/API contracts

- `radial_displacement(range)(rng, a, b, point)`:
  - **New:** computes `edge_length = b.position.sub(a.position).length()`
  - **Changed:** computes `delta = edge_length * rng.random_range(range.low()..=range.high())` (previously `delta = rng.random_range(range.low()..=range.high())` directly, ignoring `a`/`b` entirely)
  - Unchanged: the `radius == 0.0` guard (returns `point` unchanged before ever computing `delta`), the `MIN_VERTEX_RADIUS` floor clamp on `new_radius`, and the single-ratio-scale application (`point.position.scale(new_radius / radius)`)
  - **New guarantee:** when `edge_length == 0.0`, `delta` is always exactly `0.0` regardless of `range`, so the result is bit-identical to `point` — this holds even for a non-zero-width range, subsuming the previous zero-width-range-only no-op guarantee
  - Still consumes exactly one `rng.random_range` draw per invocation, in the same call position as before — determinism and draw order (`constitution.md`'s non-negotiable constraint) are unaffected by this change
- `normal_displacement(range)(rng, a, b, point)`:
  - **New:** computes `edge_length = b.position.sub(a.position).length()`
  - **Changed:** computes `delta = edge_length * rng.random_range(range.low()..=range.high())` (previously the raw sampled value)
  - Unchanged: the `a.position.cross(b.position).normalized()` degenerate-normal guard (returns `point` unchanged when `a`, `b`, and the origin are collinear) and the displacement application (`point.position.add(normal.scale(delta))`)
  - Still consumes exactly one `rng.random_range` draw per invocation, same call position as before
- Both functions' signatures (`fn(range: X) -> VertexOperator`) are unchanged — this is a pure behavior change, not an interface change. No caller (`compose`, `RadialRandomSplit::new`, `RedGreenSplit::new`) requires any code change
- **New invariant, replacing `007-radial-randomness.md`'s `radius <= 1.0 + steps * elevation_noise_range.high()`:** in any single subdivision round, a newly created vertex's radial contribution is bounded by `parent_edge_length * elevation_noise_range.high()`, not a step-count-independent constant. Because every child edge this codebase's strategies produce is bounded (via the triangle inequality: a new vertex's total displacement from its exact, undisplaced split point has magnitude at most `parent_edge_length * (elevation_noise_range.high() + normal_noise_range.high())`) by `child_edge_length <= parent_edge_length * (0.5 + elevation_noise_range.high() + normal_noise_range.high())` when the split point is the exact midpoint (`RadialRandomSplit` always, `RedGreenSplit` when `split_point_variance` is `0.0`), edge lengths shrink geometrically round-over-round whenever `elevation_noise_range.high() + normal_noise_range.high() < 0.5` — true for all three current `Preset`s (`Earthy`: `0.15+0.05=0.20`; `Volcano`: `0.35+0.10=0.45`; `Rocky`: `0.2+0.15=0.35`). Consequently, the **total** accumulated radial displacement across unboundedly many rounds converges to a finite limit — `1.0 + L0 * (eh + nh) / (1 - (0.5 + eh + nh))` for base icosahedron edge length `L0` and per-round highs `eh`/`nh` — instead of growing linearly with `steps` the way the old fixed-magnitude formula did. This is the mathematical shape of the fix: previously, more rounds (a preset's tighter `min_edge_length`) meant strictly more accumulated noise; now it means the *same, bounded* amount of accumulated noise, spread across progressively finer local detail
  - A nonzero `split_point_variance` (`RedGreenSplit` with `Rocky`'s `0.25`, e.g.) loosens the `0.5` halving factor in the bound above — an off-center Gaussian split point can leave one child edge close to the parent's full length — so the closed-form convergence argument above is only established here for `split_point_variance == 0.0`; the general nonzero-variance case is not re-derived in this spec (see Requirements' note on `planet.feature`'s `Rocky` scenario)

## BDD scenarios

**Unit-test level** (`radial_displacement.rs`/`normal_displacement.rs`, matching this codebase's existing convention of unit-testing these two functions directly rather than through `cucumber` — they are not covered by any `.feature` file today, only exercised indirectly through `subdivide.feature`'s `RadialRandomSplit`/`RedGreenSplit` scenarios):

- Given edge endpoints `a = (1,0,0)`, `b = (0,1,0)` (edge length `√2 ≈ 1.41421`) and an `ElevationNoiseRange` of `(0.1, 0.1)` (zero-width, non-zero, forcing a known sampled fraction), when `radial_displacement` displaces a point at radius `1.0` along that edge, then the resulting radius equals `1.0 + √2 * 0.1 ≈ 1.14142` — proving the delta scales by edge length, not a fixed `0.1` (this is a **new** test; no existing test currently asserts a non-clamped, non-zero displaced value)
- Given the same edge endpoints and a `NormalNoiseRange` of `(0.05, 0.05)`, when `normal_displacement` displaces the fixture point already covered by `displaces_along_the_edge_plane_normal_by_the_drawn_delta`, then the expected displaced position updates from `Vec3::new(0.5, 0.5, 0.05)` to `Vec3::new(0.5, 0.5, √2 * 0.05 ≈ 0.070711)` (existing test, expected value updated)
- Given a degenerate edge whose two endpoints coincide (`edge_length == 0.0`) and any `ElevationNoiseRange`/`NormalNoiseRange` (including a wide, non-zero-width one), when either operator displaces a point on that edge, then the resulting position is bit-identical to the input — a **new** boundary scenario, since previously only a *zero-width range* guaranteed a no-op; now a *zero-length edge* guarantees one too, independent of the range
- The three existing `zero_width_range_leaves_position_bit_identical` and `radius_is_clamped_to_min_vertex_radius` tests in both files remain valid unmodified — a zero-width range still produces `delta == 0.0` regardless of edge length (`edge_length * 0.0 == 0.0`), and an extreme negative range still clamps to `MIN_VERTEX_RADIUS` regardless of edge length (the clamp fires either way)

**`planet-core/tests/features/subdivide.feature`** (existing scenarios' numeric bounds recomputed; one new scenario appended after the existing `RedGreenSplit` bound scenario):

Updated (line 83-87 today):
```gherkin
  Scenario: Subdividing the icosahedron mesh with SubdivisionMode::RadialRandomSplit keeps every vertex radius within the configured bound
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::RadialRandomSplit with seed 7, an ElevationNoiseRange of low -0.1 and high 0.1, and the default NormalNoiseRange
    Then every vertex of the resulting Mesh has a radius less than or equal to 1.27
    And every vertex of the resulting Mesh has a radius greater than or equal to 0.05
```
(Bound moves from `1.3` to `1.27` — derived from the icosahedron's actual edge length `L0 ≈ 1.05146`: round 1 contributes `L0 * 0.15 ≈ 0.15772`; round 2's edges are bounded by `L0 * 0.65 ≈ 0.68345`, contributing `0.68345 * 0.15 ≈ 0.10252`; total `1.0 + 0.15772 + 0.10252 ≈ 1.26024`, rounded up to `1.27`.)

Updated (line 151-155 today):
```gherkin
  Scenario: SubdivisionMode::RedGreenSplit keeps every vertex radius within the configured bound
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low -0.1 and high 0.1, the default NormalNoiseRange, a MinEdgeLength of 0.5, and a SplitPointVariance of 0.0
    Then every vertex of the resulting Mesh has a radius less than or equal to 1.16
    And every vertex of the resulting Mesh has a radius greater than or equal to 0.05
```
(Bound moves from `1.15` to `1.16` — barely changes at 1 round since the icosahedron's edges, at `≈1.05`, are still close to `1.0`; this scenario alone does not show the fix, the new scenario below does.)

New scenario (appended after the one above):
```gherkin
  Scenario: SubdivisionMode::RedGreenSplit's vertex radius bound does not grow with additional subdivision rounds
    Given an icosahedron mesh
    When the mesh is subdivided with 8 steps using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low -0.1 and high 0.1, the default NormalNoiseRange, a MinEdgeLength of 0.05, and a SplitPointVariance of 0.0
    Then every vertex of the resulting Mesh has a radius less than or equal to 1.46
    And every vertex of the resulting Mesh has a radius greater than or equal to 0.05
```
(`8` is `MAX_SUBDIVISION_STEPS`; `MinEdgeLength` of `0.05` forces roughly 5 full-mesh rounds before edges converge below threshold, exercising the same "tight-threshold, many-rounds" shape as `Volcano`/`Rocky`. `1.46` is the geometric series' convergence limit — `1.0 + L0 * 0.15 / (1 - 0.65) ≈ 1.0 + 0.45063 ≈ 1.45063` — rounded up, and holds regardless of step count once edges have started shrinking. Contrast: the **old**, pre-this-feature formula would have predicted `1.0 + 8 * 0.15 = 2.2` at this step count — this scenario is the direct proof that displacement no longer compounds unboundedly with subdivision depth.)

**`planet-core/tests/features/planet.feature`** (existing `Rocky`-preset scenario, line 18-21): its asserted bound (`<= 1.4`) is reverified against real `cargo test` output during `planet-tdd` rather than hand-derived here (see Requirements) — the scenario's Given/When/Then shape is unchanged, only the asserted numeric value is expected to change and must be confirmed empirically, not assumed.

## Acceptance criteria

1. `radial_displacement(range)(rng, a, b, point)` computes `delta = b.position.sub(a.position).length() * rng.random_range(range.low()..=range.high())`, not `delta = rng.random_range(...)` alone
2. `normal_displacement(range)(rng, a, b, point)` computes `delta = b.position.sub(a.position).length() * rng.random_range(range.low()..=range.high())`, not the raw sampled value alone
3. Neither function's signature changes; `compose`, `identity`, `RadialRandomSplit`, `RedGreenSplit`, `UniformRedSplit`, `subdivide`, and `Planet::subdivide` require no code changes and automatically exhibit the new behavior
4. For edge endpoints `a = (1,0,0)`, `b = (0,1,0)` and `ElevationNoiseRange::new(0.1, 0.1)`, `radial_displacement` applied to a point at radius `1.0` produces a radius of `1.0 + √2 * 0.1` (within float tolerance), not `1.0 + 0.1`
5. For the same edge endpoints and `NormalNoiseRange::new(0.05, 0.05)`, `normal_displacement`'s existing displaced-position test (`displaces_along_the_edge_plane_normal_by_the_drawn_delta`) is updated to expect `Vec3::new(0.5, 0.5, √2 * 0.05)`, not `Vec3::new(0.5, 0.5, 0.05)`
6. A zero-length edge (`a.position == b.position`) produces `delta == 0.0` in both functions for any configured range, including a wide, non-zero-width one — the resulting position is bit-identical to the input
7. The existing zero-width-range and extreme-negative-range clamp tests in both files continue to pass unmodified (same expected outcome, since both a `0.0` delta and a floor-clamped delta are edge-length-independent in their final effect)
8. `subdivide.feature`'s `RadialRandomSplit` radius-bound scenario (2 steps, seed 7, `ElevationNoiseRange(-0.1, 0.1)`, default `NormalNoiseRange`) asserts an upper bound of `1.27` (not `1.3`), verified against actual `cargo test` output
9. `subdivide.feature`'s `RedGreenSplit` radius-bound scenario (1 step, seed 7, `ElevationNoiseRange(-0.1, 0.1)`, default `NormalNoiseRange`, `MinEdgeLength(0.5)`, `SplitPointVariance(0.0)`) asserts an upper bound of `1.16` (not `1.15`), verified against actual `cargo test` output
10. A new `subdivide.feature` scenario subdivides the icosahedron with `SubdivisionMode::RedGreenSplit`, seed 7, `ElevationNoiseRange(-0.1, 0.1)`, default `NormalNoiseRange`, `MinEdgeLength(0.05)`, `SplitPointVariance(0.0)`, for `8` steps (`MAX_SUBDIVISION_STEPS`), and asserts every vertex radius is `<= 1.46` — verified against actual output, and the exact value tightened if the real maximum is lower
11. `planet.feature`'s `Rocky`-preset radius-bound scenario's asserted upper bound is re-verified (and updated if necessary) against real output from the implemented code, not assumed unchanged
12. `docs/specs/007-radial-randomness.md`'s documented radius invariant is updated to reference the new length-relative formula (documentation-only change)
13. All scenarios in `radial_displacement.rs`'s and `normal_displacement.rs`'s unit test modules, plus every updated/new `subdivide.feature` and `planet.feature` scenario, pass
14. `cargo test --workspace`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` all pass
15. `cargo build --target wasm32-unknown-unknown -p planet-renderer` still succeeds
16. No new `unwrap()`/`panic!()` in production code outside tests
17. Every other existing `.feature` file's scenarios (`vec3`, `mesh`, `icosahedron`, `steps`, `seed`, `elevation_noise_range`, `normal_noise_range`, `min_edge_length`, `split_point_variance`, `subdivision_args`, `color_gradient`, `preset`, `preset_params`, `ocean_quota`, `apply_ocean_quota`, `vertex_scramble`, `vertex_scramble_range`, `rgb`, and every `subdivide.feature`/`planet.feature` scenario not touched by this feature) still pass unmodified
