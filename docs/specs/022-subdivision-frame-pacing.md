# 022 — Subdivision Frame Pacing

**Status:** Ready for review
**Feature slug:** `subdivision-frame-pacing`

This is an ad-hoc corrective feature, not the next sequential `docs/roadmap.md` phase — it fixes a bug reported against `005-subdivision-facade.md`'s animated growth reveal: "when I start to subdivide a planet, I see the icosahedron, then it jumps straight to the final version, skipping all the intermediate transformations."

## Investigation

`005-subdivision-facade.md` replaced interactive per-keystroke subdivision stepping with a single upfront `subdivide` call whose `update_cb` collects one mesh snapshot per round into a frame sequence, which `planet-renderer`'s `App` then "reveals automatically, one round per subsequent redraw, producing an animated build-up." That spec explicitly deferred the reveal's pacing: "Exact frame-advance pacing/timing for `planet-renderer`'s animated build-up (e.g. throttling to a fixed interval) — `app.rs` is thin wiring, not BDD-tested per `rules.md`, and the precise cadence is an implementation detail decided during `planet-tdd` and verified manually in-browser." That deferred pacing was never added.

Confirmed in the current code:

- `App::wire_controls`'s `#start-button` handler calls `generate()` (`planet-renderer/src/app.rs:184-265`), which runs `Planet::subdivide` synchronously to completion, collecting every round's post-processed mesh into `collected_frames` via the `on_progress` callback, then stores `(new_frames, 0)` in `self.frames` (`Frames = Rc<RefCell<(Vec<(Mesh, Vec<Rgb>)>, usize)>>`) and pushes the first frame to the GPU.
- `ApplicationHandler::window_event`'s `WindowEvent::RedrawRequested` arm (`app.rs:532-568`) unconditionally advances `current_frame` by exactly 1 every time it runs (guarded only by `current_frame + 1 < frame_list.len()`), then always calls `renderer.render(...)` and re-requests a redraw — a self-sustaining loop with **no wall-clock throttle** between advances.
- The event loop is created with `ControlFlow::Poll` (`planet-renderer/src/lib.rs:16`), so `RedrawRequested` fires as fast as the platform will deliver it. The GPU surface config resolves to its adapter default, which is vsync (`Fifo`) on every backend this project targets, so in practice each redraw is paced by the display's refresh interval (~16.6ms at 60Hz) rather than truly unthrottled — but with at most `MAX_SUBDIVISION_STEPS = 8` rounds (`planet-core/src/subdivision/steps.rs:3`, so at most 9 frames including the base icosahedron), the entire sequence completes in under ~150ms. That is well below the threshold of human perception of discrete steps, so it reads as "jumps straight to the final version" even though every intermediate mesh is, technically, set on the GPU and rendered for one frame.
- Depth `0` (`MIN_DEPTH` in `planet-renderer/src/controls/depth_slider.rs`) produces exactly 1 frame (the base icosahedron snapshot is overwritten in-place by the final post-processed mesh before display), so there is nothing to animate in that case — this must remain instantaneous, not artificially delayed.

**Root cause:** the reveal mechanism itself works correctly; it is missing the pacing `005` deferred. The fix is to gate `current_frame` advancement on elapsed wall-clock time instead of redraw frequency, while leaving every-redraw rendering (needed for live camera responsiveness) untouched.

## Requirements

