# 006 — By-Concern File Layout

**Status:** Ready for review
**Feature slug:** `by-concern-file-layout`

This is an ad-hoc structural spec, not the next numbered roadmap phase (`docs/roadmap.md`'s own "006 — Irregular subdivision" is unaffected and will simply become the next spec number whenever it is written) — it reorganizes both crates' existing `src/` trees from a flat file list into concern subdirectories, and documents a `rules.md` rule that prevents the flat layout from creeping back in. It also moves `Mesh::cube`'s implementation and the existing free function `icosahedron()` into a `primitives/` sub-concern nested under `geometry/` as `pub(crate)` functions, with `Mesh` gaining `pub fn cube` and `pub fn icosahedron` facade methods that delegate to them — mirroring the public-facade-over-crate-private-implementation pattern `005-subdivision-facade` already established for `SubdivisionMode`/`SubdivisionStrategy`. `Mesh::cube`'s public call form is unchanged; `Mesh::icosahedron` is a new addition (`icosahedron()` was previously a free function, never a `Mesh` method).

## Requirements

- Both crates' `src/` directories move from "one file per type, flat under `src/`" (`rules.md`'s current wording) to **by-concern subdirectories**. Every existing `.rs` file moves into a concern folder; no type, field, method, visibility keyword, or algorithm changes — this is a pure module-path relocation, with one exception (see the `cube`/`icosahedron` bullet below)
- `planet-core` gets two top-level concerns, each declared via a sibling `<concern>.rs` module file (Rust 2024 module style — no `mod.rs`):
  - `geometry/` — `vec3.rs` (`Vec3`), `mesh.rs` (`Vertex`, `Triangle`, `Mesh`, `MeshError`), plus a nested `primitives/` sub-concern: `icosahedron.rs` (`pub(crate) fn icosahedron()`, new visibility — see below), `cube.rs` (`pub(crate) fn cube()`, new file and visibility — see below). `primitives` nests inside `geometry` rather than sitting alongside it as a third top-level concern, since every primitive is built entirely from `geometry`'s own types (`Vec3`, `Mesh`, `Vertex`, `Triangle`) and returns a `Mesh` — it's `geometry`'s own mesh-construction toolkit, not an independent concern
  - `subdivision/` — `edge.rs` (`EdgeKey`, `EdgeCache`, `pub(crate)`), `steps.rs` (`Steps`, `StepsError`, `MAX_SUBDIVISION_STEPS`), `subdivision_mode.rs` (`SubdivisionMode`), `subdivision_args.rs` (`SubdivisionArgs`, `UpdateCallback`), `subdivide.rs` (`SubdivisionStrategy` `pub(crate)`, `subdivide()`), `uniform_red_split.rs` (`UniformRedSplit`, `pub(crate)`)
- `planet-renderer` gets two concerns plus its existing top-level composition root:
  - `scene/` — `camera.rs` (`Camera`)
  - `gpu/` — `buffers.rs`, `uniforms.rs`, `render.rs`, `shader.wgsl` (everything wgpu-facing: mesh/preset-to-GPU-data mapping and the actual device/pipeline/draw calls)
  - `app.rs` stays directly under `src/`, unchanged in role (winit event loop, wasm-bindgen entry point, HTML control wiring — the composition root, not a concern bucket)
