# 005 — Subdivision Facade

**Status:** Ready for review
**Feature slug:** `subdivision-facade`

This is an ad-hoc refactor spec, not the next numbered roadmap phase (`docs/roadmap.md`'s own "005 — Radial randomness" is unaffected and will simply become spec `006` whenever it is written) — it revisits `004-icosahedron-subdivision`'s public API before that phase builds on it.

## Requirements

- `planet-core` adopts the **Facade design pattern** to stop leaking the `SubdivisionStrategy` trait and its concrete implementations as public vocabulary: a new public `SubdivisionMode` enum (`planet-core/src/subdivision_mode.rs`) is the only subdivision-algorithm-selection type external callers ever see. `SubdivisionStrategy` (the trait) and `UniformRedSplit` (its sole concrete implementation) become `pub(crate)` — reachable only from within `planet-core`. `EdgeKey`/`EdgeCache` (`edge.rs`), which `004-icosahedron-subdivision` was forced to make `pub` purely as a side effect of `SubdivisionStrategy` itself being `pub`, revert to `pub(crate)` too, now that the root cause no longer applies — restoring `000-architecture.md`'s original framing of the edge cache as "not public domain vocabulary"
- `SubdivisionMode` has exactly one variant today, `UniformRedSplit`, matching the crate's sole implemented strategy; it implements `Default`, resolving to `UniformRedSplit`. Future strategies (`005-radial-randomness`, `006-irregular-subdivision` on the roadmap) each add a variant here, never a new public type
- `planet-core` gains a new validated newtype `Steps` (`planet-core/src/steps.rs`) wrapping the subdivision round count, instead of a bare `usize`: `pub struct Steps(usize)` (private field), constructed only via `Steps::new(usize) -> Result<Steps, StepsError>` — per `rules.md`'s "constructors that validate invariants return `Result` with a dedicated `Error` type" — which rejects any value greater than a new hard cap `MAX_SUBDIVISION_STEPS = 8` (`pub const`, per `000-architecture.md`'s "e.g. 1..=8" example range for the never-built `SubdivisionDepth` newtype — `Steps` now fulfills that newtype's intended role directly) with `StepsError::ExceedsMaximum { steps, max }`. `Steps` also implements `Default`, resolving to `Steps(3)` (the value `planet-renderer` previously hardcoded as `MAX_SUBDIVISION_DEPTH`) — since `Default` already guarantees a valid value, no fallible path is needed when a caller omits a step count
- `planet-core` gains a new `SubdivisionArgs` struct (`planet-core/src/subdivision_args.rs`) bundling everything a call to `subdivide` needs, replacing its old three-parameter `(mesh, depth: u32, strategy: &mut dyn SubdivisionStrategy)` signature with `(mesh, args: SubdivisionArgs)`:
  - `steps: Steps` — how many subdivision rounds to run; a caller who wants a non-default value first validates it via `Steps::new`, so by the time `SubdivisionArgs` sees a `Steps` value, it is already guaranteed within range
  - `mode: SubdivisionMode` — which algorithm to use; resolved from `None` to `SubdivisionMode::UniformRedSplit` (the only strategy implemented so far)
  - `update_cb: Option<UpdateCallback>` (`UpdateCallback = Box<dyn FnMut(&Mesh, usize)>`) — an optional callback invoked once per completed subdivision round, in round order, receiving the mesh as it exists immediately after that round and the round's 1-based number. This is what "solves the feedback loop with a single step": a caller that previously had to drive an external per-round loop (as `planet-renderer`'s `SubdivisionStepper` did) to observe intermediate meshes can now get the same round-by-round feedback from one call to `subdivide`
  - Constructed only via `SubdivisionArgs::new(steps: Option<Steps>, mode: Option<SubdivisionMode>, update_cb: Option<UpdateCallback>) -> SubdivisionArgs` — **infallible**, since all step-count validation already happened when the caller built the `Steps` value (or omitted it, taking `Steps::default()`); resolves `steps.unwrap_or_default()` and `mode.unwrap_or_default()`. Fields are `pub(crate)`, not `pub`, so `new` is the only way to obtain an instance
  - `SubdivisionArgs::steps(&self) -> Steps` and `SubdivisionArgs::mode(&self) -> SubdivisionMode` are `pub` read accessors (needed since the fields themselves are `pub(crate)`, and BDD coverage for `SubdivisionArgs` lives in an external `tests/` crate)
