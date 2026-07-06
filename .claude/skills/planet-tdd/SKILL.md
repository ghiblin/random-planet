---
name: planet-tdd
description: Use when implementing any Fractal Planet task — guides the RED-GREEN-REFACTOR cycle with build gate verification before every commit
---

# Fractal Planet TDD

## Workflow state file — blocking gate

This skill coordinates with `planet-spec` and `planet-spec-review` through a single state file at `<repo-root>/.claude/fractal-planet-workflow-state.json` — untracked (see `.gitignore`).

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
  "branch": "<actual git branch name backing this feature's worktree>",
  "worktree_path": "<actual worktree path, relative to repo-root>",
  "stage": "drafting-spec | ready-for-review | changes-requested | approved | implementing | complete | pr-changes-requested | validated",
  "updated_at": "<date>"
}
```

**HARD GATE:** read the file before writing any code.
- Missing file, or `stage` is `"drafting-spec"`, `"ready-for-review"`, or `"changes-requested"`: STOP. This feature's spec has not been reviewed and approved yet — implementation cannot start. Tell the user the current stage and that `planet-spec-review` must pass first.
- `stage == "approved"`: this is the first implementation session for this feature. Update `stage: "implementing"` and proceed, using `spec_file` from the state as the source of truth for tasks.
- `stage == "implementing"`: resume — a previous session already started; continue from where the task checklist in `spec_file` left off.
- `stage` is `"complete"`, `"pr-changes-requested"`, or `"validated"`: this feature is already implemented (and possibly already in PR review). Ask the user before redoing any work.

When every task for this feature/phase is implemented and the final build gate passes, update `stage: "complete"`. This is what unblocks `planet-pr-validate`.

## The Iron Law

**No production code without a confirmed failing test. If you wrote code before the test, delete it. Start over from the test.**

Violating the letter of this rule is violating the spirit of it.

Note: rendering glue that is genuinely untestable without a GPU/browser (per `rules.md` and `docs/specs/000-architecture.md` — `render.rs`, `app.rs`, the `wasm-bindgen`/DOM wiring) is exempt from the Iron Law but must stay thin: no domain or generation logic may live there. If you find yourself writing non-trivial logic in one of these files, extract it into a pure, testable module first.

## RED — Write a failing test

Write a test (or `cucumber` scenario step) that describes one expected behaviour. Run it. Confirm it fails for the right reason:
- Not a compilation error
- Not a missing import
- The test runs, the assertion fails, and the failure message describes the missing behaviour

**Hard gate: if the test does not fail, do not write production code.**

## GREEN — Write minimal code

Write the minimal code that makes the test pass. Nothing more. Run only the test in question. Confirm it passes.

## REFACTOR — Clean up

Clean up the code without changing behaviour. Re-run the test. It must still pass.

## Repeat

One behaviour per cycle. If the task requires multiple behaviours, run a full RED-GREEN-REFACTOR cycle for each one before moving to the next.

## Before committing

**HARD GATE: no commit goes on `main`. If the active branch is `main`, stop.**

- Run the full test suite for the affected crate(s). Zero regressions.
- Invoke `superpowers:verification-before-completion` and run the build gate:
  ```bash
  cargo test --workspace && cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo build --target wasm32-unknown-unknown -p planet-renderer
  ```
  You must produce fresh evidence (actual command output) before claiming the build gate passes. "It should pass" is not evidence.
- Semantic commit on the worktree branch. Never on `main`. Never add a `Co-Authored-By` trailer.
