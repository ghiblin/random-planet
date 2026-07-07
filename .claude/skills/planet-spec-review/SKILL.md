---
name: planet-spec-review
description: Use when reviewing a Fractal Planet spec before starting implementation — checklist across four categories with a hard-gate verdict
---

# Fractal Planet Spec Review

## Overview

A spec that passes this review is ready for implementation. A spec that fails is returned to the author. Do not start implementation until every item is pass.

## Workflow state file — blocking gate

This skill coordinates with `planet-spec` and `planet-tdd` through a single state file at `<repo-root>/.claude/fractal-planet-workflow-state.json` — untracked (see `.gitignore`).

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
  "branch": "feat/<slug> (enforced by planet-spec Phase 0)",
  "worktree_path": "<actual worktree path, relative to repo-root>",
  "stage": "drafting-spec | ready-for-review | changes-requested | approved | implementing | complete | pr-changes-requested | validated",
  "updated_at": "<date>"
}
```

**HARD GATE:** read the file before doing anything else.
- Missing file, or `stage == "drafting-spec"`: STOP. `planet-spec` has not finished this feature yet — there is nothing ready to review. Tell the user and do not proceed.
- `stage == "ready-for-review"` or `"changes-requested"`: proceed with the checklist below, reviewing `spec_file` from the state.
- `stage` already `"approved"`, `"implementing"`, `"complete"`, `"pr-changes-requested"`, or `"validated"`: this spec was already reviewed. Ask the user whether they want a re-review before proceeding.

At the end (Verdict), write the result back to the state file — see "Verdict" below.

## Checklist

Work through each item. Mark it pass or fail.

### 1. Completeness

- Does the spec have all five sections: Requirements, Domain model involved, Function/API contracts, BDD scenarios, Acceptance criteria?
- Is every section filled in — no "TBD", no empty sections?

### 2. Consistency with `constitution.md` (at repo root: `constitution.md`)

- Does the feature respect the non-negotiable constraints?
  - `planet-core` stays free of I/O, GPU, and WASM/browser dependencies
  - `Planet::generate` (or any new generation function) stays deterministic for a given seed
  - Any browser-only code is `#[cfg(target_arch = "wasm32")]`-gated
  - Subdivision recursion stays bounded by an explicit max-depth cap
- Does the domain logic involved belong in `planet-core`, not `planet-renderer` (or vice versa for rendering/input logic)?

### 3. BDD scenarios

- Is there at least one happy path scenario (Given/When/Then)?
- Is there at least one boundary/edge-case scenario?
- Does every scenario use Given/When/Then format without ambiguity?
- Does the phrasing follow the BDD scenario style in `rules.md` (explicit fixtures, consistent core scenario set/order across sibling feature files)?

### 4. Acceptance criteria

- Is every criterion testable — answerable with pass/fail by a unit, integration, or BDD test?
- Is no criterion vague? ("the mesh looks right" is not testable)

## Verdict

**Hard gate:** if even one item above is fail, the spec is not ready.

- All pass → implementation may begin. Update `<repo-root>/.claude/fractal-planet-workflow-state.json`: `stage: "approved"`. This is what unblocks `planet-tdd`.
- Any fail → return to the author with the specific failed items listed. Do not start implementation. Update the state file: `stage: "changes-requested"` — `planet-tdd` stays blocked until a re-review passes.