- `edge.rs` and `uniform_red_split.rs` keep their non-`pub` module declarations (`mod edge;`, `mod uniform_red_split;`), but nested one level deeper — inside `subdivision.rs` instead of `lib.rs`. Rust's privacy rule (an item with no `pub` is visible to its defining module and that module's descendants) means this nesting actually **tightens** their visibility: today they're reachable from anywhere in the crate (declared at crate root); after this move they're reachable only from within `subdivision` and its submodules, which is the only place they're used. This is a side effect of the move, not a new requirement to design for
- `Mesh::cube`'s implementation moves out of `geometry/mesh.rs` into `geometry/primitives/cube.rs` as a `pub(crate) fn cube(side: f32) -> Result<Mesh, MeshError>`; the existing free function `icosahedron()` (currently `planet_core::icosahedron::icosahedron`, `pub`) moves into `geometry/primitives/icosahedron.rs` and becomes `pub(crate) fn icosahedron() -> Result<Mesh, MeshError>`. `Mesh` (`geometry/mesh.rs`) gains two `pub` facade methods that delegate to them:
  ```rust
  use super::primitives::{cube::cube, icosahedron::icosahedron};

  impl Mesh {
      pub fn cube(side: f32) -> Result<Mesh, MeshError> {
          cube(side)
      }

      pub fn icosahedron() -> Result<Mesh, MeshError> {
          icosahedron()
      }
  }
  ```
  This mirrors `005-subdivision-facade`'s existing pattern: a `pub` type in the owning concern (`Mesh` in `geometry`, `SubdivisionMode` in `subdivision`) is the only externally visible entry point, delegating to `pub(crate)` implementations that live in a sub-concern (here, `geometry/primitives`). `Mesh::cube`'s public call form (`Mesh::cube(side)`) is completely unchanged — only its implementation moves. `Mesh::icosahedron()` is a genuinely new public API surface (an addition, not a rename): every existing call site that used the free `icosahedron()` function switches to `Mesh::icosahedron()` instead. `MeshError::NegativeCubeSide` stays in `geometry/mesh.rs`'s `MeshError` enum — it's a cross-cutting error shared by any mesh-construction site, exactly like `icosahedron()` already returns `Result<Mesh, MeshError>` without owning any variant of it. `mesh.rs` and `primitives/{cube,icosahedron}.rs` referencing each other (`mesh.rs` via `super::primitives::...`, `primitives/*.rs` via `crate::geometry::mesh::...`) is now ordinary intra-concern file organization, not cross-concern coupling — both live inside `geometry`
- Every internal `use crate::<old_path>::...` (in `src/`) and `use planet_core::<old_path>::...` / `use planet_renderer::<old_path>::...` (in `tests/*.rs` step-definition files) updates to the new `<concern>::<module>` path, and every call site of the free `icosahedron()` function (`planet-renderer/src/app.rs`, `planet-core/tests/icosahedron.rs`, `planet-core/tests/subdivide.rs`) drops its `use planet_core::icosahedron::icosahedron;` import and calls `Mesh::icosahedron()` instead. `tests/` and `tests/features/` directory layout itself is **out of scope** — those stay flat, both because the ask was specifically about production-code (`src/`) organization and because Cargo's own integration-test harness already requires each `tests/<name>.rs` to be a distinct top-level file matching a `[[test]] name = "<name>"` Cargo.toml entry
- **Enforcement is documentation-only:** `rules.md`'s "Module structure" section is rewritten to describe the by-concern layout (see Domain model involved below) and states the rule that no new top-level `.rs` file may be added to either crate's `src/` outside the allowed list. This is enforced the same way every other convention in `rules.md` already is — by `planet-pr-validate`'s spec-adherence review — not by a new automated test; no `tests/file_layout.rs` or similar is added in this feature

Out of scope:
- Any change to `tests/`/`tests/features/` layout or naming beyond fixing `icosahedron()` call sites to `Mesh::icosahedron()` (see Requirements) and updating `use` paths; no Cargo.toml `[[test]]` entries change, and no `.feature` file wording changes — `mesh.feature`'s cube scenarios already say "`Mesh::cube`" (still accurate) and `icosahedron.feature` already says "an icosahedron mesh" (already abstract, no function name to update)
- Any new domain type, public API contract change, or algorithm change beyond the `Mesh::icosahedron()` addition described above — `planet_core::geometry::mesh::Mesh` behaves identically to today's `planet_core::mesh::Mesh`, just at a new path; `Mesh::cube`'s and `Mesh::icosahedron()`'s behavior (vertex/triangle layout, error conditions) is byte-for-byte identical to today's `Mesh::cube`/free `icosahedron()`
- `planet-core/RULES.md` / `planet-renderer/RULES.md` (the dependency allow/blocklists) — unaffected, since no dependency changes
- A `mod.rs`-per-folder style — this project uses the sibling-file module style (`geometry.rs` next to `geometry/`), consistent with `lib.rs` already being "module declarations only, no logic" one level up
- Any automated build-gate check for file placement — the rule is documentation-only (see Enforcement above); enforcing concern-internal file naming or a max-files-per-concern rule is likewise out of scope, staying a documented convention exercised at `planet-pr-validate` review time when a new concern is proposed

## Domain model involved

