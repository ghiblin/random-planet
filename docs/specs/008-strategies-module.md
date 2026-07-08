# 008 — Strategies Module

**Status:** Ready for review
**Feature slug:** `strategies-module`

This is an ad-hoc structural spec, like `005-subdivision-facade` and `006-by-concern-file-layout` before it — a pure file/module relocation with zero behavior change, not a roadmap phase.

**Worktree/branch deviation (explicit, user-directed):** every other spec in this project gets its own worktree and `feat/<slug>` branch per `planet-spec`'s Phase 0. This one does not: `RadialRandomSplit` (the type this reorg needs to move) exists only on the unmerged `feat/radial-randomness` branch — it hasn't landed on `main` yet — so a fresh worktree from `main` couldn't touch it. The user explicitly chose to fold this reorg into the existing `feat/radial-randomness` worktree/branch and combine it into that same not-yet-merged PR, re-running `planet-spec-review` and `planet-pr-validate` on the combined result before merge, rather than waiting for `radial-randomness` to merge first and opening a separate PR. This spec's `branch`/`worktree_path` in the workflow state file therefore point at `feat/radial-randomness` / `.claude/worktrees/radial-randomness`, not `feat/strategies-module` — mirroring the one documented exception `constitution.md` already carves out for its own worktree rule (the bootstrap commit), applied here to a second, narrower case.

## Requirements

- `planet-core`'s `subdivision/` concern gains a nested `strategies/` sub-concern, mirroring `geometry/`'s existing `primitives/` sub-concern pattern exactly: a public-facing type (`SubdivisionMode`, already the sole facade for algorithm selection since `005-subdivision-facade`) is backed by `pub(crate)` implementation structs that live one directory deeper, invisible outside the crate
- `UniformRedSplit` (`planet-core/src/subdivision/uniform_red_split.rs`) moves to `planet-core/src/subdivision/strategies/uniform_red_split.rs` — struct body, `split_triangle` impl, and the private `exact_midpoint` helper are byte-for-byte unchanged; only its `use` statements update to reflect the new nesting depth
- `RadialRandomSplit` and `MIN_VERTEX_RADIUS` (`planet-core/src/subdivision/radial_random_split.rs`) move to `planet-core/src/subdivision/strategies/radial_random_split.rs` — struct body, `split_triangle` impl, the private `displaced_midpoint` helper, and the `MIN_VERTEX_RADIUS` constant are byte-for-byte unchanged; only its `use` statements update
- A new `planet-core/src/subdivision/strategies.rs` declares both moved modules as `pub(crate)` — not bare private — for the same reason `geometry/primitives.rs` already does: `subdivision_mode.rs` (the sole consumer of both strategies) is a *sibling* of `strategies` under `subdivision`, not a descendant of `strategies` itself, so it needs the path `subdivision::strategies::uniform_red_split` to be reachable. This does not widen either struct's true reachability — both remain `pub(crate)`, invisible outside `planet-core`, and neither is referenced anywhere outside `subdivision_mode.rs` today (confirmed via `grep -rln "uniform_red_split\|radial_random_split" --include="*.rs" .`, which returns only `subdivision.rs` and `subdivision_mode.rs`)
- `planet-core/src/subdivision.rs`'s top-level module list drops the two direct `mod uniform_red_split;` / `mod radial_random_split;` declarations and gains a single `mod strategies;` in their place (alphabetical position preserved: `strategies` sorts between `steps` and `subdivide`)
- `subdivision_mode.rs`'s imports repoint from `super::uniform_red_split::UniformRedSplit` / `super::radial_random_split::RadialRandomSplit` to `super::strategies::uniform_red_split::UniformRedSplit` / `super::strategies::radial_random_split::RadialRandomSplit`. `SubdivisionMode::strategy()`'s match arms and every other line of `subdivision_mode.rs` are unchanged
- `rules.md`'s "Module structure" section is updated to document `strategies/` under `subdivision/`'s concern list, matching how `primitives/` is already documented under `geometry/`'s