- `planet-renderer` gains a new `scene/growth_animation.rs` concern (`scene/` already houses render-loop-adjacent pure logic, e.g. `camera.rs`) defining `GrowthAnimation`, a small state type that owns the collected frame sequence, the current playback index, and the wall-clock timestamp of the last advance. It replaces the ad-hoc `Frames = Rc<RefCell<(Vec<(Mesh, Vec<Rgb>)>, usize)>>` tuple currently defined in `app.rs`.
- `GrowthAnimation` exposes a pure `tick(&mut self, now_ms: f64) -> bool` method: advances `current_frame` by exactly 1 and returns `true` only when there is a next frame available **and** at least `FRAME_INTERVAL_MS` has elapsed since the last advance (or since construction, for the first tick); otherwise leaves state unchanged and returns `false`. `now_ms` is supplied by the caller (dependency injection), so `GrowthAnimation` itself makes no browser/DOM/GPU calls and is natively unit-testable, matching this crate's existing convention for pure logic (`controls/seed_from_timestamp.rs`, `controls/depth_slider.rs`).
- A new `pub const FRAME_INTERVAL_MS: f64 = 150.0;` in `growth_animation.rs` sets the fixed pacing interval between revealed rounds. This is a concrete starting value per this project's established convention of shipping tunable starting constants (`018-restore-tangential-jitter.md`), not a frozen aesthetic contract — it may be retuned during this feature's own `planet-tdd` REFACTOR step against real in-browser output.
- `App::generate` constructs a `GrowthAnimation` from the collected frames and the current wall-clock timestamp (`web_sys::window().and_then(|w| w.performance())`, read once at generation time), instead of storing a raw `(Vec<_>, usize)` tuple. This is a new `web-sys` feature (`Performance`) — add it to the `web-sys` feature list in `planet-renderer/Cargo.toml` and to the dependency table in `tech-stack.md`.
- The `RedrawRequested` handler (`app.rs:532-568`) reads the current timestamp via the same `Performance` API, calls `GrowthAnimation::tick(now_ms)`, and calls `renderer.set_mesh` only when `tick` returns `true`. It continues to call `renderer.render(...)` and `window.request_redraw()` unconditionally on every `RedrawRequested`, regardless of `tick`'s result — camera orbit/zoom and the wireframe/shading toggles must remain fully live and responsive while a growth animation is still revealing frames. If the `Performance` lookup fails (`None`), log the error via the existing `log_error` helper and skip that redraw's `tick` call entirely (advancing on a later redraw once the lookup succeeds again) — no `unwrap()`, per `rules.md`'s DOM-lookup error-handling rule.
- Starting a new generation (clicking Start again, even mid-animation) replaces `self.frames` with a freshly constructed `GrowthAnimation` seeded with the current timestamp — the new animation's pacing starts clean from frame 0, never inheriting the previous generation's last-advance timestamp or index.

Out of scope:
- Any change to `Planet::subdivide`, `subdivide()`, or the `update_cb`/`GenerationProgress` contract in `planet-core` — this feature only changes how `planet-renderer` paces its already-collected frames, not how they are produced.
- Any change to `Renderer::new`, `Renderer::render`, or `Renderer::set_mesh`'s signatures.
- A user-facing control for the pacing interval (e.g. an animation-speed slider) — `FRAME_INTERVAL_MS` is a fixed constant for this feature.
- Skipping/scrubbing/pausing the animation, or replaying it after it completes — the animation still always runs forward-only, exactly once, from frame 0 to the last frame.
- Any change to how `depth = 0` (single-frame) generation behaves — it already displays immediately today and continues to do so, since `tick` naturally returns `false` when there is no next frame.

## Domain model involved