No new types. Existing types relocate as follows (identifiers, visibility, and behavior unchanged):

**`planet-core/src/` (was flat, now by concern):**
```
planet-core/src/
  lib.rs                        # pub mod geometry; pub mod subdivision;
  geometry.rs                   # pub mod vec3; pub mod mesh; mod primitives;
                                 # (primitives is a private sub-concern of geometry — nothing
                                 # inside it is reachable outside geometry, let alone outside
                                 # the crate; a private declaration here is visible to geometry
                                 # and all of geometry's descendants, including mesh.rs)
  geometry/
    vec3.rs                     # Vec3
    mesh.rs                     # Vertex, Triangle, Mesh, MeshError
                                 # Mesh gains: pub fn cube(side) { cube(side) }
                                 #             pub fn icosahedron() { icosahedron() }
                                 # (delegating to primitives::cube / primitives::icosahedron
                                 # via `use super::primitives::{cube::cube, icosahedron::icosahedron};`)
    primitives.rs                # pub(crate) mod icosahedron; pub(crate) mod cube;
                                  # (must be `pub(crate)`, not bare private: mesh.rs is a
                                  # *sibling* of primitives under geometry, not a descendant of
                                  # primitives, so it needs the path geometry::primitives::cube
                                  # to be reachable — unlike edge/uniform_red_split below, whose
                                  # only consumers already live inside subdivision's own subtree)
    primitives/
      icosahedron.rs             # pub(crate) fn icosahedron() — moved from the old free
                                  # pub fn icosahedron() in icosahedron.rs; now crate-private
      cube.rs                    # pub(crate) fn cube() — moved from Mesh::cube's body in mesh.rs
  subdivision.rs                # mod edge; pub mod steps; pub mod subdivision_mode;
                                 # pub mod subdivision_args; pub mod subdivide; mod uniform_red_split;
  subdivision/
    edge.rs                     # EdgeKey, EdgeCache (pub(crate))
    steps.rs                    # Steps, StepsError, MAX_SUBDIVISION_STEPS
    subdivision_mode.rs         # SubdivisionMode
    subdivision_args.rs         # SubdivisionArgs, UpdateCallback
    subdivide.rs                # SubdivisionStrategy (pub(crate)), subdivide()
    uniform_red_split.rs        # UniformRedSplit (pub(crate))
```

Path changes example: `use crate::mesh::Mesh;` (in `subdivide.rs`) → `use crate::geometry::mesh::Mesh;`; `use crate::edge::EdgeCache;` (in `subdivide.rs`, now `subdivision/subdivide.rs`) → `use super::edge::EdgeCache;`. `Mesh::cube(side)` call sites (`planet-core/tests/mesh.rs`, `planet-renderer/tests/mesh_render_indices.rs`, `mesh_render_line_indices.rs`, `mesh_render_vertices.rs`) are **unchanged** — `Mesh::cube` keeps its exact public call form; only its implementation moves (to `geometry::primitives::cube::cube`, `pub(crate)`, unreachable from those test files directly).

**`planet_core::icosahedron::icosahedron` call sites — the one call-site-visible change:**
- `planet-renderer/src/app.rs`, `planet-core/tests/icosahedron.rs`, `planet-core/tests/subdivide.rs` each currently do `use planet_core::icosahedron::icosahedron;` and call bare `icosahedron()`. All three drop that import (each file already separately imports `Mesh` — `use planet_core::mesh::Mesh;`, becoming `use planet_core::geometry::mesh::Mesh;`) and call `Mesh::icosahedron()` instead. No `.feature` file changes: `icosahedron.feature` already reads "Given an icosahedron mesh" / "Then the Mesh is constructed successfully" with no literal function name to update

**`planet-renderer/src/` (was flat, now by concern):**
```
planet-renderer/src/
  lib.rs                         # #[cfg(wasm32)] pub mod app; pub mod scene; pub mod gpu;
                                  # (wasm_bindgen `start` fn unchanged)
  app.rs                         # App (composition root, stays top-level)
  scene.rs                       # pub mod camera;
  scene/
    camera.rs                    # Camera
  gpu.rs                         # pub mod buffers; pub mod uniforms; pub mod render;
  gpu/
    buffers.rs                   # Vertex, pack_vertex_buffer, pack_index_buffer,
                                  # mesh_render_vertices, mesh_render_indices, mesh_render_line_indices
    uniforms.rs                  # pack_view_projection_uniform
    render.rs                    # Renderer
    shader.wgsl                  # unchanged; render.rs's include_str!("shader.wgsl") stays valid
                                  # (path is relative to render.rs's own file location)
```

