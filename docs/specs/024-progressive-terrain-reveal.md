# 024 — Progressive Terrain Reveal

**Status:** Ready for review
**Feature slug:** `progressive-terrain-reveal`

This is an ad-hoc corrective feature, not the next sequential `docs/roadmap.md` phase — triggered by a visual bug reported directly against the live-rendering growth animation shipped in `022-subdivision-frame-pacing.md`/`023-nonblocking-subdivision-generation.md`: every intermediate subdivision round shown during generation renders as a bare, undisplaced icosahedron-derived mesh; the full terrain elevation only appears in one atomic jump on the very last frame.

## Investigation

`Planet::subdivide` (`planet-core/src/planets/planet.rs`) currently runs `subdivide()` (the full `max_depth`-round combinatorial subdivision loop) to completion first, and only *after* that loop returns does it call `apply_terrain_noise` — once, as a whole-mesh pass — per `017-geodesic-terrain-rework.md`'s architecture. The round-by-round `on_progress` callback (`subdivision/subdivide.rs`'s `update_cb`, invoked once per completed round from *inside* `subdivide()`'s loop) therefore always receives a mesh with every vertex still at (approximately) unit radius — the only position perturbation present at that point is `processor/jitter.rs`'s small tangential/normal nudge applied during the split itself, unrelated to elevation. Since this is exactly the callback `022`/`023`'s growth animation renders frame-by-frame, the user-visible result is a perfectly round, undetailed sphere growing facet-by-facet, followed by a single jarring "pop" to full mountains/oceans/color once the terminal, fully-postprocessed `Planet` is delivered.

The requested fix: interleave terrain-noise application into the subdivision loop itself, so every round's mesh — the same mesh hand ed to `on_progress` and therefore rendered by the growth animation — already carries its own share of elevation detail, growing progressively rather than popping in at the end. Two designs were considered for how each round's noise gets "a smaller factor" than the previous round's; the chosen design (confirmed directly, since it is a real architectural fork with no single obviously-correct answer) is:

**Reveal one additional fBm octave per completed subdivision round.** Round `r`'s terrain-noise application uses `octaves = min(r, terrain_noise.octaves())` — round 1 reveals just the coarsest octave, round 2 adds the next, and so on, until `terrain_noise.octaves()` is reached (after which every further round keeps resampling the same full function). `TerrainNoise`'s existing `persistence` field (already validated `0.0..=1.0`, `processor/terrain_noise.rs`) already scales each successive octave's raw contribution by `persistence^i` before the `noise` crate's `Fbm` sums them — so "each new round's factor is smaller than the last" falls directly out of each preset's already-configured, already-tuned knobs, with **no new `TerrainNoise` field**. Each round's application is an absolute, deterministic function of `(direction, octaves-through-r)` — it *replaces* a vertex's radius outright, rather than adding a delta on top of whatever the previous round left there — which avoids reintroducing the class of unbounded cross-round compounding drift `017` deliberately eliminated (the pre-017 per-split Bernoulli-draw radial-displacement model `008`/`009`/`016` documented and then removed). Because the *next* round's chord-midpoint calculation runs on the mesh this round's transform just returned, elevation still genuinely feeds forward into subsequent geometry — later, finer vertices are chord midpoints of already-mountain-shaped neighbors, not of a perfect sphere — giving the result real fractal self-similarity (matching this project's name, and the user's literally described split → noise → split → noise workflow), while each individual noise *application* stays a bounded, non-drifting, pure function of vertex direction and revealed-octave count.

The alternative considered — a new explicit per-round amplitude-decay field, resampling all configured octaves at every round but scaling the overall amplitude down each time — was rejected: it reveals every octave's frequency content immediately at round 1 on a coarse (12–42-vertex) mesh, risking visible aliasing before there are enough vertices to represent that detail, and it would add a new per-preset tunable knob where the octave-reveal design needs none.

## Requirements