**`planet-renderer/src/scene/growth_animation.rs` (new):**
- `pub const FRAME_INTERVAL_MS: f64 = 150.0;`
- `pub struct GrowthAnimation { frames: Vec<(Mesh, Vec<Rgb>)>, current_frame: usize, last_advance_ms: f64 }`
- `impl GrowthAnimation`:
  - `pub fn new(frames: Vec<(Mesh, Vec<Rgb>)>, started_ms: f64) -> GrowthAnimation` — `current_frame` starts at `0`, `last_advance_ms` starts at `started_ms`. `frames` is expected non-empty (mirrors today's invariant: `generate()` always collects at least the base-icosahedron frame); an empty `frames` vec is a caller bug, not a case this type validates.
  - `pub fn current(&self) -> &(Mesh, Vec<Rgb>)` — returns the frame at `current_frame` (panics only if `frames` was empty, which callers must not do — no `Result` needed since this mirrors an existing, already-relied-upon invariant, not new fallibility)
  - `pub fn tick(&mut self, now_ms: f64) -> bool` — as described in Requirements
- `Mesh` and `Rgb` are reused unchanged from `planet_core::geometry::mesh` / `planet_core::color::rgb`.

**`planet-renderer/src/app.rs` (updated):**
- Remove the `Frames` type alias and its raw-tuple shape; `App::frames` becomes `Rc<RefCell<Option<GrowthAnimation>>>` (starts `None` before the first generation).
- `generate()`: after collecting `new_frames` and swapping in the final post-processed frame (unchanged), construct `GrowthAnimation::new(new_frames, now_ms)` using a `Performance` timestamp read at the top of `generate()`, and store it in `self.frames`. The first frame is pushed to the GPU exactly as today, via `animation.current()`.
- `RedrawRequested` arm: read the current `Performance` timestamp; if `self.frames` holds `Some(animation)`, call `animation.tick(now_ms)`, and if it returned `true`, call `renderer.set_mesh` with `animation.current()`. `renderer.render(...)` and `window.request_redraw()` remain unconditional, exactly as today.

**`planet-renderer/Cargo.toml` (updated):**
- Add `Performance` to the `web-sys` feature list.
- Add `[[test]] name = "growth_animation" harness = false`.

**`tech-stack.md` (updated):**
- Add `Performance` to the `web-sys` feature-flag cell in the dependency table.

**`planet-renderer/tests/features/growth_animation.feature` / `planet-renderer/tests/growth_animation.rs` (new):**
- BDD coverage for `GrowthAnimation::tick`'s pacing contract, following the existing `seed_from_timestamp`/`depth_slider` convention (cucumber `World`, regex `when`/`then` steps, pure — no browser API).

No changes to `Renderer`, `Camera`, `gpu/buffers.rs`, `gpu/uniforms.rs`, `shader.wgsl`, or any `planet-core` type.

## Function/API contracts

- `GrowthAnimation::new(frames, started_ms)` never panics for non-empty `frames`; sets `current_frame = 0`, `last_advance_ms = started_ms`.
- `GrowthAnimation::tick(now_ms)`:
  - Returns `true` and increments `current_frame` by exactly 1, and sets `last_advance_ms = now_ms`, if and only if `current_frame + 1 < frames.len()` **and** `now_ms - last_advance_ms >= FRAME_INTERVAL_MS`.
  - Otherwise returns `false` and leaves `current_frame`/`last_advance_ms` unchanged.
  - Never advances `current_frame` past `frames.len() - 1`, and never advances by more than 1 per call — matching the existing pre-fix invariant already enforced by `current_frame + 1 < frame_list.len()`.
  - For `frames.len() == 1`, always returns `false` regardless of `now_ms` (no next frame exists).
- `GrowthAnimation::current()` returns the frame at the current index; the returned `(Mesh, Vec<Rgb>)` reference is exactly what was collected/swapped-in by `generate()`, unchanged from today's per-frame content.
- `planet-renderer`'s `RedrawRequested` handling calls `renderer.render(...)` on every invocation, independent of whether `tick` returned `true` or `false` — camera/toggle responsiveness during the animation is unaffected by this feature.

## BDD scenarios

`planet-renderer/tests/features/growth_animation.feature`:

```gherkin
Feature: Pacing the subdivision growth-animation frame reveal

  Scenario: Ticking after the pacing interval has elapsed advances to the next frame
    Given a GrowthAnimation constructed with 3 frames and started at 0.0ms
    When the GrowthAnimation is ticked at 150.0ms
    Then the tick returns true
    And the GrowthAnimation's current frame index is 1

  Scenario: Ticking before the pacing interval has elapsed does not advance
    Given a GrowthAnimation constructed with 3 frames and started at 0.0ms
    When the GrowthAnimation is ticked at 50.0ms
    Then the tick returns false
    And the GrowthAnimation's current frame index is 0

  Scenario: Ticking a single-frame GrowthAnimation never advances
    Given a GrowthAnimation constructed with 1 frame and started at 0.0ms
    When the GrowthAnimation is ticked at 1000.0ms
    Then the tick returns false
    And the GrowthAnimation's current frame index is 0

  Scenario: Ticking at the last frame never advances past the end
    Given a GrowthAnimation constructed with 2 frames and started at 0.0ms
    And the GrowthAnimation has already been ticked at 150.0ms
    When the GrowthAnimation is ticked at 300.0ms
    Then the tick returns false
    And the GrowthAnimation's current frame index is 1
```

## Acceptance criteria

1. `GrowthAnimation::new(frames, started_ms)` sets `current_frame = 0` and `last_advance_ms = started_ms` for any non-empty `frames`.
2. `GrowthAnimation::tick(now_ms)` returns `true` and advances `current_frame` by exactly 1 if and only if a next frame exists and `now_ms - last_advance_ms >= FRAME_INTERVAL_MS`; otherwise returns `false` with no state change.
3. Repeatedly calling `tick` with monotonically increasing `now_ms` values each spaced `>= FRAME_INTERVAL_MS` apart visits every index `0..frames.len()` exactly once, in order, never skipping or repeating an index out of sequence.
4. For `frames.len() == 1`, `tick` always returns `false` regardless of `now_ms`.
5. `tick` never advances `current_frame` beyond `frames.len() - 1`.
6. `planet-renderer`'s `RedrawRequested` handler calls `renderer.set_mesh` only on redraws where `tick` returned `true`, but calls `renderer.render` and `window.request_redraw` on every `RedrawRequested`, regardless of `tick`'s result.
7. Starting a new generation while a previous animation is still revealing frames constructs a fresh `GrowthAnimation` (index `0`, timer reset to the new generation's start time) — verified by manual in-browser check: clicking Start again mid-animation restarts the visible build-up from the icosahedron rather than continuing or skipping.
8. Manual in-browser check: generating a planet at the default depth shows each intermediate subdivision round on screen for a perceptible duration (not an instantaneous flash) before settling on the final mesh, and orbiting/zooming the camera or toggling wireframe/shading remains responsive throughout the animation.
9. `cargo test --workspace`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo build --target wasm32-unknown-unknown -p planet-renderer` all pass.
