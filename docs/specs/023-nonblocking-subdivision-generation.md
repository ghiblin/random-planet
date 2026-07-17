# 023 — Non-blocking Subdivision Generation

**Status:** Ready for review
**Feature slug:** `nonblocking-subdivision-generation`

This is an ad-hoc corrective feature, not the next sequential `docs/roadmap.md` phase — it fixes a bug reported against the Start-button flow: "when I hit the Start button, the UI freezes." Confirmed at max depth (8).

## Investigation

Systematic debugging (headless-Chromium responsiveness probes against a live Trunk dev server) measured the click-to-responsive latency at every depth setting:

| Depth | Blocked time |
|---|---|
| 3 (default) | 35ms |
| 4 | 57ms |
| 5 | 123ms |
| 6 | 388ms |
| 7 | 1.4s |
| 8 (max) | **5.65s** |

The page is instantly responsive again the moment the click handler returns — not a hang, deadlock, or infinite loop, purely compute-bound blocking of the browser's single main thread. `App::generate` (`planet-renderer/src/app.rs`) calls `Planet::subdivide` fully synchronously from inside the `#start-button` click handler, with no yielding; this pattern predates `022-subdivision-frame-pacing.md` (confirmed against the pre-022 `main` commit), so it is a pre-existing architectural gap, not a regression from that fix.

A follow-up native profiling run (`cargo test -p planet-core`, timing `subdivide(...)` and the postprocessing pipeline separately at depth 8, Earthy preset, 655,362 final vertices) found the blocking time is **not** dominated by the subdivision rounds themselves:

| Stage | Time |
|---|---|
| Subdivision rounds (all 8) | 1.63s |
| Postprocessing (terrain noise + ocean quota) | **4.52s** |