- **`planet-core/src/subdivision/subdivision_args.rs`:** `UpdateCallback` changes from a read-only observer (`Box<dyn FnMut(&Mesh, usize)>`) to a mesh transform (`Box<dyn FnMut(Mesh, usize) -> Result<Mesh, MeshError>>`).
- **`planet-core/src/subdivision/subdivide.rs`:** `subdivide()`'s loop threads the transform's return value back into `current` before the next round, propagating any `Err` immediately. The strategy instance (and therefore its RNG state, for `UniformRedSplit`'s jitter) is still constructed exactly once per `subdivide()` call — this feature does not change how `subdivide()` is *invoked* (`Planet::subdivide` still makes one call covering the whole `max_depth`), only what its per-round hook is allowed to do.
- **`planet-core/src/processor/terrain_noise.rs`** gains `pub fn apply_terrain_noise_for_round(mesh: &Mesh, seed: Seed, terrain_noise: TerrainNoise, revealed_octaves: u32) -> Result<Mesh, MeshError>` — identical per-vertex math to `apply_terrain_noise`, but the underlying `Fbm`'s octave count is `revealed_octaves.clamp(1, terrain_noise.octaves())` instead of always `terrain_noise.octaves()`. Both functions delegate to one new private helper so `apply_terrain_noise`'s own existing contract/tests are untouched — it becomes a one-line wrapper around the same helper called with the full octave count.
- **`planet-core/src/planets/planet.rs`:** `Planet::subdivide` restructured — terrain noise is applied once per round (via the new `UpdateCallback` transform), not once after the loop. `apply_ocean_quota` is unaffected: still applied exactly once, after the full round loop, since sea level is a whole-mesh percentile over *final* elevations.
- **`planet-core/src/planets/postprocess_stage.rs`:** `PostprocessStage` shrinks to its one remaining variant, `OceanQuota` (mirroring `017`'s precedent for `SubdivisionMode`: shrink an enum to what's left, rather than delete the type, when further collapsing it is its own out-of-scope cleanup). Terrain noise is no longer a discrete, separately-observable post-subdivision "stage" — it is folded into every subdivision round, already covered by the round-by-round `on_progress` callback that fires regardless. `on_postprocess` is therefore invoked 0 times (no ocean quota configured) or exactly 1 time (`OceanQuota`), never 2.
- **`planet-renderer/src/worker/protocol.rs`:** `WorkerMessage::PostprocessStage`'s `JsValue` string conversion drops its `"TerrainNoise"` match arm on both directions (no longer a valid variant).
- **`rules.md`** updated: `subdivision/`'s `subdivide.rs` entry describes the new transform-and-continue `update_cb` contract; `processor/`'s `terrain_noise.rs` entry gains `apply_terrain_noise_for_round`; `planets/`'s `postprocess_stage.rs` entry describes the one remaining `OceanQuota`-only variant.

**Out of scope:**
- Any change to `processor/jitter.rs` itself (the tangential/normal nudge applied during subdivision, unrelated to terrain elevation)
- Any change to `GrowthAnimation`/`FRAME_INTERVAL_MS` pacing (`022`/`023`) — frames still arrive at the same cadence; only their *content* (elevation/colors) changes, since this feature is scoped entirely to `planet-core`'s subdivision/postprocessing pipeline
- Restoring per-round *additive* compounding elevation (the pre-`017` model `008`/`009`/`016` used and `017` deliberately removed) — this feature's octave-reveal design is deliberately absolute-set per round, not additive, specifically to avoid reintroducing that class of bug
- Re-tuning any preset's `TerrainNoise` constants — visual tuning is out of BDD scope per `000-architecture.md`, "manually verified in-browser per milestone," same as every prior terrain-related spec
- Any change to `apply_ocean_quota`'s own algorithm or its single, whole-mesh, end-of-pipeline placement
- `planet-renderer/src/bin/generation_worker.rs`'s exact frame-forwarding behavior on the last round (its current `if round == last_round { return; }` skip, added because the raw pre-`017`-postprocessing last round was uninteresting to show — now that every round's mesh already carries full accumulated elevation, whether to keep, relax, or adjust that skip to avoid a pre-ocean-quota/pre-sea-level-color flash is an implementation nuance for `planet-tdd`/manual in-browser verification, not a spec-level contract; `planet-renderer` has no BDD suite for worker/wasm glue, per its established convention for `app.rs`)

