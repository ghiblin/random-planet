# 015 — Control Panel

**Status:** Ready for review
**Feature slug:** `control-panel`

This is the fourth and final slice of `docs/roadmap.md`'s "007 — Planet presets" phase, continuing after `012-preset-color-gradient`, `013-planet-aggregate-root`, and `014-ocean-quota`. `013` explicitly deferred this exact work: *"A Preset dropdown and a depth slider remain out of scope — those are genuine UI-control work for a later spec."* This is that spec.

**JS/HTML vs. wgpu-native investigation (resolved before this spec was drafted):** because this app is intended to eventually be served as an external dependency inside a microfrontend host, two implementation strategies were compared for the control panel itself:

- **HTML/DOM via `web-sys`** — real `<fieldset>`/radio-button/`<input type="range">` elements, wired from `planet-renderer` exactly the way `rules.md` already anticipates (`document.get_element_by_id` lookups, handled `None` explicitly). For microfrontend embedding, the standard isolation mechanism is wrapping the whole widget as a Custom Element with a Shadow DOM (or an iframe) — either isolates the panel's markup/CSS from whatever host page consumes it. No new dependency; native accessibility (keyboard nav, screen readers) for free; small WASM payload.
- **wgpu-native UI** (e.g. `egui` via `egui-wgpu`/`egui-winit`, drawn into the same render pass) — total isolation from host CSS/DOM, but a new dependency, a larger WASM payload, materially weaker accessibility (no native focus/screen-reader semantics for a canvas-drawn widget), and a reversal of `000-architecture.md`'s existing UI-controls sketch (a native "Preset selector dropdown").

**Decision: HTML/DOM via `web-sys`, with a Custom Element + Shadow DOM as the eventual embedding wrapper.** This spec implements the control panel itself (preset radio group + depth slider + Start button) inside the existing Trunk-served `index.html`; the Custom Element/Shadow DOM wrapper that lets a host microfrontend actually mount this app is explicitly **out of scope** here (see "Out of scope" below) — narrow-increment precedent per `013`/`014`.

**Preset selector shape: a radio button group, not a `<select>` dropdown.** Each preset option renders as a label + a short description + an image placeholder (a real preset preview image is a future feature — this spec only reserves the slot), rather than a plain `<option>` text string. A dropdown can't show a description or an image per option without non-native custom-dropdown machinery; a radio group is the native HTML control that can carry rich per-option content (an image, a label, a description) while an `<option>` cannot contain arbitrary markup. This also fits the microfrontend-isolation rationale above unchanged, since it's still plain DOM/`web-sys`, no new dependency.

**Why this spec also wires per-vertex color into the GPU pipeline, not just DOM controls:** inspecting the current renderer (`gpu/buffers.rs`, `gpu/render.rs`, `gpu/shader.wgsl`) shows that `Planet::colors()` — shipped by `012`/`013`/`014` — has never actually been wired to the GPU. `Renderer` only ever consumes a bare `Mesh`; the fragment shader hardcodes a fixed blue (`vec3(0.3, 0.55, 0.9)`) regardless of preset. A preset control whose only observable effect is a change in noise/edge-length geometry, with every preset rendering the same flat blue, would be a half-finished feature — the color difference is each preset's most visible identity (compare Earthy's water-to-snow gradient against Volcano's basalt-to-lava gradient in `preset.rs`). This spec therefore extends the existing `gpu/` concern (not a new concern — `buffers.rs`/`render.rs`/`shader.wgsl` already own mesh-to-GPU-data mapping) with a `color` vertex attribute sourced from `Planet::colors()`.

**Interaction model (configure → generate, not live-preview):** the control panel and the rendered planet are mutually exclusive, never shown together:

1. **Configuring** (the state on page load): the preset radio group, the depth slider, and a **Start** button are visible, overlaid on the canvas. The canvas itself is mounted and rendering (so it's visible in the background — the clear-color backdrop, camera still nominally active) but holds an **empty mesh** — no planet, since nothing has been generated yet. `#start-button` is disabled until the renderer's one-time async GPU/device setup finishes (mirrors this file's existing async-boundary handling for `Renderer::new`).
2. **Clicking Start** reads whichever preset radio is checked and the depth slider's current value, computes a **fresh seed from the click's own timestamp** (`js_sys::Date::now()` — see below), and unconditionally builds a brand-new `Planet` from scratch (never reuses or extends a previous one — there is nothing to reuse, since every click gets its own seed). The existing per-round growth animation (`on_progress`-collected, colored frames, played back via `RedrawRequested`) plays once, ending on the final post-processed `(mesh, colors)`. The controls are hidden the moment Start is clicked; a small **Change settings** button appears in their place.
3. **Viewing**: only the canvas (now showing the generated, colored planet) and the "Change settings" button are visible; camera orbit/zoom/wireframe work as before.
4. **Clicking Change settings** re-shows the preset/depth/Start controls (with whichever preset/depth were last selected still intact, since the underlying `<input>` elements are only hidden, never destroyed) and hides "Change settings" again — the canvas keeps rendering the last generated planet behind the reopened controls, satisfying "visible in the background." Clicking Start again always performs a full fresh build with a new timestamp-derived seed, exactly like the first click — no special-casing for "first vs. subsequent" generation.

**Seed: derived from the Start-click timestamp, not a fixed constant.** `000-architecture.md` already anticipated this: *"No seed input exposed in the UI — 'regenerate' re-seeds internally."* There is still no manual seed **input field** (nothing the user types or sees the value of), but every Start click now re-seeds via `Seed::from(js_sys::Date::now() as u64)` — the browser-side, impure timestamp read happens in `app.rs`; the numeric conversion itself is factored into a small pure, testable helper (`controls::seed_from_timestamp`, see below) so at least the non-panicking conversion behavior is unit/BDD-tested even though the timestamp source itself is not. This does not conflict with `constitution.md`'s determinism requirement — `Planet::generate`/`Planet::subdivide` remain pure functions of their explicit `(seed, preset, max_depth)` arguments and still never read system time internally; only the *caller* (browser UI code, not `planet-core`) chooses to derive its `seed` argument from the clock.

Because every Start click now gets an independent, unconditionally-fresh `Planet`, `013`'s open question about depth-slider semantics ("does re-subdividing an already-subdivided `Planet` compound instead of resetting?") no longer applies to this feature — there is no persisted "base planet" to accidentally re-subdivide across interactions; each click starts from `Mesh::icosahedron()` via a brand-new `PlanetBuilder::build()` call.

## Out of scope

- The Custom Element + Shadow DOM (or iframe) microfrontend embedding wrapper — deferred to a later spec. This spec keeps Trunk's single-page `index.html` as the only supported host.
- A wgpu-native/`egui` UI — rejected by the investigation above.
- A manual seed **entry field** or a way to view/copy/share the exact seed value used — `000-architecture.md` is explicit that no seed input is exposed in the UI; the seed is always auto-derived from the Start-click timestamp and never surfaced to the user.
- Any change to camera orbit/zoom, the wireframe toggle, or existing keyboard shortcuts.
- Accessibility beyond what a native `<fieldset>`/`<input type="radio">` group, `<input type="range">`, and `<button>` provide for free (no ARIA live region on the depth-value label, no focus management when toggling Configuring/Viewing, etc.).
- Any change to `Planet`/`PresetParams`/ocean-quota domain logic beyond the small additive `Preset` helpers below.
- The actual preset preview images — this spec only reserves an `<img>` placeholder slot per preset (empty `src`, fixed size, `alt` text); generating/sourcing real images is explicitly future work, per the user's own framing ("we are going to generate an image for each preset later").
- Any persistence of a previous generation's settings across a full page reload (only within a single page load, via "Change settings," per the interaction model above).

## Requirements

### `planet-core` — `presets/preset.rs` (modified, no new file)

- `Preset` gains `pub const ALL: [Preset; 3] = [Preset::Earthy, Preset::Volcano, Preset::Rocky];` — the single source of truth for "every preset that exists," in a fixed, stable order (`Earthy` first, matching its `#[default]` position)
- `Preset` gains `pub fn name(&self) -> &'static str`, returning `"Earthy"`, `"Volcano"`, `"Rocky"` respectively — the exact strings used as both each radio `<input>`'s `value`/visible label and the round-trip key `controls::preset_select::parse_preset` matches against
- `Preset` gains `pub fn description(&self) -> &'static str` — a short, human-readable blurb rendered next to each radio option's label (e.g. `"Oceans, grasslands, and snow-capped peaks."` for `Earthy`). Cosmetic UI copy, not a domain invariant — freely editable later without touching any contract this spec defines

### `planet-renderer` — new `controls/` concern

Per `rules.md`'s existing pattern of keeping BDD-testable logic GPU/DOM-free (`scene/camera.rs`, `gpu/buffers.rs`, `gpu/uniforms.rs`), `controls/` holds **pure DOM-value parsing/validation only** — no `web_sys`/`js_sys` calls, no browser API — so it is natively `cargo test`-able. The actual `web_sys`/`js_sys` calls (element lookups, event-listener registration, reading the clock) stay in `app.rs`, consistent with `rules.md`'s existing `app.rs` role ("wasm-bindgen entry point, HTML control wiring").

- `controls/preset_select.rs`: `pub fn parse_preset(value: &str) -> Option<Preset>` — matches `value` against every `Preset::ALL` member's `.name()`, `None` if nothing matches
- `controls/depth_slider.rs`: `pub const MIN_DEPTH: usize = 0;`, `pub const MAX_DEPTH: usize = planet_core::subdivision::steps::MAX_SUBDIVISION_STEPS;`, `pub enum DepthParseError { NotANumber { value: String }, InvalidSteps(StepsError) }` (`Display`/`std::error::Error` impls, `From<StepsError>`), `pub fn parse_depth(value: &str) -> Result<Steps, DepthParseError>` — parses `value` as `usize`, then validates via `Steps::new`
- `controls/seed_from_timestamp.rs` (new): `pub fn seed_from_timestamp(timestamp_millis: f64) -> Seed` — wraps a millisecond timestamp (as returned by `js_sys::Date::now()`) into a `Seed` via a saturating `as u64` cast (Rust float-to-int casts have been saturating, never panicking, since Rust 1.45 — negative/NaN inputs saturate to `0`, values beyond `u64::MAX` saturate to `u64::MAX`)
- `controls.rs` sibling module file: `pub mod preset_select; pub mod depth_slider; pub mod seed_from_timestamp;`, declared in `lib.rs` alongside the existing `pub mod gpu; pub mod scene;`
- `rules.md` gains a new `planet-renderer` concern-list entry: `controls/` — `preset_select.rs` (`parse_preset`), `depth_slider.rs` (`MIN_DEPTH`, `MAX_DEPTH`, `DepthParseError`, `parse_depth`), `seed_from_timestamp.rs` (`seed_from_timestamp`); pure DOM-value parsing/validation and timestamp-to-seed conversion, no browser API calls — the actual element lookups/listeners/clock reads stay in `app.rs`

### `planet-renderer` — `gpu/buffers.rs` (modified)

- `Vertex` gains a third field: `pub color: [f32; 3]`
- `mesh_render_vertices`'s signature changes to `pub fn mesh_render_vertices(mesh: &Mesh, colors: &[Rgb]) -> Vec<Vertex>` — for each triangle corner, in addition to the existing position/flat-shaded-normal lookup, looks up `colors[triangle.a]`/`colors[triangle.b]`/`colors[triangle.c]` (via `Rgb`'s `r()`/`g()`/`b()` accessors, since its fields are `pub(crate)` to `planet-core`) and assigns each corner's own color — unlike the shared flat normal, a triangle's three corners generally get **different** colors (colors are per-source-vertex, not per-face)
- **Precondition, not defensively checked:** `colors.len() == mesh.vertices().len()` — guaranteed by every caller in this codebase, since `colors` always comes from `Planet::colors()` (or, before any generation, an empty slice paired with an empty `Mesh` — see `app.rs` below). Consistent with this function's existing, unchanged trust in `mesh`'s own triangle-index validity (`mesh.vertices()[triangle.a]` is not bounds-checked here either) — per this codebase's "trust internal invariants, validate only at system boundaries" convention
- `pack_vertex_buffer` is unchanged (it already packs whatever fields `Vertex` has via `std::mem::size_of_val`/iterating `.position`/`.normal`; this feature adds a third `.chain(vertex.color.iter())` to that byte-writing loop)

### `planet-renderer` — `gpu/render.rs` (modified)

- `Renderer::new`'s signature gains a `colors: &[Rgb]` parameter (alongside the existing `mesh: &Mesh`), threaded into its `mesh_render_vertices(mesh, colors)` call. Called at startup with an empty `Mesh`/empty colors slice (see `app.rs` below) — `mesh_render_vertices`'s existing empty-mesh behavior (empty input ⇒ empty render-vertex list) already covers this with no new logic
- `Renderer::set_mesh`'s signature gains the same `colors: &[Rgb]` parameter
- `vertex_layout.array_stride` changes from `24` to `36`; a third `wgpu::VertexAttribute { format: Float32x3, offset: 24, shader_location: 2 }` is added

### `planet-renderer` — `gpu/shader.wgsl` (modified, manually verified in-browser only, per `000-architecture.md`'s existing GPU-pixel-output exemption)

- `VertexInput` gains `@location(2) color: vec3<f32>`; `VertexOutput` gains `@location(1) color: vec3<f32>`
- `vs_main` passes `input.color` through to `out.color`
- `fs_main` replaces the hardcoded `vec3<f32>(0.3, 0.55, 0.9)` with `input.color` — the rest of the lighting math (`brightness` from the fixed light direction) is unchanged

### New dependency and `web-sys` feature flags (`Cargo.toml`, `tech-stack.md`)

- `planet-renderer/Cargo.toml` gains a new direct dependency `js-sys = "0.3.103"` (version-matched to the existing `web-sys = "0.3.103"`, per the wasm-bindgen ecosystem's lockstep 0.2.x/0.3.x release scheme) — needed for `js_sys::Date::now()`
- `web-sys`'s `features` list grows from `["console"]` to `["console", "Document", "Element", "Node", "EventTarget", "HtmlInputElement", "HtmlDialogElement"]` — `Document`/`Element` for `get_element_by_id`/`create_element`/attribute get-set, `Node` for `append_child`/`set_text_content` (both defined on `Node`, inherited by `Element` but requiring `Node`'s own feature flag enabled in `web-sys`), `EventTarget` for `add_event_listener_with_callback`, `HtmlInputElement` for the depth slider's `.value()`/`.set_min()`/`.set_max()`/`.set_value()` and each preset radio's `.value()`/`.checked()`, `HtmlDialogElement` for the error modal's `.show_modal()`/`.close()` (see "Error handling" below). A generic `web_sys::Element` (not `HtmlSelectElement`, not needed here) suffices for the preset `<label>`/`<img>`/description elements and the Start/Change-settings `<button>`s, since only `set_attribute`/`append_child`/generic property access is needed on those
- `tech-stack.md`'s dependency table gains a `js-sys` row (`default` features, used in `planet-renderer` — "reads `Date.now()` to seed each Start-click's `Planet`"); its `web-sys` row's feature-flags cell is updated to the new list above

### `planet-renderer` — `app.rs` (modified)

- `DEMO_SEED`/`DEMO_PRESET` are both removed — there is no fixed seed or fixed initial-generation preset anymore. `Preset::default()` (`Earthy`) is still used, but only as the initial **checked** radio button in the markup, not as an eagerly-generated planet
- `App`'s `resumed()` still performs the existing one-time async `Renderer::new(window, &mesh, &colors)` device/GPU setup (unchanged boundary/pattern), but now passes an **empty** mesh/colors pair instead of a generated demo planet. Building that empty mesh goes through `Mesh::new(vec![], vec![])`'s own `Result<Mesh, MeshError>` return value, matched explicitly — never `.expect()`/`.unwrap()` (per `rules.md`'s "no unwrap/panic in production code," and consistent with `013`'s precedent of propagating even a call that "cannot fail in practice" rather than special-casing it, since an empty triangle list can never actually violate `Mesh::new`'s index-bounds check). On `Ok(mesh)`, `resumed()` proceeds exactly as before with `&mesh`/an empty `&[]` colors slice. On the (practically unreachable) `Err(error)` branch, `resumed()` calls the new `show_error_modal` helper below with a descriptive message and returns, leaving the page in its pre-render state rather than panicking. Once the async `Renderer::new` future resolves, `#start-button`'s `disabled` attribute is cleared

#### Error handling: a modal dialog, not a console-only log

- A new private `fn show_error_modal(message: &str)` helper in `app.rs`: looks up `#error-modal` (cast to `HtmlDialogElement`) and `#error-modal-message` (`Element`) — guarded against `None` at both lookups, logged via `web_sys::console::error_1` and returning without showing anything if either is missing (can't show a modal about a missing modal; this mirrors the file's existing DOM-lookup-`None` handling elsewhere). On success: sets `#error-modal-message`'s text content to `message` and calls `.show_modal()` on the dialog
- A `click` listener registered once at startup on `#error-modal-dismiss` calls `.close()` on `#error-modal`
- This mechanism is deliberately scoped to the one call site above (`Mesh::new`'s startup failure) — it is not retrofitted onto this file's other, pre-existing error branches (e.g. `Planet::builder()...build()`/`Renderer::new` failing), which keep their existing console-log-and-return behavior unchanged; those are out of scope for this feature
- `frames: Vec<Mesh>` becomes `frames: Vec<(Mesh, Vec<Rgb>)>` — unchanged rationale from the color-wiring requirement above: each animation frame carries its own colors, computed as `mesh.vertices().iter().map(|v| params.color_gradient().sample(v.position.length())).collect()` from the preset used for that generation (a pure function of a vertex's radius, valid for every intermediate round, not just the final one — ocean-quota flattening is a post-subdivision-only step, so intermediate rounds never show a flattened ocean, only the final one does)
- A new private `generate(preset: Preset, depth: Steps, seed: Seed)`-shaped helper (exact name/signature is an implementation detail) does the actual work a Start click triggers: `PlanetBuilder::build()` a fresh base `Planet` from `(preset, seed)`, call `.subdivide(depth, Some(on_progress))` (collecting colored frames as above), replace the **last** collected frame's `(Mesh, Vec<Rgb>)` with `(subdivided.mesh().clone(), subdivided.colors().to_vec())` (so the animation's final frame always matches `Planet::subdivide`'s true, fully-post-processed output — differs from the raw last round only for `Earthy`'s ocean quota), then set `self.frames`/`self.current_frame = 0` for `RedrawRequested` to play back
- New `#[cfg(target_arch = "wasm32")]`-gated DOM wiring in `resumed()`, guarded against `None` at every lookup (per `rules.md`'s existing DOM-lookup rule — logged via `web_sys::console::error_1` and left non-fatal, mirroring this file's existing error-handling style):
  - Looks up `#controls` (the container div — preset group, depth slider, Start button), `#preset-group` (the `<fieldset>`), `#depth-slider`/`#depth-value`, `#start-button`, and `#change-settings-button`
  - Populates `#preset-group` with one option per `Preset::ALL` member instead of hardcoding them in `index.html` — a single source of truth, avoiding the classic "added a preset, forgot the control" drift. For each `preset`, builds and appends a `<label class="preset-option">` wrapping: a `<input type="radio" name="preset" value={preset.name()}>` (`id="preset-{preset.name()}"`; the one matching `Preset::default()` gets `checked = true`), an `<img class="preset-image-placeholder" alt="{preset.name()} preset preview">` with no `src` set (this spec reserves the slot only; see "Out of scope"), a label-text element containing `preset.name()`, and a description element containing `preset.description()`
  - Sets the slider's `min`/`max`/`value` attributes from `depth_slider::MIN_DEPTH`/`MAX_DEPTH`/`Steps::default().value()` — likewise driven from Rust's own constants rather than duplicated literals in `index.html`
  - Registers an `input` listener on `#depth-slider` that only updates `#depth-value`'s text content (cheap, no generation triggered) — no `change` listener is needed on the slider or on `#preset-group`'s radios at all, since neither triggers generation directly; both are read only at Start-click time
  - Registers a `click` listener on `#start-button`:
    1. Reads the checked radio via `document.query_selector("input[name=preset]:checked")`, casts to `HtmlInputElement`, reads `.value()`, parses via `controls::preset_select::parse_preset` (logged and ignored if `None` — unreachable in practice)
    2. Reads `#depth-slider`'s `.value()`, parses via `controls::depth_slider::parse_depth` (logged and ignored if `Err` — unreachable in practice, since the slider's own `min`/`max`/`step="1"` already constrain every reachable value)
    3. Computes `seed = controls::seed_from_timestamp::seed_from_timestamp(js_sys::Date::now())`
    4. If `renderer.borrow().is_none()` (the async GPU-setup future hasn't resolved yet — an accepted, effectively-unreachable-in-practice race, same tolerance this file's existing async boundary already has elsewhere), logs and returns without generating
    5. Otherwise calls the `generate(preset, depth, seed)` helper above, then hides `#controls` and un-hides `#change-settings-button`
  - Registers a `click` listener on `#change-settings-button`: un-hides `#controls`, hides `#change-settings-button`. No regeneration, no state reset — the previously-selected radio/slider values are still there (the elements were only hidden, never removed), and the canvas keeps rendering whatever was last generated behind the reopened controls

### `index.html` (modified)

Adds bare control markup (containers + labels only — no hardcoded radio `<input>`s or `min`/`max`/`value`, populated from Rust as above) and minimal overlay CSS. `#controls` and `#change-settings-button` are toggled via the HTML `hidden` attribute (one always hidden while the other is shown) rather than custom CSS classes. `.preset-option`/`.preset-image-placeholder` are styled with a small fixed-size box (dashed border, muted background) so the reserved image slot is visually obvious as a placeholder rather than a missing/broken image:

```html
<div id="controls">
  <fieldset id="preset-group">
    <legend>Preset</legend>
  </fieldset>

  <label for="depth-slider">Depth (<span id="depth-value"></span>)</label>
  <input id="depth-slider" type="range" step="1" />

  <button id="start-button" type="button" disabled>Start</button>
</div>

<button id="change-settings-button" type="button" hidden>Change settings</button>

<dialog id="error-modal">
  <p id="error-modal-message"></p>
  <button id="error-modal-dismiss" type="button">OK</button>
</dialog>
```

## Domain model involved

**`planet-core/src/presets/preset.rs` (modified — additions only):**
```rust
impl Preset {
    pub const ALL: [Preset; 3] = [Preset::Earthy, Preset::Volcano, Preset::Rocky];

    pub fn name(&self) -> &'static str {
        match self {
            Preset::Earthy => "Earthy",
            Preset::Volcano => "Volcano",
            Preset::Rocky => "Rocky",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Preset::Earthy => "Oceans, grasslands, and snow-capped peaks.",
            Preset::Volcano => "Charred basalt and glowing molten rock.",
            Preset::Rocky => "Barren gray stone, no water or lava.",
        }
    }
}
```

**`planet-renderer/src/controls/preset_select.rs` (new):**
```rust
use planet_core::presets::preset::Preset;

pub fn parse_preset(value: &str) -> Option<Preset> {
    Preset::ALL.into_iter().find(|preset| preset.name() == value)
}
```

**`planet-renderer/src/controls/depth_slider.rs` (new):**
```rust
use std::fmt;

use planet_core::subdivision::steps::{MAX_SUBDIVISION_STEPS, Steps, StepsError};

pub const MIN_DEPTH: usize = 0;
pub const MAX_DEPTH: usize = MAX_SUBDIVISION_STEPS;

#[derive(Debug, Clone, PartialEq)]
pub enum DepthParseError {
    NotANumber { value: String },
    InvalidSteps(StepsError),
}

impl fmt::Display for DepthParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DepthParseError::NotANumber { value } => {
                write!(f, "depth value {value:?} is not a number")
            }
            DepthParseError::InvalidSteps(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for DepthParseError {}

impl From<StepsError> for DepthParseError {
    fn from(error: StepsError) -> DepthParseError {
        DepthParseError::InvalidSteps(error)
    }
}

pub fn parse_depth(value: &str) -> Result<Steps, DepthParseError> {
    let raw: usize = value
        .parse()
        .map_err(|_| DepthParseError::NotANumber {
            value: value.to_string(),
        })?;
    Ok(Steps::new(raw)?)
}
```

**`planet-renderer/src/controls/seed_from_timestamp.rs` (new):**
```rust
use planet_core::subdivision::seed::Seed;

pub fn seed_from_timestamp(timestamp_millis: f64) -> Seed {
    Seed::from(timestamp_millis as u64)
}
```

**`planet-renderer/src/controls.rs` (new, sibling module-declaration file):**
```rust
pub mod depth_slider;
pub mod preset_select;
pub mod seed_from_timestamp;
```

**`planet-renderer/src/lib.rs` (modified):**
```rust
#[cfg(target_arch = "wasm32")]
pub mod app;
pub mod controls;
pub mod gpu;
pub mod scene;
```

**`planet-renderer/src/gpu/buffers.rs` (modified):**
```rust
use planet_core::color::rgb::Rgb;
use planet_core::geometry::mesh::Mesh;
use planet_core::geometry::vec3::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
}

pub fn mesh_render_vertices(mesh: &Mesh, colors: &[Rgb]) -> Vec<Vertex> {
    mesh.triangles()
        .iter()
        .flat_map(|triangle| {
            let a = mesh.vertices()[triangle.a].position;
            let b = mesh.vertices()[triangle.b].position;
            let c = mesh.vertices()[triangle.c].position;
            let normal = b
                .sub(a)
                .cross(c.sub(a))
                .normalized()
                .unwrap_or(Vec3::new(0.0, 0.0, 0.0));
            let normal = [normal.x, normal.y, normal.z];
            let corner_colors = [
                colors[triangle.a],
                colors[triangle.b],
                colors[triangle.c],
            ];
            [a, b, c]
                .into_iter()
                .zip(corner_colors)
                .map(move |(position, color)| Vertex {
                    position: [position.x, position.y, position.z],
                    normal,
                    color: [color.r(), color.g(), color.b()],
                })
        })
        .collect()
}
```
(`mesh_render_indices`, `mesh_render_line_indices`, `pack_index_buffer` are unchanged; `pack_vertex_buffer` gains `.chain(vertex.color.iter())` in its per-vertex byte-writing loop.)

Existing types this feature calls but does not modify: `Mesh`/`Vec3` (`geometry/`), `Rgb` (`color/rgb.rs`), `Planet`/`PlanetBuilder`/`Preset`/`Seed`/`Steps`/`StepsError` (aside from `Preset`'s two additions and `MAX_SUBDIVISION_STEPS` already being `pub`).

## Function/API contracts

### `Preset::name`
```rust
pub fn name(&self) -> &'static str
```
- **Pre:** none
- **Post:** returns a fixed, non-empty string unique per variant (`"Earthy"`, `"Volcano"`, `"Rocky"`); stable across calls (pure function of `self`)

### `Preset::description`
```rust
pub fn description(&self) -> &'static str
```
- **Pre:** none
- **Post:** returns a fixed, non-empty string per variant; stable across calls (pure function of `self`). No uniqueness requirement (unlike `name()`, which must round-trip through `parse_preset`) — this string is display-only, never parsed back

### `controls::preset_select::parse_preset`
```rust
pub fn parse_preset(value: &str) -> Option<Preset>
```
- **Pre:** `value` is any `&str`
- **Post:** `Some(preset)` iff `value == preset.name()` for exactly one `preset` in `Preset::ALL`; `None` otherwise (including case-mismatches and unrecognized strings — comparison is exact, no case-folding). For every `p` in `Preset::ALL`, `parse_preset(p.name()) == Some(p)` (round-trip law)

### `controls::depth_slider::parse_depth`
```rust
pub fn parse_depth(value: &str) -> Result<Steps, DepthParseError>
```
- **Pre:** `value` is any `&str`
- **Post:** `Ok(Steps::new(n).unwrap())` iff `value` parses as a `usize` `n` with `n <= MAX_DEPTH`; `Err(DepthParseError::NotANumber { value })` if `value` does not parse as a `usize` (including negative numbers, decimals, empty string, non-numeric text); `Err(DepthParseError::InvalidSteps(_))` if it parses but exceeds `MAX_DEPTH`

### `controls::seed_from_timestamp::seed_from_timestamp`
```rust
pub fn seed_from_timestamp(timestamp_millis: f64) -> Seed
```
- **Pre:** `timestamp_millis` is any `f64`, including negative values, `NaN`, and infinities
- **Post:** never panics, for any input. Returns `Seed::from(timestamp_millis as u64)`: for a finite, non-negative value representable in `u64`, this is simple truncation towards zero; a negative value or `NaN` saturates to `Seed::from(0)`; a value at or beyond `u64::MAX` (including `+inf`) saturates to `Seed::from(u64::MAX)`. Two calls with the same `timestamp_millis` always return an equal `Seed` (pure function)

### `mesh_render_vertices` (updated contract)
```rust
pub fn mesh_render_vertices(mesh: &Mesh, colors: &[Rgb]) -> Vec<Vertex>
```
- **Pre:** `mesh` is any valid `Mesh`; `colors.len() == mesh.vertices().len()` (see "Requirements" — trusted, not checked)
- **Post:** all of this function's pre-existing postconditions (one vertex per triangle corner, shared flat per-triangle normal, unit-length or zero normal, empty mesh ⇒ empty list, no panic on a degenerate triangle) continue to hold unchanged. Additionally: for every render vertex produced from triangle corner referencing source-vertex index `i`, that vertex's `color` equals `[colors[i].r(), colors[i].g(), colors[i].b()]` — unlike `normal`, `color` is **not** forced identical across a triangle's three render vertices

## BDD scenarios

### `planet-core/tests/features/preset.feature` (extended)
```gherkin
  Scenario: Preset::ALL lists all three presets in a fixed order
    When Preset::ALL is requested
    Then Preset::ALL equals Earthy, Volcano, Rocky in that order

  Scenario: Every Preset has a human-readable name
    When each Preset's name is requested
    Then Preset::Earthy's name is "Earthy"
    And Preset::Volcano's name is "Volcano"
    And Preset::Rocky's name is "Rocky"

  Scenario: Every Preset has a non-empty description
    When each Preset's description is requested
    Then Preset::Earthy's description is non-empty
    And Preset::Volcano's description is non-empty
    And Preset::Rocky's description is non-empty
```

### `planet-renderer/tests/features/preset_select.feature` (new)
```gherkin
Feature: Parsing a preset-select DOM value into a Preset

  Scenario: Parsing a recognized preset name returns the matching Preset
    When the preset-select value "Volcano" is parsed
    Then the parsed Preset is Volcano

  Scenario: Every Preset::ALL member's own name round-trips to itself
    When each of Preset::ALL's names is parsed
    Then every parsed Preset equals its source Preset

  Scenario: Parsing an unrecognized value returns no Preset
    When the preset-select value "Unknown" is parsed
    Then no Preset is returned
```

### `planet-renderer/tests/features/depth_slider.feature` (new)
```gherkin
Feature: Parsing a depth-slider DOM value into validated Steps

  Scenario: Parsing a value within range succeeds
    When the depth-slider value "5" is parsed
    Then the parsed Steps has value 5

  Scenario: Parsing the minimum boundary value succeeds
    When the depth-slider value "0" is parsed
    Then the parsed Steps has value 0

  Scenario: Parsing the maximum boundary value succeeds
    When the depth-slider value "8" is parsed
    Then the parsed Steps has value 8

  Scenario: Parsing a value above the maximum fails
    When the depth-slider value "9" is parsed
    Then the parsing fails with an invalid-steps error

  Scenario: Parsing a non-numeric value fails
    When the depth-slider value "abc" is parsed
    Then the parsing fails with a not-a-number error
```

### `planet-renderer/tests/features/seed_from_timestamp.feature` (new)
```gherkin
Feature: Converting a millisecond timestamp into a Seed

  Scenario: Converting a typical timestamp produces the expected Seed
    When the timestamp 1752400000000.0 is converted to a Seed
    Then the resulting Seed has value 1752400000000

  Scenario: Converting a negative timestamp saturates to zero
    When the timestamp -1.0 is converted to a Seed
    Then the resulting Seed has value 0

  Scenario: Converting NaN saturates to zero
    When the timestamp NaN is converted to a Seed
    Then the resulting Seed has value 0

  Scenario: Converting a timestamp beyond u64's range saturates to the maximum
    When the timestamp 1e30 is converted to a Seed
    Then the resulting Seed has the maximum u64 value

  Scenario: Converting the same timestamp twice produces equal Seeds
    When the timestamp 1752400000000.0 is converted to a Seed twice
    Then both resulting Seeds are equal
```

### `planet-renderer/tests/features/mesh_render_vertices.feature` (extended)

Existing scenarios' "converted into render vertices" step gains an implicit uniform-white `colors` fixture (one per source vertex) so they keep asserting position/normal behavior unchanged; one new scenario asserts real per-vertex color mapping:

```gherkin
  Scenario: Converting a Mesh into render vertices assigns each render vertex the color of its source vertex
    Given a Mesh constructed by Mesh::cube with side 1.0
    And a distinct Rgb color for each of the mesh's vertices
    When the mesh is converted into render vertices using those colors
    Then each render vertex's color equals its source vertex's Rgb
```

### `planet-renderer/tests/features/buffers.feature` (unchanged scenarios, updated fixture)

The existing `Vertex { position, normal }` struct literals in `tests/buffers.rs`'s step definitions gain a `color: [0.0, 0.0, 0.0]` field — a compile fix, not a behavioral change, since `assert_vertex_buffer_len`'s `std::mem::size_of::<Vertex>()` already generically covers whatever fields `Vertex` has.

### Not BDD-tested (manually verified in-browser only, per `000-architecture.md`'s existing exemption for GPU pixel output and DOM/browser-facing wiring)

- `gpu/shader.wgsl`'s color-passthrough change
- `gpu/render.rs`'s pipeline/vertex-layout changes
- `app.rs`'s DOM element lookup, radio-option/attribute population, Configuring/Viewing toggle, and `click`/`input` listener wiring — verified manually: reload the page and confirm the canvas is empty with `#controls` visible and `#start-button` disabled until GPU setup finishes; pick a preset/depth and click Start, confirm the colored growth animation plays, ends on the correct preset's colors, and `#controls` is replaced by "Change settings"; click "Change settings," confirm the controls reappear with the same preset/depth still selected and the previous planet still visible behind them; click Start again and confirm a visibly different mesh is generated for the same preset/depth (different timestamp seed)
- `show_error_modal`'s wiring and `#error-modal`/`#error-modal-dismiss` — verified by code inspection (the `Mesh::new(vec![], vec![])` `Err` branch this guards is practically unreachable, since an empty triangle list can never violate its index-bounds check) and, if feasible, by temporarily forcing the `Err` branch during manual testing to confirm the modal displays the message and `#error-modal-dismiss` closes it

## Acceptance criteria

1. `Preset::ALL` exists as `[Preset::Earthy, Preset::Volcano, Preset::Rocky]`; `Preset::name()` returns `"Earthy"`/`"Volcano"`/`"Rocky"` for the respective variants; `Preset::description()` returns a non-empty string for every variant (unit/BDD test)
2. `planet-renderer` gains a `controls/` concern (`controls/preset_select.rs`, `controls/depth_slider.rs`, `controls/seed_from_timestamp.rs`, declared via sibling `controls.rs`), added to `rules.md`'s module-structure list; none of the three files calls any `web_sys`/`js_sys`/browser API
3. `controls::preset_select::parse_preset(value)` returns `Some(preset)` iff `value` exactly matches `preset.name()` for some `preset` in `Preset::ALL`, `None` otherwise; round-trips for every `Preset::ALL` member (unit/BDD test)
4. `controls::depth_slider::{MIN_DEPTH, MAX_DEPTH}` equal `0` and `planet_core::subdivision::steps::MAX_SUBDIVISION_STEPS` (8) respectively; `parse_depth(value)` succeeds for every integer in `MIN_DEPTH..=MAX_DEPTH` and fails (with the correct `DepthParseError` variant) for non-numeric input and for values above `MAX_DEPTH` (unit/BDD test)
5. `controls::seed_from_timestamp::seed_from_timestamp(timestamp)` never panics for any `f64` input (including negative, `NaN`, and values beyond `u64::MAX`) and is a pure, deterministic function of its input (unit/BDD test)
6. `gpu::buffers::Vertex` gains a `color: [f32; 3]` field; `mesh_render_vertices(mesh, colors)` assigns each render vertex the `Rgb` of its own source vertex (not forced identical per triangle, unlike `normal`); all four pre-existing `mesh_render_vertices` scenarios continue to pass (unit/BDD test)
7. `pack_vertex_buffer`'s byte-length postcondition (`vertex count * size_of::<Vertex>()`) continues to hold with the enlarged `Vertex` (unit/BDD test, no logic change needed — `size_of::<Vertex>()` already reflects the new field)
8. `Renderer::new`/`Renderer::set_mesh` both accept a `colors: &[Rgb]` parameter and thread it into `mesh_render_vertices`; the vertex buffer's `array_stride` is `36` with a third `Float32x3` attribute at `shader_location: 2`
9. `gpu/shader.wgsl`'s fragment shader outputs `input.color * brightness` instead of a hardcoded fixed color (manually verified in-browser — different presets visibly render different colors)
10. On page load, `#controls` is visible, `#change-settings-button` is hidden, `#start-button` is disabled, and the canvas renders with an empty mesh (no planet) until GPU setup completes and re-enables `#start-button` (manually verified in-browser)
11. Clicking Start (once enabled) reads the checked preset radio and the depth slider's value, computes a seed via `seed_from_timestamp(js_sys::Date::now())`, unconditionally builds and subdivides a fresh `Planet` from those three values, plays the colored growth animation ending on the true post-processed result, then hides `#controls` and shows `#change-settings-button` (manually verified in-browser)
12. Clicking "Change settings" re-shows `#controls` (previously selected preset/depth still intact) and hides `#change-settings-button`, with the canvas continuing to render the last generated planet behind the reopened controls (manually verified in-browser)
13. Clicking Start again after "Change settings" always performs a full fresh build (never reuses a prior `Planet`) and, for the same preset/depth, produces a visibly/numerically different mesh from the previous generation, since each click's timestamp-derived seed differs (manually verified in-browser)
14. `#preset-group`'s radio options (each with a label, description, and an image-placeholder slot) and the depth slider's `min`/`max`/`value` attributes are populated from `Preset::ALL`/`controls::depth_slider`'s constants/`Steps::default()` at startup, not hardcoded in `index.html` (manually verified: `index.html`'s `#preset-group` contains no `<input>` elements and the slider has no `min`/`max`/`value` attributes prior to script execution)
15. Every generated preset option includes an `<img class="preset-image-placeholder">` with no `src` and a descriptive `alt` (`"{name} preset preview"`) — reserving the slot for a future real preview image without rendering a broken-image icon today (manually verified in-browser)
16. `js-sys` is added as a direct `planet-renderer` dependency and appears in `tech-stack.md`; `web-sys`'s feature list includes `Document`, `Element`, `Node`, `EventTarget`, `HtmlInputElement`, `HtmlDialogElement` in addition to the existing `console`, reflected in `tech-stack.md`
17. `resumed()` never calls `.expect()`/`.unwrap()` on `Mesh::new(vec![], vec![])`'s `Result`; on `Err`, it calls `show_error_modal` with a descriptive message and returns, rather than panicking (unit-inspectable: no `.expect()`/`.unwrap()` on that call in `app.rs`'s source)
18. `show_error_modal(message)` sets `#error-modal-message`'s text content to `message` and calls `.show_modal()` on `#error-modal`; clicking `#error-modal-dismiss` calls `.close()` on it; both DOM lookups are guarded against `None` (logged, non-fatal) rather than unwrapped (manually verified / code inspection, per the note above about this path's practical unreachability)
19. No other DOM lookup/cast in `app.rs` uses `.unwrap()`/`.expect()`/`panic!()` either — every `None`/`Err`, and the `renderer.borrow().is_none()` async-setup race at Start-click time, is logged via `web_sys::console::error_1` and handled non-fatally, per `rules.md`
20. Camera orbit/zoom, the wireframe toggle, and existing keyboard shortcuts are unaffected by this feature (manually verified — no regression)
21. All new/extended BDD scenarios above are backed by real `cucumber` step definitions (`preset.feature` extension, new `preset_select.feature`/`depth_slider.feature`/`seed_from_timestamp.feature`, extended `mesh_render_vertices.feature`) — no scenario left as markdown prose
22. Build gate passes: `cargo test --workspace && cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo build --target wasm32-unknown-unknown -p planet-renderer`