Path changes example: `use crate::camera::Camera;` (in `app.rs`) → `use crate::scene::camera::Camera;`; `use crate::render::Renderer;` (in `app.rs`) → `use crate::gpu::render::Renderer;`; `use crate::buffers::{...};` (in `render.rs`, now `gpu/render.rs`) → `use super::buffers::{...};`; `planet_renderer::camera::Camera` (external, in `tests/camera.rs`, `tests/uniforms.rs`) → `planet_renderer::scene::camera::Camera`; `planet_renderer::buffers::{...}` (in `tests/buffers.rs`, `tests/mesh_render_*.rs`) → `planet_renderer::gpu::buffers::{...}`; `planet_renderer::uniforms::pack_view_projection_uniform` (in `tests/uniforms.rs`) → `planet_renderer::gpu::uniforms::pack_view_projection_uniform`.

**`rules.md`'s "Module structure" section — rewritten:**
```markdown
## Module structure

Both crates organize `src/` by concern, not as a flat file list: every module lives
under a concern subdirectory, declared via a sibling `<concern>.rs` file (Rust 2024
module style — no `mod.rs`). The only files allowed directly under `src/` are
`lib.rs` (both crates) and `app.rs` (`planet-renderer`'s composition root — wasm-bindgen
entry point + winit event loop, wiring only). This is a documentation rule, enforced
at `planet-pr-validate` review time — the same way every other convention in this
file (naming, one-type-per-file) is enforced — not by an automated test.

`planet-core`'s concerns:
- `geometry/` — `vec3.rs` (`Vec3`), `mesh.rs` (`Vertex`, `Triangle`, `Mesh`, `MeshError`):
  spatial value types, no algorithm; plus a nested `primitives/` sub-concern
  (`icosahedron.rs`, `cube.rs`, both `pub(crate)` — exposed publicly only via
  `Mesh::icosahedron()` / `Mesh::cube()`, never directly) for mesh-construction
  functions built entirely from `geometry`'s own types
- `subdivision/` — `edge.rs` (`EdgeKey`, `EdgeCache`, `pub(crate)`), `steps.rs`
  (`Steps`, `StepsError`), `subdivision_mode.rs` (`SubdivisionMode`),
  `subdivision_args.rs` (`SubdivisionArgs`), `subdivide.rs` (`SubdivisionStrategy`
  `pub(crate)`, `subdivide`), `uniform_red_split.rs` (`UniformRedSplit`, `pub(crate)`):
  the recursive subdivision algorithm and its public configuration facade

`planet-renderer`'s concerns:
- `scene/` — `camera.rs` (`Camera`): orbit/zoom input math
- `gpu/` — `buffers.rs`, `uniforms.rs`, `render.rs`, `shader.wgsl`: everything
  wgpu-facing — mesh/preset-to-GPU-data mapping and the actual device/pipeline/draw calls
- `app.rs` (top-level) — winit event loop, wasm-bindgen entry point, HTML control wiring

Adding a new type: put it in the file for its existing concern if one fits; only
create a new concern subdirectory (and a `rules.md` entry for it, in this same list)
when no existing concern fits — never add a bare `.rs` file directly under `src/` as
a shortcut.

One type per file, everywhere (unchanged).
```

## Function/API contracts

- No `pub` function, method, struct, enum, or trait changes its name, signature, or visibility keyword anywhere in `planet-core` or `planet-renderer` as a result of this feature, with two related exceptions — `cargo doc -p planet-core --no-deps` and `cargo doc -p planet-renderer --no-deps` list the same public items as before this feature (just under new module paths, e.g. `planet_core::mesh::Mesh` → `planet_core::geometry::mesh::Mesh`), plus one addition and one visibility tightening:
  - Addition: `Mesh::icosahedron() -> Result<Mesh, MeshError>` is a new `pub` associated function. The free function `planet_core::icosahedron::icosahedron` it replaces is removed from the public surface entirely (not just moved) — its logic continues to exist, but only as `pub(crate) fn icosahedron()` inside `planet_core::geometry::primitives::icosahedron`
  - Visibility tightening: `Mesh::cube(side: f32) -> Result<Mesh, MeshError>` keeps its exact existing public signature and call form (`Mesh::cube(side)`) — no external change at all. Only its implementation body relocates, into `pub(crate) fn cube(side: f32) -> Result<Mesh, MeshError>` at `planet_core::geometry::primitives::cube`