## Domain model involved

### Changed

- **`planet-core/src/subdivision/subdivision_args.rs`:**
  ```rust
  pub type UpdateCallback = Box<dyn FnMut(Mesh, usize) -> Result<Mesh, MeshError>>;
  ```
  (was `Box<dyn FnMut(&Mesh, usize)>`; needs `MeshError` imported alongside the existing `Mesh` import)

- **`planet-core/src/subdivision/subdivide.rs`:**
  ```rust
  pub fn subdivide(mesh: &Mesh, mut args: SubdivisionArgs) -> Result<Mesh, MeshError> {
      let mut strategy = args.mode.strategy(args.seed);
      let mut current = mesh.clone();
      for step in 1..=args.steps.value() {
          current = split_round(&current, strategy.as_mut())?;
          if let Some(update_cb) = args.update_cb.as_mut() {
              current = update_cb(current, step)?;
          }
      }
      Ok(current)
  }
  ```
  Only the `if let` body changes (`update_cb(&current, step)` observer call becomes `update_cb(current, step)?`, reassigning `current`); everything else is unchanged.

- **`planet-core/src/processor/terrain_noise.rs`:** the existing body of `apply_terrain_noise` (the `Fbm` construction through `Ok(mesh.with_repositioned(positions))`) moves into a new private helper, parameterized on octave count:
  ```rust
  fn sample_and_apply(
      mesh: &Mesh,
      seed: Seed,
      terrain_noise: TerrainNoise,
      octaves: u32,
  ) -> Result<Mesh, MeshError> {
      let noise = Fbm::<Perlin>::new(seed.value() as u32)
          .set_frequency(terrain_noise.frequency() as f64)
          .set_octaves(octaves as usize)
          .set_persistence(terrain_noise.persistence() as f64)
          .set_lacunarity(terrain_noise.lacunarity() as f64);
      // ...same per-vertex direction/sample/clamp/redistribution/terracing/floor logic as today...
  }

  pub fn apply_terrain_noise(
      mesh: &Mesh,
      seed: Seed,
      terrain_noise: TerrainNoise,
  ) -> Result<Mesh, MeshError> {
      sample_and_apply(mesh, seed, terrain_noise, terrain_noise.octaves())
  }

  pub fn apply_terrain_noise_for_round(
      mesh: &Mesh,
      seed: Seed,
      terrain_noise: TerrainNoise,
      revealed_octaves: u32,
  ) -> Result<Mesh, MeshError> {
      let octaves = revealed_octaves.clamp(1, terrain_noise.octaves());
      sample_and_apply(mesh, seed, terrain_noise, octaves)
  }
  ```
  No change to `TerrainNoise`/`TerrainNoiseError`'s own fields, validation, or accessors.