Out of scope:
- Any change to `SubdivisionStrategy` (the trait, staying in `subdivide.rs` — it is not a strategy *implementation*, it's the abstraction both implementations satisfy, and `subdivide.rs` already sits correctly at the `subdivision/` concern level, not nested under `strategies/`), `EdgeCache`/`EdgeKey`, `Steps`, `Seed`, `ElevationNoiseRange`, `SubdivisionArgs`, or `subdivide()`'s own logic — none of their bodies, signatures, or visibility change
- Any change to `UniformRedSplit`'s or `RadialRandomSplit`'s actual subdivision math, RNG usage, or output — this is a pure relocation; every existing BDD scenario's expected numbers (triangle counts, radius bounds, determinism) are unaffected
- Any test file (`planet-core/tests/**`) changes — confirmed via grep that no test file references either struct's module path directly (both are `pub(crate)`, reachable only through the `SubdivisionMode` facade), so none need updating
- `planet-renderer` — untouched; it only ever calls `subdivide()`/`SubdivisionArgs`/`SubdivisionMode`, never the strategy structs directly
- Any new concern, sub-concern, or file beyond `strategies.rs` and the two moved files

## Domain model involved

No new types; existing types relocate as follows (identifiers, fields, methods, and bodies unchanged):

**`planet-core/src/subdivision.rs` (updated):**
```rust
mod edge;
pub mod elevation_noise_range;
pub mod seed;
pub mod steps;
mod strategies;
pub mod subdivide;
pub mod subdivision_args;
pub mod subdivision_mode;
```

**`planet-core/src/subdivision/strategies.rs` (new):**
```rust
pub(crate) mod radial_random_split;
pub(crate) mod uniform_red_split;
```

**`planet-core/src/subdivision/strategies/uniform_red_split.rs` (moved from `subdivision/uniform_red_split.rs`):**
```rust
use crate::geometry::mesh::{Triangle, Vertex};
use crate::subdivision::edge::EdgeCache;
use crate::subdivision::subdivide::SubdivisionStrategy;

fn exact_midpoint(a: &Vertex, b: &Vertex) -> Vertex {
    // unchanged body
}

pub(crate) struct UniformRedSplit;

impl SubdivisionStrategy for UniformRedSplit {
    // unchanged body
}
```
(`use super::edge::EdgeCache;` / `use super::subdivide::SubdivisionStrategy;` become `crate::subdivision::edge`/`crate::subdivision::subdivide` — absolute paths, matching exactly how `geometry/primitives/cube.rs` already reaches its sibling concern via `crate::geometry::mesh::...` rather than `super::super::...`. Both `edge` and `subdivide` stay visible to this new location because Rust's default module privacy extends to *all* descendants of the declaring module, not just direct children — `strategies::uniform_red_split` is a descendant of `subdivision` two levels down, exactly as `geometry::primitives::cube` is a descendant of `geometry` two levels down today)

**`planet-core/src/subdivision/strategies/radial_random_split.rs` (moved from `subdivision/radial_random_split.rs`):**
```rust
use rand::{RngExt, SeedableRng};
use rand_pcg::Pcg32;

use crate::geometry::mesh::{Triangle, Vertex};
use crate::subdivision::edge::EdgeCache;
use crate::subdivision::elevation_noise_range::ElevationNoiseRange;
use crate::subdivision::seed::Seed;
use crate::subdivision::subdivide::SubdivisionStrategy;

pub(crate) const MIN_VERTEX_RADIUS: f32 = 0.05;

fn displaced_midpoint(/* unchanged signature */) -> Vertex {
    // unchanged body
}

pub(crate) struct RadialRandomSplit {
    // unchanged fields
}

impl RadialRandomSplit {
    // unchanged
}

impl SubdivisionStrategy for RadialRandomSplit {
    // unchanged body
}
```

**`planet-core/src/subdivision/subdivision_mode.rs` (updated — imports only):**
```rust
use super::elevation_noise_range::ElevationNoiseRange;
use super::seed::Seed;
use super::strategies::radial_random_split::RadialRandomSplit;
use super::strategies::uniform_red_split::UniformRedSplit;
use super::subdivide::SubdivisionStrategy;
```
(`SubdivisionMode` enum definition and `strategy()`'s match arms are unchanged — only the two `use` lines that named `super::uniform_red_split`/`super::radial_random_split` now name `super::strategies::uniform_red_split`/`super::strategies::radial_random_split`)

**`rules.md`'s "Module structure" section — `subdivision/`'s concern-list bullet updated:**
```markdown
- `subdivision/` — `edge.rs` (`EdgeKey`, `EdgeCache`, `pub(crate)`), `steps.rs`
  (`Steps`, `StepsError`), `seed.rs` (`Seed`), `elevation_noise_range.rs`
  (`ElevationNoiseRange`, `ElevationNoiseRangeError`), `subdivision_mode.rs`
  (`SubdivisionMode`), `subdivision_args.rs` (`SubdivisionArgs`), `subdivide.rs`
  (`SubdivisionStrategy` `pub(crate)`, `subdivide`); plus a nested `strategies/`
  sub-concern (`uniform_red_split.rs`, `radial_random_split.rs`, both `pub(crate)`
  — exposed publicly only via `SubdivisionMode`, never directly) for the concrete
  subdivision-algorithm implementations: the recursive subdivision algorithm and
  its public configuration facade
```

## Function/API contracts

- No `pub` function, method, struct, enum, or trait anywhere in `planet-core` changes its name, signature, or visibility keyword as a result of this feature — `cargo doc -p planet-core --no-deps`'s public item listing is byte-identical before and after this change (verified: `UniformRedSplit`, `RadialRandomSplit`, and `MIN_VERTEX_RADIUS` are all `pub(crate)` today and remain `pub(crate)` after the move — none were ever part of the public surface, so none can be removed from it)
- `crate::subdivision::strategies::uniform_red_split::UniformRedSplit` and `crate::subdivision::strategies::radial_random_split::RadialRandomSplit` are reachable from `subdivision_mode.rs` (the only place that constructs either); neither is reachable from outside `planet-core` (still `pub(crate)`, and now additionally nested one level deeper, which only *narrows* the set of internal paths that happen to type-check trivially — it does not add any new external reachability)
- `SubdivisionMode::strategy(&self) -> Box<dyn SubdivisionStrategy>` has identical behavior before and after: same match arms, same constructed values, same `Box<dyn SubdivisionStrategy>` return type
- `UniformRedSplit::split_triangle` and `RadialRandomSplit::split_triangle` (via `displaced_midpoint`) produce byte-identical output to their pre-move implementations, for any input, since neither function's body changes — only the `use` statements that resolve `EdgeCache`/`SubdivisionStrategy`/`ElevationNoiseRange`/`Seed`/`Triangle`/`Vertex` change from relative (`super::`) to absolute (`crate::subdivision::...`/`crate::geometry::...`) paths, which resolve to the exact same items

## BDD scenarios

This feature introduces no new domain behavior — per `constitution.md`'s BDD requirement, `cucumber` scenarios are reserved for domain behavior, and there is none new here to add, so no new `.feature` file is introduced (the same reasoning `006-by-concern-file-layout` already used for its own pure-relocation scope).

Unlike `006-by-concern-file-layout` (which needed step-definition files' `use` statements repointed, since `Mesh::icosahedron()`'s free-function predecessor was called by name from three test files), this move requires **zero test file changes**: `UniformRedSplit` and `RadialRandomSplit` are `pub(crate)` and were never referenced from `planet-core/tests/**` or `planet-renderer` — confirmed via `grep -rln "uniform_red_split\|radial_random_split" --include="*.rs" .` returning only `subdivision.rs` and `subdivision_mode.rs`. The move's correctness is instead exercised entirely by every pre-existing scenario that already exercises `UniformRedSplit`/`RadialRandomSplit` through the unchanged `SubdivisionMode` facade, quoted here as this feature's required happy-path and boundary coverage:

**Happy path** (`planet-core/tests/features/subdivide.feature`, already passing, exercises the relocated `UniformRedSplit`):
```gherkin
Scenario: Subdividing the icosahedron mesh by 1 step using SubdivisionMode::UniformRedSplit quadruples the triangle count
  Given an icosahedron mesh
  When the mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit
  Then the resulting Mesh has 80 triangles
```

**Boundary/edge case** (`planet-core/tests/features/subdivide.feature`, already passing, exercises the relocated `RadialRandomSplit`'s zero-radius guard — the most delicate code path in either strategy):
```gherkin
Scenario: SubdivisionMode::RadialRandomSplit never panics when an edge's midpoint is exactly the origin
  Given a Mesh with an edge whose midpoint is the origin
  And a Triangle referencing indices 0, 1, 2
  When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7 and the default ElevationNoiseRange
  Then no panic occurs
```

All other scenarios in `subdivide.feature` (22 total), `seed.feature` (2), and `elevation_noise_range.feature` (4) provide the same regression guarantee — if the move introduced any drift (a wrong import resolving to a different item, a dropped `pub(crate)`, a typo in a relocated body), one of these would fail.

## Acceptance criteria

1. `planet-core/src/subdivision/` contains exactly `edge.rs`, `elevation_noise_range.rs`, `seed.rs`, `steps.rs`, `strategies.rs`, `strategies/`, `subdivide.rs`, `subdivision_args.rs`, `subdivision_mode.rs` at its top level — no `uniform_red_split.rs` or `radial_random_split.rs` directly under `subdivision/` anymore
2. `planet-core/src/subdivision/strategies/uniform_red_split.rs` and `planet-core/src/subdivision/strategies/radial_random_split.rs` exist, each with its struct/fn/const bodies byte-for-byte identical to the pre-move files (only `use` statements differ)
3. `cargo doc -p planet-core --no-deps`'s public item listing is unchanged — identical set of public item names, at identical paths, to before this feature (verified: `UniformRedSplit`, `RadialRandomSplit`, `MIN_VERTEX_RADIUS` do not appear in either listing, matching their pre-move absence)
4. `grep -rn "subdivision::uniform_red_split\|subdivision::radial_random_split\|super::uniform_red_split\|super::radial_random_split"` across the repo returns zero matches outside this spec file and `docs/`
5. `rules.md`'s "Module structure" section documents `strategies/` as a nested sub-concern of `subdivision/`, listing both moved files and their `pub(crate)` visibility, mirroring `primitives/`'s existing documentation under `geometry/`
6. Every scenario in `subdivide.feature` (22), `seed.feature` (2), and `elevation_noise_range.feature` (4) passes unmodified in content and outcome — same Given/When/Then text, same expected numbers
7. No `planet-core/tests/**` file changes are required or made
8. `cargo test --workspace`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` all pass
9. `cargo build --target wasm32-unknown-unknown -p planet-renderer` still succeeds
10. No new `unwrap()`/`panic!()` in production code