- Both `pub(crate)` functions (`geometry::primitives::cube::cube`, `geometry::primitives::icosahedron::icosahedron`) are byte-for-byte identical in behavior to today's `Mesh::cube` body and today's free `icosahedron()` body respectively — same vertex positions, same triangle winding, same error conditions
- `planet_core::subdivision::edge::EdgeKey`, `EdgeCache`, `planet_core::subdivision::subdivide::SubdivisionStrategy`, and `planet_core::subdivision::uniform_red_split::UniformRedSplit` remain unreachable from outside `planet-core` (still `pub(crate)`, now additionally scoped one level deeper via non-`pub` `mod` declarations inside `subdivision.rs`) — verified the same way `005-subdivision-facade` verified it, via `cargo doc -p planet-core --no-deps` listing none of them

## BDD scenarios

This feature introduces no new domain behavior. `Mesh`, `Planet`, subdivision math, camera math, and buffer/uniform packing all behave exactly as before; `Mesh::cube`'s and `Mesh::icosahedron()`'s output is byte-for-byte identical to today's `Mesh::cube` and free `icosahedron()` respectively, only reachable through a new path (`icosahedron()`) or an unchanged one (`cube`). Per `constitution.md`'s BDD requirement, `cucumber` scenarios are reserved for domain behavior — there is none new here to add, so no new `.feature` file is introduced.

The by-concern layout rule itself (Requirements → Enforcement) is a documentation rule in `rules.md`, not a testable runtime behavior, so it has no BDD scenario either — consistent with how `rules.md`'s other conventions (naming, one-type-per-file) have never had BDD scenarios of their own.

`Mesh::cube`'s relocation and `Mesh::icosahedron()`'s addition are instead exercised by these pre-existing, already-`cucumber`-backed scenarios, whose step definitions get repointed at the new locations (per Domain model involved) but whose `Given`/`When`/`Then` text and expected outcomes are otherwise unchanged — quoted verbatim here as this feature's required happy-path and boundary coverage:

`planet-core/tests/features/icosahedron.feature` (happy path — exercises the new `Mesh::icosahedron()` facade method once its step definition is repointed):
```gherkin
Scenario: Constructing the icosahedron produces the expected vertex and triangle counts
  Given an icosahedron mesh
  Then the Mesh is constructed successfully
  And the Mesh has 12 vertices
  And the Mesh has 20 triangles
```

`planet-core/tests/features/mesh.feature` (happy path for the relocated `Mesh::cube`):
```gherkin
Scenario: Constructing a cube mesh with side 1.0 succeeds with the expected vertex and triangle counts
  Given a Mesh constructed by Mesh::cube with side 1.0
  Then the Mesh is constructed successfully
  And the Mesh has 8 vertices
  And the Mesh has 12 triangles
```

`planet-core/tests/features/mesh.feature` (boundary/edge case — exercises the moved error path, proving `MeshError::NegativeCubeSide` still fires correctly from `geometry::primitives::cube::cube` after the move):
```gherkin
Scenario: Constructing a cube mesh with a negative side fails
  When a Mesh is constructed by Mesh::cube with side -1.0
  Then the construction fails with a negative-cube-side error
```

These three scenarios (plus `mesh.feature`'s remaining cube scenarios and the three `mesh_render_*.feature` files' cube scenarios) are this feature's full regression coverage for the `cube`/`icosahedron` relocation: if `planet-tdd`'s move introduces any behavioral drift — a wrong vertex count, a dropped error check, a broken delegation from `Mesh::icosahedron()` to `primitives::icosahedron::icosahedron` — one of these fails. No new scenario is needed beyond repointing the step definitions, since the relocation adds no new behavior to cover.