- **`planet-core/src/planets/planet.rs`:** `Planet::subdivide` restructured to build one closure — capturing `self.seed`, `params.terrain_noise()`, and the caller's own `on_progress` — supplied as `SubdivisionArgs`'s `update_cb`:
  ```rust
  pub fn subdivide(
      &self,
      max_depth: Steps,
      on_progress: Option<GenerationProgress>,
      on_postprocess: Option<PostprocessProgress>,
  ) -> Result<Planet, PlanetError> {
      let params = self.preset.params();
      let terrain_noise = params.terrain_noise();
      let seed = self.seed;

      let mut on_progress = on_progress;
      if let Some(callback) = on_progress.as_mut() {
          callback(&self.mesh, 0);
      }

      let round_transform: UpdateCallback = Box::new(move |mesh, round| {
          let revealed_octaves = (round as u32).min(terrain_noise.octaves());
          let noised = apply_terrain_noise_for_round(&mesh, seed, terrain_noise, revealed_octaves)?;
          if let Some(callback) = on_progress.as_mut() {
              callback(&noised, round);
          }
          Ok(noised)
      });

      let args = SubdivisionArgs::new(
          Some(max_depth),
          Some(params.subdivision_mode()),
          Some(seed),
          Some(round_transform),
      );
      let mesh = subdivide(&self.mesh, args)?;

      let mut on_postprocess = on_postprocess;
      let mesh = match params.ocean_quota() {
          Some(quota) => {
              if let Some(callback) = on_postprocess.as_mut() {
                  callback(PostprocessStage::OceanQuota);
              }
              apply_ocean_quota(&mesh, quota)?
          }
          None => mesh,
      };
      // ...sea-level computation, colors, finalize_normals: unchanged...
  }
  ```
  The standalone post-loop `apply_terrain_noise` call is removed — the final round's own transform already leaves the mesh fully elevated through `terrain_noise.octaves()` (since `revealed_octaves` saturates there once `max_depth >= octaves`). Round 0's callback invocation (the pre-loop `callback(&self.mesh, 0)`) is unaffected — still the plain, unmodified base mesh; the first noise application happens after round 1's split, matching the user's literal "split, then noise" ordering.

- **`planet-core/src/planets/postprocess_stage.rs`:** `PostprocessStage` shrinks to `pub enum PostprocessStage { OceanQuota }`.

- **`planet-renderer/src/worker/protocol.rs`:** the `PostprocessStage <-> String` conversion's `"TerrainNoise"` arms (both directions) are removed.

- **`rules.md`:** as described in Requirements.

### Unchanged

`Mesh`/`Vertex`/`Edge`/`Face`/`Vec3`, `TerrainNoise`/`TerrainNoiseError` (fields/validation/accessors), `apply_ocean_quota`/`OceanQuota`, `SubdivisionStrategy`/`UniformRedSplit`/`jitter`, `Seed`, `Steps`, `SubdivisionMode`, `PresetParams`/`Preset` (no new/changed fields), `finalize_normals`, `GenerationProgress`'s own type shape (still `Box<dyn FnMut(&Mesh, usize)>` — only `SubdivisionArgs`'s *internal* `update_cb` type changes; the public callback the renderer supplies to `Planet::subdivide` is untouched), `GrowthAnimation`/`FRAME_INTERVAL_MS`, `PackedFrame`/`pack_frame`, `Renderer`.

## Function/API contracts

### `subdivide(mesh, args)` (updated contract)

- **Pre:** unchanged (`args.steps` within `[0, MAX_SUBDIVISION_STEPS]`)
- **Post:** for `step` in `1..=args.steps.value()`: computes `split_round`, then — if `args.update_cb` is `Some` — replaces `current` with the callback's returned `Mesh` (propagating `Err` immediately via `?`, aborting remaining rounds) before continuing to the next round's `split_round`. If `args.update_cb` is `None`, behavior is identical to before this feature.
- A transform that returns its input mesh unchanged reproduces the *observer* behavior every existing caller relied on before this feature — this is exactly what the existing "recording update callback" test double (`subdivide.feature`/`subdivide.rs`) needs to do going forward: clone the mesh for its own recording, then return the original mesh unchanged. Its existing 2-scenario contract ("invoked once per completed round with that round's mesh" / "never invoked at 0 steps") continues to hold verbatim.
- **New guarantee this feature adds:** a transform that returns a *different* mesh than it received causes every subsequent round to subdivide that different mesh, not the plain combinatorial result — `subdivide()` has no opinion on what the transform does, only that it re-threads the return value.

### `apply_terrain_noise_for_round(mesh, seed, terrain_noise, revealed_octaves)`

