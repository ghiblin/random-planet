# 001 — Cube Render

**Status:** Ready for review
**Feature slug:** `cube-render`

## Requirements

- Scaffold the Cargo workspace: `planet-core` (empty domain crate, populated starting spec `002`) and `planet-renderer` (wgpu/winit/wasm-bindgen crate)
- Wire up Trunk to build `planet-renderer` to `wasm32-unknown-unknown` and serve it via `index.html`
- Render a single static cube in the browser via wgpu
- Camera orbits the cube on mouse drag and zooms on scroll, per the UI controls agreed in `docs/specs/000-architecture.md`
- Establish the crate split from `rules.md`: pure/testable logic (`camera.rs`, `buffers.rs`, `uniforms.rs`) separate from thin GPU/browser glue (`render.rs`, `app.rs`, the wasm-bindgen entry point)

Out of scope for this phase (later roadmap phases): any generated `Mesh` from `planet-core`, presets, color gradients, subdivision-depth UI, preset dropdown.

## Domain model involved

All new — this is the first code in the project.

**`planet-renderer` (new, pure/testable):**
- `Camera` (`camera.rs`) — holds `yaw`, `pitch`, `distance`
  - `Camera::orbit(&mut self, delta_yaw: f32, delta_pitch: f32)` — applies a mouse-drag delta; pitch is clamped to avoid gimbal-lock flip (just short of ±90°)
  - `Camera::zoom(&mut self, scroll_delta: f32)` — applies a scroll delta to `distance`, clamped to `[MIN_DISTANCE, MAX_DISTANCE]`
  - `Camera::view_projection_matrix(&self, aspect_ratio: f32) -> [[f32; 4]; 4]` — combines view (derived from yaw/pitch/distance, looking at the origin) and perspective projection
- Cube fixture + buffer packing (`buffers.rs`) — a fixed 24-vertex/36-index cube (4 vertices per face for flat per-face normals), and:
  - `pack_vertex_buffer(vertices: &[Vertex]) -> Vec<u8>`
  - `pack_index_buffer(indices: &[u16]) -> Vec<u8>`
- Uniform buffer packing (`uniforms.rs`):
  - `pack_view_projection_uniform(matrix: &[[f32; 4]; 4]) -> Vec<u8>` — serializes the view-projection matrix into the byte layout the WGSL uniform expects (column-major, 64 bytes)

**`planet-renderer` (new, thin, not BDD-tested):**
- `render.rs` — wgpu instance/device/pipeline setup, draw call
- `app.rs` — winit event loop; translates browser mouse-drag/scroll events into `Camera::orbit`/`Camera::zoom` calls
- `lib.rs` — `#[wasm_bindgen(start)]` entry point, `pub mod` declarations only

**`planet-core` (new, empty):**
- `lib.rs` with no public items yet — exists so the workspace shape is final and spec `002` doesn't need further workspace restructuring

## Function/API contracts

- `Camera::orbit` and `Camera::zoom` never panic and never produce `NaN`/`infinite` fields for finite input deltas, regardless of magnitude (large deltas saturate at the clamp bounds rather than overflowing)
- `Camera::view_projection_matrix` never contains `NaN` for any valid (already-clamped) camera state and a positive, finite `aspect_ratio`
- `pack_vertex_buffer`/`pack_index_buffer` output length is always exactly `input.len() * size_of::<T>()`, for any input slice length including zero
- `pack_view_projection_uniform` output is always exactly 64 bytes (16 `f32`s) for any finite matrix input

## BDD scenarios

`planet-renderer/tests/features/camera.feature`:

```gherkin
Feature: Camera orbit and zoom

  Scenario: Orbiting updates yaw and pitch
    Given a Camera constructed with default orbit parameters
    When the camera is orbited by a mouse delta of (0.2, 0.1)
    Then the camera's yaw increases by 0.2
    And the camera's pitch increases by 0.1

  Scenario: Orbiting clamps pitch to avoid gimbal-lock flip
    Given a Camera constructed with default orbit parameters
    When the camera is orbited upward past the maximum pitch
    Then the camera's pitch stays at the maximum allowed pitch

  Scenario: Zooming in decreases distance
    Given a Camera constructed with default orbit parameters
    When the camera is zoomed in by a scroll delta of 1.0
    Then the camera's distance decreases
    And the camera's distance stays at or above the minimum distance

  Scenario: Zooming in past the minimum distance clamps
    Given a Camera constructed at the minimum distance
    When the camera is zoomed in by a scroll delta of 100.0
    Then the camera's distance stays at the minimum distance

  Scenario: Zooming out past the maximum distance clamps
    Given a Camera constructed at the maximum distance
    When the camera is zoomed out by a scroll delta of 100.0
    Then the camera's distance stays at the maximum distance
```

`planet-renderer/tests/features/buffers.feature`:

```gherkin
Feature: Vertex and index buffer packing

  Scenario: Packing the cube's vertex list produces a correctly sized buffer
    Given the cube's fixed vertex list
    When the vertex list is packed into a vertex buffer
    Then the buffer's byte length equals the vertex count times the vertex stride

  Scenario: Packing the cube's index list produces a correctly sized buffer
    Given the cube's fixed index list
    When the index list is packed into an index buffer
    Then the buffer's byte length equals the index count times the index size

  Scenario: Packing an empty vertex list produces an empty buffer
    Given an empty vertex list
    When the vertex list is packed into a vertex buffer
    Then the buffer is empty
```

`planet-renderer/tests/features/uniforms.feature`:

```gherkin
Feature: Uniform buffer packing

  Scenario: Packing a view-projection matrix produces a correctly sized uniform buffer
    Given a view-projection matrix computed from a Camera
    When the matrix is packed into a uniform buffer
    Then the buffer's byte length equals 64 bytes
```

## Acceptance criteria

1. `cargo test --workspace` passes, including all scenarios in `camera.feature`, `buffers.feature`, and `uniforms.feature` via real `cucumber` step definitions (no stub steps)
2. `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings` pass
3. `cargo build --target wasm32-unknown-unknown -p planet-renderer` succeeds
4. `trunk build` succeeds and produces a loadable `index.html` + WASM artifact
5. Manually verified in a browser: a cube is visible, mouse-drag orbits it, scroll wheel zooms in/out and stops at the configured bounds
6. `planet-core` exists in the workspace as a compiling (empty) crate