- `subdivide(mesh: &Mesh, args: SubdivisionArgs) -> Result<Mesh, MeshError>` keeps its existing return type and its existing "0 steps leaves the mesh unchanged" behavior (renamed from `depth` to `steps`, matching `SubdivisionArgs`'s field), and remains algorithm-agnostic — it resolves `args.mode` to a boxed strategy once via a new `pub(crate)` `SubdivisionMode::strategy(&self)` method, then loops `args.steps.value()` times exactly as before, calling `args.update_cb` (if present) after each round
- `planet-renderer` is updated to compile against the new facade and to replace its previous **interactive, per-keystroke** subdivision stepping (`SubdivisionStepper`, Space key advances one round) with a **single upfront `subdivide` call** at startup whose `update_cb` collects every intermediate mesh into a frame sequence that the app then reveals automatically, one round per subsequent redraw, producing an animated build-up instead of requiring manual key presses. `SubdivisionStepper` (`planet-renderer/src/subdivision_stepper.rs`), its BDD feature file, and its Cargo test entry are deleted; the Space key binding is removed (the `W` wireframe toggle is unaffected)

Out of scope for this feature:
- Any new `SubdivisionMode` variant beyond `UniformRedSplit`, or any change to the uniform red-split algorithm itself (`005-radial-randomness`, `006-irregular-subdivision` on the roadmap)
- A depth/step-count UI control — `planet-renderer` continues to pass `None` for `steps`, relying on `SubdivisionArgs`'s default of `3`, until `007-planet-presets` wires up a slider
- Any seed/RNG or determinism change — `UniformRedSplit` remains stateless and deterministic; `SubdivisionMode`/`SubdivisionArgs` introduce no randomness
- `Send`/`Sync` bounds or thread-safety for `UpdateCallback` — the app runs single-threaded in one browser tab, per the constitution
- Cancelling or early-exiting subdivision from within `update_cb` — the callback is a pure observer, it cannot stop the loop
- Exact frame-advance pacing/timing for `planet-renderer`'s animated build-up (e.g. throttling to a fixed interval) — `app.rs` is thin wiring, not BDD-tested per `rules.md`, and the precise cadence is an implementation detail decided during `planet-tdd` and verified manually in-browser
- Changing `Mesh`, `MeshError`, `Vec3`, or `icosahedron()` — none of those public contracts change

## Domain model involved

**`planet-core/src/subdivision_mode.rs` (new):**
- `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum SubdivisionMode { UniformRedSplit }` — the Facade's only forward-facing vocabulary for "which subdivision algorithm to use"
- `impl Default for SubdivisionMode { fn default() -> Self { SubdivisionMode::UniformRedSplit } }`
- `pub(crate) fn strategy(&self) -> Box<dyn SubdivisionStrategy>` — maps a variant to its concrete, crate-internal `SubdivisionStrategy` implementation (`SubdivisionMode::UniformRedSplit => Box::new(UniformRedSplit)`); the only place in the crate that constructs a `dyn SubdivisionStrategy`, and not reachable from outside the crate

**`planet-core/src/steps.rs` (new):**
- `pub const MAX_SUBDIVISION_STEPS: usize = 8;` — hard recursion cap (constitution: subdivision "always bounded by an explicit max-depth cap")
- private `const DEFAULT_STEPS: usize = 3;`
- `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct Steps(usize);` — private field, so a valid `Steps` can only come from `Steps::new` or `Steps::default()`
- `#[derive(Debug, Clone, PartialEq)] pub enum StepsError { ExceedsMaximum { steps: usize, max: usize } }` with `Display`/`std::error::Error` impls mirroring `MeshError`
- `impl Steps`:
  - `pub fn new(steps: usize) -> Result<Steps, StepsError>` — returns `Err(StepsError::ExceedsMaximum { steps, max: MAX_SUBDIVISION_STEPS })` if `steps > MAX_SUBDIVISION_STEPS`, otherwise `Ok(Steps(steps))`
  - `pub fn value(&self) -> usize` — returns the wrapped count
- `impl Default for Steps { fn default() -> Self { Steps(DEFAULT_STEPS) } }`

**`planet-core/src/subdivision_args.rs` (new):**
- `pub type UpdateCallback = Box<dyn FnMut(&Mesh, usize)>;`
- `pub struct SubdivisionArgs { pub(crate) steps: Steps, pub(crate) mode: SubdivisionMode, pub(crate) update_cb: Option<UpdateCallback> }`
- `impl SubdivisionArgs`:
  - `pub fn new(steps: Option<Steps>, mode: Option<SubdivisionMode>, update_cb: Option<UpdateCallback>) -> SubdivisionArgs` — infallible: resolves `steps.unwrap_or_default()` and `mode.unwrap_or_default()`, and returns `SubdivisionArgs { steps, mode, update_cb }`
  - `pub fn steps(&self) -> Steps`
  - `pub fn mode(&self) -> SubdivisionMode`

**`planet-core/src/subdivide.rs` (updated):**
- `SubdivisionStrategy` trait: visibility `pub` → `pub(crate)`; method signature unchanged
- `pub fn subdivide(mesh: &Mesh, mut args: SubdivisionArgs) -> Result<Mesh, MeshError>` (was `subdivide(mesh: &Mesh, depth: u32, strategy: &mut dyn SubdivisionStrategy)`): resolves a boxed strategy once via `args.mode.strategy()`, then for `step in 1..=args.steps.value()`, calls the private `split_round` helper and, if `args.update_cb` is `Some`, invokes it with `(&current_mesh, step)` immediately after that round succeeds. `args.steps.value() == 0` returns `Ok(mesh.clone())` without calling `split_round` or the callback, exactly as `depth == 0` did before
- `split_round` (private helper): unchanged signature and body, still takes `&mut dyn SubdivisionStrategy`

**`planet-core/src/uniform_red_split.rs` (updated):**
- `pub struct UniformRedSplit;` → `pub(crate) struct UniformRedSplit;`; body unchanged

**`planet-core/src/edge.rs` (updated):**
- `EdgeKey`, `EdgeCache`, and their methods: `pub` → `pub(crate)`

**`planet-core/src/lib.rs` (updated):**
```
mod edge;
pub mod icosahedron;
pub mod mesh;
pub mod steps;
pub mod subdivide;
pub mod subdivision_args;
pub mod subdivision_mode;
mod uniform_red_split;
pub mod vec3;
```
(`edge` and `uniform_red_split` become private modules; `steps`, `subdivision_args`, and `subdivision_mode` are new public modules)

**`planet-core/tests/features/subdivide.feature` / `planet-core/tests/subdivide.rs` (updated):**
- Given/When steps reworded to construct `SubdivisionArgs` (via `Steps`/`SubdivisionMode`) and reference `SubdivisionMode::UniformRedSplit` instead of a raw `depth`/`&mut UniformRedSplit`; existing core scenario set (face-count growth, no duplicate vertices, no cracks, radius bound) and algorithm-specific scenarios preserved with identical expected counts; new scenarios cover default-args resolution and `update_cb` invocation

**`planet-core/tests/features/steps.feature` / `planet-core/tests/steps.rs` (new):**
- BDD coverage for `Steps::new`'s validation contract and `Steps::default()`, independent of `SubdivisionArgs` or the subdivision algorithm

**`planet-core/tests/features/subdivision_args.feature` / `planet-core/tests/subdivision_args.rs` (new):**
- BDD coverage for `SubdivisionArgs::new`'s defaulting behavior (now infallible, since `Steps` validates itself before `SubdivisionArgs` ever sees it)

**`planet-core/Cargo.toml` (updated):**
- Add `[[test]] name = "steps" harness = false` and `[[test]] name = "subdivision_args" harness = false`

**`planet-renderer/src/subdivision_stepper.rs`, `planet-renderer/tests/subdivision_stepper.rs`, `planet-renderer/tests/features/subdivision_stepper.feature` — deleted.**

**`planet-renderer/Cargo.toml` (updated):**
- Remove the `[[test]] name = "subdivision_stepper"` entry

**`planet-renderer/src/lib.rs` (updated):**
- Remove `pub mod subdivision_stepper;`

**`planet-renderer/src/app.rs` (updated):**
- Imports: remove `use planet_core::uniform_red_split::UniformRedSplit;` and `use crate::subdivision_stepper::SubdivisionStepper;`; add `use planet_core::mesh::Mesh;`, `use planet_core::subdivide::subdivide;`, `use planet_core::subdivision_args::SubdivisionArgs;`
- Remove `const MAX_SUBDIVISION_DEPTH: u32 = 3;` — `SubdivisionArgs`'s own default (`3`) now governs this until `007-planet-presets`'s depth slider passes an explicit value
- `App` struct: remove `stepper: Option<SubdivisionStepper>`; add `frames: Vec<Mesh>` (default empty) and `current_frame: usize` (default `0`)
- `resumed()`: after building `base_mesh` via `icosahedron()`, construct an `update_cb` that appends each round's mesh to a shared collector seeded with `base_mesh.clone()`, build `let args = SubdivisionArgs::new(None, None, Some(update_cb));` (infallible — no `?` needed here), and call `subdivide(&base_mesh, args)` once, synchronously, before spawning the async `Renderer::new` task. On success, move the collected frames into `self.frames` and set `self.current_frame = 0`; the async `Renderer::new` task is initialized with `self.frames[0]` exactly as it previously used `stepper.mesh().clone()`. Any `Err` (from `icosahedron()` or `subdivide`, the only two fallible calls left in this path) is logged via `web_sys::console::error_1` and the function returns early, matching the existing error-handling idiom in this function
- `window_event`: remove the `PhysicalKey::Code(KeyCode::Space)` match arm entirely; the `PhysicalKey::Code(KeyCode::KeyW)` wireframe-toggle arm is unchanged
- `RedrawRequested` arm: if `self.current_frame + 1 < self.frames.len()`, increments `self.current_frame` and calls `renderer.set_mesh(&self.frames[self.current_frame])` before rendering — this produces the animated build-up; exact pacing is an implementation detail (see Out of scope)

No changes to `camera.rs`, `render.rs`'s pipeline/buffer logic, `uniforms.rs`, or `shader.wgsl`.

## Function/API contracts

- `SubdivisionMode` is a `pub` enum in `planet-core` with exactly one variant, `UniformRedSplit`, and implements `Default` resolving to `UniformRedSplit`
- `SubdivisionMode::strategy` is `pub(crate)` and unreachable from outside `planet-core`; `SubdivisionStrategy`, `UniformRedSplit`, `EdgeKey`, and `EdgeCache` are all `pub(crate)` — none appear in `planet-core`'s public API surface (verified via `cargo doc -p planet-core --no-deps` listing only public items)
- `Steps::new(steps)` never panics:
  - `steps <= 8` returns `Ok(Steps(steps))`
  - `steps > 8` returns `Err(StepsError::ExceedsMaximum { steps, max: 8 })` and constructs nothing
- `Steps::default()` always returns a valid `Steps(3)` — it cannot fail, since `3 <= MAX_SUBDIVISION_STEPS`
- `SubdivisionArgs::new(steps, mode, update_cb)` never panics and never fails (returns `SubdivisionArgs` directly, not a `Result`):
  - `steps: None` resolves to `Steps::default()` (`Steps(3)`); `steps: Some(s)` resolves to `s` (already valid, since only `Steps::new`/`Steps::default()` can produce one)
  - `mode: None` resolves to `SubdivisionMode::UniformRedSplit`; `mode: Some(m)` resolves to `m`
  - `update_cb` is stored as given (`None` or `Some`), with no validation
- `subdivide(mesh, args)`:
  - `args.steps().value() == 0` returns `Ok(mesh.clone())`, calling neither the internal `split_round` helper nor `args`'s callback
  - `args.steps().value() == N >= 1` calls `split_round` exactly `N` times in sequence, using a single boxed strategy resolved once from `args.mode()` at the start of the call
  - when a callback was supplied, it is called exactly `N` times, once immediately after each successful round, in round order, with `(&mesh_after_that_round, round_number)` where `round_number` runs `1, 2, ..., N`
  - when no callback was supplied, `subdivide`'s observable output (triangle/vertex counts, determinism) is unchanged from before this spec
  - is functionally equivalent, for `SubdivisionMode::UniformRedSplit`, to the pre-existing `depth`-round loop — no change to the subdivision math itself
- `planet-renderer` no longer imports `planet_core::uniform_red_split::UniformRedSplit` or any `SubdivisionStrategy`-related item, and the `subdivision_stepper` module no longer exists
- `planet-renderer`'s `App` collects one mesh snapshot per completed round (plus the pre-subdivision base mesh) via a single `subdivide` call at startup, then reveals them one per subsequent redraw via `Renderer::set_mesh`, with no change to `Renderer::new`, `Renderer::render`, or `Renderer::set_mesh`'s existing signatures

## BDD scenarios

`planet-core/tests/features/subdivide.feature`:

```gherkin
Feature: Recursive subdivision via the SubdivisionMode facade

  Scenario: Subdividing the icosahedron mesh by 1 step using SubdivisionMode::UniformRedSplit quadruples the triangle count
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit
    Then the resulting Mesh has 80 triangles

  Scenario: Subdividing the icosahedron mesh by 2 steps grows the triangle count geometrically
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit
    Then the resulting Mesh has 320 triangles

  Scenario: Subdividing the icosahedron mesh by 1 step does not duplicate vertices at shared edges
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit
    Then the resulting Mesh has 42 vertices

  Scenario: Subdividing the icosahedron mesh never creates cracks between adjacent triangles
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit
    Then no two vertices in the resulting Mesh have the same position

  Scenario: Subdividing the icosahedron mesh never pushes vertices beyond the base radius
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit
    Then every vertex of the resulting Mesh has a radius less than or equal to 1.0

  Scenario: A new vertex sits at the exact arithmetic mean of its edge's endpoints
    Given an icosahedron mesh
    And the two vertices of the first triangle's first edge in the icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit
    Then a vertex exists in the resulting Mesh at the exact midpoint of the two given vertices

  Scenario: SubdivisionMode::UniformRedSplit subdivides an arbitrary single-triangle mesh, proving subdivide is not icosahedron-specific
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit
    Then the resulting Mesh has 4 triangles
    And the resulting Mesh has 6 vertices

  Scenario: Subdividing with 0 steps leaves the mesh unchanged
    Given an icosahedron mesh
    When the mesh is subdivided with 0 steps using SubdivisionMode::UniformRedSplit
    Then the resulting Mesh is identical to the icosahedron mesh

  Scenario: Omitting steps and mode falls back to the default of 3 steps using the default SubdivisionMode
    Given an icosahedron mesh
    When the mesh is subdivided with default SubdivisionArgs
    Then the resulting Mesh has 1280 triangles

  Scenario: The update callback is invoked once per completed round with that round's mesh
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit and a recording update callback
    Then the update callback was invoked 2 times
    And the update callback's 1st invocation received a Mesh with 80 triangles
    And the update callback's 2nd invocation received a Mesh with 320 triangles

  Scenario: Subdividing with 0 steps never invokes the update callback
    Given an icosahedron mesh
    When the mesh is subdivided with 0 steps using SubdivisionMode::UniformRedSplit and a recording update callback
    Then the update callback was invoked 0 times
```

`planet-core/tests/features/steps.feature`:

```gherkin
Feature: Constructing validated Steps

  Scenario: Constructing Steps within the allowed range succeeds
    When Steps is constructed with 5
    Then the Steps is constructed successfully
    And the Steps has value 5

  Scenario: The maximum allowed step count is accepted
    When Steps is constructed with 8
    Then the Steps is constructed successfully
    And the Steps has value 8

  Scenario: Requesting more steps than the maximum fails
    When Steps is constructed with 9
    Then the construction fails with an exceeds-maximum error of 9 steps and max 8

  Scenario: The default Steps value is 3
    Given the default Steps
    Then the Steps has value 3
```

`planet-core/tests/features/subdivision_args.feature`:

```gherkin
Feature: Constructing SubdivisionArgs with defaults

  Scenario: Constructing SubdivisionArgs with explicit steps and mode
    Given Steps constructed with 5
    When SubdivisionArgs is constructed with those steps and the UniformRedSplit mode
    Then the SubdivisionArgs has 5 steps
    And the SubdivisionArgs has the UniformRedSplit mode

  Scenario: Omitting steps defaults to 3
    When SubdivisionArgs is constructed with no steps and the UniformRedSplit mode
    Then the SubdivisionArgs has 3 steps

  Scenario: Omitting mode defaults to UniformRedSplit
    Given Steps constructed with 2
    When SubdivisionArgs is constructed with those steps and no mode
    Then the SubdivisionArgs has the UniformRedSplit mode
```

## Acceptance criteria

1. `SubdivisionMode` is a `pub` enum with exactly one variant, `UniformRedSplit`, and implements `Default` resolving to it
2. `SubdivisionStrategy`, `UniformRedSplit`, `EdgeKey`, and `EdgeCache` are `pub(crate)` — none are reachable from a crate outside `planet-core` (verified via `cargo doc -p planet-core --no-deps`, which lists none of them)
3. `Steps::default()` equals `Steps::new(3).unwrap()`; `SubdivisionArgs::new(None, None, None)` succeeds (no `Result`) with `steps() == Steps::default()` and `mode() == SubdivisionMode::UniformRedSplit`
4. `Steps::new(8)` succeeds with `value() == 8`; `SubdivisionArgs::new(Some(Steps::new(8).unwrap()), None, None).steps().value() == 8`
5. `Steps::new(9)` returns `Err(StepsError::ExceedsMaximum { steps: 9, max: 8 })` and constructs no `Steps` — this is the only place the step-count invariant can fail; `SubdivisionArgs::new` itself has no failure mode
6. `subdivide(&icosahedron, SubdivisionArgs::new(Some(Steps::new(0).unwrap()), None, None))` returns a `Mesh` equal to the input icosahedron mesh, and no callback is invoked
7. `subdivide(&icosahedron, SubdivisionArgs::new(Some(Steps::new(1).unwrap()), None, None))` produces exactly 80 triangles and 42 vertices; `Steps::new(2)` produces exactly 320 triangles; `None` (default, `Steps(3)`) produces exactly 1280 triangles
8. No two vertices in any `subdivide` output share the same position, and every vertex's radius is `<= 1.0 + 1e-5`, at any tested step count `>= 1`
9. A new vertex's position exactly equals the arithmetic mean of its edge's two endpoint positions, unchanged from `004-icosahedron-subdivision`
10. `subdivide` applied to a non-icosahedron single-triangle `Mesh` produces exactly 4 triangles and 6 vertices, demonstrating genericity over both the mesh and the mode
11. Given a 2-step `subdivide` call with an `update_cb`, the callback is invoked exactly twice, in order, first with the 80-triangle mesh and round number `1`, then with the 320-triangle mesh and round number `2`
12. `subdivide` never panics and never produces a triangle with an out-of-bounds vertex index, for any valid input `Mesh`, any `steps` value, and `SubdivisionMode::UniformRedSplit`
13. `planet-renderer` contains no reference to `SubdivisionStrategy`, `UniformRedSplit`, or `SubdivisionStepper` anywhere (verified via `grep -r` returning no matches outside this spec file and `docs/`)
14. `planet-renderer/src/subdivision_stepper.rs`, `planet-renderer/tests/subdivision_stepper.rs`, and `planet-renderer/tests/features/subdivision_stepper.feature` no longer exist
15. The Space key no longer has a handler in `planet-renderer/src/app.rs`; the `W` wireframe toggle still works exactly as before
16. On loading the app in-browser, the planet mesh visibly builds up through each subdivision round (an animated reveal) starting from the base icosahedron, without any key press required (manual/in-browser check, per `000-architecture.md`'s exemption for GPU/DOM wiring — not BDD-tested)
17. All scenarios in `subdivide.feature` and the new `steps.feature`/`subdivision_args.feature` pass via real `cucumber` step definitions — no undefined/stub steps
18. `cargo test --workspace`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` all pass
19. `cargo build --target wasm32-unknown-unknown -p planet-renderer` still succeeds
20. No new `unwrap()`/`panic!()` in production code outside tests
21. Existing `mesh.feature`, `vec3.feature`, `icosahedron.feature`, `camera.feature`, `buffers.feature`, `uniforms.feature`, `mesh_render_vertices.feature`, `mesh_render_indices.feature`, and `mesh_render_line_indices.feature` BDD scenarios still pass unmodified