- **Pre:** `mesh` any valid `Mesh`; `terrain_noise` any already-validated `TerrainNoise`; `seed` any `Seed`; `revealed_octaves` any `u32`
- **Post:** identical per-vertex math to `apply_terrain_noise` (direction normalization with the same zero-radius guard, fBm sample, `[-1,1]` clamp, redistribution exponent, optional terracing, `MIN_VERTEX_RADIUS` floor), except the underlying `Fbm`'s octave count is `revealed_octaves.clamp(1, terrain_noise.octaves())` — so `revealed_octaves == 0` behaves identically to `1` (never an octave-less/all-flat sample), and any `revealed_octaves >= terrain_noise.octaves()` behaves identically to `terrain_noise.octaves()` (never more detail than the preset configures).
- **Consistency with `apply_terrain_noise`:** `apply_terrain_noise_for_round(mesh, seed, tn, tn.octaves())` produces a bit-identical `Mesh` to `apply_terrain_noise(mesh, seed, tn)`, for any `mesh`/`seed`/`tn` — same bound, same determinism guarantee, same zero-vertex/origin-vertex/empty-mesh handling as `apply_terrain_noise`'s existing (`017`) contract; none of that changes.
- **Bound:** every output vertex's radius lies in `[max(MIN_VERTEX_RADIUS, 1.0 - amplitude), 1.0 + amplitude]`, at every `revealed_octaves` value — unchanged from `apply_terrain_noise`'s own bound, since octave count doesn't affect the `[-1,1]` clamp/redistribution/terracing steps that establish it.
- **Determinism:** identical `(mesh, seed, terrain_noise, revealed_octaves)` always produces a bit-identical output `Mesh`.

### `Planet::subdivide(max_depth, on_progress, on_postprocess)` (updated contract)

- **Pre:** unchanged
- **Post:**
  1. Calls `on_progress` (if `Some`) with round `0` and the unmodified base mesh, exactly as before.
  2. For each round `r` in `1..=max_depth.value()`: subdivides one round, applies `apply_terrain_noise_for_round(mesh, self.seed, params.terrain_noise(), min(r as u32, params.terrain_noise().octaves()))`, then calls `on_progress` (if `Some`) with round `r` and the resulting *already-elevated* mesh.
  3. After all rounds complete: if `params.ocean_quota()` is `Some(quota)`, calls `on_postprocess` (if `Some`) with `PostprocessStage::OceanQuota`, then applies `apply_ocean_quota`. If `None`, `on_postprocess` is never called.
  4. Sea-level computation, per-vertex colors, and `finalize_normals` proceed exactly as before, over whatever mesh step 3 produced.
- **Determinism:** identical `(self, max_depth)` still always produces a bit-identical final `Planet` — unaffected by whether `on_progress`/`on_postprocess` are supplied.
- **Behavioral change from before this feature (intentional — affects existing BDD scenario *values*, not their pass/fail status):** the final mesh's exact vertex positions are no longer guaranteed identical to what this same `(seed, preset, max_depth)` produced before this spec, even when `max_depth >= params.terrain_noise().octaves()` — because later rounds' chord-midpoint calculations now run on already-elevated neighbor positions instead of a perfectly combinatorial (elevation-free) mesh, so the *set* of directions later octaves sample at differs from before this feature. Every existing `planet.feature` scenario asserting *bounds*, *counts*, or *same-seed-implies-identical-result* continues to hold; none asserts an exact vertex position or exact color value, so none require value changes — only the two `PostprocessStage`-observing scenarios need rewriting, since `TerrainNoise` is no longer a reported stage.

## BDD scenarios

### `planet-core/tests/features/subdivide.feature` (extended)

```gherkin
  Scenario: An update callback that returns a modified mesh causes the next round to subdivide that modified mesh
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit with seed 7 and an update callback that doubles every vertex's radius after round 1
    Then every vertex of the resulting Mesh has exactly double the radius that an unmodified 2-step subdivision with the same seed would have produced at the corresponding vertex
```

The existing "The update callback is invoked once per completed round with that round's mesh" / "Subdividing with 0 steps never invokes the update callback" scenarios are unchanged in wording; their step definitions' recording test double is updated to clone the received mesh for its own bookkeeping, then return it unchanged (an identity transform), preserving both scenarios' pass/fail status verbatim.