Postprocessing — a single, atomic, whole-mesh pass applied once after all subdivision rounds complete — is ~74% of the total blocking time. Any fix that only paces/yields between subdivision rounds (as `022`'s `GrowthAnimation` already does for the *reveal*, not the *computation*) leaves the dominant cost untouched.

## Requirements

**Move generation off the main thread entirely**, via a Web Worker, so `App`'s main thread never blocks regardless of depth — and give the postprocessing stage coarse progress observability, so the worker (and therefore the growth-animation reveal) has something to report during that dominant 4.5s stretch, without the substantially larger, unwarranted refactor of turning `apply_terrain_noise`/`apply_ocean_quota`'s per-vertex math into resumable batches (ocean quota's sea-level computation is inherently a whole-mesh, two-pass operation; batching it is a materially bigger change than the payoff justifies here). Concretely:

- **`planet-core`**: `Planet::subdivide` gains a third parameter, `on_postprocess: Option<PostprocessProgress>` (`PostprocessProgress = Box<dyn FnMut(PostprocessStage)>`, `PostprocessStage` a new `Copy` enum with variants `TerrainNoise` and `OceanQuota`), invoked immediately before each of the two postprocessing stages runs — `TerrainNoise` always, `OceanQuota` only when `PresetParams::ocean_quota()` is `Some`. `subdivide`'s existing round-by-round callback (`on_progress` / `GenerationProgress`) is **unchanged** — moving generation to a worker already delivers those callbacks to the main thread progressively for free (a worker's `postMessage` is dispatched to the main thread independently of what the worker computes next, since the two run on genuinely separate threads), so no restructuring of the subdivision-round mechanism itself is needed.
- Calling `apply_terrain_noise`/`apply_ocean_quota` directly, sequentially, with `on_postprocess` invocations interleaved, replaces the existing `postprocessing_pipeline`/`compose_mesh`/`identity_mesh`/`MeshProcessor` indirection in `planet.rs` — that machinery becomes **dead code** once nothing else composes an ad-hoc `MeshProcessor` chain (confirmed via `grep`: `compose_mesh`, `identity_mesh`, and `MeshProcessor` are used nowhere else in the codebase) and is deleted, along with its own unit tests, per this project's practice of not leaving orphaned abstractions behind.
- **`planet-renderer`**: a new Web Worker entry point (`src/bin/generation_worker.rs`, a second `wasm-bindgen` binary target alongside `app.rs`'s main-thread one, wired into Trunk's build as a worker asset in `index.html`) receives a `StartRequest { preset, depth, seed }` from the main thread, runs `Planet::builder()...build()` + `.subdivide(depth, Some(on_round), Some(on_postprocess))` (this still blocks *that* thread for the full 1.6s+4.5s — irrelevant, since it is no longer the main thread), and for every `on_round` invocation and the final `Planet`, packs the mesh into GPU-ready byte buffers via the existing pure `gpu/buffers.rs` functions (`mesh_render_vertices`, `mesh_render_indices`, `mesh_render_line_indices`, `pack_vertex_buffer`, `pack_index_buffer` — unchanged) into a new `PackedFrame` struct, and posts a `WorkerMessage` back to the main thread as transferable `ArrayBuffer`s (zero-copy). `on_postprocess` invocations post a lightweight `WorkerMessage::PostprocessStage` (no mesh payload).
- A new pure, natively-testable message-protocol module (`planet-renderer/src/worker/protocol.rs`) defines `StartRequest` and `WorkerMessage` as plain Rust types (no `JsValue` in their own definitions); the actual `JsValue` postMessage encode/decode conversions live in the same file, `#[cfg(target_arch = "wasm32")]`-gated, exempt from the Iron Law as thin browser glue (same class as `app.rs`'s DOM wiring) — mirroring how `controls/seed_from_timestamp.rs` stays pure while the `Date.now()` read itself stays in `app.rs`.
- `Renderer::set_mesh` (`gpu/render.rs`) and `Renderer::new` change contract: instead of accepting `(&Mesh, &[Rgb])` and packing buffers internally, both now accept a `&PackedFrame` directly — packing happens exactly once, in the worker, the only place mesh generation occurs now. `app.rs`'s one remaining main-thread mesh construction (the empty startup placeholder mesh in `resumed()`) packs its own trivial empty `PackedFrame` via the same pure `gpu::buffers::pack_frame` function, so `Renderer`'s two entry points take a consistent input shape.
- `GrowthAnimation` (`scene/growth_animation.rs`, from `022-subdivision-frame-pacing.md`) changes its stored frame representation from `(Mesh, Vec<Rgb>)` to `PackedFrame`, and gains a `push_frame(&mut self, frame: PackedFrame, now_ms: f64)` method so frames can arrive progressively (from worker `WorkerMessage::Frame` events, with real, variable, network/compute-driven latency) instead of only from a fully-collected upfront `Vec`. The very first frame ever pushed is revealed immediately (no pacing delay — matches `022`'s existing behavior of showing frame 0 immediately at construction); every subsequent pushed frame queues and is revealed one at a time by the existing `tick(now_ms)` pacing gate (`FRAME_INTERVAL_MS`, unchanged), preserving `022`'s pacing contract exactly — only the frame *source* changes, from pre-collected to streaming. `new()` no longer takes a `Vec`/timestamp (nothing is known upfront); `current()` returns `Option<&PackedFrame>` (`None` before the first frame arrives, instead of assuming non-empty).
- `App::generate` (`app.rs`) no longer calls `Planet::subdivide` or depends on `planet_core::planets::planet` beyond the input-parsing types already used for UI validation (`Preset`, `Steps`, `Seed`). It posts a `StartRequest` to a persistent `web_sys::Worker` handle (created once, lazily on first Start click or eagerly in `resumed()`), and disables `#start-button` for the duration of the in-flight generation (re-enabled on receiving a `WorkerMessage::Frame { is_final: true, .. }` or `WorkerMessage::Error`) — this is a new, in-scope correctness requirement: the previous synchronous code could never receive an overlapping Start click (the browser was blocked until it returned), but an async worker genuinely can, and a second in-flight request racing the first is a new failure mode this change itself introduces, not a pre-existing one deferred from scope.
- A `message` event listener (wired once, alongside the worker's creation) decodes incoming `WorkerMessage`s: `Frame` pushes into `GrowthAnimation` via `push_frame`; `PostprocessStage` may update a status label (nice-to-have, not required to fix the reported freeze); `Error` logs via the existing `log_error` helper and re-enables the Start button.
- New `tech-stack.md` dependency-table additions: `web-sys` features `Worker`, `MessageEvent`, `DedicatedWorkerGlobalScope` (exact feature-flag names confirmed against actual compiler errors during `planet-tdd` — this project's established practice for wasm-bindgen/web-sys specifics, per `022`'s own precedent). No new external crate — `js-sys` (already a dependency) covers the typed-array transfer.
- New `rules.md` entries: a `worker/` concern (`protocol.rs`) under `planet-renderer`'s concern list; a documented exception alongside `app.rs`'s existing one, noting `src/bin/generation_worker.rs` as the Web Worker's own wasm-bindgen entry point (Cargo's standard `src/bin/` convention, a distinct location from the `src/`-direct-files restriction `app.rs`/`lib.rs` are exempted under).

Out of scope:
- Restructuring `apply_terrain_noise`/`apply_ocean_quota`'s internal per-vertex math into resumable, chunk-at-a-time batches — see Investigation; the coarse two-stage signal is enough given both stages already run off the main thread
- Cancelling an in-flight generation, or supporting a worker pool / multiple concurrent generations — disabling `#start-button` during generation is the simplest, sufficient guard against the one new race this change introduces
- Any change to `planet-core`'s subdivision or terrain/ocean-quota *algorithms* themselves — purely a threading/messaging/observability concern
- Any change to `022-subdivision-frame-pacing.md`'s `FRAME_INTERVAL_MS` value or `GrowthAnimation::tick`'s pacing semantics — only its frame-source/ingestion model changes (streaming vs. pre-collected), not the pacing contract itself
- A user-visible progress bar/percentage for postprocessing — the stage-progress message is plumbed for a future nice-to-have status label, not mandated as new UI in this spec
- Exact Trunk worker-asset directive syntax and exact `web-sys` feature names — implementation details confirmed during `planet-tdd` against real compiler/build output, matching this project's established precedent (`005`, `022`) for deferring wasm-bindgen/browser-plumbing specifics

## Domain model involved

**`planet-core/src/planets/postprocess_stage.rs` (new):**
- `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum PostprocessStage { TerrainNoise, OceanQuota }`

**`planet-core/src/planets/planet.rs` (updated):**
- `pub type PostprocessProgress = Box<dyn FnMut(PostprocessStage)>;` (alongside the existing `GenerationProgress` type alias)
- `pub fn subdivide(&self, max_depth: Steps, on_progress: Option<GenerationProgress>, on_postprocess: Option<PostprocessProgress>) -> Result<Planet, PlanetError>` — after the existing subdivision-round loop (unchanged), calls `on_postprocess` (if present) with `PostprocessStage::TerrainNoise` then `apply_terrain_noise(&mesh, self.seed, params.terrain_noise())`; then, only if `params.ocean_quota()` is `Some(quota)`, calls `on_postprocess` with `PostprocessStage::OceanQuota` then `apply_ocean_quota(&mesh, quota)`. The rest of the method (sea-level computation, per-vertex colors, `finalize_normals`) is unchanged.
- `postprocessing_pipeline` helper function: **removed** (inlined into `subdivide` as above).

**`planet-core/src/processor/compose_mesh.rs`, `identity_mesh.rs`, `mesh_processor.rs` — deleted** (and their `mod` declarations removed from `planet-core/src/processor.rs`), now unreferenced anywhere in the crate.

**`planet-core/src/lib.rs` / `planets.rs` (updated):** add `pub mod postprocess_stage;` (or the crate's equivalent module-declaration file for the `planets/` concern).

**`planet-renderer/src/gpu/buffers.rs` (updated):**
- `#[derive(Debug, Clone, PartialEq)] pub struct PackedFrame { pub vertex_bytes_smooth: Vec<u8>, pub vertex_bytes_flat: Vec<u8>, pub index_bytes: Vec<u8>, pub line_index_bytes: Vec<u8> }`
- `pub fn pack_frame(mesh: &Mesh, colors: &[Rgb]) -> PackedFrame` — bundles the four existing packing calls (`mesh_render_vertices`/`mesh_render_indices`/`mesh_render_line_indices`/`pack_vertex_buffer`/`pack_index_buffer`, all unchanged) into one `PackedFrame`. Reused by both the worker (packing every generated frame) and `app.rs`'s one remaining main-thread mesh construction (the empty startup placeholder).

**`planet-renderer/src/gpu/render.rs` (updated):**
- `Renderer::new(window: Arc<Window>, initial_frame: &PackedFrame) -> Result<Self, String>` (was `(window, mesh: &Mesh, colors: &[Rgb])`) — creates the initial GPU buffers directly from `initial_frame`'s already-packed bytes, no internal packing call.
- `Renderer::set_mesh(&mut self, frame: &PackedFrame)` (was `(&mut self, mesh: &Mesh, colors: &[Rgb])`) — same simplification.

**`planet-renderer/src/scene/growth_animation.rs` (updated, revising `022-subdivision-frame-pacing.md`):**
- `GrowthAnimation`'s stored frame type changes from `(Mesh, Vec<Rgb>)` to `crate::gpu::buffers::PackedFrame`.
- `pub fn new() -> GrowthAnimation` (was `new(frames: Vec<_>, started_ms: f64)`) — starts with no revealed frames, an empty pending queue, and no last-advance timestamp yet.
- `pub fn push_frame(&mut self, frame: PackedFrame, now_ms: f64)` (new) — if no frame has been revealed yet, reveals `frame` immediately (`last_advance_ms = Some(now_ms)`); otherwise enqueues it as pending.
- `pub fn tick(&mut self, now_ms: f64) -> bool` (contract unchanged in spirit) — if a pending frame exists and `now_ms - last_advance_ms >= FRAME_INTERVAL_MS` (only meaningful once at least one frame has already been revealed), reveals the next pending frame (FIFO) and returns `true`; otherwise returns `false`.
- `pub fn current(&self) -> Option<&PackedFrame>` (was `-> &(Mesh, Vec<Rgb>)`) — `None` until the first frame is pushed.

**`planet-renderer/src/worker.rs` (new, declares the new concern) / `planet-renderer/src/worker/protocol.rs` (new):**
- `#[derive(Debug, Clone, Copy)] pub struct StartRequest { pub preset: Preset, pub depth: Steps, pub seed: Seed }`
- `#[derive(Debug, Clone)] pub enum WorkerMessage { Frame { frame: PackedFrame, is_final: bool }, PostprocessStage(PostprocessStage), Error(String) }`
- `#[cfg(target_arch = "wasm32")]`-gated conversions between these plain types and `wasm_bindgen::JsValue` for the actual `postMessage`/`onmessage` boundary (exact encoding — e.g. `js_sys::Object`/`Reflect`/`Uint8Array` — decided during `planet-tdd`).

**`planet-renderer/src/bin/generation_worker.rs` (new):** the Web Worker's `wasm-bindgen` entry point — thin wiring (exempt from the Iron Law, same class as `app.rs`): decodes an incoming `StartRequest`, runs `Planet::builder()...with_preset(...).with_seed(...).build()` then `.subdivide(depth, Some(on_round), Some(on_postprocess))`, packs every intermediate and the final mesh via `gpu::buffers::pack_frame`, posts `WorkerMessage`s back via transferable `ArrayBuffer`s.

Unlike `app.rs` — whose *inclusion* is `#[cfg(target_arch = "wasm32")]`-gated at its `mod app;` declaration in `lib.rs`, so the file simply doesn't exist as far as a native compile is concerned — a `src/bin/*.rs` file is its own independent Cargo compilation unit with no equivalent `mod` line to gate. Left unaddressed, `cargo test --workspace`/`cargo build` (native, part of the mandatory build gate) would try to compile this binary for the host target and fail the moment it references a real wasm-bindgen/web-sys Worker API. So `generation_worker.rs`'s `fn main()` itself is split by target:
```rust
#[cfg(target_arch = "wasm32")]
fn main() { /* real worker entry point */ }

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
```
— mirroring how `Cargo.toml`'s `getrandom`/`console_error_panic_hook` dependencies are already scoped to `[target.'cfg(target_arch = "wasm32")'.dependencies]` for the same reason (wasm32-only concerns must not break the native build). Every other item in this file (the `StartRequest` decode, the `Planet` calls, the packing, the `WorkerMessage` posting) lives inside that `#[cfg(target_arch = "wasm32")]` real `fn main()`, or behind its own `#[cfg(target_arch = "wasm32")]` gate if factored into helper functions — the file must compile (trivially, as a near-empty binary) on the native target, exactly as `cargo test --workspace` already expects of the rest of the crate.

**`planet-renderer/src/app.rs` (updated):**
- New field: `worker: Option<web_sys::Worker>` (or equivalent handle), created once.
- `generate()` restructured: no `planet_core::planets::planet` generation call; posts a `StartRequest` to the worker; disables `#start-button`.
- A `message` event listener (wired once): decodes `WorkerMessage`s, calls `GrowthAnimation::push_frame` for `Frame`, re-enables `#start-button` on `Frame { is_final: true, .. }` or `Error` (logging the latter via `log_error`).
- `resumed()`'s empty-mesh bootstrap packs its own `PackedFrame` via `gpu::buffers::pack_frame(&empty_mesh, &[])` before constructing `Renderer::new`.
- `RedrawRequested` arm: unchanged in spirit — reads the `Performance` timestamp, calls `GrowthAnimation::tick`, and if it returns `true`, calls `renderer.set_mesh(animation.current().expect("tick only returns true once a frame exists"))`.

**`planet-renderer/Cargo.toml` (updated):** add `Worker`, `MessageEvent`, `DedicatedWorkerGlobalScope` to the `web-sys` feature list (exact names confirmed during implementation); no `[[bin]]` stanza needed for `src/bin/generation_worker.rs` (Cargo auto-discovers `src/bin/*.rs`).

**`index.html` (updated):** a Trunk worker-asset directive building `generation_worker` as a second wasm target.

**`rules.md` / `tech-stack.md` (updated):** as described in Requirements.

## Function/API contracts

- `Planet::subdivide(max_depth, on_progress, on_postprocess)`:
  - Calls `on_progress` exactly as before (unchanged round-by-round contract).
  - Calls `on_postprocess` (if `Some`) with `PostprocessStage::TerrainNoise` exactly once, always, immediately before applying terrain noise.
  - Calls `on_postprocess` (if `Some`) with `PostprocessStage::OceanQuota` exactly once, if and only if `self.preset.params().ocean_quota()` is `Some` — never called for a preset with no ocean quota.
  - Total stage-callback invocations: 1 (no ocean quota) or 2 (has ocean quota), always in the order `TerrainNoise` then (if applicable) `OceanQuota`.
  - Observable output (final mesh, colors, determinism) is unchanged from before this spec — this is purely an added observation hook.
- `GrowthAnimation::new()` starts with `current() == None` and `tick(_)` always returning `false` until at least one frame has been pushed.
- `GrowthAnimation::push_frame(frame, now_ms)`: the first call ever makes `current() == Some(&frame)` immediately, with no `tick` needed. Every subsequent call enqueues `frame` without changing `current()` until a later `tick` reveals it.
- `GrowthAnimation::tick(now_ms)`: returns `true` and reveals exactly one pending frame (in FIFO push order) if and only if a pending frame exists **and** `now_ms - <last reveal's timestamp> >= FRAME_INTERVAL_MS`; otherwise returns `false` with no state change. Never reveals more than one frame per call, never reveals out of push order.
- `Renderer::new(window, initial_frame)` / `Renderer::set_mesh(frame)`: both take pre-packed `PackedFrame` bytes directly; neither performs any mesh-to-buffer packing internally anymore (moved to `gpu::buffers::pack_frame`, called by whoever produces the frame).
- The main thread's `#start-button` is disabled from the moment a `StartRequest` is posted until a `Frame { is_final: true }` or `Error` message is received for that request — no second `StartRequest` can be posted while one is in flight.

## BDD scenarios

`planet-core/tests/features/planet.feature` (extended):

```gherkin
  Scenario: Subdividing a Planet with an ocean-quota preset reports both postprocessing stages in order
    Given a Planet generated with seed 5 and the Earthy preset at max depth 2
    When that Planet is subdivided again with a postprocessing-stage observer
    Then the observer received [TerrainNoise, OceanQuota] in that order

  Scenario: Subdividing a Planet with a preset that has no ocean quota reports only the terrain-noise stage
    Given a Planet generated with seed 5 and the Rocky preset at max depth 2
    When that Planet is subdivided again with a postprocessing-stage observer
    Then the observer received [TerrainNoise] only
```

`planet-renderer/tests/features/growth_animation.feature` (rewritten for the streaming contract):

```gherkin
Feature: Streaming the subdivision growth-animation frame reveal

  Scenario: The first pushed frame is revealed immediately, with no pacing delay
    Given a new GrowthAnimation with no frames yet
    When a frame is pushed at 0.0ms
    Then the GrowthAnimation's current frame is that frame

  Scenario: A second frame pushed before the pacing interval has elapsed is not yet revealed
    Given a new GrowthAnimation with no frames yet
    And a frame is pushed at 0.0ms
    When a second, distinct frame is pushed at 50.0ms
    Then the GrowthAnimation's current frame is still the first frame

  Scenario: Ticking after the pacing interval has elapsed reveals the next pending frame
    Given a new GrowthAnimation with no frames yet
    And a frame is pushed at 0.0ms
    And a second, distinct frame is pushed at 50.0ms
    When the GrowthAnimation is ticked at 150.0ms
    Then the tick returns true
    And the GrowthAnimation's current frame is the second frame

  Scenario: Ticking with no pending frame never advances
    Given a new GrowthAnimation with no frames yet
    And a frame is pushed at 0.0ms
    When the GrowthAnimation is ticked at 1000.0ms
    Then the tick returns false
    And the GrowthAnimation's current frame is still the first frame
```

## Acceptance criteria

1. `Planet::subdivide` invokes `on_postprocess` with `[TerrainNoise, OceanQuota]` in order for a preset with an ocean quota, and `[TerrainNoise]` only for a preset without one — covered by the two new `planet.feature` scenarios.
2. `Planet::subdivide`'s final `Planet` (mesh, colors, determinism) is byte-identical to what it produced before this spec, for the same seed/preset/depth, whether or not `on_postprocess` is supplied — covered by the existing `planet.feature` determinism scenarios, unmodified and still passing.
3. `compose_mesh`, `identity_mesh`, and `MeshProcessor` no longer exist anywhere in `planet-core` (`grep` returns nothing) — their own unit tests are removed along with them, and `cargo build -p planet-core` succeeds without them.
4. `GrowthAnimation::new()` starts with `current() == None`.
5. `GrowthAnimation::push_frame` reveals the very first pushed frame immediately (`current()` reflects it with no `tick` call needed); every later pushed frame only becomes `current()` after a `tick` call that returns `true` — covered by the rewritten `growth_animation.feature`.
6. `GrowthAnimation::tick` never reveals more than one pending frame per call, never reveals frames out of push order, and returns `false` with no state change when there is no pending frame or the pacing interval has not yet elapsed.
7. Manual in-browser check, headless-Chromium responsiveness probe (same method as the Investigation): clicking Start at depth 8 no longer blocks the main thread — a rapid-fire `page.evaluate` probe issued immediately after the click returns in low single-digit milliseconds throughout generation, not the previously measured 5.65s.
8. Manual in-browser check: the growth animation still visibly builds up through each subdivision round (unchanged from `022`'s behavior) at every depth, and camera orbit/zoom/wireframe/shading toggles remain fully responsive throughout — now including during the postprocessing stretch, which previously wasn't even reachable without freezing.
9. Manual in-browser check: `#start-button` is disabled the moment Start is clicked and re-enabled once the final mesh (or an error) arrives; clicking it (or attempting to, via a disabled element) while a generation is in flight has no effect and does not produce a second overlapping request.
10. Manual in-browser check: an error during worker-side generation (e.g. a temporarily forced invalid input) logs via the console and re-enables `#start-button`, rather than leaving it permanently disabled.
11. `cargo test --workspace`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo build --target wasm32-unknown-unknown -p planet-renderer` all pass. In particular, `cargo test --workspace` (native target) succeeds with `src/bin/generation_worker.rs` present — its `#[cfg(not(target_arch = "wasm32"))]` no-op `fn main()` compiles trivially on the host — and separately, `cargo build --target wasm32-unknown-unknown -p planet-renderer` builds the real `#[cfg(target_arch = "wasm32")]` worker entry point cleanly.
