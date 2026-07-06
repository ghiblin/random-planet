---
name: planet-pr-validate
description: Use when validating a Fractal Planet PR before merge — checklist across spec adherence and quality/security with a hard-gate verdict
---

# Fractal Planet PR Validate

## Overview

A PR that passes this review is ready to merge. A PR that fails is returned to the author with specific findings. Do not merge until every item is pass.

## Workflow state file — blocking gate

This skill coordinates with `planet-spec`, `planet-spec-review`, and `planet-tdd` through a single state file at `<repo-root>/.claude/fractal-planet-workflow-state.json` — untracked (see `.gitignore`).

**Always resolve `<repo-root>` as the main repository root, never the current working directory.** Each feature worktree gets its own checked-out `.claude` directory, so a path relative to cwd would silently write a different file per worktree and break coordination. Resolve it with:

```bash
git rev-parse --git-common-dir   # e.g. /path/to/fractal-planet/.git — same for every worktree
```

`<repo-root>` is the parent directory of that path. Read and write `<repo-root>/.claude/fractal-planet-workflow-state.json` there regardless of which worktree the skill is currently running in.

Schema:
```json
{
  "feature": "<slug>",
  "spec_file": "docs/specs/<NNN>-<slug>.md",
  "stage": "drafting-spec | ready-for-review | changes-requested | approved | implementing | complete | pr-changes-requested | validated",
  "updated_at": "<date>"
}
```

**HARD GATE:** read the file before doing anything else.
- Missing file, or `stage` is `"drafting-spec"`, `"ready-for-review"`, `"changes-requested"`, `"approved"`, or `"implementing"`: STOP. `planet-tdd` has not finished this feature yet — there is no completed implementation to validate. Tell the user the current stage.
- `stage == "complete"` or `"pr-changes-requested"`: proceed with the checklist below.
- `stage == "validated"`: this PR already passed validation. Ask the user whether they want a re-validation before proceeding.

At the end (Verdict), write the result back to the state file — see "Verdict" below.

## Precondition — PR must exist

```bash
gh pr view --json number,title,url,headRefName
```

**If no PR exists for the current branch:** STOP. Tell the user to open the PR first — this skill validates an existing PR, it does not create one.

Fetch the diff for review:
```bash
gh pr diff
```

## Checklist — Part 1: Spec adherence

Read `spec_file` from the state file. Work through each item. Mark it pass or fail.

- Does the diff implement every item in the spec's **Requirements** section — nothing missing, nothing silently descoped?
- Does the **domain model** match the spec: correct types, correct file per `rules.md` module structure (one type per file, `planet-core` flat layout / `planet-renderer` testability split)?
- Does the diff implement the **function/API contract** the spec specifies, or a documented reason it changed?
- Are the spec's **BDD scenarios** backed by real `.feature` files with `cucumber` step definitions — not left as markdown prose? An undefined step must fail the suite; confirm no scenario has a stub step that trivially passes
- Is every **acceptance criterion** covered by a passing unit, integration, or BDD test visible in the diff?
- Does the diff respect `constitution.md`: `planet-core` free of I/O/GPU/WASM, deterministic generation, browser-only code `cfg`-gated, bounded recursion depth?
- Does the diff follow `rules.md`: naming conventions, `Error`-suffixed error types, no `unwrap()` outside tests, semantic commit messages, no `Co-Authored-By` trailers?

## Checklist — Part 2: Quality & security

- **Crate boundaries**: does `planet-core` stay free of `wgpu`, `winit`, `wasm-bindgen`, `web-sys`, `std::fs`, `std::net`, and any dependency on `planet-renderer`? Does `planet-renderer` keep generation/domain logic out of `render.rs`/`app.rs`? (See each crate's `RULES.md` once scaffolded.)
- **Determinism**: does any new randomness in `planet-core` go through the seeded RNG (never system entropy, timers, or hash-map iteration order)?
- **Recursion safety**: does any new or changed subdivision logic still respect the hard max-depth cap regardless of preset parameters?
- **Error handling**: no `unwrap()`/`panic!()` in production code paths; DOM/canvas lookups in `planet-renderer` handle `None` explicitly instead of unwrapping
- **Dependency review**: any new dependency justified and, if it's a confirmed choice, added to `tech-stack.md`?
- **Debug leftovers**: no stray `console.log`/`web_sys::console::log_*`/`println!` debug spam left in production code paths
- **WASM build**: does `cargo build --target wasm32-unknown-unknown -p planet-renderer` actually succeed (not just native `cargo test`)?

## Verdict

**Hard gate:** if even one item above is fail, the PR is not ready to merge.

- All pass → merge may proceed. Update `<repo-root>/.claude/fractal-planet-workflow-state.json`: `stage: "validated"`. This is what unblocks `planet-pr-merge`. Tell the user: "Validation passed. Run planet-pr-merge to merge."
- Any fail → post the specific failed items on the PR (`gh pr comment --body "..."`) and to the user. Do not merge. Update the state file: `stage: "pr-changes-requested"` — `planet-pr-merge` stays blocked until a re-validation passes.