### `planet-core/tests/features/apply_terrain_noise.feature` (extended)

```gherkin
  Scenario: Applying terrain noise for round 1 uses just the first octave
    Given an icosahedron mesh
    And a TerrainNoise with amplitude 0.2 and 4 octaves
    When terrain noise for round 1 is applied to that mesh with seed 7 and that TerrainNoise
    Then the resulting Mesh is identical to applying terrain noise for round 1 with a TerrainNoise with amplitude 0.2 and 1 octave

  Scenario: Applying terrain noise for a round at or beyond the configured octave count matches the whole-mesh function
    Given an icosahedron mesh
    And a TerrainNoise with amplitude 0.2 and 4 octaves
    When terrain noise for round 4 is applied to that mesh with seed 7 and that TerrainNoise
    Then the resulting Mesh is identical to applying terrain noise to that mesh with seed 7 and that TerrainNoise

  Scenario: Applying terrain noise for round 0 behaves like round 1
    Given an icosahedron mesh
    And a TerrainNoise with amplitude 0.2 and 4 octaves
    When terrain noise for round 0 is applied to that mesh with seed 7 and that TerrainNoise
    Then the resulting Mesh is identical to applying terrain noise for round 1 with that same TerrainNoise

  Scenario: Applying terrain noise for a round never displaces a vertex beyond amplitude bounds, regardless of revealed octaves
    Given an icosahedron mesh
    And a TerrainNoise with amplitude 0.2 and 4 octaves
    When terrain noise for round 2 is applied to that mesh with seed 7 and that TerrainNoise
    Then every vertex of the resulting Mesh has a radius less than or equal to 1.2
    And every vertex of the resulting Mesh has a radius greater than or equal to 0.8

  Scenario: Applying terrain noise for a round is deterministic for a given seed
    Given an icosahedron mesh
    And a TerrainNoise with amplitude 0.2 and 4 octaves
    When terrain noise for round 2 is applied to that mesh with seed 7 and that TerrainNoise, producing the first Mesh
    And terrain noise for round 2 is applied to the same icosahedron mesh with seed 7 and that TerrainNoise, producing the second Mesh
    Then the first Mesh and the second Mesh are identical
```

### `planet-core/tests/features/planet.feature` (rewritten postprocess-stage scenarios, plus new progressive-reveal scenarios)

Replaces the two `023`-added scenarios (`"... reports both postprocessing stages in order"` / `"... reports only the terrain-noise stage"`) with:

```gherkin
  Scenario: Subdividing a Planet with an ocean-quota preset reports the ocean-quota stage
    Given a Planet generated with seed 5 and the Earthy preset at max depth 2
    When that Planet is subdivided again with a postprocessing-stage observer
    Then the observer received [OceanQuota] only

  Scenario: Subdividing a Planet with a preset that has no ocean quota reports no postprocessing stage
    Given a Planet generated with seed 5 and the Rocky preset at max depth 2
    When that Planet is subdivided again with a postprocessing-stage observer
    Then the observer received no postprocessing stages
```

New scenarios — the direct regression test for this feature's whole purpose:

```gherkin
  Scenario: Each subdivision round's reported mesh already carries elevation, not just the final round
    Given a recording progress callback
    When a Planet is generated with seed 9 and the Earthy preset at max depth 3 using that callback
    Then the progress callback's 1st invocation received round 0 with the base icosahedron mesh
    And the progress callback's 2nd invocation received a Mesh where at least one vertex's radius differs from 1.0
    And the progress callback's 3rd invocation received a Mesh where at least one shared vertex's radius differs from that vertex's radius in the 2nd invocation's Mesh

  Scenario: A Planet's final mesh is unaffected by whether a progress callback was supplied
    Given a Planet generated with seed 9 and the Earthy preset at max depth 4
    When another Planet is generated with seed 9 and the Earthy preset at max depth 4 using a recording progress callback
    Then the two Planets have identical meshes
```