All pre-existing `cucumber` feature files (`vec3.feature`, `mesh.feature`, `icosahedron.feature`, `subdivide.feature`, `steps.feature`, `subdivision_args.feature`, `camera.feature`, `buffers.feature`, `uniforms.feature`, `mesh_render_vertices.feature`, `mesh_render_indices.feature`, `mesh_render_line_indices.feature`) pass unmodified in content, with identical scenario counts and outcomes — only step-definition `.rs` files' `use` statements and (for `icosahedron`-related ones) call forms change, per the path table above.

## Acceptance criteria

1. `planet-core/src/` contains exactly `lib.rs`, `geometry.rs`, `geometry/`, `subdivision.rs`, `subdivision/` at its top level — no other `.rs` file (`primitives.rs`/`primitives/` live nested under `geometry/`, not at this top level)
2. `planet-renderer/src/` contains exactly `lib.rs`, `app.rs`, `scene.rs`, `scene/`, `gpu.rs`, `gpu/` at its top level — no other `.rs` file
3. Every file listed in the Domain model section exists at its documented path (including `geometry/primitives.rs` and `geometry/primitives/{icosahedron,cube}.rs`, nested under `geometry/`) with its documented public items intact (same names, same visibility keywords), except `Mesh::icosahedron` (new addition) and the `cube`/`icosahedron` implementations' visibility (tightened to `pub(crate)`), both documented as the intentional exceptions
4. `cargo doc -p planet-core --no-deps` lists the identical set of public item names as it did before this feature, plus one addition (`Mesh::icosahedron`) and minus one removal (the free function `icosahedron` no longer appears — `planet_core::geometry::primitives::icosahedron::icosahedron` and `planet_core::geometry::primitives::cube::cube` are both `pub(crate)` and absent from the doc output); `cargo doc -p planet-renderer --no-deps` lists the identical set of public item names as before, only differing by module-path prefix
5. `EdgeKey`, `EdgeCache`, `SubdivisionStrategy`, `UniformRedSplit`, `geometry::primitives::cube::cube`, and `geometry::primitives::icosahedron::icosahedron` all remain absent from `planet-core`'s public documentation output (`cargo doc -p planet-core --no-deps`)
6. `grep -rn "planet_core::icosahedron::icosahedron\|use crate::icosahedron::icosahedron"` across the whole repo returns zero matches outside this spec file and `docs/`; every former call site (`planet-renderer/src/app.rs`, `planet-core/tests/icosahedron.rs`, `planet-core/tests/subdivide.rs`) calls `Mesh::icosahedron()` instead. `grep -rn "Mesh::cube"` in the same files returns the same call sites as before this feature (unchanged — `Mesh::cube`'s call form never changes)
7. Every existing test file under `planet-core/tests/` and `planet-renderer/tests/` (12 pre-existing `.rs` files: `icosahedron.rs`, `mesh.rs`, `steps.rs`, `subdivide.rs`, `subdivision_args.rs`, `vec3.rs` in `planet-core/tests/`; `buffers.rs`, `camera.rs`, `mesh_render_indices.rs`, `mesh_render_line_indices.rs`, `mesh_render_vertices.rs`, `uniforms.rs` in `planet-renderer/tests/`) compiles against the new module paths with no remaining reference to a pre-move flat path (verified via `grep -rn "planet_core::\(mesh\|vec3\|icosahedron\|steps\|subdivide\|subdivision_args\|subdivision_mode\)::" planet-core/tests planet-renderer/tests planet-renderer/src` and `grep -rn "planet_renderer::\(camera\|buffers\|uniforms\)::" planet-renderer/tests` returning zero matches)
8. `rules.md`'s "Module structure" section reads exactly as drafted above (by-concern description including `primitives/` — `cube.rs`/`icosahedron.rs`, both `pub(crate)` — nested under `geometry/`, and the documentation-only enforcement rule)
9. All pre-existing `cucumber` scenarios (all 12 `.feature` files) still pass, unmodified in content, with identical scenario counts and outcomes to the pre-move baseline
10. No new `unwrap()`/`panic!()` in production code
11. `cargo test --workspace`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` all pass
12. `cargo build --target wasm32-unknown-unknown -p planet-renderer` still succeeds, and `include_str!("shader.wgsl")` in `gpu/render.rs` still resolves (shader.wgsl moved alongside it)
13. On loading the app in-browser, rendering is visually unchanged from before this feature (manual/in-browser spot check — no rendering logic changed, only import paths and the `icosahedron()` call form)