## Acceptance criteria

1. `subdivide()`'s `UpdateCallback` type is `Box<dyn FnMut(Mesh, usize) -> Result<Mesh, MeshError>>`; a `None` callback, or an identity-returning callback, preserves every pre-existing `subdivide()` BDD scenario verbatim (compile-time + BDD test)
2. A transform callback that returns a different mesh causes every subsequent round to subdivide that different mesh, not the original combinatorial result (BDD test)
3. `apply_terrain_noise_for_round(mesh, seed, tn, tn.octaves())` produces a bit-identical `Mesh` to `apply_terrain_noise(mesh, seed, tn)`, for any valid inputs (BDD test) — the direct regression guard that this refactor doesn't change `apply_terrain_noise`'s existing observable behavior
4. `apply_terrain_noise_for_round`'s `revealed_octaves` is clamped to `[1, tn.octaves()]` — `0` behaves like `1`, anything `>= tn.octaves()` behaves like `tn.octaves()` (BDD test)
5. `apply_terrain_noise_for_round` obeys the same bound (`[max(MIN_VERTEX_RADIUS, 1.0 - amplitude), 1.0 + amplitude]`) and determinism guarantee as `apply_terrain_noise`, at every `revealed_octaves` value (BDD test)
6. `Planet::subdivide` no longer makes a standalone whole-mesh `apply_terrain_noise` call after the round loop (`grep` in `planet.rs` finds only `apply_terrain_noise_for_round`, never bare `apply_terrain_noise`)
7. `Planet::subdivide`'s round-0 callback invocation is unaffected — still receives the plain, unmodified base mesh (existing BDD scenario, unmodified, still passes)
8. For a `Planet` generated with a progress callback, round `r >= 1`'s reported mesh has at least one vertex with radius `!= 1.0` whenever `terrain_noise.amplitude() > 0.0` (BDD test) — the direct regression test for "displacement applied after each split, not just at the end"
9. For a `Planet` generated with a progress callback and `max_depth >= 2`, round `r+1`'s reported mesh has at least one shared vertex whose radius differs from that same vertex's radius at round `r` (BDD test) — confirms detail keeps changing round over round, not converging after round 1
10. `Planet::subdivide`'s final mesh/colors are unaffected by whether a progress callback is supplied (BDD test)
11. `PostprocessStage` has exactly one variant, `OceanQuota`; `on_postprocess` is invoked exactly once (with `OceanQuota`) for a preset with an ocean quota, and never for a preset without one (BDD test) — supersedes the two `023` scenarios asserting a `TerrainNoise` stage
12. `PostprocessStage::TerrainNoise` no longer exists anywhere in `planet-core`'s public API (compile-time check)
13. `planet-renderer/src/worker/protocol.rs`'s `WorkerMessage::PostprocessStage` string conversion has no `"TerrainNoise"` match arm on either direction (`cargo build --target wasm32-unknown-unknown -p planet-renderer` succeeds)
14. Every triangle/vertex-count invariant from `017`/`023` (e.g. `20 * 4^max_depth` triangles for every preset at every depth) continues to hold — this feature changes elevation *values*, not subdivision *topology* (existing BDD scenarios, unmodified, still pass)
15. `rules.md` updated for `subdivide.rs`'s new transform contract, `terrain_noise.rs`'s new function, and `postprocess_stage.rs`'s shrunk enum
16. `apply_terrain_noise_for_round`/the restructured `Planet::subdivide` contain no `unwrap()`/`panic!()`/`.expect()` in production code
17. All BDD scenarios above are backed by real `cucumber` step definitions in their respective `.feature` files and matching step-definition modules — no scenario left as markdown prose
18. `cargo test --workspace && cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo build --target wasm32-unknown-unknown -p planet-renderer` all pass
19. Manual in-browser check (all three presets, depth 5+): the growth animation visibly shows terrain elevation (shape and color) building up progressively round over round, with no single "pop" of full elevation on the last frame — the direct visual confirmation of this feature's whole purpose
